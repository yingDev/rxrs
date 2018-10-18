use std::sync::Arc;
use crate::*;
use std::marker::PhantomData;
use std::ops::Deref;

pub unsafe trait IntoSendSync<BY>
{
    type Output;
    fn into_ss(self) -> Self::Output;
}

pub struct ActSendSync<T, BY: RefOrVal>
{
    act: T,
    PhantomData: PhantomData<BY>
}

impl<T, BY: RefOrVal> ActSendSync<T, BY>
{
    pub fn wrap_next(act: T) -> ActSendSync<T, BY> where T : ActNext<'static, YES, BY> { ActSendSync { act, PhantomData } }
    pub fn wrap_ec(act: T) -> ActSendSync<T, BY> where T : ActEc<'static, YES, BY> { ActSendSync { act, PhantomData } }
}


impl<T, BY: RefOrVal> Deref for ActSendSync<T, BY>
{
    type Target = T;
    fn deref(&self) -> &T { &self.act }
}

unsafe impl<T, BY: RefOrVal> Send for ActSendSync<T, BY> {}
unsafe impl<T, BY: RefOrVal> Sync for ActSendSync<T, BY> {}

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