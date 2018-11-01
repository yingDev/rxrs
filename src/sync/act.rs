use crate::*;
use std::marker::PhantomData;
use std::sync::Arc;

pub unsafe trait Act<SS:YesNo, A: RefOrVal, R=()>
{
    fn call(&self, v: A::V) -> R;
}

pub unsafe trait ActOnce<SS:YesNo, A: RefOrVal, R=()>
{
    fn call_once(self, e: A::V) -> R;
}

pub unsafe trait ActBox<SS:YesNo, A: RefOrVal, R=()>
{
    fn call_box(self: Box<Self>, e: A::V) -> R;
}

pub fn act_sendsync<SS:YesNo, By: RefOrVal, R>(act: impl Act<SS, By, R>) -> impl Act<SS, By, R> + Ssmark<SS>
{
    struct X<A>(A);
    unsafe impl<SS:YesNo, A> Ssmark<SS> for X<A>{}
    unsafe impl<SS: YesNo, By:RefOrVal, R, A: Act<SS, By, R>> Act<SS, By, R> for X<A>
    {
        #[inline(always)]
        fn call(&self, v: <By as RefOrVal>::V) -> R { self.0.call(v) }
    }

    X(act)
}


unsafe impl<'a, V, R, F: Fn(V) -> R+'a> Act<NO, Val<V>, R> for F
{
    #[inline(always)] fn call(&self, v: V) -> R { self(v) }
}

unsafe impl<'a, V, R, F: Fn(V)->R+Send+Sync+'a> Act<YES, Val<V>, R> for F
{
    #[inline(always)] fn call(&self, v: V) -> R { self(v) }
}

unsafe impl<'a, V, R, F: Fn(&V) -> R+'a> Act<NO, Ref<V>, R> for F
{
    #[inline(always)] fn call(&self, v: *const V) -> R { self(unsafe{ &*v }) }
}

unsafe impl<'a, V, R, F: Fn(&V)->R+Send+Sync+'a> Act<YES, Ref<V>, R> for F
{
    #[inline(always)] fn call(&self, v: *const V) -> R { self(unsafe{ &*v }) }
}

unsafe impl <'a, SS:YesNo, BY: RefOrVal, R> Act<SS, BY, R> for Box<Act<SS, BY, R>+'a>
{
    #[inline(always)] fn call(&self, v: BY::V) -> R { Box::as_ref(self).call(v) }
}




unsafe impl<'a, V, R, F: FnOnce(V) -> R+'a> ActOnce<NO, Val<V>, R> for F
{
    #[inline(always)] fn call_once(self, v: V) -> R { self(v) }
}

unsafe impl<'a, V, R, F: FnOnce(V)->R+Send+Sync+'a> ActOnce<YES, Val<V>, R> for F
{
    #[inline(always)] fn call_once(self, v: V) -> R { self(v) }
}

unsafe impl<'a, V, R, F: FnOnce(&V) -> R+'a> ActOnce<NO, Ref<V>, R> for F
{
    #[inline(always)] fn call_once(self, v: *const V) -> R { self(unsafe{ &*v }) }
}

unsafe impl<'a, V, R, F: FnOnce(&V)->R+Send+Sync+'a> ActOnce<YES, Ref<V>, R> for F
{
    #[inline(always)] fn call_once(self, v: *const V) -> R { self(unsafe{ &*v }) }
}

unsafe impl<'a, R, F: FnOnce()->R+Send+Sync+'a> ActOnce<YES, (), R> for F
{
    #[inline(always)] fn call_once(self, _v: ()) -> R { self() }
}

unsafe impl<'a, R, F: FnOnce()->R+'a> ActOnce<NO, (), R> for F
{
    #[inline(always)] fn call_once(self, _v: ()) -> R { self() }
}



unsafe impl<'a, SS:YesNo, A: RefOrVal, R, F: ActOnce<SS, A, R>> ActBox<SS, A, R> for F
{
    #[inline(always)] fn call_box(self: Box<F>, v: A::V) -> R  { self.call_once(v) }
}

unsafe impl <'a, SS:YesNo, BY: RefOrVal, R> ActOnce<SS, BY, R> for Box<ActBox<SS, BY, R>+'a>
{
    #[inline(always)] fn call_once(self, v: BY::V) -> R { self.call_box(v) }
}


unsafe impl<SS:YesNo, By: RefOrVal, R, A: Act<SS, By, R>> Act<SS, By, R> for Arc<A>
{
    #[inline(always)]
    fn call(&self, v: <By as RefOrVal>::V) -> R { Arc::as_ref(self).call(v) }
}


unsafe impl<SS:YesNo, A: RefOrVal> Act<SS, A> for ()
{
    #[inline(always)] fn call(&self, _: A::V) {  }
}

unsafe impl<SS:YesNo, A: RefOrVal> ActOnce<SS, A> for ()
{
    #[inline(always)] fn call_once(self, _: A::V) {  }
}




pub struct WrapAct<'o, SS: YesNo, By: RefOrVal, A, R>
{
    act: A,
    PhantomData: PhantomData<&'o (SS, By, R)>
}

unsafe impl<'o, By: RefOrVal, A: Send+'o, R> Send for WrapAct<'o, YES, By, A, R> {}
unsafe impl<'o, By: RefOrVal, A: Sync+'o, R> Sync for WrapAct<'o, YES, By, A, R> {}

impl<'o, SS:YesNo, A: Fn(By)->R+'o, By: RefOrVal, R> WrapAct<'o, SS, By, A, R>
{
    #[inline(always)]
    pub unsafe fn new(act: A) -> WrapAct<'o, SS, By, A, R>
    {
        WrapAct { act, PhantomData }
    }
}

unsafe impl<'o, SS:YesNo, A: Fn(By)->R+'o, By: RefOrVal, R> Act<SS, By, R> for WrapAct<'o, SS, By, A, R>
{
    #[inline(always)]
    fn call(&self, v: By::V) -> R
    {
        (self.act)(unsafe { By::from_v(v) })
    }
}

