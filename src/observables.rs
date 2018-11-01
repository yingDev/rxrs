use crate::*;
use std::sync::Arc;
use std::rc::Rc;

impl<'o, By: RefOrVal, EBy: RefOrVal, O: Observable<'o, SS, By, EBy>, SS:YesNo>
Observable<'o, SS, By, EBy>
for Rc<O>
{
    #[inline(always)]
    fn subscribe(&self, next: impl ActNext<'o, SS, By>, ec: impl ActEc<'o, SS, EBy>) -> Unsub<'o, SS> where Self: Sized
    { Rc::as_ref(self).subscribe(next, ec) }

    #[inline(always)]
    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, By>>, ec: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>
    { Rc::as_ref(self).subscribe_dyn(next, ec) }
}

impl<'o, By: RefOrVal, EBy: RefOrVal, O: Observable<'o, SS, By, EBy>, SS:YesNo>
Observable<'o, SS, By, EBy>
for Arc<O>
{
    #[inline(always)]
    fn subscribe(&self, next: impl ActNext<'o, SS, By>, ec: impl ActEc<'o, SS, EBy>) -> Unsub<'o, SS> where Self: Sized
    { Arc::as_ref(self).subscribe(next, ec) }

    fn into_dyn<'x>(self) -> DynObservable<'x, 'o, SS, By, EBy> where Self: Sized+'x
    { DynObservable::from_arc(self) }

    #[inline(always)]
    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, By>>, ec: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>
    { Arc::as_ref(self).subscribe_dyn(next, ec) }
}


impl<'o, By: RefOrVal, EBy: RefOrVal, O: Observable<'o, SS, By, EBy>, SS:YesNo>
Observable<'o, SS, By, EBy>
for Box<O>
{
    #[inline(always)]
    fn subscribe(&self, next: impl ActNext<'o, SS, By>, ec: impl ActEc<'o, SS, EBy>) -> Unsub<'o, SS> where Self: Sized
    { Box::as_ref(self).subscribe(next, ec) }

    fn into_dyn<'x>(self) -> DynObservable<'x, 'o, SS, By, EBy> where Self: Sized+'x
    { DynObservable::from_box(self) }

    #[inline(always)]
    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, By>>, ec: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>
    { Box::as_ref(self).subscribe_dyn(next, ec) }
}

impl<'s, 'o, SS:YesNo, By: RefOrVal, EBy: RefOrVal>
Observable<'o, SS, By, EBy>
for Box<dyn Observable<'o, SS, By, EBy>+'s>
{
    #[inline(always)]
    fn subscribe(&self, next: impl ActNext<'o, SS, By>, ec: impl ActEc<'o, SS, EBy>) -> Unsub<'o, SS> where Self: Sized
    { self.subscribe_dyn(box next, box ec) }

    #[inline(always)]
    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, By>>, ec: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>
    { Box::as_ref(self).subscribe_dyn(next, ec) }
}


impl<'s, 'o, SS:YesNo, By: RefOrVal, EBy: RefOrVal>
Observable<'o, SS, By, EBy>
for Arc<dyn Observable<'o, SS, By, EBy>+'s>
{
    #[inline(always)]
    fn subscribe(&self, next: impl ActNext<'o, SS, By>, ec: impl ActEc<'o, SS, EBy>) -> Unsub<'o, SS> where Self: Sized
    { self.subscribe_dyn(box next, box ec) }

    fn into_dyn<'x>(self) -> DynObservable<'x, 'o, SS, By, EBy> where Self: Sized+'x
    { DynObservable::from_arc(self) }

    #[inline(always)]
    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, By>>, ec: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>
    { Arc::as_ref(self).subscribe_dyn(next, ec) }
}