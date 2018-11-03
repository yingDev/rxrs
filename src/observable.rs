use crate::*;

pub trait Observable<'o, SS:YesNo, By: RefOrVal>
{
    fn subscribe    (&self, next: impl ActNext<'o, SS, By>, err_or_comp: impl ActEc<'o, SS>   ) -> Unsub<'o, SS> where Self: Sized;
    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, By>>, err_or_comp: Box<ActEcBox<'o, SS>>) -> Unsub<'o, SS>;

    fn into_dyn<'s>(self) -> DynObservable<'s, 'o, SS, By> where Self: Sized+'s { DynObservable::new(self) }
}

#[derive(Clone)]
pub struct DynObservable<'s, 'o, SS:YesNo, By: RefOrVal>
{
    pub(crate) src: std::sync::Arc<Observable<'o, SS, By> + 's>,
}

unsafe impl<'s, 'o, By: RefOrVal> Send for DynObservable<'s, 'o, YES, By>{}
unsafe impl<'s, 'o, By: RefOrVal> Sync for DynObservable<'s, 'o, YES, By>{}

impl<'s, 'o, SS:YesNo, By: RefOrVal> DynObservable<'s, 'o, SS, By>
{
    pub fn new(src: impl Observable<'o, SS, By>+'s) -> Self { DynObservable{ src: std::sync::Arc::new(src) }}
    pub fn from_arc(src: std::sync::Arc<Observable<'o, SS, By>+'s>) -> Self { DynObservable{ src }}
    pub fn from_box(src: Box<Observable<'o, SS, By>+'s>) -> Self { DynObservable{ src: src.into() }}

    pub fn to_impl(&self) -> std::sync::Arc<Observable<'o, SS, By>+'s> { self.src.clone() }

    pub fn subscribe(&self, next: impl ActNext<'o, SS, By>, err_or_comp: impl ActEc<'o, SS>) -> Unsub<'o, SS> where Self: Sized
    { self.src.subscribe_dyn(box next, box err_or_comp) }

    pub fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, By>>, err_or_comp: Box<ActEcBox<'o, SS>>) -> Unsub<'o, SS>
    { self.src.subscribe_dyn(next, err_or_comp) }
}



impl<'o, By: RefOrVal, O: Observable<'o, SS, By>, SS:YesNo>
Observable<'o, SS, By>
for std::rc::Rc<O>
{
    #[inline(always)]
    fn subscribe(&self, next: impl ActNext<'o, SS, By>, ec: impl ActEc<'o, SS>) -> Unsub<'o, SS> where Self: Sized
    { std::rc::Rc::as_ref(self).subscribe(next, ec) }

    #[inline(always)]
    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, By>>, ec: Box<ActEcBox<'o, SS>>) -> Unsub<'o, SS>
    { std::rc::Rc::as_ref(self).subscribe_dyn(next, ec) }
}

impl<'o, By: RefOrVal, O: Observable<'o, SS, By>, SS:YesNo>
Observable<'o, SS, By>
for std::sync::Arc<O>
{
    #[inline(always)]
    fn subscribe(&self, next: impl ActNext<'o, SS, By>, ec: impl ActEc<'o, SS>) -> Unsub<'o, SS> where Self: Sized
    { std::sync::Arc::as_ref(self).subscribe(next, ec) }

    #[inline(always)]
    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, By>>, ec: Box<ActEcBox<'o, SS>>) -> Unsub<'o, SS>
    { std::sync::Arc::as_ref(self).subscribe_dyn(next, ec) }

    fn into_dyn<'x>(self) -> DynObservable<'x, 'o, SS, By> where Self: Sized+'x
    { DynObservable::from_arc(self) }
}


impl<'o, By: RefOrVal, O: Observable<'o, SS, By>, SS:YesNo>
Observable<'o, SS, By>
for Box<O>
{
    #[inline(always)]
    fn subscribe(&self, next: impl ActNext<'o, SS, By>, ec: impl ActEc<'o, SS>) -> Unsub<'o, SS> where Self: Sized
    { Box::as_ref(self).subscribe(next, ec) }

    #[inline(always)]
    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, By>>, ec: Box<ActEcBox<'o, SS>>) -> Unsub<'o, SS>
    { Box::as_ref(self).subscribe_dyn(next, ec) }

    fn into_dyn<'x>(self) -> DynObservable<'x, 'o, SS, By> where Self: Sized+'x
    { DynObservable::from_box(self) }
}

impl<'s, 'o, SS:YesNo, By: RefOrVal>
Observable<'o, SS, By>
for Box<dyn Observable<'o, SS, By>+'s>
{
    #[inline(always)]
    fn subscribe(&self, next: impl ActNext<'o, SS, By>, ec: impl ActEc<'o, SS>) -> Unsub<'o, SS> where Self: Sized
    { self.subscribe_dyn(box next, box ec) }

    #[inline(always)]
    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, By>>, ec: Box<ActEcBox<'o, SS>>) -> Unsub<'o, SS>
    { Box::as_ref(self).subscribe_dyn(next, ec) }
}

impl<'s, 'o, SS:YesNo, By: RefOrVal>
Observable<'o, SS, By>
for std::sync::Arc<dyn Observable<'o, SS, By>+'s>
{
    #[inline(always)]
    fn subscribe(&self, next: impl ActNext<'o, SS, By>, ec: impl ActEc<'o, SS>) -> Unsub<'o, SS> where Self: Sized
    { self.subscribe_dyn(box next, box ec) }

    #[inline(always)]
    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, By>>, ec: Box<ActEcBox<'o, SS>>) -> Unsub<'o, SS>
    { std::sync::Arc::as_ref(self).subscribe_dyn(next, ec) }

    fn into_dyn<'x>(self) -> DynObservable<'x, 'o, SS, By> where Self: Sized+'x
    { DynObservable::from_arc(self) }
}