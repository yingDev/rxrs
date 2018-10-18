use std::marker::PhantomData;
use std::ops::Deref;

pub unsafe trait RefOrVal {}


pub struct Ref<V>(*const V);
pub struct Val<V>(V);

unsafe impl<V> RefOrVal for Ref<V>{}
unsafe impl<V> RefOrVal for Val<V>{}

pub struct By<'a, T: RefOrVal>
{
    t: T,
    PhantomData:PhantomData<&'a ()>
}

//ok?
unsafe impl<'a, V: Send> Send for Ref<V>{}
unsafe impl<'a, V: Sync> Sync for Ref<V>{}

impl<'a, V> By<'a, Ref<V>>
{
    #[inline(always)]
    pub fn r(r: &'a V) -> By<'a, Ref<V>> { By{ t: Ref(r), PhantomData } }
}

impl<'a, V> By<'a, Val<V>>
{
    #[inline(always)]
    pub fn v(v: V) -> By<'a, Val<V>> { By{ t: Val(v), PhantomData } }
    #[inline(always)]
    pub fn val(self) -> V { self.t.0 }
}

impl<'a, V> Deref for By<'a, Ref<V>>
{
    type Target = V;
    #[inline(always)] fn deref(&self) -> &V { unsafe { std::mem::transmute(self.t.0) } }
}

impl<'a, V> Deref for By<'a, Val<V>>
{
    type Target = V;
    #[inline(always)] fn deref(&self) -> &V { &self.t.0 }
}
