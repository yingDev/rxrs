use crate::*;
use std::marker::PhantomData;

//todo: just use crate::sync::Act's ?

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

unsafe impl<'o, SS:YesNo, BY:RefOrVal, A: ActEc<'o, SS, BY>>
ActEcBox<'o, SS, BY>
for A
{
    fn call_box(self: Box<Self>, e: Option<BY::V>) { self.call_once(e) }
}

unsafe impl<'o, V, F: Fn(V)+'o>
ActNext<'o, NO, Val<V>>
for F
{
    fn call(&self, by: V) { self.call((by,)) }
}

unsafe impl<'o, V, F: Fn(&V)+'o>
ActNext<'o, NO, Ref<V>>
for F
{
    fn call(&self, by: *const V) { self.call((unsafe{ &*by },)) }
}

unsafe impl<'o, V, F: Fn(V)+'o+Send+Sync>
ActNext<'o, YES, Val<V>>
for F
{
    fn call(&self, by: V) { self.call((by,)) }
}

unsafe impl<'o, V, F: Fn(&V)+'o+Send+Sync>
ActNext<'o, YES, Ref<V>>
for F
{
    fn call(&self, by: *const V) { self.call((unsafe { &*by },)) }
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


unsafe impl<'o, SS:YesNo, E, F:FnOnce(Option<E>)+'o>
ActEc<'o, SS, Val<E>>
for F
{
    fn call_once(self, e: Option<E>) { self(e) }
}

unsafe impl<'o, SS:YesNo, E, F:FnOnce(Option<&E>)+'o>
ActEc<'o, SS, Ref<E>>
for F
{
    fn call_once(self, e: Option<*const E>) { self(e.map(|e| unsafe{ &*e })) }
}


unsafe impl<'o, SS:YesNo, BY:RefOrVal+'o>
ActNext<'o, SS, BY>
for Box<ActNext<'o, SS, BY>>
{
    fn call(&self, v: BY::V) { Box::as_ref(self).call(v) }
    fn stopped(&self) -> bool { Box::as_ref(self).stopped() }
}

unsafe impl<'o, SS:YesNo, BY:RefOrVal+'o>
ActEc<'o, SS, BY>
for Box<ActEcBox<'o, SS, BY>>
{
    fn call_once(self, e: Option<BY::V>) { self.call_box(e) }
}

unsafe impl<'o, SS:YesNo, BY:RefOrVal+'o>
ActEc<'o, SS, BY>
for ()
{
    fn call_once(self, _e: Option<BY::V>) {}
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

unsafe impl<'o, SS:YesNo, By: RefOrVal, A: ActEc<'o, SS, By>>
Ssmark<SS>
for SSActEcWrap<By, A> {}




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
pub fn forward_ec<'o, SS:YesNo, By: RefOrVal+'o, Caps: Ssmark<SS>+'o>
(captures: Caps, fec: fn(Caps, Option<By>))
 -> SsForward<SS, (Caps, fn(Caps, Option<By>))>
{
    SsForward::new((captures, fec))
}


unsafe impl<'o, SS:YesNo, By: RefOrVal+'o, Caps: Ssmark<SS>+'o>
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