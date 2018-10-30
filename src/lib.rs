#![feature(fn_traits, unboxed_closures, integer_atomics, optin_builtin_traits, fnbox,
    test, cell_update, box_syntax, coerce_unsized, unsize,
)]
//#![feature(arbitrary_self_types)]
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
use crate::util::any_send_sync::AnySendSync;
use std::ops::Deref;

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
//unsafe impl<SS:YesNo, A: Act<SS, Ref<Unsub<'static, SS>>>+'static>
//SchActPeriodic<SS>
//for A{}
//
//unsafe impl<SS:YesNo, A: ActOnce<SS, (), Unsub<'static, SS>>+'static>
//SchActOnce<SS>
//for A{}
//
//unsafe impl<SS:YesNo, A: ActBox<SS, (), Unsub<'static, SS>>+'static>
//SchActBox<SS>
//for A{}

pub unsafe trait SendSync<SS:YesNo> : Sized { }

unsafe impl<SS:YesNo> SendSync<SS> for () {}
unsafe impl <SS:YesNo, A: SendSync<SS>> SendSync<SS> for (A,) {}
unsafe impl <SS:YesNo, A: SendSync<SS>, B: SendSync<SS>> SendSync<SS> for (A, B) {}
unsafe impl <SS:YesNo, A: SendSync<SS>, B: SendSync<SS>, C: SendSync<SS>> SendSync<SS> for (A, B, C) {}
unsafe impl <SS:YesNo, A: SendSync<SS>, B: SendSync<SS>, C: SendSync<SS>, D: SendSync<SS>> SendSync<SS> for (A, B, C, D) {}
unsafe impl <SS:YesNo, A: SendSync<SS>, B: SendSync<SS>, C: SendSync<SS>, D: SendSync<SS>, E: SendSync<SS>> SendSync<SS> for (A, B, C, D, E) {}
unsafe impl <SS:YesNo, A: SendSync<SS>, B: SendSync<SS>, C: SendSync<SS>, D: SendSync<SS>, E: SendSync<SS>, F: SendSync<SS>> SendSync<SS> for (A, B, C, D, E, F) {}
unsafe impl <SS:YesNo, A: SendSync<SS>, B: SendSync<SS>, C: SendSync<SS>, D: SendSync<SS>, E: SendSync<SS>, F: SendSync<SS>, G: SendSync<SS>> SendSync<SS> for (A, B, C, D, E, F, F, G) {}

unsafe impl<SS:YesNo, A, B, C, R> SendSync<SS> for fn(A, B, C) -> R {}
unsafe impl<SS:YesNo, A, B, R> SendSync<SS> for fn(A, B) -> R {}
unsafe impl<SS:YesNo, A, B, R> SendSync<SS> for fn(&A, &B) -> R {}
unsafe impl<SS:YesNo, A, B, C, R> SendSync<SS> for fn(&A, &B, C) -> R {}

struct SSWrap<V:Send+Sync>(V);
unsafe impl<SS:YesNo, V: Send+Sync> SendSync<SS> for SSWrap<V> {}
impl<V: Send+Sync> SSWrap<V>
{
    fn new(v: V) -> Self { SSWrap(v) }
    fn into_inner(self) -> V { self.0 }
}
impl<V: Send+Sync> Deref for SSWrap<V>
{
    type Target = V;
    fn deref(&self) -> &V { &self.0 }
}

pub struct SSActNextWrap<SS:YesNo, By, A>
{
    next: A,
    PhantomData: PhantomData<(SS, By)>
}

impl<'o, SS:YesNo, By: RefOrVal, A> SSActNextWrap<SS, By, A>
{
    pub fn new(next: A) -> Self where A: ActNext<'o, SS, By>
    { SSActNextWrap{ next, PhantomData } }
}

unsafe impl<'o, SS:YesNo, By: RefOrVal, A: ActNext<'o, SS, By>> SendSync<SS> for SSActNextWrap<SS, By, A> {}

unsafe impl<'o, SS:YesNo, By: RefOrVal+'o, A: ActNext<'o, SS, By>> ActNext<'o, SS, By> for SSActNextWrap<SS, By, A>
{
    #[inline(always)] fn call(&self, v: <By as RefOrVal>::V) { self.next.call(v) }
    #[inline(always)] fn stopped(&self) -> bool { self.next.stopped() }
}


pub struct SSActEcWrap<By, A>
{
    pub ec: A,
    PhantomData: PhantomData<By>
}

impl<By: RefOrVal, A> SSActEcWrap<By, A>
{
    pub fn new<'o, SS:YesNo>(ec: A) -> Self where A: ActEc<'o, SS, By>
    { SSActEcWrap{ ec, PhantomData } }

    pub fn into_inner(self) -> A { self.ec }
}

unsafe impl<'o, SS:YesNo, By: RefOrVal, A: ActEc<'o, SS, By>> SendSync<SS> for SSActEcWrap<By, A> {}


pub struct SsForward<SS:YesNo, Caps: SendSync<SS>> { captures: Caps, PhantomData: PhantomData<SS> }

impl<SS:YesNo, Caps: SendSync<SS>> SsForward<SS, Caps>
{
    pub fn new(value: Caps) -> Self
    {
        SsForward { captures: value, PhantomData }
    }

    pub fn into_inner(self) -> Caps { self.captures }
}

unsafe impl<SS:YesNo, Caps: SendSync<SS>> SendSync<SS> for SsForward<SS, Caps> {}

impl<SS:YesNo, Caps: SendSync<SS>> Deref for SsForward<SS, Caps>
{
    type Target = Caps;
    fn deref(&self) -> &Caps { &self.captures }
}


unsafe impl<'o, SS:YesNo, N: SendSync<SS>+'o, Caps:SendSync<SS>+'o, FBy: RefOrVal+'o>
ActNext<'o, SS, FBy>
for SsForward<SS, (N, Caps, fn(&N, &Caps, FBy), fn(&N, &Caps) ->bool)>
{
    #[inline(always)]
    fn call(&self, v: FBy::V)
    {
        let (next, caps, fnext, _) = &self.captures;
        fnext(next, caps, unsafe { FBy::from_v(v) })
    }

    #[inline(always)]
    fn stopped(&self) -> bool
    {
        let (next, caps, _, stop) = &self.captures;
        stop(next, caps)
    }
}

#[inline(always)]
pub fn forward_next<'o, SS:YesNo, By: RefOrVal+'o, N: SendSync<SS>+'o, Caps:SendSync<SS>+'o>
(next:N, captures: Caps, fnext: fn(&N, &Caps, By), fstop: fn(&N, &Caps)->bool)
 -> SsForward<SS, (N, Caps, fn(&N, &Caps, By), fn(&N, &Caps) ->bool)>
{
    SsForward::new((next, captures, fnext, fstop))
}


#[inline(always)]
pub fn forward_ec<'o, SS:YesNo, By: RefOrVal+'o, Caps:SendSync<SS>+'o>
(captures: Caps, fec: fn(Caps, Option<By>))
 -> SsForward<SS, (Caps, fn(Caps, Option<By>))>
{
    SsForward::new((captures, fec))
}


unsafe impl<'o, SS:YesNo, By: RefOrVal+'o, Caps:SendSync<SS>+'o>
ActEc<'o, SS, By>
for SsForward<SS, (Caps, fn(Caps, Option<By>))>
{
    #[inline(always)]
    fn call_once(self, v: Option<By::V>)
    {
        let (caps, fec) = self.captures;
        fec(caps,  v.map(|v| unsafe { By::from_v(v) }))
    }
}
