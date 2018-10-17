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

pub trait Observable<'o, SS:YesNo, VBy: RefOrVal=Ref<()>, EBy: RefOrVal=Ref<()>>
{
    fn sub(&self,
           next: impl for<'x> Act<SS, By<'x, VBy>>+'o,
           ec: impl for<'x> ActOnce<SS, Option<By<'x, EBy>>>+'o) -> Unsub<'o, SS> where Self: Sized
    {
        self.sub_dyn(box next, box ec)
    }

    fn sub_dyn(&self,
               next: Box<for<'x> Act<SS, By<'x, VBy>>+'o>,
               ec: Box<for<'x> ActBox<SS, Option<By<'x, EBy>>> +'o>) -> Unsub<'o, SS>;
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



mod op;
mod util;
mod subject;
mod unsub;
mod fac;
mod action;
mod into_sendsync;