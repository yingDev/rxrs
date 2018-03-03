use std::any::Any;
use std::rc::Rc;
use std::sync::{Once, ONCE_INIT};
use std::cell::RefCell;
use unsub_ref::*;
use std::sync::Arc;
use std::marker::PhantomData;
use std::boxed::FnBox;
use util::AtomicOption;

pub trait Observable< V>
{
    fn sub(&self, dest: Arc<Observer<V>+Send+Sync>) -> UnsubRef<'static>;
}

pub trait Observer<V>
{
    fn next(&self, v:V){}
    fn err(&self, e:Arc<Any+Send+Sync>){} //todo
    fn complete(&self){}

    fn _is_closed(&self) -> bool { false }
}

pub trait ObservableSubHelper<V, F, FErr,FComp>
{
    fn subf(&self, next: F, ferr: FErr, fcomp: FComp) -> UnsubRef<'static>;
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
    fn subn(&self, next: F) -> UnsubRef<'static>;
}

pub trait ObservableSubScopedHelper<'a, Obs, V,F, FErr, FComp>
{
    fn sub_scopedf(self, next: F, ferr: FErr, fcomp: FComp) -> UnsubRef<'a>;
}
pub trait ObservableSubScopedNextHelper<'a, Obs, V,F>
{
    fn sub_scoped(self, next: F) -> UnsubRef<'a>;
}
impl<'a, Obs, V:'static, F, FErr, FComp> ObservableSubScopedHelper<'a, Obs, V,F, FErr, FComp> for Obs where
    Obs : Observable<V>, F:FnMut(V)+'a, FErr:FnMut(Arc<Any+Send+Sync>)+'a, FComp:FnMut()+'a,
{
    fn sub_scopedf(self, fnext: F, ferr: FErr, fcomp: FComp) -> UnsubRef<'a>
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

            transmute(scoped)
        }

    }
}
impl<'a, Obs, V:'static, F> ObservableSubScopedNextHelper<'a, Obs,V,F> for Obs where
    Obs : Observable<V>, F:FnMut(V)+'a,
{
    fn sub_scoped(self, fnext: F) -> UnsubRef<'a>
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

            transmute(scoped)
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

pub struct ByrefOp<'a, V: 'static>
{
    source: &'a Observable<V>,
}

pub trait ObservableByref<'a, V, Src>
{
    fn byref(&'a self) -> ByrefOp<'a, V>;
    fn rx(&'a self) -> ByrefOp<'a, V>{ self.byref() }
}

impl<'a, V, Src> ObservableByref<'a, V, Src> for Src where Src : Observable<V>
{
    fn byref(&'a self) -> ByrefOp<'a, V>
    {
        ByrefOp{ source: self }
    }
}

//impl<'b, V, Src> ObservableByref<'static, 'b, V, Src> for Arc<Src> where Src : Observable<V>+Send+Sync+'static
//{
//    fn byref(&'b self) -> ByrefOp<'static, V>
//    {
//        let clone = self.clone();
//        ByrefOp{ arc: Some(clone), source: None, PhantomData }
//    }
//}

impl<'a, V> Observable<  V> for ByrefOp<'a,  V>
{
    #[inline] fn sub(&self, dest: Arc<Observer<V>+Send+Sync>) -> UnsubRef<'static>
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


impl<V, Obs, F: Fn(V)+'static+Send+Sync, FErr: Fn(Arc<Any+Send+Sync>)+'static+Send+Sync, FComp: Fn()+'static+Send+Sync> ObservableSubHelper<V,F, FErr, FComp> for Obs where Obs : Observable<V>
{
    #[inline] fn subf(&self, next: F, ferr: FErr, fcomp: FComp) -> UnsubRef<'static> { self.sub(Arc::new((next, ferr, fcomp))) }
}

impl<V, Obs, F: Fn(V)+'static+Send+Sync> ObservableSubHelper<V,F, (), ()> for Obs where Obs : Observable<V>
{
    #[inline] fn subf(&self, next: F, ferr: (), fcomp: ()) -> UnsubRef<'static> { self.sub(Arc::new((next, (), ()))) }
}

impl<V, Obs, F: Fn(V)+'static+Send+Sync, FErr: Fn(Arc<Any+Send+Sync>)+'static+Send+Sync> ObservableSubHelper<V,F, FErr, ()> for Obs where Obs : Observable<V>
{
    #[inline] fn subf(&self, next: F, ferr: FErr, fcomp: ()) -> UnsubRef<'static> { self.sub(Arc::new((next, ferr, ()))) }
}

impl<V, Obs, F: Fn(V)+'static+Send+Sync, FComp: Fn()+'static+Send+Sync> ObservableSubHelper<V,F, (), FComp> for Obs where Obs : Observable<V>
{
    #[inline] fn subf(&self, next: F, ferr: (), fcomp: FComp) -> UnsubRef<'static> { self.sub(Arc::new((next, (), fcomp))) }
}

impl<V, Obs, FErr: Fn(Arc<Any+Send+Sync>)+'static+Send+Sync> ObservableSubHelper<V,(), FErr, ()> for Obs where Obs : Observable<V>
{
    fn subf(&self, f: (), ferr: FErr, fcomp: ()) -> UnsubRef<'static>
    {
        self.sub(Arc::new(((), ferr, ())))
    }
}

impl<V, Obs, FComp: Fn()+'static+Send+Sync> ObservableSubHelper<V,(), (), FComp> for Obs where Obs : Observable<V>
{
    #[inline] fn subf(&self, f: (), ferr: (), fcomp: FComp) -> UnsubRef<'static> { self.sub(Arc::new(((), (), fcomp))) }
}

impl<V,F, Src> ObservableSubNextHelper<V,F> for Src where
    F: Fn(V)+'static+Send+Sync,
    Src : Observable<V>
{
    #[inline] fn subn(&self, next: F) -> UnsubRef<'static> { self.sub(Arc::new(next)) }
}

impl<V, Src> Observable<V> for Arc<Src> where Src : Observable<V>
{
    #[inline] fn sub(&self, dest: Arc<Observer<V>+Send+Sync>) -> UnsubRef<'static>
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