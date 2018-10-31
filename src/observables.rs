use crate::*;
use std::sync::Arc;
use std::rc::Rc;

impl<'o, By: RefOrVal, EBy: RefOrVal, O: Observable<'o, SS, By, EBy>, SS:YesNo>
Observable<'o, SS, By, EBy>
for Rc<O>
{
    #[inline(always)]
    fn sub(&self, next: impl ActNext<'o, SS, By>, ec: impl ActEc<'o, SS, EBy>) -> Unsub<'o, SS> where Self: Sized
    { Rc::as_ref(self).sub(next, ec) }

    #[inline(always)]
    fn sub_dyn(&self, next: Box<ActNext<'o, SS, By>>, ec: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>
    { Rc::as_ref(self).sub_dyn(next, ec) }
}

impl<'o, By: RefOrVal, EBy: RefOrVal, O: Observable<'o, SS, By, EBy>, SS:YesNo>
Observable<'o, SS, By, EBy>
for Arc<O>
{
    #[inline(always)]
    fn sub(&self, next: impl ActNext<'o, SS, By>, ec: impl ActEc<'o, SS, EBy>) -> Unsub<'o, SS> where Self: Sized
    { Arc::as_ref(self).sub(next, ec) }

    #[inline(always)]
    fn sub_dyn(&self, next: Box<ActNext<'o, SS, By>>, ec: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>
    { Arc::as_ref(self).sub_dyn(next, ec) }
}


impl<'o, By: RefOrVal, EBy: RefOrVal, O: Observable<'o, SS, By, EBy>, SS:YesNo>
Observable<'o, SS, By, EBy>
for Box<O>
{
    #[inline(always)]
    fn sub(&self, next: impl ActNext<'o, SS, By>, ec: impl ActEc<'o, SS, EBy>) -> Unsub<'o, SS> where Self: Sized
    { Box::as_ref(self).sub(next, ec) }

    #[inline(always)]
    fn sub_dyn(&self, next: Box<ActNext<'o, SS, By>>, ec: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>
    { Box::as_ref(self).sub_dyn(next, ec) }
}

impl<'s, 'o, By: RefOrVal, EBy: RefOrVal, SS:YesNo>
Observable<'o, SS, By, EBy>
for Box<dyn Observable<'o, SS, By, EBy>+'s>
{
    #[inline(always)]
    fn sub(&self, next: impl ActNext<'o, SS, By>, ec: impl ActEc<'o, SS, EBy>) -> Unsub<'o, SS> where Self: Sized
    { self.sub_dyn(box next, box ec) }

    #[inline(always)]
    fn sub_dyn(&self, next: Box<ActNext<'o, SS, By>>, ec: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>
    { Box::as_ref(self).sub_dyn(next, ec) }
}
