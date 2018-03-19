use std::any::Any;
use std::rc::Rc;
use std::sync::{Once, ONCE_INIT};
use std::cell::RefCell;
use subref::*;
use std::sync::Arc;
use std::marker::PhantomData;
use std::boxed::FnBox;
use util::AtomicOption;
use std::any::TypeId;
use std::cell::UnsafeCell;
use std::cell::Cell;
use std::ops::Not;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::mem;

pub struct Yes;
pub struct No;

pub trait Observable<'a, V, SSO>
{
    fn sub(&self, o: impl Observer<V>+'a+Send+Sync) -> SubRef;
}

pub trait Observer<V>
{
    fn next(&self, v:V){}
    fn err(&self, e:Arc<Any+Send+Sync>){} //todo
    fn complete(&self){}

    fn _is_closed(&self) -> bool { false }
}

pub trait ObserverHelper<V>
{
    fn next(&self, v: V){}
    fn err(&self, e: Arc<Any+Send+Sync>){}
    fn complete(&self){}
    fn _is_closed(&self) -> bool{ false }
}

pub enum RxNoti<V>
{
    Next(V), Err(Arc<Any+Send+Sync>), Comp
}

pub trait ObservableSubNotiHelper<V, F, SSO>
{
    fn sub_noti(&self, f: F) -> SubRef;
}

pub trait ObservableSubNonSSOHelper<'a, V>
{
    fn sub_nss(&self, o: impl Observer<V>+'a) -> SubRef;
}
impl<'a,V:'a, O> ObservableSubNonSSOHelper<'a, V> for O where O : Observable<'a,V, No>
{
    fn sub_nss(&self, o: impl Observer<V>+'a) -> SubRef
    {
        self.sub_noti(move |n| {
            match n {
                RxNoti::Next(v) => o.next(v),
                RxNoti::Err(e) => o.err(e),
                RxNoti::Comp => o.complete()
            }
            return if o._is_closed() { IsClosed::True } else { IsClosed::Default };
        })
    }
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
        self.sub(MatchObserver::new(move |n| unsafe {
            (*f.0.get())(n)
        }))
    }
}

impl<'a, V:'a,F, Obs, Ret:AsIsClosed> ObservableSubNotiHelper<V, F, Yes> for Obs where Obs:Observable<'a,V, Yes>, F: Send+Sync+'a+FnMut(RxNoti<V>)->Ret
{
    #[inline(always)]
    fn sub_noti(&self, f: F) -> SubRef
    {
        let f = FnCell::new(f);
        self.sub(MatchObserver::new(move |n| unsafe {
            (*f.0.get())(n)
        }))
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
    fn err(&self, e:Arc<Any+Send+Sync>)
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


pub struct ByrefOp<'a:'b, 'b, V, Src, SSO> where Src : Observable<'a, V, SSO>+'a
{
    source: &'b Src,
    PhantomData: PhantomData<(V, &'a(), SSO)>
}


pub trait ObservableByref<'a:'b, 'b, V, Src,SSO> where Src : Observable<'a, V, SSO>+'a
{
    fn rx(&'b self) -> ByrefOp<'a, 'b, V, Src, SSO>;
}

impl<'a:'b,'b, V, Src,SSO> ObservableByref<'a, 'b, V, Src,SSO> for Src where Src : Observable<'a, V, SSO>+'a
{
    #[inline(always)]
    fn rx(&'b self) -> ByrefOp<'a, 'b,V, Src, SSO>
    {
        ByrefOp{ source: self, PhantomData }
    }
}

impl<'a:'b, 'b, V:'a, Src> Observable<'a,V,No> for ByrefOp<'a, 'b, V,Src,No> where Src: Observable<'a, V, No>+'a
{
    #[inline(always)]
    fn sub(&self, o: impl Observer<V>+'a) -> SubRef
    {
        self.source.sub_nss(o)
    }
}
impl<'a:'b, 'b, V, Src> Observable<'a,V,Yes> for ByrefOp<'a, 'b, V,Src,Yes> where Src: Observable<'a, V, Yes>+'a
{
    #[inline(always)]
    fn sub(&self, o: impl Observer<V>+'a+Send+Sync) -> SubRef
    {
        self.source.sub(o)
    }
}

impl<V, F: Fn(V), FErr: Fn(Arc<Any+Send+Sync>), FComp: Fn()> Observer<V> for (F, FErr, FComp, PhantomData<()>)
{
    #[inline] fn next(&self, v:V) { self.0(v) }
    #[inline] fn err(&self, e:Arc<Any+Send+Sync>) { self.1(e) }
    #[inline] fn complete(&self) { self.2() }
}


impl<V, F: Fn(V)> ObserverHelper<V> for F
{
    #[inline] fn next(&self, v:V) { self(v) }
}
impl<V, FErr: Fn(Arc<Any+Send+Sync>)> ObserverHelper<V> for ((), FErr)
{
    #[inline] fn err(&self, e:Arc<Any+Send+Sync>) { self.1(e) }
}
impl<V,FComp: Fn()> ObserverHelper<V> for ((), (), FComp)
{
    #[inline] fn complete(&self) { self.2() }
}
impl<V, F: Fn(V), FErr: Fn(Arc<Any+Send+Sync>),FComp: Fn()> ObserverHelper<V> for (F, FErr, FComp)
{
    #[inline] fn next(&self, v:V) { self.0(v) }
    #[inline] fn err(&self, e:Arc<Any+Send+Sync>) { self.1(e) }
    #[inline] fn complete(&self) { self.2() }
}
impl<V, F: Fn(V), FErr: Fn(Arc<Any+Send+Sync>)> ObserverHelper<V> for (F, FErr)
{
    #[inline] fn next(&self, v:V) { self.0(v) }
    #[inline] fn err(&self, e:Arc<Any+Send+Sync>) { self.1(e) }
}
impl<V, FErr: Fn(Arc<Any+Send+Sync>),FComp: Fn()> ObserverHelper<V> for ((), FErr, FComp)
{
    #[inline] fn err(&self, e:Arc<Any+Send+Sync>) { self.1(e) }
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
    fn sub(&self, o: impl Observer<V>+'a+Send+Sync) -> SubRef
    {
        Arc::as_ref(self).sub(o)
    }
}

impl<V> ObserverHelper<V> for Arc<Observer<V>>
{
    #[inline] fn next(&self, v: V){
        Arc::as_ref(self).next(v);
    }
    #[inline] fn err(&self, e: Arc<Any+Send+Sync>) {
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
    fn err(&self, e:Arc<Any+Send+Sync>){ Arc::as_ref(self).err(e); }
    fn complete(&self){ Arc::as_ref(self).complete(); }
    fn _is_closed(&self) -> bool { Arc::as_ref(self)._is_closed() }
}

pub trait SubFHelper<V,F, SSO>
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

fn _empty<V>(v:V){}
fn _comp(){}

impl<'a, Obs, V:'a, F> SubFHelper<V,F, No> for Obs
    where Obs : Observable<'a, V, No>, F: FnMut(V)+'a
{
    #[inline(always)]
    fn subf(&self, f: F)-> SubRef {
        unsafe{
            let next = FnCell::new(f);
            self.sub(( move |v| (*next.0.get())(v), _empty, _comp, PhantomData))
        }
    }
}

impl<'a, Obs, V:'a, FErr> SubFHelper<V,((),FErr), No> for Obs
    where Obs : Observable<'a, V, No>,  FErr:FnMut(Arc<Any+Send+Sync>)+'a,
{
    #[inline(always)]
    fn subf(&self, f: ((),FErr))-> SubRef
    {
        unsafe{
            let ferr = FnCell::new(f.1);
            self.sub(( _empty, move |e| (*ferr.0.get())(e), _comp, PhantomData))
        }
    }
}

impl<'a, Obs, V:'a, F,FErr, FComp> SubFHelper<V,(F,FErr,FComp), No> for Obs
    where Obs : Observable<'a, V, No>, F: FnMut(V)+'a, FErr:FnMut(Arc<Any+Send+Sync>)+'a, FComp: FnMut()+'a,
{
    #[inline(always)]
    fn subf(&self, f: (F,FErr,FComp))-> SubRef
    {
        unsafe{
            let (next, err, comp) = (FnCell::new(f.0), FnCell::new(f.1), FnCell::new(f.2));
            self.sub(( move |v| (*next.0.get())(v), move |e| (*err.0.get())(e), move || (*comp.0.get())(), PhantomData))
        }
    }
}

impl<'a, Obs, V:'a, FComp> SubFHelper<V,((),(),FComp), No> for Obs
    where Obs : Observable<'a, V, No>,  FComp: FnMut()+'a,
{
    #[inline(always)]
    fn subf(&self, f: ((),(),FComp))-> SubRef
    {
        unsafe{
            let comp = FnCell::new(f.2);
            self.sub((_empty, _empty, move || (*comp.0.get())(), PhantomData))
        }
    }
}

impl<'a, Obs, V:'a, F,FErr> SubFHelper<V,(F,FErr), No> for Obs
    where Obs : Observable<'a, V, No>, F:FnMut(V)+'a, FErr:FnMut(Arc<Any+Send+Sync>)+'a,
{
    #[inline(always)]
    fn subf(&self, f: (F,FErr))-> SubRef
    {
        unsafe{
            let (next, err) = (FnCell::new(f.0), FnCell::new(f.1));
            self.sub(( move |v| (*next.0.get())(v), move |e| (*err.0.get())(e), _comp, PhantomData))
        }
    }
}

impl<'a, Obs, V:'a, F,FComp> SubFHelper<V,(F,(), FComp), No> for Obs
    where Obs : Observable<'a, V, No>, F:FnMut(V)+'a, FComp:FnMut()+'a,
{
    #[inline(always)]
    fn subf(&self, f: (F,(), FComp))-> SubRef
    {
        unsafe{
            let (next,comp) = (FnCell::new(f.0), FnCell::new(f.2));
            self.sub(( move |v| (*next.0.get())(v), _empty, move || (*comp.0.get())(), PhantomData))
        }
    }
}

impl<'a, Obs, V:'a, FErr,FComp> SubFHelper<V,((),FErr, FComp), No> for Obs
    where Obs : Observable<'a, V, No>, FErr:FnMut(Arc<Any+Send+Sync>)+'a, FComp:FnMut()+'a,
{
    #[inline(always)]
    fn subf(&self, f: ((),FErr,FComp))-> SubRef
    {
        unsafe {
            let ( err, comp) = (FnCell::new(f.1), FnCell::new(f.2));
            self.sub(( _empty, move |e| (*err.0.get())(e), move || (*comp.0.get())(), PhantomData))
        } }
}

/////// SSO
impl<'a, Obs, V:'a, F> SubFHelper<V,F, Yes> for Obs
    where Obs : Observable<'a, V, Yes>, F: FnMut(V)+'a+Send+Sync
{
    #[inline(always)]
    fn subf(&self, f: F)-> SubRef {
        unsafe{
            let next = FnCell::new(f);
            self.sub(( move |v| (*next.0.get())(v), _empty, _comp, PhantomData))
        }
    }
}

impl<'a, Obs, V:'a, FErr> SubFHelper<V,((),FErr), Yes> for Obs
    where Obs : Observable<'a, V, Yes>,  FErr:FnMut(Arc<Any+Send+Sync>)+'a+Send+Sync,
{
    #[inline(always)]
    fn subf(&self, f: ((),FErr))-> SubRef
    {
        unsafe{
            let ferr = FnCell::new(f.1);
            self.sub(( _empty, move |e| (*ferr.0.get())(e), _comp, PhantomData))
        }
    }
}

impl<'a, Obs, V:'a, F,FErr, FComp> SubFHelper<V,(F,FErr,FComp), Yes> for Obs
    where Obs : Observable<'a, V, Yes>, F: FnMut(V)+'a+Send+Sync, FErr:FnMut(Arc<Any+Send+Sync>)+'a+Send+Sync, FComp: FnMut()+Send+Sync+'a,
{
    #[inline(always)]
    fn subf(&self, f: (F,FErr,FComp))-> SubRef
    {
        unsafe{
            let (next, err, comp) = (FnCell::new(f.0), FnCell::new(f.1), FnCell::new(f.2));
            self.sub(( move |v| (*next.0.get())(v), move |e| (*err.0.get())(e), move || (*comp.0.get())(), PhantomData))
        }
    }
}

impl<'a, Obs, V:'a, FComp> SubFHelper<V,((),(),FComp), Yes> for Obs
    where Obs : Observable<'a, V, Yes>,  FComp: FnMut()+Send+Sync+'a,
{
    #[inline(always)]
    fn subf(&self, f: ((),(),FComp))-> SubRef
    {
        unsafe{
            let comp = FnCell::new(f.2);
            self.sub((_empty, _empty, move || (*comp.0.get())(), PhantomData))
        }
    }
}

impl<'a, Obs, V:'a, F,FErr> SubFHelper<V,(F,FErr), Yes> for Obs
    where Obs : Observable<'a, V, Yes>, F:FnMut(V)+'a+Send+Sync, FErr:FnMut(Arc<Any+Send+Sync>)+'a+Send+Sync,
{
    #[inline(always)]
    fn subf(&self, f: (F,FErr))-> SubRef
    {
        unsafe{
            let (next, err) = (FnCell::new(f.0), FnCell::new(f.1));
            self.sub(( move |v| (*next.0.get())(v), move |e| (*err.0.get())(e), _comp, PhantomData))
        }
    }
}

impl<'a, Obs, V:'a, F,FComp> SubFHelper<V,(F,(), FComp), Yes> for Obs
    where Obs : Observable<'a, V, Yes>, F:FnMut(V)+'a+Send+Sync, FComp:FnMut()+'a+Send+Sync,
{
    #[inline(always)]
    fn subf(&self, f: (F,(), FComp))-> SubRef
    {
        unsafe{
            let (next,comp) = (FnCell::new(f.0), FnCell::new(f.2));
            self.sub(( move |v| (*next.0.get())(v), _empty, move || (*comp.0.get())(), PhantomData))
        }
    }
}

impl<'a, Obs, V:'a, FErr,FComp> SubFHelper<V,((),FErr, FComp), Yes> for Obs
    where Obs : Observable<'a, V, Yes>, FErr:FnMut(Arc<Any+Send+Sync>)+'a+Send+Sync, FComp:FnMut()+'a+Send+Sync,
{
    #[inline(always)]
    fn subf(&self, f: ((),FErr,FComp))-> SubRef
    {
        unsafe {
            let ( err, comp) = (FnCell::new(f.1), FnCell::new(f.2));
            self.sub(( _empty, move |e| (*err.0.get())(e), move || (*comp.0.get())(), PhantomData))
        } }
}

#[cfg(test)]
mod test
{
    use super::*;
    use test_fixture::*;

    #[test]
    fn hello_world()
    {
        let s = SimpleObservable;
        let o = SimpleObserver;
        s.sub(o);

        let i = 123;
        let ts = ThreadedObservable;
        let o = LocalObserver(&i);
        //should not compile
        //ts.sub(o);

        ts.sub_noti(|n| { println!("noti"); });

        //should not compile
        //ts.sub_noti(|n| { println!("noti: {}", i); });

        let nso = NonSendObserver(Rc::new(123));
        s.sub_nss(nso);

        //should not compile
        //ts.subf(|v| println!("ok {}", i));
    }
}