use crate::*;
use std::sync::Arc;
use std::cell::UnsafeCell;

pub struct UntilOp<'o, SS:YesNo, Src, SVBy: RefOrVal>
{
    src: Src,
    sig: DynObservable<'o, 'o, SS, SVBy>,
}

pub trait ObsUntilOp<'o, SS:YesNo, VBy: RefOrVal, SVBy: RefOrVal> : Sized
{
    fn until(self, sig: impl Observable<'o, SS, SVBy>+'o) -> UntilOp<'o, SS, Self, SVBy>
    {
        UntilOp{ src: self, sig: sig.into_dyn() }
    }
}

impl<'o, SS:YesNo, VBy: RefOrVal, SVBy: RefOrVal+'o, Src: Observable<'o, SS, VBy>>
ObsUntilOp<'o, SS, VBy, SVBy>
for Src {}


pub trait DynObsUntilOp<'o, SS:YesNo, VBy: RefOrVal, SVBy: RefOrVal> : Sized
{
    fn until(self, signal: impl Observable<'o, SS, SVBy>+'o) -> DynObservable<'o, 'o, SS, VBy>;
}

impl<'o, SS:YesNo, VBy: RefOrVal+'o+'o, SVBy: RefOrVal+'o>
DynObsUntilOp<'o, SS, VBy, SVBy>
for DynObservable<'o, 'o, SS, VBy>
{
    fn until(self, sig: impl Observable<'o, SS, SVBy>+'o) -> DynObservable<'o, 'o, SS, VBy>
    {
        UntilOp{ src: self.src, sig: sig.into_dyn() }.into_dyn()
    }
}

impl<'o, SS:YesNo, VBy: RefOrVal+'o+'o,  SVBy: RefOrVal+'o, Src: Observable<'o, SS, VBy>>
Observable<'o, SS, VBy>
for UntilOp<'o, SS, Src, SVBy>
{
    fn subscribe(&self, next: impl ActNext<'o, SS, VBy>, ec: impl ActEc<'o, SS>) -> Unsub<'o, SS> where Self: Sized
    {
        let next = SSActNextWrap::new(next);
        let sub = Unsub::new();
        //no mutex here because access is protected by Unsub's internal lock
        let ec = Arc::new(unsafe{ AnySendSync::new(UnsafeCell::new(Some(ec))) });

        sub.add_each(self.sig.subscribe(forward_next((), (sub.clone(), SSWrap::new(ec.clone())), |(), (sub, ec), _v:SVBy|{
            sub.unsub_then(|| unsafe{ &mut *Arc::as_ref(&*ec).get() }.take().map_or((), |ec| ec.call_once(None)));
        }, |(), (sub,_)| sub.is_done()), forward_ec(sub.clone(), |sub, _e| sub.unsub())));

        if sub.is_done() { return sub; }

        sub.clone().added_each(self.src.subscribe(
            forward_next(next, sub.clone(), |next, sub, v: VBy| {
                sub.if_not_done(|| next.call(v.into_v()))
            }, |next, sub| next.stopped() || sub.is_done()),

            forward_ec((sub.clone(), SSWrap::new(ec)), |(sub, ec), e:Option<RxError>| {
                sub.if_not_done(||unsafe{ &mut *Arc::as_ref(&*ec).get() }.take().map_or((), |ec| ec.call_once(e)))
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

    use std::rc::Rc;
    use std::cell::Cell;

    #[test]
    fn smoke()
    {
        let n = Cell::new(0);

        let (o, o2) = Rc::new(Of::value(123)).clones();
        let (sig, sig2) = Rc::new(Subject::<NO, i32>::new()).clones();

        o.until(sig).subscribe(|_:&_| { n.replace(n.get()+1); }, |_| { n.replace(n.get()+1); } );
        assert_eq!(n.get(), 2);

        n.replace(0);
        sig2.complete();
        o2.until(sig2).subscribe(|_:&_| { n.replace(n.get()+1); }, |_| { n.replace(n.get()+1); } );
        assert_eq!(n.get(), 0);

        Of::value(123).until(Of::value(456)).subscribe(|_: &_| { n.replace(n.get()+1); }, |_| { n.replace(n.get()+1); } );
        assert_eq!(n.get(), 1);

        n.replace(0);
        let (src, src1) = Rc::new(Subject::<NO, i32>::new()).clones();
        let (sig, sig1) = Rc::new(Subject::<NO, i32>::new()).clones();

        src.until(sig).subscribe(|_: &_| { n.replace(n.get()+1); }, |_| { n.replace(n.get()+1); } );
        src1.next(1);
        assert_eq!(n.get(), 1);
        src1.next(1);
        assert_eq!(n.get(), 2);

        sig1.next(1);
        assert_eq!(n.get(), 3);
        sig1.next(1);
        assert_eq!(n.get(), 3);
    }

    #[test]
    fn cycle()
    {
        let n = Cell::new(0);
        let (s1, s2, s3) = Rc::new(Subject::<NO, i32>::new()).clones();

        s1.until(s2).subscribe(
            |_: &_| { n.replace(n.get()+1); },
            |_| { n.replace(n.get()+100); }
        );

        s3.next(1);
        assert_eq!(n.get(), 100);

        s3.complete();
        assert_eq!(n.get(), 100);
    }

    #[test]
    fn empty_sig()
    {
        let sig = Of::<()>::empty();
        let val = Of::value(123);

        let sub = val.until(sig).subscribe(|_v:&_| assert!(false, "shouldnt next"), |_e| assert!(false, "shouldnt complete"));

        assert!(sub.is_done());
    }
}