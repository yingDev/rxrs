#![feature(fn_traits, unboxed_closures, integer_atomics, associated_type_defaults, optin_builtin_traits, fnbox, test, cell_update, box_syntax, specialization, trait_alias, option_replace)]
#![allow(non_snake_case)]

pub trait Observable<'o, V:Clone+'o, E:Clone+'o>
{
    fn sub(&self, o: impl Observer<V, E> + 'o) -> Unsub<'o, NO>;
}

pub trait ObservableSendSync<V:Clone+'static, E:Clone+'static> : 'static + Send + Sync
{
    fn sub(&self, o: impl Observer<V,E>+ Send + Sync+'static) -> Unsub<'static, YES>;
}


pub trait Observer<V:Clone, E:Clone>
{
    fn next(&self, v: V);
    fn error(&self, e: E);
    fn complete(&self);
}

trait Subscriber<V:Clone, E:Clone> : Observer<V,E>
{
    fn unsubscribe(&self);
}

pub mod sync;

mod fac;
mod op;
mod unsub;
mod yesno;
mod subject;
mod observers;
mod observables;

mod util;

pub use crate::unsub::*;
pub use crate::yesno::*;
pub use crate::fac::*;
pub use crate::subject::*;
pub use crate::observers::*;
pub use crate::observables::*;
pub use crate::op::*;