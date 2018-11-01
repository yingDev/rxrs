use crate::*;
use std::sync::Arc;
use std::rc::Rc;

pub trait Observable<'o, SS:YesNo, By: RefOrVal, EBy: RefOrVal=Ref<()>>
{
    fn subscribe(&self, next: impl ActNext<'o, SS, By>, err_or_comp: impl ActEc<'o, SS, EBy>) -> Unsub<'o, SS> where Self: Sized;
    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, By>>, err_or_comp: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>;

    fn into_dyn<'s>(self) -> DynObservable<'s, 'o, SS, By, EBy> where Self: Sized+'s { DynObservable::new(self) }
}

#[derive(Clone)]
pub struct DynObservable<'s, 'o, SS:YesNo, By: RefOrVal, EBy: RefOrVal>
{
    pub(crate) src: Arc<Observable<'o, SS, By, EBy> + 's>,
}

unsafe impl<'s, 'o, By: RefOrVal, EBy: RefOrVal> Send for DynObservable<'s, 'o, YES, By, EBy>{}
unsafe impl<'s, 'o, By: RefOrVal, EBy: RefOrVal> Sync for DynObservable<'s, 'o, YES, By, EBy>{}

impl<'s, 'o, SS:YesNo, By: RefOrVal, EBy: RefOrVal> DynObservable<'s, 'o, SS, By, EBy>
{
    pub fn new(src: impl Observable<'o, SS, By, EBy>+'s) -> Self { DynObservable{ src: Arc::new(src) }}
    pub fn from_arc(src: Arc<Observable<'o, SS, By, EBy>+'s>) -> Self { DynObservable{ src }}
    pub fn from_box(src: Box<Observable<'o, SS, By, EBy>+'s>) -> Self { DynObservable{ src: src.into() }}

    pub fn to_impl(&self) -> Arc<Observable<'o, SS, By, EBy>+'s> { self.src.clone() }

    pub fn subscribe(&self, next: impl ActNext<'o, SS, By>, err_or_comp: impl ActEc<'o, SS, EBy>) -> Unsub<'o, SS> where Self: Sized
    { self.src.subscribe_dyn(box next, box err_or_comp) }

    pub fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, By>>, err_or_comp: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>
    { self.src.subscribe_dyn(next, err_or_comp) }
}



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

    #[inline(always)]
    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, By>>, ec: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>
    { Arc::as_ref(self).subscribe_dyn(next, ec) }

    fn into_dyn<'x>(self) -> DynObservable<'x, 'o, SS, By, EBy> where Self: Sized+'x
    { DynObservable::from_arc(self) }
}


impl<'o, By: RefOrVal, EBy: RefOrVal, O: Observable<'o, SS, By, EBy>, SS:YesNo>
Observable<'o, SS, By, EBy>
for Box<O>
{
    #[inline(always)]
    fn subscribe(&self, next: impl ActNext<'o, SS, By>, ec: impl ActEc<'o, SS, EBy>) -> Unsub<'o, SS> where Self: Sized
    { Box::as_ref(self).subscribe(next, ec) }

    #[inline(always)]
    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, By>>, ec: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>
    { Box::as_ref(self).subscribe_dyn(next, ec) }

    fn into_dyn<'x>(self) -> DynObservable<'x, 'o, SS, By, EBy> where Self: Sized+'x
    { DynObservable::from_box(self) }
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

    #[inline(always)]
    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, By>>, ec: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>
    { Arc::as_ref(self).subscribe_dyn(next, ec) }

    fn into_dyn<'x>(self) -> DynObservable<'x, 'o, SS, By, EBy> where Self: Sized+'x
    { DynObservable::from_arc(self) }
}