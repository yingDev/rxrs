#![feature(fn_traits, unboxed_closures, integer_atomics, associated_type_defaults, optin_builtin_traits, fnbox, test, cell_update, box_syntax, specialization, trait_alias, option_replace)]
#![allow(non_snake_case)]

mod op;
mod util;
mod subject;
mod unsub;
mod fac;

pub mod sync;

pub use crate::util::{*, alias::*};
pub use crate::unsub::*;
//pub use crate::subject::*;
pub use crate::fac::*;
pub use crate::op::*;

pub trait Observable<'o, SS:YesNo, VBy: RefOrVal=Ref<()>, EBy: RefOrVal=Ref<()>>
{
    fn sub(&self,
           next: impl for<'x> FnNext<SS, By<'x, VBy>>+'o,
           ec: impl for<'x> FnErrComp<SS, By<'x, EBy>>+'o) -> Unsub<'o, SS> where Self: Sized
    {
        self.sub_dyn(box next, box ec)
    }

    fn sub_dyn(&self,
               next: Box<for<'x> FnNext<SS, By<'x, VBy>>+'o>,
               ec: Box<for<'x> FnErrCompBox<SS, By<'x, EBy>> +'o>) -> Unsub<'o, SS>;
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

pub unsafe trait FnNext<SS:YesNo, Arg>  { fn call(&self, v: Arg); }
pub unsafe trait FnErrComp<SS:YesNo, Arg> { fn call_once(self, e: Option<Arg>); }
pub unsafe trait FnErrCompBox<SS:YesNo, Arg> { fn call_box(self: Box<Self>, e: Option<Arg>); }

unsafe impl<'a, Arg, F: Fn(Arg)+'a> FnNext<NO, Arg> for F
{ #[inline(always)] fn call(&self, v: Arg) { self(v) } }

unsafe impl<Arg, F: Fn(Arg)+Send+Sync+'static> FnNext<YES, Arg> for F
{ #[inline(always)] fn call(&self, v: Arg) { self(v) } }

unsafe impl<'a, Arg, F: FnOnce(Option<Arg>)+'a> FnErrComp<NO, Arg> for F
{ #[inline(always)] fn call_once(self, v: Option<Arg>) { self(v) } }

unsafe impl<Arg, F: FnOnce(Option<Arg>)+Send+Sync+'static> FnErrComp<YES, Arg> for F
{ #[inline(always)] fn call_once(self, v: Option<Arg>) { self(v) } }

unsafe impl<SS:YesNo, F: FnErrComp<SS, Arg>, Arg> FnErrCompBox<SS, Arg> for F
{ #[inline(always)] fn call_box(self: Box<Self>, v: Option<Arg>) { self.call_once(v) } }


unsafe impl<'a, SS: YesNo, Arg> FnNext<SS, Arg> for Box<FnNext<SS, Arg>+'a>
{ #[inline(always)] fn call(&self, v: Arg) { self.as_ref().call(v) } }


unsafe impl<'a, SS:YesNo, Arg> FnErrComp<SS, Arg> for Box<FnErrCompBox<SS, Arg>+'a>
{ #[inline(always)] fn call_once(self, v: Option<Arg>) { self.call_box(v) } }


unsafe impl<SS:YesNo, Arg> FnNext<SS, Arg> for ()
{ #[inline(always)] fn call(&self, v: Arg) {  } }

unsafe impl<SS:YesNo, Arg> FnErrComp<SS, Arg> for ()
{ #[inline(always)] fn call_once(self, v: Option<Arg>) {  } }




