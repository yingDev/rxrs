#![feature(fn_traits, unboxed_closures, integer_atomics, associated_type_defaults, optin_builtin_traits, fnbox, test, cell_update, box_syntax, specialization, )]
#![allow(non_snake_case)]

pub trait Observable<'o, V:Clone+'o, E:Clone+'o>
{
    fn subscribe(&self, observer: impl Observer<V, E> + 'o) -> Subscription<'o, NO>;
}

pub trait ObservableSendSync<V:Clone+'static, E:Clone+'static> : Send + Sync
{
    fn subscribe(&self, observer: impl Observer<V,E>+ Send + Sync+'static) -> Subscription<'static, YES>;
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

pub mod sync;


mod fac;
mod op;
mod subscription;
mod yesno;
mod subject;
mod observers;
mod observables;

pub use crate::subscription::*;
pub use crate::yesno::*;
pub use crate::fac::*;
pub use crate::subject::*;
pub use crate::observers::*;
pub use crate::observables::*;
pub use crate::op::*;