use std::any::Any;
use subref::*;
use std::sync::Arc;
use std::marker::PhantomData;
use std::cell::UnsafeCell;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use util::mss::*;

pub type ArcErr = Arc<Box<Any+Send+Sync>>;

pub trait Observable<'a, V, SSO:?Sized>
{
    fn sub(&self, o: Mss<SSO, impl Observer<V>+'a>)-> SubRef;
}

pub trait Observer<V>
{
    fn next(&self, _:V){}
    fn err(&self, _:ArcErr){} //todo
    fn complete(&self){}

    fn _is_closed(&self) -> bool { false }

    fn next_noti(&self, n:RxNoti<V>)
    {
        use self::RxNoti::*;

        match n {
            Next(v) => self.next(v),
            Err(e) => self.err(e),
            Comp => self.complete()
        }
    }
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

pub trait ObservableSubNotiHelper<V, F, SSO:?Sized+No>
{
    fn sub_noti(&self, f: F) -> SubRef;
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
impl<'a, V:'a,F, Obs, Ret:AsIsClosed> ObservableSubNotiHelper<V, F, No> for Obs where Obs:Observable<'a,V, No>, F: 'a+FnMut(RxNoti<V>)->Ret
{
    #[inline(always)]
    fn sub_noti(&self, f: F) -> SubRef
    {
        let f = FnCell::new(f);
        self.sub(Mss::new(MatchObserver::new(move |n| unsafe {
            (*f.0.get())(n)
        })))
    }
}

impl<'a, V:'a,F, Obs, Ret:AsIsClosed> ObservableSubNotiHelper<V, F, Yes> for Obs where Obs:Observable<'a,V, Yes>, F: Send+Sync+'a+FnMut(RxNoti<V>)->Ret
{
    #[inline(always)]
    fn sub_noti(&self, f: F) -> SubRef
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
impl<'a, V,F,Ret:AsIsClosed> Observer<V> for MatchObserver<V,F> where F: Send+Sync+'a+Fn(RxNoti<V>)->Ret
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
pub struct ByrefOp<'a:'b, 'b, V, Src:'b, SSO:?Sized> //where Src : Observable<'a, V, SSO>+'b
{
    source: &'b Src,
    PhantomData: PhantomData<(*const V, &'a(), SSO)>
}
unsafe impl<'a,'b,V,Src,SSO:?Sized> Send for ByrefOp<'a,'b,V,Src,SSO> where Src:Send{}
unsafe impl<'a,'b,V,Src,SSO:?Sized> Sync for ByrefOp<'a,'b,V,Src,SSO> where Src:Sync{}


pub trait ObservableByref<'a:'b, 'b, V, Src,SSO:?Sized> where Src : Observable<'a, V, SSO>+'b
{
    fn rx(&'b self) -> ByrefOp<'a, 'b, V, Src, SSO>;
}

impl<'a:'b,'b, V, Src,SSO:?Sized> ObservableByref<'a, 'b, V, Src,SSO> for Src where Src : Observable<'a, V, SSO>+'b
{
    #[inline(always)]
    fn rx(&'b self) -> ByrefOp<'a, 'b,V, Src, SSO>
    {
        ByrefOp{ source: self, PhantomData }
    }
}

impl<'a:'b, 'b, V:'a, Src, S:?Sized> Observable<'a,V,S> for ByrefOp<'a, 'b, V,Src,S> where Src: Observable<'a, V, S>+'b
{
    #[inline(always)]
    fn sub(&self, o: Mss<S, impl Observer<V>+'a>) -> SubRef
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


impl<'a, V, Src> Observable<'a,V, No> for Arc<Src> where Src : Observable<'a,V, No>
{
    #[inline(always)]
    fn sub(&self, o: Mss<No, impl Observer<V>+'a>) -> SubRef
    {
        Arc::as_ref(self).sub(o)
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

pub trait SubFHelper<V,F, SSO:?Sized+No>
{
    fn subf(&self, f: F) -> SubRef;
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

impl<'a, Obs, V:'a, F, R> SubFHelper<V,F, No> for Obs
    where Obs : Observable<'a, V, No>, F: 'a+FnMut(V) ->R
{
    #[inline(always)]
    fn subf(&self, f: F)-> SubRef {
        unsafe{
            let next = FnCell::new(f);
            self.sub(Mss::new(( move |v| { (*next.0.get())(v); }, _empty, _comp, PhantomData)))
        }
    }
}

impl<'a, Obs, V:'a, FErr,RE> SubFHelper<V,((),FErr), No> for Obs
    where Obs : Observable<'a, V, No>,  FErr: 'a+FnMut(ArcErr) ->RE,
{
    #[inline(always)]
    fn subf(&self, f: ((),FErr))-> SubRef
    {
        unsafe{
            let ferr = FnCell::new(f.1);
            self.sub(Mss::new(( _empty, move |e| {(*ferr.0.get())(e);}, _comp, PhantomData)))
        }
    }
}

impl<'a, Obs, V:'a, F,FErr, FComp, R, RE, RC> SubFHelper<V,(F,FErr,FComp), No> for Obs
    where Obs : Observable<'a, V, No>, F: 'a+FnMut(V)->R, FErr: 'a+FnMut(ArcErr)->RE, FComp: 'a+FnMut()->RC,
{
    #[inline(always)]
    fn subf(&self, f: (F,FErr,FComp))-> SubRef
    {
        unsafe{
            let (next, err, comp) = (FnCell::new(f.0), FnCell::new(f.1), FnCell::new(f.2));
            self.sub(Mss::new(( move |v| {(*next.0.get())(v);}, move |e| {(*err.0.get())(e);}, move || {(*comp.0.get())();}, PhantomData)))
        }
    }
}

impl<'a, Obs, V:'a, FComp, RC> SubFHelper<V,((),(),FComp), No> for Obs
    where Obs : Observable<'a, V, No>,  FComp: 'a+FnMut()-> RC
{
    #[inline(always)]
    fn subf(&self, f: ((),(),FComp))-> SubRef
    {
        unsafe{
            let comp = FnCell::new(f.2);
            self.sub(Mss::new((_empty, _empty, move || {(*comp.0.get())();}, PhantomData)))
        }
    }
}

impl<'a, Obs, V:'a, F,FErr, R, RE> SubFHelper<V,(F,FErr), No> for Obs
    where Obs : Observable<'a, V, No>, F:'a+FnMut(V)->R, FErr:'a+FnMut(ArcErr)->RE,
{
    #[inline(always)]
    fn subf(&self, f: (F,FErr))-> SubRef
    {
        unsafe{
            let (next, err) = (FnCell::new(f.0), FnCell::new(f.1));
            self.sub(Mss::new(( move |v| {(*next.0.get())(v);}, move |e| {(*err.0.get())(e);}, _comp, PhantomData)))
        }
    }
}

impl<'a, Obs, V:'a, F,FComp, R, RC> SubFHelper<V,(F,(), FComp), No> for Obs
    where Obs : Observable<'a, V, No>, F:'a+FnMut(V)->R, FComp:'a+FnMut()->RC,
{
    #[inline(always)]
    fn subf(&self, f: (F,(), FComp))-> SubRef
    {
        unsafe{
            let (next,comp) = (FnCell::new(f.0), FnCell::new(f.2));
            self.sub(Mss::new(( move |v| {(*next.0.get())(v);}, _empty, move || {(*comp.0.get())();}, PhantomData)))
        }
    }
}

impl<'a, Obs, V:'a, FErr,FComp, RE, RC> SubFHelper<V,((),FErr, FComp), No> for Obs
    where Obs : Observable<'a, V, No>, FErr:'a+FnMut(ArcErr)->RE, FComp:'a+FnMut()->RC,
{
    #[inline(always)]
    fn subf(&self, f: ((),FErr,FComp))-> SubRef
    {
        unsafe {
            let ( err, comp) = (FnCell::new(f.1), FnCell::new(f.2));
            self.sub(Mss::new(( _empty, move |e| { (*err.0.get())(e);}, move || {(*comp.0.get())();}, PhantomData)))
        } }
}

/////// SSO
impl<'a, Obs, V:'a, F, R> SubFHelper<V,F, Yes> for Obs
    where Obs : Observable<'a, V, Yes>, F: 'a+Send+Sync+FnMut(V)->R
{
    #[inline(always)]
    fn subf(&self, f: F)-> SubRef {
        unsafe{
            let next = FnCell::new(f);
            self.sub(Mss::new(( move |v| { (*next.0.get())(v); }, _empty, _comp, PhantomData)))
        }
    }
}

impl<'a, Obs, V:'a, FErr, RE> SubFHelper<V,((),FErr), Yes> for Obs
    where Obs : Observable<'a, V, Yes>,  FErr:'a+Send+Sync+FnMut(ArcErr)->RE,
{
    #[inline(always)]
    fn subf(&self, f: ((),FErr))-> SubRef
    {
        unsafe{
            let ferr = FnCell::new(f.1);
            self.sub(Mss::new(( _empty, move |e| { (*ferr.0.get())(e); }, _comp, PhantomData)))
        }
    }
}

impl<'a, Obs, V:'a, F, R,FErr,RE, FComp,RC> SubFHelper<V,(F,FErr,FComp), Yes> for Obs
    where Obs : Observable<'a, V, Yes>, F: 'a+Send+Sync+FnMut(V)->R, FErr: 'a+Send+Sync+FnMut(ArcErr)->RE, FComp: 'a+Send+Sync+FnMut()->RC,
{
    #[inline(always)]
    fn subf(&self, f: (F,FErr,FComp))-> SubRef
    {
        unsafe{
            let (next, err, comp) = (FnCell::new(f.0), FnCell::new(f.1), FnCell::new(f.2));
            self.sub(Mss::new(( move |v| { (*next.0.get())(v);}, move |e| { (*err.0.get())(e);}, move || { (*comp.0.get())();}, PhantomData)))
        }
    }
}

impl<'a, Obs, V:'a, FComp, RC> SubFHelper<V,((),(),FComp), Yes> for Obs
    where Obs : Observable<'a, V, Yes>,  FComp: Send+Sync+'a+FnMut()->RC
{
    #[inline(always)]
    fn subf(&self, f: ((),(),FComp))-> SubRef
    {
        unsafe{
            let comp = FnCell::new(f.2);
            self.sub(Mss::new((_empty, _empty, move || { (*comp.0.get())();}, PhantomData)))
        }
    }
}

impl<'a, Obs, V:'a, F,FErr, R, RE> SubFHelper<V,(F,FErr), Yes> for Obs
    where Obs : Observable<'a, V, Yes>, F:'a+Send+Sync+FnMut(V)->R, FErr:'a+Send+Sync+FnMut(ArcErr) -> RE,
{
    #[inline(always)]
    fn subf(&self, f: (F,FErr))-> SubRef
    {
        unsafe{
            let (next, err) = (FnCell::new(f.0), FnCell::new(f.1));
            self.sub(Mss::new(( move |v| { (*next.0.get())(v);}, move |e| { (*err.0.get())(e);}, _comp, PhantomData)))
        }
    }
}

impl<'a, Obs, V:'a, F,FComp, R, RC> SubFHelper<V,(F,(), FComp), Yes> for Obs
    where Obs : Observable<'a, V, Yes>, F:'a+Send+Sync+FnMut(V)->R, FComp:'a+Send+Sync+FnMut()->RC,
{
    #[inline(always)]
    fn subf(&self, f: (F,(), FComp))-> SubRef
    {
        unsafe{
            let (next,comp) = (FnCell::new(f.0), FnCell::new(f.2));
            self.sub(Mss::new(( move |v| { (*next.0.get())(v);}, _empty, move || {(*comp.0.get())();}, PhantomData)))
        }
    }
}

impl<'a, Obs, V:'a, FErr,FComp, RE, RC> SubFHelper<V,((),FErr, FComp), Yes> for Obs
    where Obs : Observable<'a, V, Yes>, FErr:'a+Send+Sync+FnMut(ArcErr)->RE, FComp:'a+Send+Sync+FnMut()->RC,
{
    #[inline(always)]
    fn subf(&self, f: ((),FErr,FComp))-> SubRef
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