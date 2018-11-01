use crate::*;
use std::sync::Arc;
use std::cell::UnsafeCell;
use std::marker::PhantomData;

pub struct UntilOp<Src, SVBy, SEBy, Sig>
{
    src: Src,
    sig: Arc<Sig>,
    PhantomData: PhantomData<(SEBy, SVBy)>
}

pub trait ObsUntilOp<'o, SS:YesNo, VBy: RefOrVal, EBy: RefOrVal, SVBy: RefOrVal, SEBy: RefOrVal, Sig: Observable<'o, SS, SVBy, SEBy>> : Sized
{
    fn until(self, signal: Sig) -> UntilOp<Self, SVBy, SEBy, Sig>
    {
        UntilOp{ src: self, sig: Arc::new(signal), PhantomData }
    }
}

impl<'o, SS:YesNo, VBy: RefOrVal, EBy: RefOrVal, SVBy: RefOrVal+'o, SEBy: RefOrVal+'o, Src: Observable<'o, SS, VBy, EBy>, Sig: Observable<'o, SS, SVBy, SEBy>>
ObsUntilOp<'o, SS, VBy, EBy, SVBy, SEBy, Sig>
for Src {}

impl<'o, SS:YesNo, VBy: RefOrVal+'o, EBy: RefOrVal+'o,  SVBy: RefOrVal+'o, SEBy: RefOrVal+'o, Src: Observable<'o, SS, VBy, EBy>, Sig: Observable<'o, SS, SVBy, SEBy>>
Observable<'o, SS, VBy, EBy>
for UntilOp<Src, SVBy, SEBy, Sig>
{
    fn subscribe(&self, next: impl ActNext<'o, SS, VBy>, ec: impl ActEc<'o, SS, EBy>) -> Unsub<'o, SS> where Self: Sized
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

            forward_ec((sub.clone(), SSWrap::new(ec)), |(sub, ec), e:Option<EBy>| {
                sub.if_not_done(||unsafe{ &mut *Arc::as_ref(&*ec).get() }.take().map_or((), |ec| ec.call_once(e.map(|e|e.into_v()))))
            })
        ))
    }

    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, VBy>>, ec: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>
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

        o.until(sig).subscribe(|_:&_| { n.replace(n.get()+1); }, |_: Option<&_>| { n.replace(n.get()+1); } );
        assert_eq!(n.get(), 2);

        n.replace(0);
        sig2.complete();
        o2.until(sig2).subscribe(|_:&_| { n.replace(n.get()+1); }, |_: Option<&_>| { n.replace(n.get()+1); } );
        assert_eq!(n.get(), 0);

        Of::value(123).until(Of::value(456)).subscribe(|_: &_| { n.replace(n.get()+1); }, |_: Option<&_>| { n.replace(n.get()+1); } );
        assert_eq!(n.get(), 1);

        n.replace(0);
        let (src, src1) = Rc::new(Subject::<NO, i32>::new()).clones();
        let (sig, sig1) = Rc::new(Subject::<NO, i32>::new()).clones();

        src.until(sig).subscribe(|_: &_| { n.replace(n.get()+1); }, |_: Option<&_>| { n.replace(n.get()+1); } );
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

        s1.until(s2).subscribe(|_: &_| { n.replace(n.get()+1); }, |_:Option<&_>| { n.replace(n.get()+100); });

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

        let sub = val.until(sig).subscribe(|_v:&_| assert!(false, "shouldnt next"), |_e:Option<&_>| assert!(false, "shouldnt complete"));

        assert!(sub.is_done());
    }
}