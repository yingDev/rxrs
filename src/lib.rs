#![feature(fn_traits, unboxed_closures, integer_atomics, associated_type_defaults, optin_builtin_traits, fnbox, test, cell_update, box_syntax, specialization, trait_alias, option_replace)]
#![allow(non_snake_case)]
#![feature(untagged_unions)]

#![feature(impl_trait_in_bindings)]
#![feature(coerce_unsized)]
#![feature(unsize)]

use std::cell::UnsafeCell;
use std::sync::Arc;
use std::marker::PhantomData;
use std::boxed::FnBox;

pub trait Observable<'o, SS:YesNo, VBy: RefOrVal=Ref<()>, EBy: RefOrVal=Ref<()>>
{
    fn sub(&self, next: impl FnNext<SS, VBy>+'o, ec: impl FnErrComp<SS, EBy>+'o) -> Unsub<'o, SS> where Self: Sized
    {
        self.sub_dyn(box next, box ec)
    }

    fn sub_dyn(&self, next: Box<FnNext<SS, VBy>+'o>, ec: Box<FnErrCompBox<SS, EBy> +'o>) -> Unsub<'o, SS>;
}

pub unsafe trait FnNext<SS:YesNo, BY: RefOrVal>  { fn call(&self, v: By<BY>); }
pub unsafe trait FnErrComp<SS:YesNo, EBy: RefOrVal> { fn call_once(self, e: Option<By<EBy>>); }
pub unsafe trait FnErrCompBox<SS:YesNo, EBy: RefOrVal> { fn call_box(self: Box<Self>, e: Option<By<EBy>>); }

pub unsafe trait IntoDyn<'s, 'o, SS:YesNo, VBy: RefOrVal, EBy: RefOrVal>
    where Self: 's + Observable<'o, SS, VBy, EBy> + Sized
{
    #[inline(always)] fn into_dyn(self) -> Box<dyn Observable<'o, SS, VBy, EBy> + 's>{ box self }
    #[inline(always)] fn into_dyn_ss(self) -> Box<dyn Observable<'o, SS, VBy, EBy>+Send+Sync+'s> where Self: Send+Sync { box self }

}
unsafe impl<'s, 'o, SS:YesNo, VBy: RefOrVal, EBy: RefOrVal, O> IntoDyn<'s, 'o, SS, VBy, EBy> for O
    where O: 's + Observable<'o, SS, VBy, EBy> + Sized
{}

unsafe impl<'a, BY: RefOrVal, F> FnNext<NO, BY> for F
    where F: Fn(By<BY>)+'a
{ #[inline(always)] fn call(&self, v: By<BY>) { self(v) } }

unsafe impl<BY: RefOrVal, F> FnNext<YES, BY> for F
    where F: Fn(By<BY>)+Send+Sync+'static
{ #[inline(always)] fn call(&self, v: By<BY>) { self(v) } }

unsafe impl<'a, BY: RefOrVal, F> FnErrComp<NO, BY> for F
    where F: FnOnce(Option<By<BY>>)+'a
{ #[inline(always)] fn call_once(self, v: Option<By<BY>>) { self(v) } }

unsafe impl<BY: RefOrVal, F> FnErrComp<YES, BY> for F
    where F: FnOnce(Option<By<BY>>)+Send+Sync+'static
{ #[inline(always)] fn call_once(self, v: Option<By<BY>>) { self(v) } }

unsafe impl<SS:YesNo, F, BY: RefOrVal> FnErrCompBox<SS, BY> for F
    where F: FnErrComp<SS, BY>
{ #[inline(always)] fn call_box(self: Box<Self>, v: Option<By<BY>>) { self.call_once(v) } }


unsafe impl<SS:YesNo, BY: RefOrVal> FnNext<SS, BY> for ()
{ #[inline(always)] fn call(&self, v: By<BY>) {  } }

unsafe impl<SS:YesNo, BY: RefOrVal> FnErrComp<SS, BY> for ()
{ #[inline(always)] fn call_once(self, v: Option<By<BY>>) {  } }

//mod of;
mod map;
mod util;
mod subject;
mod unsub;
mod fac;

pub mod sync;
//
//pub use self::of::*;
pub use crate::util::{*, alias::*};
pub use crate::unsub::*;
pub use crate::subject::*;
pub use crate::fac::*;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::marker::Unsize;
//