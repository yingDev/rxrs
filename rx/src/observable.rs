use std::any::Any;
use subref::*;
use std::sync::Arc;
use std::marker::PhantomData;
use std::cell::UnsafeCell;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use util::mss::*;
use std::rc::Rc;
use std::error::Error;

pub type ArcErr = Arc<Box<Error+Send+Sync>>;

pub trait Observable<'o, V, SSO:?Sized=No, SSS:?Sized=No>
{
    fn sub(&self, o: Mss<SSO, impl Observer<V>+'o>)-> SubRef<SSS>;
}

pub trait Observer<V>
{
    fn next(&self, _:V){}
    fn err(&self, _:ArcErr){} //todo
    fn complete(&self){}

    fn _is_closed(&self) -> bool { false }
}

pub trait ObserverHelper<V>
{
    fn next(&self, _: V){}
    fn err(&self, _: ArcErr){}
    fn complete(&self){}
    fn _is_closed(&self) -> bool{ false }
}

pub enum RxNoti<V>
{
    Next(V), Err(ArcErr), Comp
}

pub trait ObservableSubNotiHelper<V, F, SSO:?Sized+'static, SSS:?Sized+'static>
{
    fn sub_noti(&self, f: F) -> SubRef<SSS>;
}

#[derive(PartialEq, Copy, Clone)]
pub enum IsClosed
{
    Default,
    True,
    False
}

pub trait AsIsClosed
{
    fn is_closed(&self) -> IsClosed;
}
impl AsIsClosed for () {
    #[inline(always)]fn is_closed(&self) -> IsClosed { IsClosed::Default }
}
impl AsIsClosed for IsClosed
{
    #[inline(always)]fn is_closed(&self) -> IsClosed{ *self }
}
//===========
impl<'o, V:'o,F, Obs, Ret:AsIsClosed, SSS:?Sized+'static> ObservableSubNotiHelper<V, F, No, SSS> for Obs where Obs:Observable<'o,V, No, SSS>, F: 'o+FnMut(RxNoti<V>)->Ret
{
    #[inline(always)]
    fn sub_noti(&self, f: F) -> SubRef<SSS>
    {
        let f = FnCell::new(f);
        self.sub(Mss::new(MatchObserver::new(move |n| unsafe {
            (*f.0.get())(n)
        })))
    }
}

impl<'o, V:'o,F, Obs, Ret:AsIsClosed, SSS:?Sized+'static> ObservableSubNotiHelper<V, F, Yes, SSS> for Obs where Obs:Observable<'o,V, Yes, SSS>, F: Send+Sync+'o+FnMut(RxNoti<V>)->Ret
{
    #[inline(always)]
    fn sub_noti(&self, f: F) -> SubRef<SSS>
    {
        let f = FnCell::new(f);
        self.sub(Mss::new(MatchObserver::new(move |n| unsafe {
            (*f.0.get())(n)
        })))
    }
}


pub struct MatchObserver<V,F>
{
    pub closed: AtomicBool,
    pub f: F,
    PhantomData: PhantomData<*const V>
}
impl<V,F> MatchObserver<V,F>
{
    #[inline(always)]
    fn new(f:F) -> MatchObserver<V,F>{ MatchObserver{ closed: AtomicBool::new(false), f, PhantomData } }
}
unsafe impl<V,F> Send for MatchObserver<V,F> where F: Send{}
unsafe impl<V,F> Sync for MatchObserver<V,F> where F: Sync{}
impl<'o, V,F,Ret:AsIsClosed> Observer<V> for MatchObserver<V,F> where F: Send+Sync+'o+Fn(RxNoti<V>)->Ret
{
    #[inline(always)]
    fn next(&self, v:V)
    {
        if ! self.closed.load(Ordering::Acquire) {
            if ((self.f)(RxNoti::Next(v))).is_closed() == IsClosed::True {
                self.closed.store(true, Ordering::Release);
            }
        }
    }

    #[inline(always)]
    fn err(&self, e:ArcErr)
    {
        if ! self.closed.compare_and_swap(false, true, Ordering::Acquire) {
            if ((self.f)(RxNoti::Err(e))).is_closed() == IsClosed::False {
                self.closed.store(false, Ordering::Release);
            }
        }
    }

    #[inline(always)]
    fn complete(&self)
    {
        if ! self.closed.compare_and_swap(false, true, Ordering::Acquire) {
            if ((self.f)(RxNoti::Comp)).is_closed() == IsClosed::False {
                self.closed.store(false, Ordering::Release);
            }
        }
    }

    #[inline(always)]
    fn _is_closed(&self) -> bool { self.closed.load(Ordering::SeqCst) }
}

#[derive(Clone)]
pub struct ByrefOp<'a:'b, 'b, V, Src:'b, SSO:?Sized, SSS:?Sized> //where Src : Observable<'o, V, SSO>+'b
{
    source: &'b Src,
    PhantomData: PhantomData<(*const V, &'a(), *const SSO, *const SSS)>
}
unsafe impl<'o,'b,V,Src,SSO:?Sized, SSS:?Sized> Send for ByrefOp<'o,'b,V,Src,SSO, SSS> where Src:Send{}
unsafe impl<'o,'b,V,Src,SSO:?Sized, SSS:?Sized> Sync for ByrefOp<'o,'b,V,Src,SSO, SSS> where Src:Sync{}


pub trait ObservableByref<'o:'b, 'b, V, Src,SSO:?Sized, SSS:?Sized> where Src : Observable<'o, V, SSO, SSS>+'b
{
    fn rx(&'b self) -> ByrefOp<'o, 'b, V, Src, SSO, SSS>;
}

impl<'o:'b,'b, V, Src,SSO:?Sized, SSS:?Sized> ObservableByref<'o, 'b, V, Src,SSO, SSS> for Src where Src : Observable<'o, V, SSO, SSS>+'b
{
    #[inline(always)]
    fn rx(&'b self) -> ByrefOp<'o, 'b,V, Src, SSO, SSS>
    {
        ByrefOp{ source: self, PhantomData }
    }
}

impl<'o:'b, 'b, V:'o, Src, SSO:?Sized, SSS:?Sized> Observable<'o,V,SSO, SSS> for ByrefOp<'o, 'b, V,Src,SSO,SSS> where Src: Observable<'o, V, SSO, SSS>+'b
{
    #[inline(always)]
    fn sub(&self, o: Mss<SSO, impl Observer<V>+'o>) -> SubRef<SSS>
    {
        self.source.sub(o)
    }
}

impl<V, F: Fn(V), FErr: Fn(ArcErr), FComp: Fn()> Observer<V> for (F, FErr, FComp, PhantomData<()>)
{
    #[inline] fn next(&self, v:V) { self.0(v) }
    #[inline] fn err(&self, e:ArcErr) { self.1(e) }
    #[inline] fn complete(&self) { self.2() }
}


impl<V, F: Fn(V)> ObserverHelper<V> for F
{
    #[inline] fn next(&self, v:V) { self(v) }
}
impl<V, FErr: Fn(ArcErr)> ObserverHelper<V> for ((), FErr)
{
    #[inline] fn err(&self, e:ArcErr) { self.1(e) }
}
impl<V,FComp: Fn()> ObserverHelper<V> for ((), (), FComp)
{
    #[inline] fn complete(&self) { self.2() }
}
impl<V, F: Fn(V), FErr: Fn(ArcErr),FComp: Fn()> ObserverHelper<V> for (F, FErr, FComp)
{
    #[inline] fn next(&self, v:V) { self.0(v) }
    #[inline] fn err(&self, e:ArcErr) { self.1(e) }
    #[inline] fn complete(&self) { self.2() }
}
impl<V, F: Fn(V), FErr: Fn(ArcErr)> ObserverHelper<V> for (F, FErr)
{
    #[inline] fn next(&self, v:V) { self.0(v) }
    #[inline] fn err(&self, e:ArcErr) { self.1(e) }
}
impl<V, FErr: Fn(ArcErr),FComp: Fn()> ObserverHelper<V> for ((), FErr, FComp)
{
    #[inline] fn err(&self, e:ArcErr) { self.1(e) }
    #[inline] fn complete(&self) { self.2() }
}
impl<V,F: Fn(V),FComp: Fn()> ObserverHelper<V> for (F, (), FComp)
{
    #[inline] fn next(&self, v:V) { self.0(v) }
    #[inline] fn complete(&self) { self.2() }
}


impl<'o, V, Src, SSO:?Sized, SSS:?Sized> Observable<'o,V, SSO, SSS> for Arc<Src> where Src : Observable<'o,V, SSO, SSS>
{
    #[inline(always)]
    fn sub(&self, o: Mss<SSO, impl Observer<V>+'o>) -> SubRef<SSS>
    {
        Arc::as_ref(self).sub(o)
    }
}

impl<'o, V, Src, SSO:?Sized, SSS:?Sized> Observable<'o,V, SSO, SSS> for Rc<Src> where Src : Observable<'o,V, SSO, SSS>
{
    #[inline(always)]
    fn sub(&self, o: Mss<SSO, impl Observer<V>+'o>) -> SubRef<SSS>
    {
        Rc::as_ref(self).sub(o)
    }
}

impl<V> ObserverHelper<V> for Arc<Observer<V>>
{
    #[inline] fn next(&self, v: V){
        Arc::as_ref(self).next(v);
    }
    #[inline] fn err(&self, e: ArcErr) {
        Arc::as_ref(self).err(e);
    }
    #[inline] fn complete(&self) {
        Arc::as_ref(self).complete();

    }
    #[inline] fn _is_closed(&self) -> bool {
        Arc::as_ref(self)._is_closed()
    }
}

impl<V, O> Observer<V> for Arc<O> where O : Observer<V>
{
    fn next(&self, v:V){ Arc::as_ref(self).next(v); }
    fn err(&self, e:ArcErr){ Arc::as_ref(self).err(e); }
    fn complete(&self){ Arc::as_ref(self).complete(); }
    fn _is_closed(&self) -> bool { Arc::as_ref(self)._is_closed() }
}

pub trait SubFHelper<V,F, SSO:?Sized, SSS:?Sized>
{
    fn subf(&self, f: F) -> SubRef<SSS>;
}

pub struct FnCell<F>(pub UnsafeCell<F>);
unsafe impl<F> Send for FnCell<F> {}
unsafe impl<F> Sync for FnCell<F> {}
impl<F> FnCell<F>
{
    #[inline(always)]
    pub fn new(f:F) -> FnCell<F> { FnCell(UnsafeCell::new(f)) }
}

fn _empty<V>(_:V){}
fn _comp(){}

impl<'o, Obs, V, F, R, SSS:?Sized> SubFHelper<V,F, No, SSS> for Obs
    where Obs : Observable<'o, V, No, SSS>, F: 'o+FnMut(V) ->R
{
    #[inline(always)]
    fn subf(&self, f: F)-> SubRef<SSS> {
        unsafe{
            let next = FnCell::new(f);
            self.sub(Mss::new(( move |v| { (*next.0.get())(v); }, _empty, _comp, PhantomData)))
        }
    }
}

impl<'o, Obs, V:'o, FErr,RE, SSS:?Sized> SubFHelper<V,((),FErr), No, SSS> for Obs
    where Obs : Observable<'o, V, No, SSS>,  FErr: 'o+FnMut(ArcErr) ->RE,
{
    #[inline(always)]
    fn subf(&self, f: ((),FErr))-> SubRef<SSS>
    {
        unsafe{
            let ferr = FnCell::new(f.1);
            self.sub(Mss::new(( _empty, move |e| {(*ferr.0.get())(e);}, _comp, PhantomData)))
        }
    }
}

impl<'o, Obs, V:'o, F,FErr, FComp, R, RE, RC, SSS:?Sized> SubFHelper<V,(F,FErr,FComp), No, SSS> for Obs
    where Obs : Observable<'o, V, No, SSS>, F: 'o+FnMut(V)->R, FErr: 'o+FnMut(ArcErr)->RE, FComp: 'o+FnMut()->RC,
{
    #[inline(always)]
    fn subf(&self, f: (F,FErr,FComp))-> SubRef<SSS>
    {
        unsafe{
            let (next, err, comp) = (FnCell::new(f.0), FnCell::new(f.1), FnCell::new(f.2));
            self.sub(Mss::new(( move |v| {(*next.0.get())(v);}, move |e| {(*err.0.get())(e);}, move || {(*comp.0.get())();}, PhantomData)))
        }
    }
}

impl<'o, Obs, V:'o, FComp, RC, SSS:?Sized> SubFHelper<V,((),(),FComp), No, SSS> for Obs
    where Obs : Observable<'o, V, No, SSS>,  FComp: 'o+FnMut()-> RC
{
    #[inline(always)]
    fn subf(&self, f: ((),(),FComp))-> SubRef<SSS>
    {
        unsafe{
            let comp = FnCell::new(f.2);
            self.sub(Mss::new((_empty, _empty, move || {(*comp.0.get())();}, PhantomData)))
        }
    }
}

impl<'o, Obs, V:'o, F,FErr, R, RE, SSS:?Sized> SubFHelper<V,(F,FErr), No, SSS> for Obs
    where Obs : Observable<'o, V, No, SSS>, F:'o+FnMut(V)->R, FErr:'o+FnMut(ArcErr)->RE,
{
    #[inline(always)]
    fn subf(&self, f: (F,FErr))-> SubRef<SSS>
    {
        unsafe{
            let (next, err) = (FnCell::new(f.0), FnCell::new(f.1));
            self.sub(Mss::new(( move |v| {(*next.0.get())(v);}, move |e| {(*err.0.get())(e);}, _comp, PhantomData)))
        }
    }
}

impl<'o, Obs, V:'o, F,FComp, R, RC, SSS:?Sized> SubFHelper<V,(F,(), FComp), No, SSS> for Obs
    where Obs : Observable<'o, V, No, SSS>, F:'o+FnMut(V)->R, FComp:'o+FnMut()->RC,
{
    #[inline(always)]
    fn subf(&self, f: (F,(), FComp))-> SubRef<SSS>
    {
        unsafe{
            let (next,comp) = (FnCell::new(f.0), FnCell::new(f.2));
            self.sub(Mss::new(( move |v| {(*next.0.get())(v);}, _empty, move || {(*comp.0.get())();}, PhantomData)))
        }
    }
}

impl<'o, Obs, V:'o, FErr,FComp, RE, RC, SSS:?Sized> SubFHelper<V,((),FErr, FComp), No, SSS> for Obs
    where Obs : Observable<'o, V, No, SSS>, FErr:'o+FnMut(ArcErr)->RE, FComp:'o+FnMut()->RC,
{
    #[inline(always)]
    fn subf(&self, f: ((),FErr,FComp))-> SubRef<SSS>
    {
        unsafe {
            let ( err, comp) = (FnCell::new(f.1), FnCell::new(f.2));
            self.sub(Mss::new(( _empty, move |e| { (*err.0.get())(e);}, move || {(*comp.0.get())();}, PhantomData)))
        } }
}

///////// SSO
impl<'o, Obs, V:'o, F, R, SSS:?Sized> SubFHelper<V,F, Yes, SSS> for Obs
    where Obs : Observable<'o, V, Yes, SSS>, F: 'o+Send+Sync+FnMut(V)->R
{
    #[inline(always)]
    fn subf(&self, f: F)-> SubRef<SSS> {
        unsafe{
            let next = FnCell::new(f);
            self.sub(Mss::new(( move |v| { (*next.0.get())(v); }, _empty, _comp, PhantomData)))
        }
    }
}

impl<'o, Obs, V:'o, FErr, RE, SSS:?Sized> SubFHelper<V,((),FErr), Yes, SSS> for Obs
    where Obs : Observable<'o, V, Yes, SSS>,  FErr:'o+Send+Sync+FnMut(ArcErr)->RE,
{
    #[inline(always)]
    fn subf(&self, f: ((),FErr))-> SubRef<SSS>
    {
        unsafe{
            let ferr = FnCell::new(f.1);
            self.sub(Mss::new(( _empty, move |e| { (*ferr.0.get())(e); }, _comp, PhantomData)))
        }
    }
}

impl<'o, Obs, V:'o, F, R,FErr,RE, FComp,RC, SSS:?Sized> SubFHelper<V,(F,FErr,FComp), Yes, SSS> for Obs
    where Obs : Observable<'o, V, Yes, SSS>, F: 'o+Send+Sync+FnMut(V)->R, FErr: 'o+Send+Sync+FnMut(ArcErr)->RE, FComp: 'o+Send+Sync+FnMut()->RC,
{
    #[inline(always)]
    fn subf(&self, f: (F,FErr,FComp))-> SubRef<SSS>
    {
        unsafe{
            let (next, err, comp) = (FnCell::new(f.0), FnCell::new(f.1), FnCell::new(f.2));
            self.sub(Mss::new(( move |v| { (*next.0.get())(v);}, move |e| { (*err.0.get())(e);}, move || { (*comp.0.get())();}, PhantomData)))
        }
    }
}

impl<'o, Obs, V:'o, FComp, RC, SSS:?Sized> SubFHelper<V,((),(),FComp), Yes, SSS> for Obs
    where Obs : Observable<'o, V, Yes, SSS>,  FComp: Send+Sync+'o+FnMut()->RC
{
    #[inline(always)]
    fn subf(&self, f: ((),(),FComp))-> SubRef<SSS>
    {
        unsafe{
            let comp = FnCell::new(f.2);
            self.sub(Mss::new((_empty, _empty, move || { (*comp.0.get())();}, PhantomData)))
        }
    }
}

impl<'o, Obs, V:'o, F,FErr, R, RE, SSS:?Sized> SubFHelper<V,(F,FErr), Yes, SSS> for Obs
    where Obs : Observable<'o, V, Yes, SSS>, F:'o+Send+Sync+FnMut(V)->R, FErr:'o+Send+Sync+FnMut(ArcErr) -> RE,
{
    #[inline(always)]
    fn subf(&self, f: (F,FErr))-> SubRef<SSS>
    {
        unsafe{
            let (next, err) = (FnCell::new(f.0), FnCell::new(f.1));
            self.sub(Mss::new(( move |v| { (*next.0.get())(v);}, move |e| { (*err.0.get())(e);}, _comp, PhantomData)))
        }
    }
}

impl<'o, Obs, V:'o, F,FComp, R, RC, SSS:?Sized> SubFHelper<V,(F,(), FComp), Yes, SSS> for Obs
    where Obs : Observable<'o, V, Yes, SSS>, F:'o+Send+Sync+FnMut(V)->R, FComp:'o+Send+Sync+FnMut()->RC,
{
    #[inline(always)]
    fn subf(&self, f: (F,(), FComp))-> SubRef<SSS>
    {
        unsafe{
            let (next,comp) = (FnCell::new(f.0), FnCell::new(f.2));
            self.sub(Mss::new(( move |v| { (*next.0.get())(v);}, _empty, move || {(*comp.0.get())();}, PhantomData)))
        }
    }
}

impl<'o, Obs, V:'o, FErr,FComp, RE, RC, SSS:?Sized> SubFHelper<V,((),FErr, FComp), Yes, SSS> for Obs
    where Obs : Observable<'o, V, Yes, SSS>, FErr:'o+Send+Sync+FnMut(ArcErr)->RE, FComp:'o+Send+Sync+FnMut()->RC,
{
    #[inline(always)]
    fn subf(&self, f: ((),FErr,FComp))-> SubRef<SSS>
    {
        unsafe {
            let ( err, comp) = (FnCell::new(f.1), FnCell::new(f.2));
            self.sub(Mss::new(( _empty, move |e| {(*err.0.get())(e);}, move || {(*comp.0.get())();}, PhantomData)))
        } }
}

#[cfg(test)]
mod test
{
    use super::*;
    use test_fixture::*;
    use util::mss::*;
    use std::rc::Rc;
    use std::sync::Mutex;
    use std::sync::Arc;
    use std::sync::atomic::AtomicIsize;

    #[test]
    fn hello_world()
    {
        {
            let s = SimpleObservable;
            let o = SimpleObserver;
            s.sub(mss!(o));
        }

        {
            let s = SimpleObservable;
            let ts = ThreadedObservable;
            let i = 123;
            let o = LocalObserver(&i);
            s.sub(mss!(o));
            //should not compile
            //ts.sub(o);
        }

        {
            let i = 123;
            let s = StoreObserverObservable::new();
            //should fail if moved here
            //let i = 123;
            let o = LocalObserver(&i);
            s.sub(mss!(o));
        }


        {
            let ts = ThreadedObservable;
            ts.sub_noti(|n| { println!("noti"); });
        }

        {
            let ts = ThreadedObservable;
            //should not compile
            //ts.sub_noti(|n| { println!("noti: {}", i); });
            //ts.subf(|v| println!("ok {}", i));
        }

        {
            let s = SimpleObservable;
            let nso = NonSendObserver(Rc::new(123));
            s.sub(mss!(nso));
        }

    }
}