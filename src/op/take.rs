use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::*;
use crate::*;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use crate::any_send_sync::AnySendSync;
use std::cell::UnsafeCell;

pub struct TakeOp<SS, Src>
{
    count: usize,
    src: Src,
    PhantomData: PhantomData<(SS)>
}

pub trait ObsTakeOp<SS, VBy, EBy> : Sized
{
    fn take(self, count: usize) -> TakeOp<SS, Self> { TakeOp{ count, src: self, PhantomData} }
}

impl<'o, VBy: RefOrVal, EBy: RefOrVal, Src: Observable<'o, SS, VBy, EBy>+'o, SS:YesNo>
ObsTakeOp<SS, VBy,EBy>
for Src {}


impl<'o, VBy: RefOrVal+'o, EBy:RefOrVal+'o, Src: Observable<'o, NO, VBy, EBy>+'o>
Observable<'o, NO, VBy, EBy>
for TakeOp<NO, Src>
{
    fn sub(&self, next: impl ActNext<'o, NO, VBy>, ec: impl ActEc<'o, NO, EBy>) -> Unsub<'o, NO> where Self: Sized
    {
        if self.count == 0 {
            ec.call_once(None);
            return Unsub::done();
        }

        let  n = Cell::new(self.count);
        let (s1, s2, s3) = Unsub::new().clones();
        let (ec, ec1) = Rc::new(RefCell::new(Some(ec))).clones();

        s1.added_each(self.src.sub(ForwardNext::new(next, move |next, v:VBy::V| {
            if !s2.is_done() {
                let mut val = n.get();
                if val != 0 {
                    val -= 1;
                    n.replace(val);
                    next.call(v);
                }

                if val == 0 {
                    if ! s2.is_done() {
                        s2.unsub();
                        ec.borrow_mut().take().map_or((), |ec| ec.call_once(None));
                    }
                }
            }

        }, move |s| (s || s3.is_done()) ), move |e:Option<EBy::V>| ec1.borrow_mut().take().map_or((), |ec| ec.call_once(e))) )
    }

    fn sub_dyn(&self, next: Box<ActNext<'o, NO, VBy>>, ec: Box<ActEcBox<'o, NO, EBy>>) -> Unsub<'o, NO>
    { self.sub(next, ec) }
}

impl<VBy: RefOrValSSs, EBy: RefOrValSSs, Src: Observable<'static, YES, VBy, EBy>+'static+Send+Sync>
Observable<'static, YES, VBy, EBy>
for TakeOp<YES, Src>
{
    fn sub(&self, next: impl ActNext<'static, YES, VBy>, ec: impl ActEc<'static, YES, EBy>) -> Unsub<'static, YES> where Self: Sized
    {
        if self.count == 0 {
            ec.call_once(None);
            return Unsub::done();
        }

        let (s1, s2, s3, s4) = Unsub::new().clones();
        let (state, state1) = Arc::new(unsafe{ AnySendSync::new(UnsafeCell::new((self.count, Some(ec)))) }).clones();

        s1.added_each(self.src.sub(ForwardNext::new(next, move |next, v:VBy::V| {
            s2.if_not_done(|| {
                let state = unsafe{ &mut *state.get() };
                let mut val = state.0;
                if val != 0 {
                    val -= 1;
                    state.0 -= 1;
                    next.call(v);
                }
                if val == 0 {
                    s2.unsub_then(|| state.1.take().map_or((), |ec| ec.call_once(None)));
                }
            });

        }, move |s| (s || s4.is_done())), move |e:Option<EBy::V>| s3.unsub_then(|| unsafe{ &mut *state1.get() }.1.take().map_or((), |ec| ec.call_once(e)))) )
    }

    fn sub_dyn(&self, next: Box<ActNext<'static, YES, VBy>>, ec: Box<ActEcBox<'static, YES, EBy>>) -> Unsub<'static, YES>
    { self.sub(next, ec) }
}


#[cfg(test)]
mod test
{
    use crate::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn smoke()
    {
        let (n, n1, n2) = Rc::new(Cell::new(0)).clones();
        let (s, s1) = Rc::new(Subject::<NO, i32>::new()).clones();
        s.take(3).sub(|v:By<_>| { n.replace(*v); }, |e:Option<By<_>>| { n1.replace(n1.get() + 100); });

        s1.next(1);
        assert_eq!(n2.get(), 1);

        s1.next(2);
        assert_eq!(n2.get(), 2);

        s1.next(3);
        assert_eq!(n2.get(), 103);

        s1.next(4);
        assert_eq!(n2.get(), 103);

        s1.complete();
        assert_eq!(n2.get(), 103);
    }

    #[test]
    fn of()
    {
        let n = Cell::new(0);
        Of::value(123).take(100).sub(|v:By<_>| { n.replace(*v); }, |e:Option<By<_>>| { n.replace(n.get() + 100); });

        assert_eq!(n.get(), 223);
    }

    #[test]
    fn zero()
    {
        let n = Cell::new(0);
        Of::value(123).take(0).sub(|v:By<_>| { n.replace(*v); }, |e:Option<By<_>>| { n.replace(n.get() + 100); });

        assert_eq!(n.get(), 100);
    }

}
