use std::sync::Arc;
use crate::*;

pub unsafe trait IntoSendSync
{
    type Output;
    fn into_sendsync(self) -> Self::Output;
}

unsafe impl<BY: RefOrVal> IntoSendSync for Arc<for<'x> Act<YES, By<'x, BY>>>
{
    type Output = Arc<for<'x> Act<YES, By<'x, BY>> + Send+Sync>;
    fn into_sendsync(self) -> Self::Output { unsafe{ ::std::mem::transmute( self )} }
}

unsafe impl<BY: RefOrVal> IntoSendSync for Box<for<'x> Act<YES, By<'x, BY>>>
{
    type Output = Box<for<'x> Act<YES, By<'x, BY>> + Send+Sync>;
    fn into_sendsync(self) -> Self::Output { unsafe{ ::std::mem::transmute( self )} }
}

unsafe impl<BY: RefOrVal> IntoSendSync for Box<for<'x> ActBox<YES, By<'x, BY>>>
{
    type Output = Box<for<'x> ActBox<YES, By<'x, BY>> + Send+Sync>;
    fn into_sendsync(self) -> Self::Output { unsafe{ ::std::mem::transmute( self )} }
}