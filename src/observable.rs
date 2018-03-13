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

pub trait Observable<'a, V>
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

pub trait ObservableSubHelper<'a, V, F, FErr,FComp>
{
    fn subf(&self, next: F, ferr: FErr, fcomp: FComp) -> SubRef;
}

pub trait ObserverHelper<V>
{
    fn next(&self, v: V) -> &Self;
    fn err(&self, e: Arc<Any+Send+Sync>);
    fn complete(&self);
    fn _is_closed(&self) -> bool;
}

pub trait ObservableSubNextHelper<V, F>
{
    fn subn(&self, next: F) -> SubRef;
}

pub struct ByrefOp<'a:'b, 'b, V, Src> where Src : Observable<'a, V>+'a
{
    source: &'b Src,
    PhantomData: PhantomData<(V, &'a())>
}

pub trait ObservableByref<'a:'b, 'b, V, Src> where Src : Observable<'a, V>+'a
{
    //todo: keep only one
    fn byref(&'b self) -> ByrefOp<'a, 'b,V, Src>;
    fn rx(&'b self) -> ByrefOp<'a, 'b, V, Src>{ self.byref() }
}

impl<'a:'b,'b, V, Src> ObservableByref<'a, 'b, V, Src> for Src where Src : Observable<'a, V>+'a
{
    fn byref(&'b self) -> ByrefOp<'a, 'b,V, Src>
    {
        ByrefOp{ source: self, PhantomData }
    }
}

impl<'a:'b, 'b, V, Src> Observable<'a,V> for ByrefOp<'a, 'b, V,Src> where Src: Observable<'a, V>+'a
{
    #[inline] fn sub(&self, o: impl Observer<V>+'a+Send+Sync) -> SubRef
    {
        self.source.sub(o)
    }
}

impl<V, F: Fn(V), FErr: Fn(Arc<Any+Send+Sync>), FComp: Fn()> Observer<V> for (F, FErr, FComp, ())
{
    #[inline] fn next(&self, v:V) { self.0(v) }
    #[inline] fn err(&self, e:Arc<Any+Send+Sync>) { self.1(e) }
    #[inline] fn complete(&self) { self.2() }
}

impl<'a, V, Src> Observable<'a,V> for Arc<Src> where Src : Observable<'a,V>
{
    #[inline] fn sub(&self, o: impl Observer<V>+'a+Send+Sync) -> SubRef
    {
        Arc::as_ref(self).sub(o)
    }
}

impl<V> ObserverHelper<V> for Arc<Observer<V>>
{
    #[inline] fn next(&self, v: V) -> &Self {
        Arc::as_ref(self).next(v);
        self
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

pub trait ObservableSubScopedHelper<V,F>
{
    fn subf(&self, f: F) -> SubRef;
}

fn sub_helper<'a,V, Src: Observable<'a, V>>(
    observable: &Src,
    fnext: impl FnMut(V)+'a+Send+Sync,
    ferr: impl FnMut(Arc<Any+Send+Sync>)+'a+Send+Sync,
    fcomp: impl FnMut()+'a+Send+Sync) -> SubRef
{
    unsafe{
        let (next, err, comp) = (FnCell::new(fnext), FnCell::new(ferr), FnCell::new(fcomp));
        observable.sub(( move |v| (*next.0.get())(v), move |e| (*err.0.get())(e), move || (*comp.0.get())(), ()))
    }
}

struct FnCell<F>(UnsafeCell<F>);
unsafe impl<F> Send for FnCell<F> {}
unsafe impl<F> Sync for FnCell<F> {}
impl<F> FnCell<F> { fn new(f:F) -> FnCell<F> { FnCell(UnsafeCell::new(f)) } }

fn _empty<V>(v:V){}
fn _comp(){}

impl<'a, Obs, V:'a, F> ObservableSubScopedHelper<V,F> for Obs
    where Obs : Observable<'a, V>, F: FnMut(V)+'a+Send+Sync
{
    fn subf(&self, f: F)-> SubRef { sub_helper(self, f, |e|{}, ||{}) }
}

impl<'a, Obs, V:'a, FErr> ObservableSubScopedHelper<V,((),FErr)> for Obs
    where Obs : Observable<'a, V>,  FErr:FnMut(Arc<Any+Send+Sync>)+'a+Send+Sync,
{
    fn subf(&self, f: ((),FErr))-> SubRef { sub_helper(self, _empty, f.1, _comp) }
}

impl<'a, Obs, V:'a, F,FErr, FComp> ObservableSubScopedHelper<V,(F,FErr,FComp)> for Obs
    where Obs : Observable<'a, V>, F: FnMut(V)+'a+Send+Sync, FErr:FnMut(Arc<Any+Send+Sync>)+'a+Send+Sync, FComp: FnMut()+Send+Sync+'a,
{
    fn subf(&self, f: (F,FErr,FComp))-> SubRef { sub_helper(self, _empty, _empty, f.2) }
}

impl<'a, Obs, V:'a, F,FErr> ObservableSubScopedHelper<V,(F,FErr)> for Obs
    where Obs : Observable<'a, V>, F:FnMut(V)+'a+Send+Sync, FErr:FnMut(Arc<Any+Send+Sync>)+'a+Send+Sync,
{
    fn subf(&self, f: (F,FErr))-> SubRef { sub_helper(self, f.0, f.1, _comp) }
}

impl<'a, Obs, V:'a, F,FComp> ObservableSubScopedHelper<V,(F,(), FComp)> for Obs
    where Obs : Observable<'a, V>, F:FnMut(V)+'a+Send+Sync, FComp:FnMut()+'a+Send+Sync,
{
    fn subf(&self, f: (F,(), FComp))-> SubRef { sub_helper(self, f.0, _empty, f.2) }
}

impl<'a, Obs, V:'a, FErr,FComp> ObservableSubScopedHelper<V,((),FErr, FComp)> for Obs
    where Obs : Observable<'a, V>, FErr:FnMut(Arc<Any+Send+Sync>)+'a+Send+Sync, FComp:FnMut()+'a+Send+Sync,
{
    fn subf(&self, f: ((),FErr,FComp))-> SubRef { sub_helper(self, _empty, f.1, f.2) }
}

//
//#[cfg(test)]
//mod test
//{
//    use super::*;
//    use std::sync::Mutex;
//    use std::marker::PhantomData;
//    use std::sync::atomic::{Ordering, AtomicIsize};
//    use op::*;
//    use fac::*;
//    use scheduler::NewThreadScheduler;
//
//    #[test]
//    fn scoped()
//    {
//
//    }
//
//    #[test]
//    fn scoped_mut()
//    {
//        let mut a = (0,0);
//
//        {
//            let src = rxfac::range(0..30);
//            //won't compile
//            //rxfac::range(0..30).take(3).observe_on(NewThreadScheduler::get()).sub_scoped(|v| a+=v);
//
//            //src.rx().take(3).observe_on(NewThreadScheduler::get()).sub_scoped(|v| a.0+=1);
//
//            {
//                src.rx().take(3).sub_scoped(|v| a.1 += v);
//            }
//        }
//
//        //rxfac::range(0..30).take(3).sub_scoped((|v| a+=v, |e|println!("err"), || println!("comp")));
//        assert_eq!(a.1, 3);
//    }
//
//}