use std::marker::PhantomData;
use std::sync::Arc;
use crate::*;
use crate::any_send_sync::AnySendSync;
use std::cell::UnsafeCell;
use std::cell::RefCell;

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

impl<'o, SS:YesNo, VBy: RefOrVal+'o, EBy: RefOrVal+'o, Src: Observable<'o, SS, VBy, EBy>>
Observable<'o, SS, VBy, EBy>
for TakeOp<SS, Src>
{
    fn sub(&self, next: impl ActNext<'o, SS, VBy>, ec: impl ActEc<'o, SS, EBy>) -> Unsub<'o, SS> where Self: Sized
    {
        if self.count == 0 {
            ec.call_once(None);
            return Unsub::done();
        }

        let next = SSActNextWrap::new(next);
        let sub = Unsub::new();
        let (state, state1) = Arc::new(unsafe{ AnySendSync::new(UnsafeCell::new((self.count, Some(ec)))) }).clones();

        sub.clone().added_each(self.src.sub(forward_next(next, (sub.clone(), SSWrap::new(state)), |next, (sub, state), v:VBy| {
                sub.if_not_done(|| {
                    let state = unsafe{ &mut *state.get() };

                    let mut val = state.0;
                    if val != 0 {
                        val -= 1;
                        state.0 -= 1;
                        next.call(v.into_v());
                    }
                    if val == 0 {
                        sub.unsub_then(|| state.1.take().map_or((), |ec| ec.call_once(None)));
                    }
                });
            }, |s, (sub, state)| (s.stopped() || sub.is_done())),

            forward_ec((sub, SSWrap::new(state1)), |(sub, state1), e:Option<EBy>| {
                sub.unsub_then(|| unsafe{ &mut *state1.get() }.1.take().map_or((), |ec| ec.call_once(e.map(|e| e.into_v()))))
            })
        ))
    }

    fn sub_dyn(&self, next: Box<ActNext<'o, SS, VBy>>, ec: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>
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

        s.take(3).sub(
            |v:&_| { n.replace(*v); },
            |e:Option<&_>| { n1.replace(n1.get() + 100); }
        );

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
        Of::value(123).take(100).sub(|v:&_| { n.replace(*v); }, |e:Option<&_>| { n.replace(n.get() + 100); });

        assert_eq!(n.get(), 223);
    }

    #[test]
    fn zero()
    {
        let n = Cell::new(0);
        Of::value(123).take(0).sub(|v:&_| { n.replace(*v); }, |e:Option<&_>| { n.replace(n.get() + 100); });

        assert_eq!(n.get(), 100);
    }

}
