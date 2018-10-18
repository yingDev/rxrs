#![feature(fn_traits, unboxed_closures, integer_atomics, associated_type_defaults, optin_builtin_traits, fnbox, test, cell_update, box_syntax, specialization, trait_alias, option_replace, coerce_unsized, unsize)]
#![allow(non_snake_case)]

pub mod sync;

pub use crate::util::*;
pub use crate::unsub::*;
pub use crate::subject::*;
pub use crate::fac::*;
pub use crate::op::*;
pub use crate::act::*;
pub use crate::into_sendsync::*;
pub use crate::observables::*;
use std::rc::Rc;
use std::sync::Arc;

pub trait ActNext <'o, SS:YesNo, BY: RefOrVal> : for<'x> Act    <SS, By<'x, BY>>+'o {}
pub trait ActEc   <'o, SS:YesNo, BY: RefOrVal> : for<'x> ActOnce<SS, Option<By<'x, BY>>>+'o {}
pub trait ActEcBox<'o, SS:YesNo, BY: RefOrVal> : for<'x> ActBox <SS, Option<By<'x, BY>>>+'o {}

pub trait Observable<'o, SS:YesNo, VBy: RefOrVal=Ref<()>, EBy: RefOrVal=Ref<()>>
{
    fn sub(&self, next: impl ActNext<'o, SS, VBy>, ec: impl ActEc<'o, SS, EBy>) -> Unsub<'o, SS> where Self: Sized
    { self.sub_dyn(box next, box ec) }

    fn sub_dyn(&self, next: Box<ActNext<'o, SS, VBy>>, ec: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>;
}

pub unsafe trait IntoDyn<'o, SS:YesNo, VBy: RefOrVal, EBy: RefOrVal>
    where Self: Observable<'o, SS, VBy, EBy> + Sized
{
    #[inline(always)] fn into_dyn(self) -> Box<Self>  { box self }
}

unsafe impl<'a, 'o, SS:YesNo, VBy: RefOrVal, EBy: RefOrVal, O> IntoDyn<'o, SS, VBy, EBy> for O
    where O: Observable<'o, SS, VBy, EBy> + Sized
{}


impl<'o, SS:YesNo, BY: RefOrVal, A: for<'x> Act<SS, By<'x, BY>>+'o> ActNext<'o, SS, BY> for A {}
impl<'o, SS:YesNo, BY: RefOrVal, A: for<'x> ActOnce<SS, Option<By<'x, BY>>>+'o> ActEc<'o, SS, BY> for A {}
impl<'o, SS:YesNo, BY: RefOrVal, A: for<'x> ActBox<SS, Option<By<'x, BY>>>+'o> ActEcBox<'o, SS, BY> for A {}


mod observables;
mod op;
mod util;
mod subject;
mod unsub;
mod fac;
mod act;
mod into_sendsync;