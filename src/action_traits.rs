use crate::*;
use std::sync::Arc;
use std::ops::Deref;
use std::marker::PhantomData;

pub unsafe trait ActOnce<SS:YesNo, A>
{
    fn call_once(self, e: A);
}
pub unsafe trait Act<SS:YesNo, A>  { fn call(&self, v: A); }
pub unsafe trait ActBox<SS:YesNo, A> { fn call_box(self: Box<Self>, e: A); }


unsafe impl<'a, A, F: Fn(A)+'a> Act<NO, A> for F
{ #[inline(always)] fn call(&self, v: A) { self(v) } }

unsafe impl<A, F: Fn(A)+Send+Sync+'static> Act<YES, A> for F
{ #[inline(always)] fn call(&self, v: A) { self(v) } }

unsafe impl<'a, A, F: FnOnce(A)+'a> ActOnce<NO, A> for F
{
    #[inline(always)] fn call_once(self, v: A) { self(v) }
}

unsafe impl<A, F: FnOnce(A)+Send+Sync+'static> ActOnce<YES, A> for F
{
    #[inline(always)] fn call_once(self, v: A) { self(v) }
}


unsafe impl<'a, SS: YesNo, A> Act<SS, A> for Box<Act<SS, A>+'a>
{ #[inline(always)] fn call(&self, v: A) { self.as_ref().call(v) } }


unsafe impl<SS:YesNo, A, F> ActBox<SS, A> for F where F: ActOnce<SS, A>
{ fn call_box(self: Box<F>, args: A)  { self.call_once(args) } }

unsafe impl<SS:YesNo, A> ActOnce<SS, A> for Box<ActBox<SS, A>>
{ fn call_once(self, args: A) { self.call_box(args) } }


unsafe impl<SS:YesNo, A> Act<SS, A> for ()
{ #[inline(always)] fn call(&self, v: A) {  } }

unsafe impl<SS:YesNo, A> ActOnce<SS, A> for ()
{
    #[inline(always)] fn call_once(self, v: A) {  }
}

