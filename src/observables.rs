use crate::*;
use std::sync::Arc;
use std::rc::Rc;

impl<'o, O: Observable<'o, SS>, SS:YesNo>
Observable<'o, SS>
for Rc<O>
{
    type By = O::By;
    type EBy = O::EBy;

    #[inline(always)]
    fn sub(&self, next: impl ActNext<'o, SS, Self::By>, ec: impl ActEc<'o, SS, Self::EBy>) -> Unsub<'o, SS> where Self: Sized
    { Rc::as_ref(self).sub(next, ec) }

    #[inline(always)]
    fn sub_dyn(&self, next: Box<ActNext<'o, SS, Self::By>>, ec: Box<ActEcBox<'o, SS, Self::EBy>>) -> Unsub<'o, SS>
    { Rc::as_ref(self).sub_dyn(next, ec) }
}

impl<'o, O: Observable<'o, SS>, SS:YesNo>
Observable<'o, SS>
for Arc<O>
{
    type By = O::By;
    type EBy = O::EBy;

    #[inline(always)]
    fn sub(&self, next: impl ActNext<'o, SS, Self::By>, ec: impl ActEc<'o, SS, Self::EBy>) -> Unsub<'o, SS> where Self: Sized
    { Arc::as_ref(self).sub(next, ec) }

    #[inline(always)]
    fn sub_dyn(&self, next: Box<ActNext<'o, SS, Self::By>>, ec: Box<ActEcBox<'o, SS, Self::EBy>>) -> Unsub<'o, SS>
    { Arc::as_ref(self).sub_dyn(next, ec) }
}


impl<'o, O: Observable<'o, SS>, SS:YesNo>
Observable<'o, SS>
for Box<O>
{
    type By = O::By;
    type EBy = O::EBy;

    #[inline(always)]
    fn sub(&self, next: impl ActNext<'o, SS, Self::By>, ec: impl ActEc<'o, SS, Self::EBy>) -> Unsub<'o, SS> where Self: Sized
    { Box::as_ref(self).sub(next, ec) }

    #[inline(always)]
    fn sub_dyn(&self, next: Box<ActNext<'o, SS, Self::By>>, ec: Box<ActEcBox<'o, SS, Self::EBy>>) -> Unsub<'o, SS>
    { Box::as_ref(self).sub_dyn(next, ec) }
}

impl<'o, SS:YesNo, By: RefOrVal, EBy: RefOrVal>
Observable<'o, SS>
for Box<dyn Observable<'o, SS, By=By, EBy=EBy>>
{
    type By = By;
    type EBy = EBy;

    #[inline(always)]
    fn sub_dyn(&self, next: Box<ActNext<'o, SS, Self::By>>, ec: Box<ActEcBox<'o, SS, Self::EBy>>) -> Unsub<'o, SS>
    { Box::as_ref(self).sub_dyn(next, ec) }
}
