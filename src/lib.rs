#![feature(fn_traits, unboxed_closures, integer_atomics, associated_type_defaults, optin_builtin_traits, fnbox,
    test, cell_update, box_syntax, specialization, trait_alias, option_replace, coerce_unsized, unsize,impl_trait_in_bindings,
)]
#![feature(arbitrary_self_types)]
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

pub unsafe trait ActNext <'o, SS:YesNo, BY: RefOrVal> : 'o //: for<'x> Act    <SS, By<'x, BY>>+'o
{
    fn call(&self, by: By<BY>);
    fn stopped(&self) -> bool { false }
}
pub unsafe trait ActEc<'o, SS:YesNo, BY: RefOrVal=Ref<()>> : 'o
{
    fn call_once(self, e: Option<By<BY>>);
}
pub unsafe trait ActEcBox<'o, SS:YesNo, BY: RefOrVal=Ref<()>> : 'o
{
    fn call_box(self: Box<Self>, e: Option<By<BY>>);
}

unsafe impl<'o, SS:YesNo, BY:RefOrVal, A: ActEc<'o, SS, BY>> ActEcBox<'o, SS, BY> for A
{
    fn call_box(self: Box<A>, e: Option<By<BY>>) { self.call_once(e) }
}

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
use std::marker::PhantomData;

mod observables;
mod op;
mod util;
mod subject;
mod unsub;
mod fac;
mod act;
mod act_helpers;
mod scheduler;


impl<'a, 'o, SS:YesNo, VBy: RefOrVal, EBy: RefOrVal, O: Observable<'o, SS, VBy, EBy>+'a>
IntoDyn<'o, SS, VBy, EBy>
for O {}



unsafe impl<'o, BY: RefOrVal, A: for<'x> Fn(By<'x, BY>)+'o>
ActNext<'o, NO, BY>
for A {
    fn call(&self, by: By<BY>) { self.call((by,)) }
}

unsafe impl<'o, BY: RefOrVal, A: for<'x> Fn(By<'x, BY>)+'o+Send+Sync>
ActNext<'o, YES, BY>
for A {
    fn call(&self, by: By<BY>) { self.call((by,)) }
}

unsafe impl<'o, SS:YesNo, BY: RefOrVal>
ActNext<'o, SS, BY>
for () {
    fn call(&self, by: By<BY>) {  }
}




unsafe impl<'o, BY: RefOrVal, A: FnOnce(Option<By<BY>>)+'o>
ActEc<'o, NO, BY>
for A {
    fn call_once(self, by: Option<By<BY>>) { self.call_once((by,)) }
}

unsafe impl<'o, BY: RefOrVal, A: FnOnce(Option<By<BY>>)+'o+Send+Sync>
ActEc<'o, YES, BY>
for A {
    fn call_once(self, by: Option<By<BY>>) { self.call_once((by,)) }
}

unsafe impl<'o, SS:YesNo, BY: RefOrVal>
ActEc<'o, SS, BY>
for () {
    fn call_once(self, by: Option<By<BY>>) {  }
}



pub struct ForwardNext<'o, SS:YesNo, NBY:RefOrVal, FBY: RefOrVal, N: ActNext<'o, SS, NBY>, F: Fn(&N, By<FBY>)+'o, S: Fn(bool)->bool+'o>
{
    old: N,
    next: F,
    stop: S,
    PhantomData: PhantomData<&'o(SS, NBY, FBY)>
}

unsafe impl<'o, NBY:RefOrValSSs, FBY: RefOrValSSs, N: ActNext<'o, YES, NBY>, F: Fn(&N, By<FBY>)+'o+Send+Sync, S: Fn(bool)->bool+'o+Send+Sync> Send for ForwardNext<'o, YES, NBY, FBY, N, F, S> {}
unsafe impl<'o, NBY:RefOrValSSs, FBY: RefOrValSSs, N: ActNext<'o, YES, NBY>, F: Fn(&N, By<FBY>)+'o+Send+Sync, S: Fn(bool)->bool+'o+Send+Sync> Sync for ForwardNext<'o, YES, NBY, FBY, N, F, S> {}

impl<'o, SS:YesNo, NBY:RefOrVal, FBY: RefOrVal, N: ActNext<'o, SS, NBY>, F: Fn(&N, By<FBY>)+'o, S: Fn(bool)->bool+'o> ForwardNext<'o, SS, NBY, FBY, N, F, S>
{
    #[inline(always)] pub fn new(old: N, next: F, stop: S) -> Self
    {
        ForwardNext{ old, next, stop, PhantomData }
    }
}

unsafe impl<'o, SS:YesNo, NBY:RefOrVal, FBY: RefOrVal, N: ActNext<'o, SS, NBY>, F: Fn(&N, By<FBY>)+'o, S: Fn(bool)->bool+'o> ActNext<'o, SS, FBY> for ForwardNext<'o, SS, NBY, FBY, N, F, S>
{
    #[inline(always)]fn call(&self, by: By<FBY>) { self.next.call((&self.old, by)) }
    #[inline(always)]fn stopped(&self) -> bool
    {
        self.stop.call((self.old.stopped(),))
    }
}

unsafe impl<'o, SS:YesNo, BY: RefOrVal, N: ActNext<'o, SS, BY>, STOP: Act<SS, (), bool>+'o> ActNext<'o, SS, BY> for (N, STOP)
{
    #[inline(always)]fn call(&self, v: By<BY>) { self.0.call(v); }
    #[inline(always)]fn stopped(&self) -> bool {self.1.call(()) }
}

unsafe impl<'o, SS:YesNo, BY:RefOrVal+'o> ActNext<'o, SS, BY> for Box<ActNext<'o, SS, BY>>
{
    #[inline(always)] fn call<'x>(&self, v: By<'x, BY>) { Box::as_ref(self).call(v) }
    #[inline(always)] fn stopped(&self) -> bool { Box::as_ref(self).stopped() }
}

unsafe impl<'o, SS:YesNo, BY:RefOrVal+'o> ActEc<'o, SS, BY> for Box<ActEcBox<'o, SS, BY>>
{
    #[inline(always)] fn call_once<'x>(self, e: Option<By<'x, BY>>) { self.call_box(e) }
}






//todo:...
unsafe impl<SS:YesNo, A: for<'x> Act<SS, &'x Unsub<'static, SS>>+'static>
SchActPeriodic<SS>
for A{}

unsafe impl<SS:YesNo, A: ActOnce<SS, (), Unsub<'static, SS>>+'static>
SchActOnce<SS>
for A{}

unsafe impl<SS:YesNo, A: ActBox<SS, (), Unsub<'static, SS>>+'static>
SchActBox<SS>
for A{}