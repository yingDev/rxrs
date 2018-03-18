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
use std::ops::Deref;
use std::ops::CoerceUnsized;
use std::marker::Unsize;
use util::traicks::*;
use util::sarc::*;


pub trait Observable<'a>
{
    type V;
    type SSO: YesNo;

    fn sub(&self, o: Sarc<Self::SSO, Observer<Self::V>+'a>) -> SubRef;
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

pub trait ObservableSubNotiHelper<V, F>
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

impl<'a:'b, 'b, V:'a,F, Ret:AsIsClosed, S: YesNo> ObservableSubNotiHelper<V, F> for Sarc<S, Observable<'a,V=V,SSO=S>+'b> where F: 'a+FnMut(RxNoti<V>)->Ret
{
    fn sub_noti(&self, f: F) -> SubRef
    {
        let f = FnCell::new(f);
        self.sub(Sarc::new(MatchObserver::new(move |n| unsafe { (*f.0.get())(n) })))
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
impl<'a, V,F,Ret:AsIsClosed> Observer<V> for MatchObserver<V,F> where F: 'a+Fn(RxNoti<V>)->Ret
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

//
//pub trait AsRx<'a:'b, 'b, V>
//{
//    fn rx(self) -> Arc<Observable<'a, V>+'b+Send+Sync>;
//}
//
//pub trait CloneRx<'a:'b, 'b, V>
//{
//    fn rx(&self) -> Arc<Observable<'a, V>+'b+Send+Sync>;
//
//}
//
//impl<'a:'b, 'b, V, Src> AsRx<'a,'b,V> for Src where Src : Observable<'a, V>+'b+Send+Sync
//{
//    fn rx(self) -> Arc<Observable<'a, V>+'b+Send+Sync>
//    {
//        Arc::new(self)
//    }
//}
//
//impl<'a:'b, 'b, V, Src> CloneRx<'a,'b,V> for Arc<Src> where Src : Observable<'a, V>+'b+Send+Sync
//{
//    fn rx(&self) -> Arc<Observable<'a, V>+'b+Send+Sync>
//    {
//        let clone : Arc<Observable<'a, V>+'b+Send+Sync> = self.clone();
//        clone
//    }
//}

impl<V, F: Fn(V), FErr: Fn(Arc<Any+Send+Sync>), FComp: Fn()> Observer<V> for (F, FErr, FComp, PhantomData<()>)
{
    #[inline] fn next(&self, v:V) { self.0(v) }
    #[inline] fn err(&self, e:Arc<Any+Send+Sync>) { self.1(e) }
    #[inline] fn complete(&self) { self.2() }
}

//
//impl<V, F: Fn(V)> ObserverHelper<V> for F
//{
//    #[inline] fn next(&self, v:V) { self(v) }
//}
//impl<V, FErr: Fn(Arc<Any+Send+Sync>)> ObserverHelper<V> for ((), FErr)
//{
//    #[inline] fn err(&self, e:Arc<Any+Send+Sync>) { self.1(e) }
//}
//impl<V,FComp: Fn()> ObserverHelper<V> for ((), (), FComp)
//{
//    #[inline] fn complete(&self) { self.2() }
//}
//impl<V, F: Fn(V), FErr: Fn(Arc<Any+Send+Sync>),FComp: Fn()> ObserverHelper<V> for (F, FErr, FComp)
//{
//    #[inline] fn next(&self, v:V) { self.0(v) }
//    #[inline] fn err(&self, e:Arc<Any+Send+Sync>) { self.1(e) }
//    #[inline] fn complete(&self) { self.2() }
//}
//impl<V, F: Fn(V), FErr: Fn(Arc<Any+Send+Sync>)> ObserverHelper<V> for (F, FErr)
//{
//    #[inline] fn next(&self, v:V) { self.0(v) }
//    #[inline] fn err(&self, e:Arc<Any+Send+Sync>) { self.1(e) }
//}
//impl<V, FErr: Fn(Arc<Any+Send+Sync>),FComp: Fn()> ObserverHelper<V> for ((), FErr, FComp)
//{
//    #[inline] fn err(&self, e:Arc<Any+Send+Sync>) { self.1(e) }
//    #[inline] fn complete(&self) { self.2() }
//}
//impl<V,F: Fn(V),FComp: Fn()> ObserverHelper<V> for (F, (), FComp)
//{
//    #[inline] fn next(&self, v:V) { self.0(v) }
//    #[inline] fn complete(&self) { self.2() }
//}


//impl<'a, V, Src> Observable<'a,V> for Arc<Src> where Src : Observable<'a,V>
//{
//    #[inline(always)]
//    fn sub(&self, o: Arc<Observer<V>+'a+Send+Sync>) -> SubRef
//    {
//        Arc::as_ref(self).sub(o)
//    }
//}

//impl<V> ObserverHelper<V> for Arc<Observer<V>>
//{
//    #[inline] fn next(&self, v: V){
//        Arc::as_ref(self).next(v);
//    }
//    #[inline] fn err(&self, e: Arc<Any+Send+Sync>) {
//        Arc::as_ref(self).err(e);
//    }
//    #[inline] fn complete(&self) {
//        Arc::as_ref(self).complete();
//
//    }
//    #[inline] fn _is_closed(&self) -> bool {
//        Arc::as_ref(self)._is_closed()
//    }
//}

impl<V, O> Observer<V> for Arc<O> where O : Observer<V>
{
    fn next(&self, v:V){ Arc::as_ref(self).next(v); }
    fn err(&self, e:Arc<Any+Send+Sync>){ Arc::as_ref(self).err(e); }
    fn complete(&self){ Arc::as_ref(self).complete(); }
    fn _is_closed(&self) -> bool { Arc::as_ref(self)._is_closed() }
}

pub trait SubFHelper<V,F>
{
    fn subf(&self, f: F) -> SubRef;
}

pub struct FnCell<F>(pub UnsafeCell<F>);
unsafe impl<F> Send for FnCell<F> where F: Send{}
unsafe impl<F> Sync for FnCell<F> where F: Sync{}
impl<F> FnCell<F>
{
    pub fn new(f:F) -> FnCell<F> { FnCell(UnsafeCell::new(f)) }
}

fn _empty<V>(v:V){}
fn _comp(){}


impl<'a:'b, 'b, V:'a, F, S:YesNo, Src> SubFHelper<V,F> for Sarc<S, Src> where F: FnMut(V)+'a, Src: Observable<'a,V=V,SSO=S>+'b
{
    fn subf(&self, f: F)-> SubRef {
        unsafe{
            let next = FnCell::new(f);
            self.sub(Sarc::new( (move |v| (*next.0.get())(v), _empty, _comp, PhantomData) ))
        }
    }
}


//impl<'a, V:'a, FErr> SubFHelper<V,((),FErr)> for Arc<Observable<O=ObserverArc<'a,V>>+Send+Sync> where FErr:FnMut(Arc<Any+Send+Sync>)+'a+Send+Sync,
//{
//    #[inline(always)]
//    fn subf(&self, f: ((),FErr))-> SubRef
//    {
//        unsafe{
//            let ferr = FnCell::new(f.1);
//            self.sub(Arc::new(( _empty, move |e| (*ferr.0.get())(e), _comp, PhantomData)))
//        }
//    }
//}
//
//impl<'a, V:'a, F,FErr, FComp> SubFHelper<V,(F,FErr,FComp)> for Arc<Observable<'a, V>+Send+Sync> where F: FnMut(V)+'a+Send+Sync, FErr:FnMut(Arc<Any+Send+Sync>)+'a+Send+Sync, FComp: FnMut()+Send+Sync+'a,
//{
//    #[inline(always)]
//    fn subf(&self, f: (F,FErr,FComp))-> SubRef
//    {
//        unsafe{
//            let (next, err, comp) = (FnCell::new(f.0), FnCell::new(f.1), FnCell::new(f.2));
//            self.sub(Arc::new(( move |v| (*next.0.get())(v), move |e| (*err.0.get())(e), move || (*comp.0.get())(), PhantomData)))
//        }
//    }
//}
//
//impl<'a, V:'a, FComp> SubFHelper<V,((),(),FComp)> for Arc<Observable<'a, V>+Send+Sync> where  FComp: FnMut()+Send+Sync+'a,
//{
//    #[inline(always)]
//    fn subf(&self, f: ((),(),FComp))-> SubRef
//    {
//        unsafe{
//            let comp = FnCell::new(f.2);
//            self.sub(Arc::new((_empty, _empty, move || (*comp.0.get())(), PhantomData)))
//        }
//    }
//}
//
//impl<'a, V:'a, F,FErr> SubFHelper<V,(F,FErr)> for Arc<Observable<'a, V>+Send+Sync> where F:FnMut(V)+'a+Send+Sync, FErr:FnMut(Arc<Any+Send+Sync>)+'a+Send+Sync,
//{
//    #[inline(always)]
//    fn subf(&self, f: (F,FErr))-> SubRef
//    {
//        unsafe{
//            let (next, err) = (FnCell::new(f.0), FnCell::new(f.1));
//            self.sub(Arc::new(( move |v| (*next.0.get())(v), move |e| (*err.0.get())(e), _comp, PhantomData)))
//        }
//    }
//}
//
//impl<'a, V:'a, F,FComp> SubFHelper<V,(F,(), FComp)> for Arc<Observable<'a, V>+Send+Sync> where F:FnMut(V)+'a+Send+Sync, FComp:FnMut()+'a+Send+Sync,
//{
//    #[inline(always)]
//    fn subf(&self, f: (F,(), FComp))-> SubRef
//    {
//        unsafe{
//            let (next,comp) = (FnCell::new(f.0), FnCell::new(f.2));
//            self.sub(Arc::new(( move |v| (*next.0.get())(v), _empty, move || (*comp.0.get())(), PhantomData)))
//        }
//    }
//}
//
//impl<'a, V:'a, FErr,FComp> SubFHelper<V,((),FErr, FComp)> for Arc<Observable<'a, V>+Send+Sync> where FErr:FnMut(Arc<Any+Send+Sync>)+'a+Send+Sync, FComp:FnMut()+'a+Send+Sync,
//{
//    #[inline(always)]
//    fn subf(&self, f: ((),FErr,FComp))-> SubRef
//    {
//        unsafe {
//            let ( err, comp) = (FnCell::new(f.1), FnCell::new(f.2));
//            self.sub(Arc::new(( _empty, move |e| (*err.0.get())(e), move || (*comp.0.get())(), PhantomData)))
//        } }
//}

//
#[cfg(test)]
mod test
{
    use super::*;
    use std::sync::Mutex;
    use std::marker::PhantomData;
    use std::sync::atomic::{Ordering, AtomicIsize};
    use std::time::Duration;
    use std::ops::CoerceUnsized;
    use std::marker::Unsize;

    struct Src<'a>(PhantomData<&'a()>);
    impl<'a> Src<'a>
    {
        fn sarc() -> Sarc<No, Src<'a>> { Sarc::new(Src(PhantomData)) }
    }

    impl<'a> Observable<'a> for Src<'a>
    {
        type V = i32;
        type SSO = No;

        fn sub(&self, o: Sarc<Self::SSO, Observer<Self::V>+'a>) -> SubRef
        {
            o.next(1);
            o.next(2);
            o.next(3);
            o.complete();

            SubRef::empty()
        }
    }

    struct ThreadedSrc;
    impl ThreadedSrc
    {
        fn sarc() -> Sarc<Yes, ThreadedSrc> { Sarc::new(ThreadedSrc) }
    }
    impl Observable<'static> for ThreadedSrc
    {
        type V = i32;
        type SSO=Yes;

        fn sub(&self, o: Sarc<Self::SSO, Observer<Self::V>+'static>) -> SubRef
        {
            let u = SubRef::signal();
            let u2 = u.clone();

            ::std::thread::spawn(move ||{
                for i in 1..4 {
                    if u2.disposed() { break; }
                    o.next(i);
                }
                o.complete();
            });

            u
        }
    }

    struct StaticDest;
    impl Observer<i32> for StaticDest
    {
        fn next(&self, v: i32){ println!("v={}", v); }
        fn complete(&self){ println!("comoplete"); }
    }

    struct ScopedDest<'a>(&'a i32);
    impl<'a> Observer<i32> for ScopedDest<'a>
    {
        fn next(&self, v: i32){ println!("v={}", v); }
        fn complete(&self){ println!("comoplete"); }
    }

    #[test]
    fn should_complile()
    {
        {
            let src = Src(PhantomData);
            let o = Sarc::new(StaticDest);// Sarc::new(Observer<_>, StaticDest);
            src.sub(o);
        }

        {
            let src = ThreadedSrc;
            let o = Sarc::new(StaticDest); //Sarc::new(Observer<_>,StaticDest);
            src.sub(o);
        }

        {
            let i = 123;

            let src = Src(PhantomData);
            let o = Sarc::new(ScopedDest(&i));
            src.sub(o);
        }

        {
            let src = Src::sarc(); //Sarc::new(Observable<_,_>, Src(PhantomData), Yes);
            let o = Sarc::new(StaticDest);
            src.sub(o);
        }


        //should not compile
        {
//        let i = 123;
//
//        let src = ThreadedSrc;
//        let o = Arc::new(ScopedDest(&i));
//        src.sub(o);
        }

    }

    #[test]
    fn sub_tuple()
    {
        let src = Src::sarc();
        src.sub(Sarc::new((|v|{}, |e|{}, ||{}, PhantomData)));
    }

    #[test]
    fn sub_f()
    {
        let mut out = 0;
        {
            let src  = Src::sarc();
            src.subf(|v| out += v);
        }

        assert_eq!(out, 6);

        let out = Arc::new(AtomicIsize::new(0));
        {
            let out = out.clone();
            let src  = ThreadedSrc::sarc();
            src.subf(move |v| { out.fetch_add(v as isize, Ordering::SeqCst); });

            //expect compile fail: "closure may outlive the current function, but it borrows `out`, which is owned by the current function"
            //src.subf(|v| { out.fetch_add(v as isize, Ordering::SeqCst); });

            ::std::thread::sleep(Duration::from_millis(500));
        }

        assert_eq!(out.load(Ordering::SeqCst), 6);
    }

}