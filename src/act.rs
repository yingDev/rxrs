use crate::*;

pub unsafe trait ActOnce<SS:YesNo, A=(), R=()>
{
    fn call_once(self, e: A) -> R;
}

pub unsafe trait Act<SS:YesNo, A=(), R=()>
{
    fn call(&self, v: A) -> R;
}

pub unsafe trait ActBox<SS:YesNo, A=(), R=()>
{
    fn call_box(self: Box<Self>, e: A) -> R;
}

unsafe impl<'a, A, R, F: Fn(A) -> R+'a> Act<NO, A, R> for F
{
    #[inline(always)] fn call(&self, v: A) -> R { self(v) }
}

unsafe impl<A, R, F: Fn(A)->R+Send+Sync> Act<YES, A, R> for F
{
    #[inline(always)] fn call(&self, v: A) -> R { self(v) }
}

unsafe impl<'a, A, R, F: FnOnce(A) -> R+'a> ActOnce<NO, A, R> for F
{
    #[inline(always)] fn call_once(self, v: A) -> R { self(v) }
}

unsafe impl<A, R, F: FnOnce(A)->R+Send+Sync> ActOnce<YES, A, R> for F
{
    #[inline(always)] fn call_once(self, v: A) -> R { self(v) }
}

unsafe impl<SS:YesNo, A, R, F: ActOnce<SS, A, R>> ActBox<SS, A, R> for F
{
    #[inline(always)] fn call_box(self: Box<F>, args: A) -> R  { self.call_once(args) }
}

unsafe impl<SS:YesNo, A> Act<SS, A> for ()
{
    #[inline(always)] fn call(&self, _: A) {  }
}

unsafe impl<SS:YesNo, A> ActOnce<SS, A> for ()
{
    #[inline(always)] fn call_once(self, _: A) {  }
}

