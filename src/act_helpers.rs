use crate::*;
use crate::any_send_sync::AnySendSync;
use std::ops::Deref;
use std::cell::UnsafeCell;
use std::marker::PhantomData;

////todo
//pub fn sendsync_act<'o, A: RefOrVal, R>(act: impl Act<YES, A, R>+'o) -> impl Act<YES, A, R>+'o + Send + Sync
//{
//    let a = unsafe{ AnySendSync::new(act) };
//    move |v: A::V| a.call(v)
//}
////todo
//pub fn sendsync_actonce<'o, A: RefOrVal, R>(act: impl ActOnce<YES, A, R>+'o) -> impl ActOnce<YES, A, R>+'o + Send + Sync
//{
//    let a = unsafe{ AnySendSync::new(act) };
//    move |v: A::V| a.into_inner().call_once(v)
//}

//pub fn sendsync_next<'o, BY: RefOrValSSs>(next: impl ActNext<'o, YES, BY>) -> impl ActNext<'o, YES, BY> + Send + Sync
//{
//    //let a = unsafe{ AnySendSync::new(next) };
//    ForwardNext::new(next, |n,v:BY| n.call(v.into_v()), |s|s)
//    //move |v:BY| a.call(v)
//}
//
//pub fn sendsync_ec<'o, V, BY: RefOrValSSs<V=V>>(act: impl ActEc<'o, YES, BY>) -> impl ActEc<'o, YES, BY> + Send + Sync
//{
//    ForwardEc::new(act)
//
//    //let a = unsafe{ AnySendSync::new(act) };
//    //move |e:Option<V>| a.into_inner().call_once(e)
//}

pub fn sendsync_next_box<'o, BY: RefOrVal+'o>(next: Box<ActNext<'o, YES, BY>>) -> Box<ActNext<'o, YES, BY>+Send+Sync>
{
    unsafe{ ::std::mem::transmute(next) }
}

pub fn sendsync_ec_box<'o, BY: RefOrVal+'o>(ec: Box<ActEcBox<'o, YES, BY>>) -> Box<ActEcBox<'o, YES, BY>+Send+Sync>
{
    unsafe{ ::std::mem::transmute(ec) }
}


//pub fn dyn_to_impl_next<'o, BY: RefOrVal+'o>(next: Box<ActNext<'o, NO, BY>>) -> impl ActNext<'o, NO, BY>
//{
//    ForwardNext::new(next, |next, v| next.call(v), |s|s)
//}

//pub fn dyn_to_impl_next_ss<'o, BY: RefOrVal+'o>(next: Box<ActNext<'o, YES, BY>>) -> impl ActNext<'o, YES, BY>
//{
//    let next = sendsync_next_box(next);
//    move |v: BY| next.call(v)
//}
//
//pub fn dyn_to_impl_ec<'o, BY: RefOrVal+'o>(ec: Box<ActEcBox<'o, NO, BY>>) -> impl ActEc<'o, NO, BY>
//{
//    move |e: Option<BY>| ec.call_box(e)
//}
//
//pub fn dyn_to_impl_ec_ss<'o, BY: RefOrVal+'o>(ec: Box<ActEcBox<'o, YES, BY>>) -> impl ActEc<'o, YES, BY>
//{
//    let ec = sendsync_ec_box(ec);
//    move |e: Option<BY>| ec.call_box(e)
//}