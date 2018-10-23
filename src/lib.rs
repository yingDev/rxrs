#![feature(fn_traits, unboxed_closures, integer_atomics, associated_type_defaults, optin_builtin_traits, fnbox,
    test, cell_update, box_syntax, specialization, trait_alias, option_replace, coerce_unsized, unsize,impl_trait_in_bindings,
)]
#![allow(non_snake_case)]


pub trait Observable<'o, SS:YesNo, VBy: RefOrVal=Ref<()>, EBy: RefOrVal=Ref<()>>
{
    fn sub(&self, next: impl ActNext<'o, SS, VBy>, err_or_comp: impl ActEc<'o, SS, EBy>) -> Unsub<'o, SS> where Self: Sized
    {
        self.sub_dyn(box next, box err_or_comp)
    }

    fn sub_dyn(&self, next: Box<ActNext<'o, SS, VBy>>, err_or_comp: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>;
}

pub trait IntoDyn<'o, SS, VBy, EBy> : Sized
{
    #[inline(always)]
    fn into_dyn(self) -> Box<Self>  { box self }
}




pub unsafe trait ActNext <'o, SS:YesNo, BY: RefOrVal> : for<'x> Act    <SS, By<'x, BY>>+'o {}
pub unsafe trait ActEc   <'o, SS:YesNo, BY: RefOrVal=Ref<()>> : for<'x> ActOnce<SS, Option<By<'x, BY>>>+'o {}
pub unsafe trait ActEcBox<'o, SS:YesNo, BY: RefOrVal=Ref<()>> : for<'x> ActBox <SS, Option<By<'x, BY>>>+'o {}

pub mod sync;

pub use crate::util::*;
pub use crate::unsub::*;
pub use crate::subject::*;
pub use crate::fac::*;
pub use crate::op::*;
pub use crate::act::*;
pub use crate::act_helpers::*;
pub use crate::observables::*;
pub use crate::scheduler::*;
mod observables;
mod op;
mod util;
mod subject;
mod unsub;
mod fac;
mod act;
mod act_helpers;
mod scheduler;


impl<'a, 'o, SS:YesNo, VBy: RefOrVal, EBy: RefOrVal, O: Observable<'o, SS, VBy, EBy>+'a> IntoDyn<'o, SS, VBy, EBy> for O {}
unsafe impl<'o, SS:YesNo, BY: RefOrVal, A: for<'x> Act    <SS, By<'x, BY>>+'o>         ActNext<'o, SS, BY>  for A {}
unsafe impl<'o, SS:YesNo, BY: RefOrVal, A: for<'x> ActOnce<SS, Option<By<'x, BY>>>+'o> ActEc<'o, SS, BY>    for A {}
unsafe impl<'o, SS:YesNo, BY: RefOrVal, A: for<'x> ActBox <SS, Option<By<'x, BY>>>+'o> ActEcBox<'o, SS, BY> for A {}