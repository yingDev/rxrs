use std::sync::Arc;
use crate::*;

pub unsafe trait IntoSendSync
{
    type Output;
    fn into_ss(self) -> Self::Output;
}

unsafe impl<BY: RefOrVal> IntoSendSync for Arc<ActNext<'static, YES, BY>>
{
    type Output = Arc<ActNext<'static, YES, BY> + Send+Sync>;
    fn into_ss(self) -> Self::Output { unsafe{ ::std::mem::transmute( self )} }
}

unsafe impl<BY: RefOrVal> IntoSendSync for Box<ActNext<'static, YES, BY>>
{
    type Output = Box<ActNext<'static, YES, BY> + Send+Sync>;
    fn into_ss(self) -> Self::Output { unsafe{ ::std::mem::transmute( self )} }
}

unsafe impl<BY: RefOrVal> IntoSendSync for Box<ActEcBox<'static, YES, BY>>
{
    type Output = Box<ActEcBox<'static, YES, BY> + Send+Sync>;
    fn into_ss(self) -> Self::Output { unsafe{ ::std::mem::transmute( self )} }
}