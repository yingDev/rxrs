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
pub mod fac;

mod op;
mod subscription;
mod yesno;
mod subject;

pub use crate::subscription::*;
pub use crate::yesno::*;
pub use crate::fac::*;
pub use crate::subject::*;

use crate::subject::Subject;
use crate::sync::ArcCell;
use crate::sync::ReSpinLock;

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


impl<'s, 'o, V:Clone, E:Clone, RC: Observable<'s, 'o, V, E>> Observable<'s, 'o, V, E> for Rc<RC>
{
    #[inline(always)] fn subscribe(&'s self, observer: impl Observer<V,E>+'o) -> Subscription<'o,NO> { Rc::as_ref(self).subscribe(observer) }
}

impl<'s, V:Clone+Send+Sync, E:Clone+Send+Sync, RC: ObservableSendSync<'s, V, E>> ObservableSendSync<'s, V, E> for Arc<RC>
{
    #[inline(always)] fn subscribe(&'s self, observer: impl Observer<V,E>+ Send + Sync+'static) -> Subscription<'static, YES>{ Arc::as_ref(self).subscribe(observer) }
}


impl<V:Clone, E:Clone, R, FN:Fn(V)->R> Observer<V,E> for FN
{
    #[inline(always)] fn next(&self, value: V) { self(value); }
    #[inline(always)] fn error(&self, _: E) {}
    #[inline(always)] fn complete(&self){}
}

impl<V:Clone, E:Clone, R, FN:Fn(V)->R> Observer<V,E> for (FN,())
{
    #[inline(always)] fn next(&self, value: V) { self.0(value); }
    #[inline(always)] fn error(&self, _: E) {}
    #[inline(always)] fn complete(&self){}
}

impl<V:Clone, E:Clone, R, FN:Fn(V)->R> Observer<V,E> for (FN,(), ())
{
    #[inline(always)] fn next(&self, value: V) { self.0(value); }
    #[inline(always)] fn error(&self, _: E) {}
    #[inline(always)] fn complete(&self){}
}

impl<V:Clone, E:Clone, RN, RE, FN:Fn(V)->RN, FE:Fn(E)->RE> Observer<V,E> for (FN,FE)
{
    #[inline(always)] fn next(&self, value: V) { self.0(value); }
    #[inline(always)] fn error(&self, error: E){ self.1(error); }
    #[inline(always)] fn complete(&self){}
}

impl<V:Clone, E:Clone, RN, RE, FN:Fn(V)->RN, FE:Fn(E)->RE> Observer<V,E> for (FN,FE, ())
{
    #[inline(always)] fn next(&self, value: V) { self.0(value); }
    #[inline(always)] fn error(&self, error: E){ self.1(error); }
    #[inline(always)] fn complete(&self){}
}

impl<V:Clone, E:Clone, RN, RC, FN:Fn(V)->RN, FC:Fn()->RC> Observer<V,E> for (FN,(),FC)
{
    #[inline(always)] fn next(&self, value: V) { self.0(value); }
    #[inline(always)] fn error(&self, _: E) {}
    #[inline(always)] fn complete(&self){ self.2(); }
}

impl<V:Clone, E:Clone, RE, RC, FE:Fn(E)->RE, FC:Fn()->RC> Observer<V,E> for ((),FE,FC)
{
    #[inline(always)] fn next(&self, _:V) {}
    #[inline(always)] fn error(&self, error: E){ self.1(error); }
    #[inline(always)] fn complete(&self){ self.2(); }
}

impl<V:Clone, E:Clone, RE, FE:Fn(E)->RE> Observer<V,E> for ((),FE,())
{
    #[inline(always)] fn next(&self, _:V) {}
    #[inline(always)] fn error(&self, error: E){ self.1(error); }
    #[inline(always)] fn complete(&self){ }
}

impl<V:Clone, E:Clone, RC, FC:Fn()->RC> Observer<V,E> for ((),(),FC)
{
    #[inline(always)] fn next(&self, _:V) {}
    #[inline(always)] fn error(&self, _: E) {}
    #[inline(always)] fn complete(&self){ self.2(); }
}

impl<V:Clone, E:Clone, RN, RE, RC, FN:Fn(V)->RN, FE:Fn(E)->RE, FC:Fn()->RC> Observer<V,E> for (FN,FE,FC)
{
    #[inline(always)] fn next(&self, value: V) { self.0(value); }
    #[inline(always)] fn error(&self, error: E){ self.1(error); }
    #[inline(always)] fn complete(&self){ self.2(); }
}




