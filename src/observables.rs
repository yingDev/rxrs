use crate::*;
use std::sync::Arc;
use std::rc::Rc;

impl<'o, O, SS:YesNo, VBy:RefOrVal, EBy: RefOrVal> Observable<'o, SS, VBy, EBy> for Rc<O>
    where O: Observable<'o, SS, VBy, EBy>
{
    #[inline(always)]
    fn sub(&self, next: impl ActNext<'o, SS, VBy>, ec: impl ActEc<'o, SS, EBy>) -> Unsub<'o, SS> where Self: Sized
    { Rc::as_ref(self).sub(next, ec) }

    #[inline(always)]
    fn sub_dyn(&self, next: Box<ActNext<'o, SS, VBy>>, ec: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>
    { Rc::as_ref(self).sub_dyn(next, ec) }
}

impl<'o, O, SS:YesNo, VBy:RefOrVal, EBy: RefOrVal> Observable<'o, SS, VBy, EBy> for Arc<O>
    where O: Observable<'o, SS, VBy, EBy>
{
    #[inline(always)]
    fn sub(&self, next: impl ActNext<'o, SS, VBy>, ec: impl ActEc<'o, SS, EBy>) -> Unsub<'o, SS> where Self: Sized
    { Arc::as_ref(self).sub(next, ec) }

    #[inline(always)]
    fn sub_dyn(&self, next: Box<ActNext<'o, SS, VBy>>, ec: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>
    { Arc::as_ref(self).sub_dyn(next, ec) }
}


impl<'o, O, SS:YesNo, VBy:RefOrVal, EBy: RefOrVal> Observable<'o, SS, VBy, EBy> for Box<O>
    where O: Observable<'o, SS, VBy, EBy>
{
    #[inline(always)]
    fn sub(&self, next: impl ActNext<'o, SS, VBy>, ec: impl ActEc<'o, SS, EBy>) -> Unsub<'o, SS> where Self: Sized
    { Box::as_ref(self).sub(next, ec) }

    #[inline(always)]
    fn sub_dyn(&self, next: Box<ActNext<'o, SS, VBy>>, ec: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>
    { Box::as_ref(self).sub_dyn(next, ec) }
}

impl<'o, SS:YesNo, VBy:RefOrVal, EBy: RefOrVal> Observable<'o, SS, VBy, EBy> for Box<dyn Observable<'o, SS, VBy, EBy>>
{
    #[inline(always)]
    fn sub_dyn(&self, next: Box<ActNext<'o, SS, VBy>>, ec: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>
    { Box::as_ref(self).sub_dyn(next, ec) }
}
