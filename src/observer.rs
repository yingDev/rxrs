use crate::*;
use std::marker::PhantomData;

//todo: just use crate::sync::Act's ?

pub unsafe trait ActNext <'o, SS:YesNo, BY: RefOrVal> : 'o
{
    fn call(&self, v: BY::V);
    fn stopped(&self) -> bool { false }
}

pub unsafe trait ActEc<'o, SS:YesNo> : 'o
{
    fn call_once(self, e: Option<RxError>);
}

pub unsafe trait ActEcBox<'o, SS:YesNo> : 'o
{
    fn call_box(self: Box<Self>, e: Option<RxError>);
}

unsafe impl<'o, SS:YesNo, A: ActEc<'o, SS>>
ActEcBox<'o, SS>
for A
{
    fn call_box(self: Box<Self>, e:  Option<RxError>) { self.call_once(e) }
}

unsafe impl<'o, V, R, F: Fn(V)->R+'o>
ActNext<'o, NO, Val<V>>
for F
{
    fn call(&self, by: V) { self.call((by,)); }
}

unsafe impl<'o, V, R, F: Fn(&V)->R+'o>
ActNext<'o, NO, Ref<V>>
for F
{
    fn call(&self, by: *const V) { self.call((unsafe{ &*by },)); }
}

unsafe impl<'o, V, R, F: Fn(V)->R+'o+Send+Sync>
ActNext<'o, YES, Val<V>>
for F
{
    fn call(&self, by: V) { self.call((by,)); }
}

unsafe impl<'o, V, R, F: Fn(&V)->R+'o+Send+Sync>
ActNext<'o, YES, Ref<V>>
for F
{
    fn call(&self, by: *const V) { self.call((unsafe { &*by },)); }
}

unsafe impl<'o, SS:YesNo, By: RefOrVal>
ActNext<'o, SS, By>
for ()
{
    fn call(&self, _by: By::V) { }
}

unsafe impl<'o, SS:YesNo, BY: RefOrVal, N: ActNext<'o, SS, BY>, STOP: Act<SS, (), bool>+'o>
ActNext<'o, SS, BY>
for (N, STOP)
{
    #[inline(always)]fn call(&self, v: BY::V) { self.0.call(v); }
    #[inline(always)]fn stopped(&self) -> bool {self.1.call(()) }
}


unsafe impl<'o, SS:YesNo, R, F:FnOnce(Option<RxError>)->R+'o>
ActEc<'o, SS>
for F
{
    fn call_once(self, e: Option<RxError>) { self(e); }
}


unsafe impl<'o, SS:YesNo, BY:RefOrVal+'o>
ActNext<'o, SS, BY>
for Box<ActNext<'o, SS, BY>>
{
    fn call(&self, v: BY::V) { Box::as_ref(self).call(v) }
    fn stopped(&self) -> bool { Box::as_ref(self).stopped() }
}

unsafe impl<'o, SS:YesNo>
ActEc<'o, SS>
for Box<ActEcBox<'o, SS>>
{
    fn call_once(self, e: Option<RxError>) { self.call_box(e) }
}

unsafe impl<'o, SS:YesNo>
ActEc<'o, SS>
for ()
{
    fn call_once(self, _e: Option<RxError>) { }
}


//todo: ergonomics

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

unsafe impl<'o, SS:YesNo, By: RefOrVal, A: ActNext<'o, SS, By>>
Ssmark<SS>
for SSActNextWrap<SS, By, A> {}

unsafe impl<'o, SS:YesNo, By: RefOrVal+'o, A: ActNext<'o, SS, By>>
ActNext<'o, SS, By>
for SSActNextWrap<SS, By, A>
{
    #[inline(always)] fn call(&self, v: <By as RefOrVal>::V) { self.next.call(v) }
    #[inline(always)] fn stopped(&self) -> bool { self.next.stopped() }
}


pub struct SSActEcWrap<A>
{
    pub ec: A,
}

impl<A> SSActEcWrap<A>
{
    pub fn new<'o, SS:YesNo>(ec: A) -> Self where A: ActEc<'o, SS>
    { SSActEcWrap{ ec } }

    pub fn into_inner(self) -> A { self.ec }
}

unsafe impl<'o, SS:YesNo, A: ActEc<'o, SS>>
Ssmark<SS>
for SSActEcWrap<A> {}




//todo: rename & organize

unsafe impl<'o, SS:YesNo, N: Ssmark<SS>+'o, Caps: Ssmark<SS>+'o, FBy: RefOrVal+'o>
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
pub fn forward_next<'o, SS:YesNo, By: RefOrVal+'o, N: Ssmark<SS>+'o, Caps: Ssmark<SS>+'o>
(next:N, captures: Caps, fnext: fn(&N, &Caps, By), fstop: fn(&N, &Caps)->bool)
 -> SsForward<SS, (N, Caps, fn(&N, &Caps, By), fn(&N, &Caps) ->bool)>
{
    SsForward::new((next, captures, fnext, fstop))
}


#[inline(always)]
pub fn forward_ec<'o, SS:YesNo, Caps: Ssmark<SS>+'o>
(captures: Caps, fec: fn(Caps, Option<RxError>))
 -> SsForward<SS, (Caps, fn(Caps, Option<RxError>))>
{
    SsForward::new((captures, fec))
}


unsafe impl<'o, SS:YesNo, Caps: Ssmark<SS>+'o>
ActEc<'o, SS>
for SsForward<SS, (Caps, fn(Caps, Option<RxError>))>
{
    #[inline(always)]
    fn call_once(self, e: Option<RxError>)
    {
        let (caps, fec) = self.captures;
        fec(caps,  e)
    }
}


#[inline(always)]
pub fn forward_act<'o, SS:YesNo, By: RefOrVal+'o, Caps: Ssmark<SS>+'o>
(captures: Caps, fact: fn(&Caps, By))
 -> SsForward<SS, (Caps, fn(&Caps, By))>
{
    SsForward::new((captures, fact))
}


unsafe impl<SS:YesNo, By: RefOrVal, Caps: Ssmark<SS>>
Act<SS, By>
for SsForward<SS, (Caps, fn(&Caps, By))>
{
    #[inline(always)]
    fn call(&self, v: By::V)
    {
        let (caps, fec) = &self.captures;
        fec(caps,  unsafe { By::from_v(v) })
    }
}