use crate::*;
use std::ops::Deref;

/// Send+Sync Mark
pub unsafe trait Ssmark<SS:YesNo> : Sized { }

unsafe impl<SS:YesNo> Ssmark<SS> for () {}
unsafe impl <SS:YesNo, A: Ssmark<SS>> Ssmark<SS> for (A,) {}
unsafe impl <SS:YesNo, A: Ssmark<SS>, B: Ssmark<SS>> Ssmark<SS> for (A, B) {}
unsafe impl <SS:YesNo, A: Ssmark<SS>, B: Ssmark<SS>, C: Ssmark<SS>> Ssmark<SS> for (A, B, C) {}
unsafe impl <SS:YesNo, A: Ssmark<SS>, B: Ssmark<SS>, C: Ssmark<SS>, D: Ssmark<SS>> Ssmark<SS> for (A, B, C, D) {}
unsafe impl <SS:YesNo, A: Ssmark<SS>, B: Ssmark<SS>, C: Ssmark<SS>, D: Ssmark<SS>, E: Ssmark<SS>> Ssmark<SS> for (A, B, C, D, E) {}
unsafe impl <SS:YesNo, A: Ssmark<SS>, B: Ssmark<SS>, C: Ssmark<SS>, D: Ssmark<SS>, E: Ssmark<SS>, F: Ssmark<SS>> Ssmark<SS> for (A, B, C, D, E, F) {}
unsafe impl <SS:YesNo, A: Ssmark<SS>, B: Ssmark<SS>, C: Ssmark<SS>, D: Ssmark<SS>, E: Ssmark<SS>, F: Ssmark<SS>, G: Ssmark<SS>> Ssmark<SS> for (A, B, C, D, E, F, F, G) {}

unsafe impl<SS:YesNo, A, B, C, R> Ssmark<SS> for fn(A, B, C) -> R {}
unsafe impl<SS:YesNo, A, B, R> Ssmark<SS> for fn(A, B) -> R {}
unsafe impl<SS:YesNo, A, B, R> Ssmark<SS> for fn(&A, &B) -> R {}
unsafe impl<SS:YesNo, A, B, R> Ssmark<SS> for fn(&A, B) -> R {}
unsafe impl<SS:YesNo, A, B, C, R> Ssmark<SS> for fn(&A, &B, C) -> R {}

pub struct SSWrap<V:Send+Sync>(V);
unsafe impl<SS:YesNo, V: Send+Sync> Ssmark<SS> for SSWrap<V> {}
impl<V: Send+Sync> SSWrap<V>
{
    pub fn new(v: V) -> Self { SSWrap(v) }
    pub fn into_inner(self) -> V { self.0 }
}
impl<V: Send+Sync> Deref for SSWrap<V>
{
    type Target = V;
    fn deref(&self) -> &V { &self.0 }
}
