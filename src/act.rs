use crate::*;

pub unsafe trait ActOnce<SS:YesNo, A: RefOrVal, R=()>
{
    fn call_once(self, e: A::V) -> R;
}

pub unsafe trait Act<SS:YesNo, A: RefOrVal, R=()>
{
    fn call(&self, v: A::V) -> R;
}

pub unsafe trait ActBox<SS:YesNo, A: RefOrVal, R=()>
{
    fn call_box(self: Box<Self>, e: A::V) -> R;
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
    #[inline(always)] fn call_once(self, v: ()) -> R { self() }
}

unsafe impl<'a, R, F: FnOnce()->R+'a> ActOnce<NO, (), R> for F
{
    #[inline(always)] fn call_once(self, v: ()) -> R { self() }
}



unsafe impl<'a, SS:YesNo, A: RefOrVal, R, F: ActOnce<SS, A, R>> ActBox<SS, A, R> for F
{
    #[inline(always)] fn call_box(self: Box<F>, v: A::V) -> R  { self.call_once(v) }
}

unsafe impl <'a, SS:YesNo, BY: RefOrVal, R> ActOnce<SS, BY, R> for Box<ActBox<SS, BY, R>+'a>
{
    #[inline(always)] fn call_once(self, v: BY::V) -> R { self.call_box(v) }
}


//unsafe impl<'a, SS:YesNo, V, R, F: ActOnce<SS, Ref<V>, R>+'a> ActBox<SS, Ref<V>, R> for F
//{
//    #[inline(always)] fn call_box(self: Box<F>, v: *const V) -> R  { self.call_once(unsafe{ &*v }) }
//}


unsafe impl<SS:YesNo, A: RefOrVal> Act<SS, A> for ()
{
    #[inline(always)] fn call(&self, _: A::V) {  }
}

unsafe impl<SS:YesNo, A: RefOrVal> ActOnce<SS, A> for ()
{
    #[inline(always)] fn call_once(self, _: A::V) {  }
}
