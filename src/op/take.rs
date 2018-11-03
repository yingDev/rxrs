use crate::*;
use std::marker::PhantomData;
use std::sync::Arc;
use std::cell::UnsafeCell;
use std::error::Error;

pub struct TakeOp<SS, Src>
{
    count: usize,
    src: Src,
    PhantomData: PhantomData<(SS)>
}

pub trait ObsTakeOp<SS, VBy> : Sized
{
    fn take(self, count: usize) -> TakeOp<SS, Self> { TakeOp{ count, src: self, PhantomData} }
}

impl<'o, VBy: RefOrVal, Src: Observable<'o, SS, VBy>+'o, SS:YesNo>
ObsTakeOp<SS, VBy>
for Src {}


pub trait DynObsTakeOp<'o, SS: YesNo, VBy: RefOrVal+'o>
{
    fn take(self, count: usize) -> Self;
}

impl<'o, SS:YesNo, VBy: RefOrVal+'o>
DynObsTakeOp<'o, SS, VBy>
for DynObservable<'o, 'o, SS, VBy>
{
    fn take(self, count: usize) -> Self
    { TakeOp{ count, src: self.src, PhantomData }.into_dyn() }
}

impl<'o, SS:YesNo, VBy: RefOrVal+'o, Src: Observable<'o, SS, VBy>>
Observable<'o, SS, VBy>
for TakeOp<SS, Src>
{
    fn subscribe(&self, next: impl ActNext<'o, SS, VBy>, ec: impl ActEc<'o, SS>) -> Unsub<'o, SS> where Self: Sized
    {
        if self.count == 0 {
            ec.call_once(None);
            return Unsub::done();
        }

        let next = SSActNextWrap::new(next);
        let sub = Unsub::new();
        let state = Arc::new(unsafe{ AnySendSync::new(UnsafeCell::new((self.count, Some(ec)))) });

        sub.clone().added_each(self.src.subscribe(
            forward_next(next, (sub.clone(), SSWrap::new(state.clone())), |next, (sub, state), v:VBy| {
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
            }, |s, (sub, _state)| (s.stopped() || sub.is_done())),

            forward_ec((sub, SSWrap::new(state)), |(sub, state), e:Option<RxError>| {
                sub.unsub_then(|| unsafe{ &mut *state.get() }.1.take().map_or((), |ec| ec.call_once(e)))
            })
        ))
    }

    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, VBy>>, ec: Box<ActEcBox<'o, SS>>) -> Unsub<'o, SS>
    { self.subscribe(next, ec) }
}

#[cfg(test)]
mod test
{
    use crate::*;
    use crate::util::clones::*;

    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn smoke()
    {
        let (n, n1, n2) = Rc::new(Cell::new(0)).clones();
        let (s, s1) = Rc::new(Subject::<NO, i32>::new()).clones();

        s.take(3).subscribe(
            |v:&_| { n.replace(*v); },
            |_e| { n1.replace(n1.get() + 100); }
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
        Of::value(123).take(100).subscribe(|v:&_| { n.replace(*v); }, |_e| { n.replace(n.get() + 100); });

        assert_eq!(n.get(), 223);
    }

    #[test]
    fn zero()
    {
        let n = Cell::new(0);
        Of::value(123).take(0).subscribe(|v:&_| { n.replace(*v); }, |_e| { n.replace(n.get() + 100); });

        assert_eq!(n.get(), 100);
    }

}
