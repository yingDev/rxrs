use std::sync::Arc;
use crate::*;
use std::ops::Deref;

pub unsafe trait IntoSendSync<BY>
{
    type Output;
    fn into_ss(self) -> Self::Output;
}

pub struct ActSendSync<T>
{
    act: T
}

unsafe impl<T : ActOnce<YES, A>, A> ActOnce<YES, A> for ActSendSync<T>
{
    fn call_once(self, e: A) { self.act.call_once(e) }
}

unsafe impl<T : Act<YES, A>, A> Act<YES, A> for ActSendSync<T>
{
    fn call(&self, v: A) { self.act.call(v) }
}

impl<T> ActSendSync<T>
{
    pub fn wrap_next<BY: RefOrVal>(act: T) -> ActSendSync<T> where T : ActNext<'static, YES, BY> { ActSendSync { act } }
    pub fn wrap_ec<BY: RefOrVal>(act: T) -> ActSendSync<T> where T : ActEc<'static, YES, BY> { ActSendSync { act } }
    pub fn into_inner(self) -> T { self.act }
}


impl<T> Deref for ActSendSync<T>
{
    type Target = T;
    fn deref(&self) -> &T { &self.act }
}

unsafe impl<T> Send for ActSendSync<T> {}
unsafe impl<T> Sync for ActSendSync<T> {}

unsafe impl<BY: RefOrVal> IntoSendSync<BY> for Arc<ActNext<'static, YES, BY>>
{
    type Output = Arc<ActNext<'static, YES, BY> + Send+Sync>;
    fn into_ss(self) -> Self::Output { unsafe{ ::std::mem::transmute( self )} }
}

unsafe impl<BY: RefOrVal> IntoSendSync<BY> for Box<ActNext<'static, YES, BY>>
{
    type Output = Box<ActNext<'static, YES, BY> + Send+Sync>;
    fn into_ss(self) -> Self::Output { unsafe{ ::std::mem::transmute( self )} }
}

unsafe impl<BY: RefOrVal> IntoSendSync<BY> for Box<ActEcBox<'static, YES, BY>>
{
    type Output = Box<ActEcBox<'static, YES, BY> + Send+Sync>;
    fn into_ss(self) -> Self::Output { unsafe{ ::std::mem::transmute( self )} }
}