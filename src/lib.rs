#![feature(fn_traits, unboxed_closures, integer_atomics, associated_type_defaults, optin_builtin_traits, fnbox, test, cell_update, box_syntax, specialization, )]
#![allow(non_snake_case)]

pub trait Observable<V:Clone, E:Clone>
{
    fn subscribe<'o>(&self, observer: impl Observer<V,E>+'o) -> Subscription<'o,NO>;
}

pub trait ObservableSendSync<V:Clone, E:Clone> : Send + Sync
{
    fn subscribe(&self, observer: impl Observer<V,E>+ Send + Sync+'static) -> Subscription<'static, YES>;
}

pub trait DynObservable<'s, 'o, V:Clone, E:Clone, SS: YesNo, O: Observer<V,E>+'o = ::std::rc::Rc<Observer<V,E>+'o>>
{
    fn subscribe(&'s self, observer: O) -> Subscription<'o, SS>;
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

//struct Wrap<V:Clone, E:Clone, Src: Observable<'s, 'o, > O: Observer<V,E>>
//{
//    src:
//}
//
//fn shit<'s, 'o>() -> Box<DynObservable<'s, 'o, i32, (), NO>>
//{
//    let o = of(123, NO);
//
//}

pub mod sync;


mod fac;
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