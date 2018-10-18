#![feature(fn_traits, unboxed_closures, integer_atomics, associated_type_defaults, optin_builtin_traits, fnbox, test, cell_update, box_syntax, specialization, trait_alias, option_replace, coerce_unsized, unsize)]
#![allow(non_snake_case)]

pub mod sync;

pub use crate::util::*;
pub use crate::unsub::*;
pub use crate::subject::*;
pub use crate::fac::*;
pub use crate::op::*;
pub use crate::action::*;
pub use crate::into_sendsync::*;
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
    #[inline(always)] fn into_dyn_ss(self) -> Box<Self> where Self: Send+Sync { box self }
}

unsafe impl<'a, 'o, SS:YesNo, VBy: RefOrVal, EBy: RefOrVal, O> IntoDyn<'o, SS, VBy, EBy> for O
    where O: Observable<'o, SS, VBy, EBy> + Sized
{}


impl<'o, SS:YesNo, BY: RefOrVal, A: for<'x> Act<SS, By<'x, BY>>+'o> ActNext<'o, SS, BY> for A {}
impl<'o, SS:YesNo, BY: RefOrVal, A: for<'x> ActOnce<SS, Option<By<'x, BY>>>+'o> ActEc<'o, SS, BY> for A {}
impl<'o, SS:YesNo, BY: RefOrVal, A: for<'x> ActBox<SS, Option<By<'x, BY>>>+'o> ActEcBox<'o, SS, BY> for A {}

impl<'o, O, SS:YesNo, VBy:RefOrVal, EBy: RefOrVal> Observable<'o, SS, VBy, EBy> for Rc<O>
    where O: Observable<'o, SS, VBy, EBy>
{
    #[inline(always)]
    fn sub(&self, next: impl ActNext<'o, SS, VBy>, ec: impl ActEc<'o, SS, EBy>) -> Unsub<'o, SS> where Self: Sized
    { Rc::as_ref(self).sub(next, ec) }

    #[inline(always)]
    fn sub_dyn(&self, next: Box<ActNext<'o, SS, VBy>>, ec: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>
    { Rc::as_ref(self).sub_dyn(next, ec) }
}

impl<'o, O, SS:YesNo, VBy:RefOrVal, EBy: RefOrVal> Observable<'o, SS, VBy, EBy> for Arc<O>
    where O: Observable<'o, SS, VBy, EBy>
{
    #[inline(always)]
    fn sub(&self, next: impl ActNext<'o, SS, VBy>, ec: impl ActEc<'o, SS, EBy>) -> Unsub<'o, SS> where Self: Sized
    { Arc::as_ref(self).sub(next, ec) }

    #[inline(always)]
    fn sub_dyn(&self, next: Box<ActNext<'o, SS, VBy>>, ec: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>
    { Arc::as_ref(self).sub_dyn(next, ec) }
}


impl<'o, O, SS:YesNo, VBy:RefOrVal, EBy: RefOrVal> Observable<'o, SS, VBy, EBy> for Box<O>
    where O: Observable<'o, SS, VBy, EBy>
{
    #[inline(always)]
    fn sub(&self, next: impl ActNext<'o, SS, VBy>, ec: impl ActEc<'o, SS, EBy>) -> Unsub<'o, SS> where Self: Sized
    { Box::as_ref(self).sub(next, ec) }

    #[inline(always)]
    fn sub_dyn(&self, next: Box<ActNext<'o, SS, VBy>>, ec: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>
    { Box::as_ref(self).sub_dyn(next, ec) }
}

impl<'o, SS:YesNo, VBy:RefOrVal, EBy: RefOrVal> Observable<'o, SS, VBy, EBy> for Box<dyn Observable<'o, SS, VBy, EBy>>
{
    #[inline(always)]
    fn sub_dyn(&self, next: Box<ActNext<'o, SS, VBy>>, ec: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>
    { Box::as_ref(self).sub_dyn(next, ec) }
}

mod op;
mod util;
mod subject;
mod unsub;
mod fac;
mod action;
mod into_sendsync;