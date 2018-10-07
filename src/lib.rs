#![feature(fn_traits, unboxed_closures, integer_atomics, associated_type_defaults, optin_builtin_traits, fnbox)]
#![feature(test)]
#![feature(cell_update)]
#![feature(box_syntax)]
#![feature(specialization)]

#![allow(non_snake_case)]

pub mod sync;
pub mod fac;

mod op;
mod subscription;
mod yesno;
mod subject;
mod observer_fn;
mod observable_rc;

pub use crate::subscription::*;
pub use crate::yesno::*;
pub use crate::fac::*;
pub use crate::subject::*;
pub use crate::observer_fn::*;
pub use crate::observable_rc::*;

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


