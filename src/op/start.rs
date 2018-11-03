use crate::*;
use std::sync::Mutex;
use std::marker::PhantomData;
use std::error::Error;
use std::fmt::Display;
use std::sync::Arc;

pub mod StartOpType {
    pub struct ONCE;
    pub struct CLONE;
    pub struct FN;
    pub struct VAL_REF;
    pub struct REF;
}

use self::StartOpType::*;


pub struct StartOp<Src, V, TYPE>
{
    src: Src,
    v: V,
    PhantomData: PhantomData<TYPE>
}

pub trait ObsStartValOp<'o, V:'o, SS:YesNo> : Sized
{
    fn start_once(self, v: V) -> StartOp<Self, Mutex<Option<V>>, ONCE>;
    fn start(self, v: V) -> StartOp<Self, V, CLONE> where V: Clone+'o;
    fn start_fn(self, f: V) -> StartOp<Self, V, FN> where V: Fn()->V+'o;
}

pub trait ObsStartRefOp<'o, V:'o, SS:YesNo> : Sized
{
    fn start(self, v: V) -> StartOp<Self, V, VAL_REF>;
    fn start_ref(self, v: &'o V) -> StartOp<Self, &'o V, REF>;
}

impl<'o, V:'o, SS:YesNo, Src: Observable<'o, SS, Val<V>>>
ObsStartValOp<'o, V, SS>
for Src
{
    fn start_once(self, v: V) -> StartOp<Self, Mutex<Option<V>>, ONCE>
    { StartOp{ src: self, v: Mutex::new(Some(v)), PhantomData } }

    fn start(self, v: V) -> StartOp<Self, V, CLONE> where V: Clone + 'o
    { StartOp{ src: self, v, PhantomData} }

    fn start_fn(self, f: V) -> StartOp<Self, V, FN> where V: Fn() -> V + 'o
    { StartOp{ src: self, v: f, PhantomData} }
}

impl<'o, V:'o, SS:YesNo, Src: Observable<'o, SS, Ref<V>>>
ObsStartRefOp<'o, V, SS>
for Src
{
    fn start(self, v: V) -> StartOp<Self, V, VAL_REF>
    { StartOp{ src: self, v, PhantomData} }

    fn start_ref(self, v: &'o V) -> StartOp<Self, &'o V, REF>
    { StartOp{ src: self, v, PhantomData} }
}

impl<'o, V:'o, SS:YesNo, Src: Observable<'o, SS, Val<V>>+'o>
Observable<'o, SS, Val<V>>
for StartOp<Src, Mutex<Option<V>>, ONCE>
{
    fn subscribe(&self, next: impl ActNext<'o, SS, Val<V>>, err_or_comp: impl ActEc<'o, SS>) -> Unsub<'o, SS> where Self: Sized {
        if ! next.stopped() {
            if let Some(v) = self.v.lock().unwrap().take() {
                next.call(v);
                if ! next.stopped() {
                    return self.src.subscribe(next, err_or_comp);
                }

            } else {
                err_or_comp.call_once(Some(RxError::simple(None, "value consumed")));
            }
        }

        Unsub::done()
    }

    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, Val<V>>>, err_or_comp: Box<ActEcBox<'o, SS>>) -> Unsub<'o, SS>
    { self.subscribe(next, err_or_comp) }
}