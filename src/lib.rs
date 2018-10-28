#![feature(fn_traits, unboxed_closures, integer_atomics, associated_type_defaults, optin_builtin_traits, fnbox,
    test, cell_update, box_syntax, specialization, trait_alias, option_replace, coerce_unsized, unsize,impl_trait_in_bindings,
)]
#![feature(arbitrary_self_types)]
#![allow(non_snake_case)]


pub trait Observable<'o, SS:YesNo, By: RefOrVal, EBy: RefOrVal=Ref<()>>
{
    fn sub(&self, next: impl ActNext<'o, SS, By>, err_or_comp: impl ActEc<'o, SS, EBy>) -> Unsub<'o, SS> where Self: Sized
    {
        self.sub_dyn(box next, box err_or_comp)
    }

    fn sub_dyn(&self, next: Box<ActNext<'o, SS, By>>, err_or_comp: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>;
}

pub trait IntoDyn<'o, SS: YesNo, By: RefOrVal, EBy: RefOrVal> : Sized
{
    #[inline(always)]
    fn into_dyn(self) -> Box<Self>  { box self }
}

pub unsafe trait ActNext <'o, SS:YesNo, BY: RefOrVal> : 'o
{
    fn call(&self, v: BY::V);
    fn stopped(&self) -> bool { false }
}
pub unsafe trait ActEc<'o, SS:YesNo, BY: RefOrVal=Ref<()> > : 'o
{
    fn call_once(self, e: Option<BY::V>);
}
pub unsafe trait ActEcBox<'o, SS:YesNo, BY: RefOrVal=Ref<()>> : 'o
{
    fn call_box(self: Box<Self>, e: Option<BY::V>);
}

unsafe impl<'o, SS:YesNo, BY:RefOrVal, A: ActEc<'o, SS, BY>> ActEcBox<'o, SS, BY> for A
{
    fn call_box(self: Box<Self>, e: Option<BY::V>) { self.call_once(e) }
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
use std::marker::PhantomData;
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


impl<'a, 'o, SS:YesNo, By: RefOrVal, EBy: RefOrVal, O: Observable<'o, SS, By, EBy>+'a>
IntoDyn<'o, SS, By, EBy>
for O {}


unsafe impl<'o, V, A: Fn(V)+'o>
ActNext<'o, NO, Val<V>>
for A
{
    fn call(&self, by: V) { self.call((by,)) }
}

unsafe impl<'o, V, A: Fn(&V)+'o>
ActNext<'o, NO, Ref<V>>
for A
{
    fn call(&self, by: *const V) { self.call((unsafe{ &*by },)) }
}

unsafe impl<'o, V, A: Fn(V)+'o+Send+Sync>
ActNext<'o, YES, Val<V>>
for A
{
    fn call(&self, by: V) { self.call((by,)) }
}

unsafe impl<'o, V, A: Fn(&V)+'o+Send+Sync>
ActNext<'o, YES, Ref<V>>
for A
{
    fn call(&self, by: *const V) { self.call((unsafe { &*by },)) }
}

unsafe impl<'o, SS:YesNo, By: RefOrVal>
ActNext<'o, SS, By>
for ()
{
    fn call(&self, by: By::V) { }
}



pub struct ForwardNext<'o, SS:YesNo, NBY:RefOrVal, FBY: RefOrVal, N: ActNext<'o, SS, NBY>, F: Fn(&N, FBY)+'o, S: Fn(bool)->bool+'o>
{
    old: N,
    next: F,
    stop: S,
    PhantomData: PhantomData<&'o(SS, NBY, FBY)>
}

unsafe impl<'o, NBY:RefOrValSSs, FBY: RefOrValSSs, N: ActNext<'o, YES, NBY>, F: Fn(&N, FBY)+'o+Send+Sync, S: Fn(bool)->bool+'o+Send+Sync> Send for ForwardNext<'o, YES, NBY, FBY, N, F, S> {}
unsafe impl<'o, NBY:RefOrValSSs, FBY: RefOrValSSs, N: ActNext<'o, YES, NBY>, F: Fn(&N, FBY)+'o+Send+Sync, S: Fn(bool)->bool+'o+Send+Sync> Sync for ForwardNext<'o, YES, NBY, FBY, N, F, S> {}

impl<'o, SS:YesNo, NBY:RefOrVal, FBY: RefOrVal, N: ActNext<'o, SS, NBY>, F: Fn(&N, FBY)+'o, S: Fn(bool)->bool+'o> ForwardNext<'o, SS, NBY, FBY, N, F, S>
{
    #[inline(always)] pub fn new(old: N, next: F, stop: S) -> Self
    {
        ForwardNext{ old, next, stop, PhantomData }
    }
}

unsafe impl<'o, SS:YesNo, NBY:RefOrVal, FBY: RefOrVal, N: ActNext<'o, SS, NBY>, F: Fn(&N, FBY)+'o, S: Fn(bool)->bool+'o>
ActNext<'o, SS, FBY>
for ForwardNext<'o, SS, NBY, FBY, N, F, S>
{
    #[inline(always)]fn call(&self, by: FBY::V) { self.next.call((&self.old, unsafe { FBY::from_v(by) })) }
    #[inline(always)]fn stopped(&self) -> bool
    {
        self.stop.call((self.old.stopped(),))
    }
}

#[inline(always)]
pub fn forward_next<'o, SS:YesNo, NBY:RefOrVal, FBY: RefOrVal, N: ActNext<'o, SS, NBY>, F: Fn(&N, FBY)+'o, S: Fn(bool)->bool+'o>
(old: N, next: F, stop: S) -> ForwardNext<'o, SS, NBY, FBY, N, F, S>
{
    ForwardNext::new(old, next, stop)
}




pub struct ForwardEc<'o, SS: YesNo, By: RefOrVal, F>
{
    f: F,
    PhantomData: PhantomData<&'o (SS, By)>
}

unsafe impl<'o, By: RefOrVal, F: Send> Send for ForwardEc<'o, YES, By, F> {}
unsafe impl<'o, By: RefOrVal, F: Sync> Sync for ForwardEc<'o, YES, By, F> {}

impl<'o, SS:YesNo, By: RefOrVal, F: FnOnce(Option<By>)> ForwardEc<'o, SS,By, F>
{
    #[inline(always)]
    fn new(f: F) -> ForwardEc<'o, SS, By, F>
    {
        ForwardEc{ f, PhantomData }
    }
}

unsafe impl<'o, SS:YesNo, By: RefOrVal+'o, F: FnOnce(Option<By>)+'o> ActEc<'o, SS, By> for ForwardEc<'o, SS, By, F>
{
    #[inline(always)] fn call_once(self, e: Option<By::V>)  { self.f.call_once((e.map(|e| unsafe { By::from_v(e) }), )) }
}


#[inline(always)]
pub fn forward_ec<'o, SS:YesNo, By: RefOrVal, F: FnOnce(Option<By>)>
(f: F) -> ForwardEc<'o, SS, By, F>
{
    ForwardEc::new(f)
}




unsafe impl<'o, SS:YesNo, BY: RefOrVal, N: ActNext<'o, SS, BY>, STOP: Act<SS, (), bool>+'o> ActNext<'o, SS, BY> for (N, STOP)
{
    #[inline(always)]fn call(&self, v: BY::V) { self.0.call(v); }
    #[inline(always)]fn stopped(&self) -> bool {self.1.call(()) }
}


unsafe impl<'o, SS:YesNo, E, F:FnOnce(Option<E>)+'o> ActEc<'o, SS, Val<E>> for F
{
    fn call_once(self, e: Option<E>) { self(e) }
}

unsafe impl<'o, SS:YesNo, E, F:FnOnce(Option<&E>)+'o> ActEc<'o, SS, Ref<E>> for F
{
    fn call_once(self, e: Option<*const E>) { self(e.map(|e| unsafe{ &*e })) }
}


unsafe impl<'o, SS:YesNo, BY:RefOrVal+'o> ActNext<'o, SS, BY> for Box<ActNext<'o, SS, BY>>
{
    fn call(&self, v: BY::V) { Box::as_ref(self).call(v) }
    fn stopped(&self) -> bool { Box::as_ref(self).stopped() }
}

unsafe impl<'o, SS:YesNo, BY:RefOrVal+'o> ActEc<'o, SS, BY> for Box<ActEcBox<'o, SS, BY>>
{
    fn call_once(self, e: Option<BY::V>) { self.call_box(e) }
}

unsafe impl<'o, SS:YesNo, BY:RefOrVal+'o> ActEc<'o, SS, BY> for ()
{
    fn call_once(self, e: Option<BY::V>) {}
}



////todo:...
unsafe impl<SS:YesNo, A: Act<SS, Ref<Unsub<'static, SS>>>+'static>
SchActPeriodic<SS>
for A{}

unsafe impl<SS:YesNo, A: ActOnce<SS, (), Unsub<'static, SS>>+'static>
SchActOnce<SS>
for A{}

unsafe impl<SS:YesNo, A: ActBox<SS, (), Unsub<'static, SS>>+'static>
SchActBox<SS>
for A{}


#[cfg(test)]
mod test
{
    use crate::*;

    fn inference()
    {

    }
}
