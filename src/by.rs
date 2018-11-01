pub unsafe trait RefOrVal
{
    type V: Sized;
    type RAW: Sized;

    #[inline(always)] fn as_ref(&self) -> &Self::RAW;
    #[inline(always)] fn into_v(self) -> Self::V;
    #[inline(always)] unsafe fn from_v(v: Self::V) -> Self;
}

pub struct Ref<V>(*const V);
pub struct Val<V>(V);

unsafe impl<V> RefOrVal for Ref<V>
{
    type V = *const V;
    type RAW = V;

    #[inline(always)] fn as_ref(&self) -> &V { unsafe{ &*self.0 } }
    #[inline(always)] fn into_v(self) -> Self::V { self.0 }
    #[inline(always)] unsafe fn from_v(v: Self::V) -> Self { Ref(v) }
}
unsafe impl<V> RefOrVal for Val<V>
{
    type V = V;
    type RAW = V;

    #[inline(always)] fn as_ref(&self) -> &V { &self.0 }
    #[inline(always)] fn into_v(self) -> Self::V { self.0 }
    #[inline(always)] unsafe fn from_v(v: Self::V) -> Self { Val(v) }

}
unsafe impl RefOrVal for ()
{
    type V = ();
    type RAW = ();

    #[inline(always)] fn as_ref(&self) -> &() { &self }
    #[inline(always)] fn into_v(self) -> Self::V { self }
    #[inline(always)] unsafe fn from_v(v: Self::V) -> Self { () }
}
