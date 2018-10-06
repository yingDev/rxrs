#![feature(fn_traits, unboxed_closures, integer_atomics, associated_type_defaults, optin_builtin_traits, fnbox)]
#![feature(test)]
#![feature(cell_update)]
#![feature(box_syntax)]
#![feature(specialization)]

#![allow(non_snake_case)]

use std::borrow::Cow;
use std::cell::Cell;
use std::cell::UnsafeCell;
use std::error::Error;
use std::hash::Hash;
use std::hash::Hasher;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Once;
use std::sync::ONCE_INIT;
use std::sync::RwLock;
use std::thread;
use std::thread::ThreadId;
use std::collections::LinkedList;
use std::boxed::FnBox;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::atomic::AtomicUsize;

pub mod sync;
pub mod subject;
mod collection_ext;


use crate::subject::Subject;
use crate::sync::ArcCell;
use crate::collection_ext::CollectionExt;
use crate::sync::ReSpinLock;

pub trait YesNo { const VALUE: bool; }
pub struct YES;
pub struct NO;
impl YesNo for YES { const VALUE: bool = true; }
impl YesNo for NO { const VALUE: bool = false; }


pub struct Subscription<'a, SS:YesNo>
{
    lock: ReSpinLock<SS>,
    done: AtomicBool,
    cbs: UnsafeCell<Option<Vec<Box<FnBox()+'a>>>>,
    PhantomData: PhantomData<SS>
}

unsafe impl Send for Subscription<'static, YES> {}
unsafe impl Sync for Subscription<'static, YES> {}

impl<'a, SS:YesNo> Subscription<'a, SS>
{
    pub fn new() -> Subscription<'a, SS>
    {
        Subscription{ lock: ReSpinLock::new(), done: AtomicBool::new(false), cbs: UnsafeCell::new(None), PhantomData }
    }

    pub fn done() -> Subscription<'a, SS>
    {
        let val = Self::new();
        val.unsubscribe();
        val
    }

    pub fn is_done(&self) -> bool
    {
        self.done.load(Ordering::Relaxed)
    }

    pub fn unsubscribe(&self)
    {
        self.lock.enter();
        self.done.store(true, Ordering::Release);

        unsafe{
            let mut old : &mut Option<_> = &mut *self.cbs.get();
            if let Some(mut vec) = old.take() {
                vec.reverse();
                for cb in vec.drain(..) {
                    cb();
                }
            }

        }
        self.lock.exit();
    }

    fn add_internal(&self, cb: Box<FnBox()+'a>)
    {
        if self.is_done() {
            return cb.call_box(());
        }

        self.lock.enter();

        unsafe {
            let cbs = &mut * self.cbs.get();
            if let Some(mut vec) = cbs.as_mut() {
                vec.push(cb);
            } else {
                let mut vec = Vec::new();
                vec.push(cb);
                *cbs = Some(vec);
            }
        }

        self.lock.exit();
    }

    pub fn add_sendsync(&'static self, cb: impl FnBox() + Send + Sync + 'static) { self.add_internal(Box::new(cb)); }
}

impl<'a> Subscription<'a, NO>
{
    pub fn add(&self, cb: impl FnBox() + 'a)
    {
        self.add_internal(Box::new(cb));
    }
}

impl Subscription<'static, YES>
{
    pub fn add(&self, cb: impl FnBox() + Send + Sync + 'static)
    {
        self.add_internal(Box::new(cb));
    }
}

//todo: rm E ?
pub trait Observable<'s, 'o, V:Clone, E:Clone>
{
    fn subscribe(&'s self, observer: impl Observer<V,E>+'o) -> Subscription<'o,NO>;
}

pub trait ObservableSendSync<'s, V:Clone, E:Clone> : Send + Sync
{
    fn subscribe(&'s self, observer: impl Observer<V,E>+ Send + Sync+'static) -> Subscription<'static, YES>;
}

pub trait Observer<V:Clone, E:Clone>
{
    fn next(&self, value: V);
    fn error(&self, error: E);
    fn complete(&self);
}

trait Subscriber<V:Clone, E:Clone> : Observer<V,E>
{
    fn unsubscribe(&self);
}



impl<V:Clone, E:Clone, FN:Fn(V)> Observer<V,E> for FN
{
    fn next(&self, value: V) { self(value) }
    fn error(&self, _: E) {}
    fn complete(&self){}
}

impl<V:Clone, E:Clone, FN:Fn(V)> Observer<V,E> for (FN,())
{
    fn next(&self, value: V) { self.0(value) }
    fn error(&self, _: E) {}
    fn complete(&self){}
}

impl<V:Clone, E:Clone, FN:Fn(V)> Observer<V,E> for (FN,(), ())
{
    fn next(&self, value: V) { self.0(value) }
    fn error(&self, _: E) {}
    fn complete(&self){}
}

impl<V:Clone, E:Clone, FN:Fn(V), FE:Fn(E)> Observer<V,E> for (FN,FE)
{
    fn next(&self, value: V) { self.0(value) }
    fn error(&self, error: E){ self.1(error) }
    fn complete(&self){}
}

impl<V:Clone, E:Clone, FN:Fn(V), FE:Fn(E)> Observer<V,E> for (FN,FE, ())
{
    fn next(&self, value: V) { self.0(value) }
    fn error(&self, error: E){ self.1(error) }
    fn complete(&self){}
}

impl<V:Clone, E:Clone, FN:Fn(V), FC:Fn()> Observer<V,E> for (FN,(),FC)
{
    fn next(&self, value: V) { self.0(value) }
    fn error(&self, _: E) {}
    fn complete(&self){ self.2() }
}

impl<V:Clone, E:Clone, FE:Fn(E), FC:Fn()> Observer<V,E> for ((),FE,FC)
{
    fn next(&self, _:V) {}
    fn error(&self, error: E){ self.1(error) }
    fn complete(&self){ self.2() }
}

impl<V:Clone, E:Clone, FE:Fn(E)> Observer<V,E> for ((),FE,())
{
    fn next(&self, _:V) {}
    fn error(&self, error: E){ self.1(error) }
    fn complete(&self){ }
}

impl<V:Clone, E:Clone, FC:Fn()> Observer<V,E> for ((),(),FC)
{
    fn next(&self, _:V) {}
    fn error(&self, _: E) {}
    fn complete(&self){ self.2() }
}

impl<V:Clone, E:Clone, FN:Fn(V), FE:Fn(E), FC:Fn()> Observer<V,E> for (FN,FE,FC)
{
    fn next(&self, value: V) { self.0(value) }
    fn error(&self, error: E){ self.1(error) }
    fn complete(&self){ self.2() }
}

#[derive(Copy, Clone)]
pub struct Of<V: Clone, SS>(V, SS);

pub fn of<V:Clone, SS>(v:V, s:SS) -> Of<V, SS>
{
    Of(v, s)
}

impl<V> !Send for Of<V, NO> {}
impl<V> !Sync for Of<V, NO> {}

impl<'s, 'o, V: Clone> Observable<'s, 'o, V, ()> for Of<V, NO>
{
    fn subscribe(&'s self, observer: impl Observer<V,()>+'o) -> Subscription<'o, NO>
    {
        observer.next(self.0.clone());
        observer.complete();
        Subscription::new()
    }
}

impl<'s, 'o, V: Clone+Send+Sync> ObservableSendSync<'s, V, ()> for Of<V, YES>
{
    fn subscribe(&'s self, observer: impl Observer<V,()>+Send+Sync+'static) -> Subscription<'static, YES>
    {
        observer.next(self.0.clone());
        observer.complete();
        Subscription::new()
    }
}


pub trait Mapper<V, VOut, E, EOut>
{
    fn next(&self, v:V) -> VOut;
    fn error(&self, e:E) -> EOut;
}

impl<V, VOut, E, FN> Mapper<V, VOut, E, E> for FN where FN: Fn(V)->VOut
{
    fn next(&self, v:V) -> VOut { self.call((v,)) }
    fn error(&self, e:E) -> E { e }
}

impl<V, E, EOut, FE> Mapper<V, V, E, EOut> for ((), FE) where FE: Fn(E)->EOut
{
    fn next(&self, v:V) -> V { v }
    fn error(&self, e:E) -> EOut { self.1.call((e,)) }
}

impl<V, VOut, E, EOut, FN, FE> Mapper<V, VOut, E, EOut> for (FN, FE) where FN: Fn(V)->VOut, FE: Fn(E)->EOut
{
    fn next(&self, v:V) -> VOut { self.0.call((v,)) }
    fn error(&self, e:E) -> EOut { self.1.call((e,)) }
}



pub trait ObservableOpMap<V, VOut, E, EOut, SS> : Sized
{
    fn map<'s, F: Mapper<V, VOut, E, EOut>>(&'s self,f: F) -> OpMap<'s, V, VOut, E, EOut, Self, F, SS>
    {
        OpMap { src: self, f, PhantomData }
    }
}

impl<'s, 'o, V:Clone,VOut:Clone, E:Clone, EOut:Clone, Src> ObservableOpMap<V, VOut, E, EOut, NO> for Src where Src : Observable<'s, 'o, V, E> {}
impl<'s, V:Clone+Send+Sync,VOut:Clone+Send+Sync, E:Clone+Send+Sync, EOut:Clone+Send+Sync, Src> ObservableOpMap<V, VOut, E, EOut, YES> for Src where Src : ObservableSendSync<'s, V, E> {}

pub struct OpMap<'s, V, VOut, E, EOut, Src, F, SS> where F : Mapper<V,VOut,E,EOut>
{
    src: &'s Src,
    f: F,
    PhantomData: PhantomData<(V, VOut, E, EOut, SS)>
}

impl<'s, 'o, V:Clone+'o, E:Clone+'o, VOut:Clone+'o, EOut:Clone+'o, Src: Observable<'s, 'o, V, E>, F: Mapper<V,VOut, E, EOut>+Clone+'o> Observable<'s, 'o, VOut, EOut> for OpMap<'s, V, VOut, E, EOut, Src, F, NO>
{
    fn subscribe(&'s self, observer: impl Observer<VOut,EOut>+'o) -> Subscription<'o, NO>
    {
        self.src.subscribe( OpMapSubscriber { observer, f: self.f.clone(), PhantomData })
    }
}

impl<'s, V:Clone+Send+Sync+'static, E:Clone+Send+Sync+'static, EOut: Clone+Send+Sync+'static, VOut:Clone+Send+Sync+'static, Src: ObservableSendSync<'s, V, E>, F: Mapper<V,VOut, E, EOut>+Send+Sync+Clone+'static> ObservableSendSync<'s, VOut, EOut> for OpMap<'s, V, VOut, E, EOut, Src, F, YES>
{
    fn subscribe(&'s self, observer: impl Observer<VOut,EOut>+Send+Sync+'static) -> Subscription<'static, YES>
    {
        self.src.subscribe( OpMapSubscriber { observer, f: self.f.clone(), PhantomData})
    }
}

struct OpMapSubscriber<V, VOut, E, EOut, Dest, F: Mapper<V,VOut, E, EOut>>
{
    observer: Dest,
    f: F,
    PhantomData: PhantomData<(V, VOut, E, EOut)>
}

impl<'a, V:Clone, VOut:Clone, E:Clone, EOut:Clone, Dest: Observer<VOut,EOut>, F: Mapper<V,VOut, E, EOut>> Observer<V,E> for OpMapSubscriber<V, VOut, E, EOut, Dest, F>
{
    fn next(&self, v:V) { self.observer.next(self.f.next(v)) }
    fn error(&self, e: E) { self.observer.error(self.f.error(e)) }
    fn complete(&self) { self.observer.complete() }
}




