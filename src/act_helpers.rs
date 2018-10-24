use crate::*;
use crate::any_send_sync::AnySendSync;
use std::ops::Deref;
use std::cell::UnsafeCell;

//todo
pub fn sendsync_act<'o, A>(act: impl Act<YES, A>+'o) -> impl Act<YES, A>+'o + Send + Sync
{
    let a = unsafe{ AnySendSync::new(act) };
    move |v:A| a.call(v)
}
//todo
pub fn sendsync_actonce<'o, A>(act: impl ActOnce<YES, A>+'o) -> impl ActOnce<YES, A>+'o + Send + Sync
{
    let a = unsafe{ AnySendSync::new(act) };
    move |v:A| a.into_inner().call_once(v)
}

pub fn sendsync_next<'o, BY: RefOrVal>(next: impl ActNext<'o, YES, BY>) -> impl ActNext<'o, YES, BY> + Send + Sync
{
    let a = unsafe{ AnySendSync::new(next) };
    move |v:By<BY>| a.call(v)
}

pub fn sendsync_ec<'o, BY: RefOrVal>(act: impl ActEc<'o, YES, BY>) -> impl ActEc<'o, YES, BY> + Send + Sync
{
    let a = unsafe{ AnySendSync::new(act) };
    move |e:Option<By<BY>>| a.into_inner().call_once(e)
}

pub fn sendsync_next_box<'o, BY: RefOrVal+'o>(next: Box<ActNext<'o, YES, BY>>) -> Box<ActNext<'o, YES, BY>+Send+Sync>
{
    unsafe{ ::std::mem::transmute(next) }
}

pub fn sendsync_ec_box<'o, BY: RefOrVal+'o>(ec: Box<ActEcBox<'o, YES, BY>>) -> Box<ActEcBox<'o, YES, BY>+Send+Sync>
{
    unsafe{ ::std::mem::transmute(ec) }
}


pub fn dyn_to_impl_next<'o, BY: RefOrVal+'o>(next: Box<ActNext<'o, NO, BY>>) -> impl ActNext<'o, NO, BY>
{
    move |v: By<BY>| next.call(v)
}

pub fn dyn_to_impl_next_ss<'o, BY: RefOrVal+'o>(next: Box<ActNext<'o, YES, BY>>) -> impl ActNext<'o, YES, BY>
{
    let next = sendsync_next_box(next);
    move |v: By<BY>| next.call(v)
}

pub fn dyn_to_impl_ec<'o, BY: RefOrVal+'o>(ec: Box<ActEcBox<'o, NO, BY>>) -> impl ActEc<'o, NO, BY>
{
    move |e: Option<By<BY>>| ec.call_box(e)
}

pub fn dyn_to_impl_ec_ss<'o, BY: RefOrVal+'o>(ec: Box<ActEcBox<'o, YES, BY>>) -> impl ActEc<'o, YES, BY>
{
    let ec = sendsync_ec_box(ec);
    move |e: Option<By<BY>>| ec.call_box(e)
}