use crate::*;
use std::marker::PhantomData;
use std::cell::Cell;
use std::error::Error;

pub struct SkipOp<SS, Src>
{
    count: usize,
    src: Src,
    PhantomData: PhantomData<(SS)>
}

pub trait ObsSkipOp<SS, VBy> : Sized
{
    fn skip(self, count: usize) -> SkipOp<SS, Self> { SkipOp{ count, src: self, PhantomData} }
}

impl<'o, VBy: RefOrVal, Src: Observable<'o, SS, VBy>+'o, SS:YesNo>
ObsSkipOp<SS, VBy>
for Src {}


pub trait DynObsSkipOp<'o, SS: YesNo, VBy: RefOrVal+'o>
{
    fn skip(self, count: usize) -> Self;
}

impl<'o, SS:YesNo, VBy: RefOrVal+'o>
DynObsSkipOp<'o, SS, VBy>
for DynObservable<'o, 'o, SS, VBy>
{
    fn skip(self, count: usize) -> Self
    { SkipOp{ count, src: self.src, PhantomData }.into_dyn() }
}

impl<'o, SS:YesNo, VBy: RefOrVal+'o, Src: Observable<'o, SS, VBy>>
Observable<'o, SS, VBy>
for SkipOp<SS, Src>
{
    fn subscribe(&self, next: impl ActNext<'o, SS, VBy>, ec: impl ActEc<'o, SS>) -> Unsub<'o, SS> where Self: Sized
    {
        if self.count == 0 {
            return self.src.subscribe(next, ec);
        }

        let count = unsafe{ AnySendSync::new(Cell::new(self.count)) };
        let next = SSActNextWrap::new(next);
        let sub = Unsub::new();

        sub.added_each(self.src.subscribe(
            forward_next(next, SSWrap::new(count), |next, count, v:VBy| {
                if count.get() == 0 {
                    next.call(v.into_v());
                } else {
                    count.replace(count.get() - 1);
                }
            }, |s, _| s.stopped()),

            ec
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

        s.skip(3).subscribe(
            |v:&_| { n.replace(*v); },
            |_e| { n1.replace(n1.get() + 100); }
        );

        s1.next(1);
        assert_eq!(n2.get(), 0);

        s1.next(2);
        assert_eq!(n2.get(), 0);

        s1.next(3);
        assert_eq!(n2.get(), 0);

        s1.next(4);
        assert_eq!(n2.get(), 4);

        s1.complete();
        assert_eq!(n2.get(), 104);
    }

    #[test]
    fn of()
    {
        let n = Cell::new(0);
        Of::value(123).skip(100).subscribe(
            |v:&_| { n.replace(*v); },
            |_e| { n.replace(n.get() + 100); }
        );

        assert_eq!(n.get(), 100);
    }

    #[test]
    fn zero()
    {
        let n = Cell::new(0);
        Of::value(123).skip(0).subscribe(
            |v:&_| { n.replace(*v); },
            |_e| { n.replace(n.get() + 100); }
        );

        assert_eq!(n.get(), 223);
    }

}
