use std::any::Any;
use std::rc::Rc;
use std::sync::{Once, ONCE_INIT};
use std::cell::RefCell;
use unsub_ref::*;
use std::sync::Arc;
use std::marker::PhantomData;
use std::boxed::FnBox;
use util::AtomicOption;

pub trait Observable<'a, V>
{
    fn sub(&self, dest: Arc<Observer<V>+Send+Sync+'a>) -> UnsubRef;
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
    fn subf(&self, next: F, ferr: FErr, fcomp: FComp) -> UnsubRef;
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
    fn subn(&self, next: F) -> UnsubRef;
}

pub struct ByrefOp<'a:'b, 'b, V, Src:'a> where Src : Observable<'a, V>+'a
{
    source: &'b Src,
    PhantomData: PhantomData<(V, &'a())>
}

//impl<'a, V, Src> Clone for ByrefOp<'a, V, Src>
//{
//    fn clone(&self) -> ByrefOp<'a, V, Src>
//    {
//        ByrefOp{source: self.source, PhantomData}
//    }
//}

pub trait ObservableByref<'a:'b, 'b, V, Src> where Src : Observable<'a, V>+'a
{
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
    #[inline] fn sub<'c>(&self, dest: Arc<Observer<V>+Send+Sync+'a>) -> UnsubRef
    {
        self.source.sub(dest)
    }
}

impl<V, F : Fn(V)> Observer<V> for F
{
    #[inline] fn next(&self, v:V) { self(v) }
}

impl<V, F: Fn(V), FErr: Fn(Arc<Any+Send+Sync>), FComp: Fn()> Observer<V> for (F, FErr, FComp)
{
    #[inline] fn next(&self, v:V) { self.0(v) }
    #[inline] fn err(&self, e:Arc<Any+Send+Sync>) { self.1(e) }
    #[inline] fn complete(&self) { self.2() }
}

impl<V, F: Fn(V), FErr: Fn(Arc<Any+Send+Sync>)> Observer<V> for (F, FErr)
{
    #[inline] fn next(&self, v:V) { self.0(v) }
    #[inline] fn err(&self, e:Arc<Any+Send+Sync>) { self.1(e) }
}

impl<V, F: Fn(V), FErr: Fn(Arc<Any+Send+Sync>)> Observer<V> for (F, FErr, ())
{
    #[inline] fn next(&self, v:V) { self.0(v) }
    #[inline] fn err(&self, e:Arc<Any+Send+Sync>) { self.1(e) }
}

impl<V, FErr: Fn(Arc<Any+Send+Sync>)> Observer<V> for ((), FErr)
{
    #[inline] fn err(&self, e:Arc<Any+Send+Sync>) { self.1(e) }
}

impl<V, FErr: Fn(Arc<Any+Send+Sync>)> Observer<V> for ((), FErr, ())
{
    #[inline] fn err(&self, e:Arc<Any+Send+Sync>) { self.1(e) }
}

impl<V, F: Fn(V), FComp: Fn()> Observer<V> for (F, (), FComp)
{
    #[inline] fn next(&self, v:V) { self.0(v) }
    #[inline] fn complete(&self) { self.2() }
}

impl<V, FErr: Fn(Arc<Any+Send+Sync>), FComp: Fn()> Observer<V> for ((), FErr, FComp)
{
    #[inline] fn err(&self, e:Arc<Any+Send+Sync>) { self.1(e) }
    #[inline] fn complete(&self) { self.2() }
}

impl<V, FComp: Fn()> Observer<V> for ((), (), FComp)
{
    #[inline] fn complete(&self) { self.2() }
}

impl<V, F: Fn(V)> Observer<V> for (F, (), ())
{
    #[inline] fn next(&self, v:V) { self.0(v) }
}

impl<'a, V, Obs, F: Fn(V)+'a+Send+Sync, FErr: Fn(Arc<Any+Send+Sync>)+'a+Send+Sync, FComp: Fn()+'a+Send+Sync> ObservableSubHelper<'a, V,F, FErr, FComp> for Obs where Obs : Observable<'a, V>
{
    #[inline] fn subf(&self, next: F, ferr: FErr, fcomp: FComp) -> UnsubRef { self.sub(Arc::new((next, ferr, fcomp))) }
}

impl<'a, V, Obs, F: Fn(V)+'a+Send+Sync> ObservableSubHelper<'a, V,F, (), ()> for Obs where Obs : Observable<'a,V>
{
    #[inline] fn subf(&self, next: F, ferr: (), fcomp: ()) -> UnsubRef { self.sub(Arc::new((next, (), ()))) }
}

impl<'a, V, Obs, F: Fn(V)+'a+Send+Sync, FErr: Fn(Arc<Any+Send+Sync>)+'a+Send+Sync> ObservableSubHelper<'a, V,F, FErr, ()> for Obs where Obs : Observable<'a,V>
{
    #[inline] fn subf(&self, next: F, ferr: FErr, fcomp: ()) -> UnsubRef { self.sub(Arc::new((next, ferr, ()))) }
}

impl<'a, V, Obs, F: Fn(V)+'a+Send+Sync, FComp: Fn()+'a+Send+Sync> ObservableSubHelper<'a, V,F, (), FComp> for Obs where Obs : Observable<'a,V>
{
    #[inline] fn subf(&self, next: F, ferr: (), fcomp: FComp) -> UnsubRef { self.sub(Arc::new((next, (), fcomp))) }
}

impl<'a, V, Obs, FErr: Fn(Arc<Any+Send+Sync>)+'a+Send+Sync> ObservableSubHelper<'a, V,(), FErr, ()> for Obs where Obs : Observable<'a,V>
{
    fn subf(&self, f: (), ferr: FErr, fcomp: ()) -> UnsubRef
    {
        self.sub(Arc::new(((), ferr, ())))
    }
}

impl<'a, V, Obs, FComp: Fn()+'a+Send+Sync> ObservableSubHelper<'a, V,(), (), FComp> for Obs where Obs : Observable<'a,V>
{
    #[inline] fn subf(&self, f: (), ferr: (), fcomp: FComp) -> UnsubRef { self.sub(Arc::new(((), (), fcomp))) }
}

impl<'a, V,F, Src> ObservableSubNextHelper<V,F> for Src where
    F: Fn(V)+'a+Send+Sync,
    Src : Observable<'a,V>
{
    #[inline] fn subn(&self, next: F) -> UnsubRef { self.sub(Arc::new(next)) }
}

impl<'a, V, Src> Observable<'a,V> for Arc<Src> where Src : Observable<'a,V>
{
    #[inline] fn sub(&self, dest: Arc<Observer<V>+Send+Sync+'a>) -> UnsubRef
    {
        Arc::as_ref(self).sub(dest)
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

pub trait ObservableSubScopedHelper<'a, Obs, V,F, FErr, FComp>
{
    fn sub_scopedf(&self, next: F, ferr: FErr, fcomp: FComp) -> UnsubRef;
}
pub trait ObservableSubScopedNextHelper<'a, Obs, V,F>
{
    fn sub_scoped(&self, next: F) -> UnsubRef;
}
impl<'a, Obs, V:'static, F, FErr, FComp> ObservableSubScopedHelper<'a, Obs, V,F, FErr, FComp> for Obs where
    Obs : Observable<'a, V>, F:FnMut(V)+'a, FErr:FnMut(Arc<Any+Send+Sync>)+'a, FComp:FnMut()+'a,
{
    fn sub_scopedf(&self, fnext: F, ferr: FErr, fcomp: FComp) -> UnsubRef
    {
        unsafe {
            use ::std::mem::transmute;

            let fnext : Box<FnMut(V) + 'a> = Box::new(fnext);
            let fnext: Box<Fn(V) + Send> = transmute(fnext);

            let ferr : Box<FnMut(Arc<Any+Send+Sync>)+'a> = Box::new(ferr);
            let ferr: Box<Fn(Arc<Any+Send+Sync>) + Send> = transmute(ferr);

            let fcomp : Box<FnMut()+'a> = Box::new(fcomp);
            let fcomp: Box<Fn() + Send> = transmute(fcomp);

            let o = Arc::new(ScopedObserver{
                fnext: Some(fnext), ferr: Some(ferr), fcomp: Some(fcomp),
                PhantomData
            });

            let sub = self.sub(o);
            let scoped = UnsubRef::scoped();
            scoped.add(sub);
            scoped
        }

    }
}
impl<'a, Obs, V:'static, F> ObservableSubScopedNextHelper<'a, Obs,V,F> for Obs where
    Obs : Observable<'a, V>, F:FnMut(V)+'a,
{
    fn sub_scoped(&self, fnext: F) -> UnsubRef
    {
        unsafe {
            use ::std::mem::transmute;

            let fnext : Box<FnMut(V) + 'a> = Box::new(fnext);
            let fnext: Box<Fn(V) + Send> = transmute(fnext);

            let o = Arc::new(ScopedObserver{
                fnext: Some(fnext), ferr: None, fcomp: None,
                PhantomData
            });

            let sub = self.sub(o);
            let scoped = UnsubRef::scoped();
            scoped.add(sub);

            scoped
        }

    }
}

pub struct ScopedObserver< V>
{
    fnext: Option<Box<Fn(V)+Send>>,
    ferr: Option<Box<Fn(Arc<Any+Send+Sync>)>>,
    fcomp: Option<Box<Fn()>>,
    PhantomData: PhantomData<V>
}
impl<V> Observer<V> for ScopedObserver<V>
{
    fn next(&self, v:V){ self.fnext.as_ref().map(|f| f(v)); }
    fn err(&self, e:Arc<Any+Send+Sync>) { self.ferr.as_ref().map(|f| f(e)); }
    fn complete(&self){ self.fcomp.as_ref().map(|f| f()); }

    fn _is_closed(&self) -> bool { false }
}
unsafe impl<V> Sync for ScopedObserver<V>{}
unsafe impl<V> Send for ScopedObserver<V>{}


#[cfg(test)]
mod test
{
    use super::*;
    use std::sync::Mutex;
    use std::marker::PhantomData;
    use std::sync::atomic::{Ordering, AtomicIsize};
    use op::*;
    use fac::*;
    use scheduler::NewThreadScheduler;

    #[test]
    fn scoped()
    {
        let s = 123;
        let a = StoresObserverObservable{ o: Mutex::new(None) };
        let o = Arc::new(ScopedObserver{ s: &s  });

        a.rx().take(1).sub(o.clone());
        a.sub(o);
    }

    #[test]
    fn scoped_mut()
    {
        let mut a = 0;

        //won't compile
        //rxfac::range(0..30).take(3).observe_on(NewThreadScheduler::get()).sub_scoped(|v| a+=v);

        rxfac::range(0..30).take(3).observe_on(NewThreadScheduler::get()).sub_scoped(|v| println!("{}",v));

        rxfac::range(0..30).take(3).sub_scoped(|v| a+=v);
        assert_eq!(a, 3);
    }

    #[test]
    fn threaded()
    {
        let x = Arc::new(AtomicIsize::new(0));

        let r = 0;
        let a = ThreadedObservable;
        a.subn(move |v| println!("{}", v + r + x.fetch_add(1, Ordering::SeqCst) as i32));
    }

    struct ThreadedObservable;
    impl Observable<'static, i32> for ThreadedObservable
    {
        fn sub(&self, o: Arc<Observer<i32>+Send+Sync+'static>) -> UnsubRef
        {
            ::std::thread::spawn(move ||{
                o.next(123);
            });
            UnsubRef::empty()
        }
    }

    struct StoresObserverObservable<'a>
    {
        o: Mutex<Option<Arc<Observer<i32>+Send+Sync+'a>>>
    }
    impl<'a> Observable<'a, i32> for StoresObserverObservable<'a>
    {
        fn sub(&self, o: Arc<Observer<i32> + Send + Sync + 'a>)-> UnsubRef
        {
            o.next(123);
            *self.o.lock().unwrap() = Some(o);
            UnsubRef::empty()
        }
    }


    struct ScopedObserver<'x>
    {
        s: &'x i32
    }
    impl<'x> Observer<i32> for ScopedObserver<'x>
    {
        fn next(&self, v:i32)
        {
            println!("{}", v + self.s);
        }
    }
}