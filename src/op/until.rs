use crate::*;
use std::sync::Arc;
use std::rc::Rc;
use std::cell::RefCell;
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
    fn sub(&self, next: impl ActNext<'o, SS, VBy>, ec: impl ActEc<'o, SS, EBy>) -> Unsub<'o, SS> where Self: Sized
    {
        let next = SSActNextWrap::new(next);
        let sub = Unsub::new();
        //no mutex here because access is protected by Unsub's internal lock
        let ec = Arc::new(AnySendSync(UnsafeCell::new(Some(ec))));

        sub.add_each(self.sig.sub(forward_next((), (sub.clone(), SSWrap::new(ec.clone())), |(), (sub, ec), v:SVBy|{
            sub.unsub_then(|| unsafe{ &mut *Arc::as_ref(&*ec).0.get() }.take().map_or((), |ec| ec.call_once(None)));
        }, |(), (sub,_)| sub.is_done()), forward_ec(sub.clone(), |sub, e| sub.unsub())));

        if sub.is_done() { return sub; }

        sub.clone().added_each(self.src.sub(
            forward_next(next, (sub.clone()), |next, (sub), v: VBy| {
                sub.if_not_done(|| next.call(v.into_v()))
            }, |next, (sub)| next.stopped() || sub.is_done()),

            forward_ec((sub.clone(), SSWrap::new(ec)), |(sub, ec), e:Option<EBy>| {
                sub.if_not_done(||unsafe{ &mut *Arc::as_ref(&*ec).0.get() }.take().map_or((), |ec| ec.call_once(e.map(|e|e.into_v()))))
            })
        ))
    }

    fn sub_dyn(&self, next: Box<ActNext<'o, SS, VBy>>, ec: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>
    { self.sub(next, ec) }
}

struct AnySendSync<T>(UnsafeCell<T>);
unsafe impl<T> Send for AnySendSync<T>{}
unsafe impl<T> Sync for AnySendSync<T>{}

#[cfg(test)]
mod test
{
    use crate::*;
    use std::rc::Rc;
    use std::cell::Cell;

    #[test]
    fn smoke()
    {
        let n = Cell::new(0);

        let (o, o2) = Rc::new(Of::value(123)).clones();
        let (sig, sig2) = Rc::new(Subject::<NO, i32>::new()).clones();

        o.until(sig).sub(|_:&_| { n.replace(n.get()+1); }, |_: Option<&_>| { n.replace(n.get()+1); } );
        assert_eq!(n.get(), 2);

        n.replace(0);
        sig2.complete();
        o2.until(sig2).sub(|_:&_| { n.replace(n.get()+1); }, |_: Option<&_>| { n.replace(n.get()+1); } );
        assert_eq!(n.get(), 0);

        Of::value(123).until(Of::value(456)).sub(|_: &_| { n.replace(n.get()+1); }, |_: Option<&_>| { n.replace(n.get()+1); } );
        assert_eq!(n.get(), 1);

        n.replace(0);
        let (src, src1) = Rc::new(Subject::<NO, i32>::new()).clones();
        let (sig, sig1) = Rc::new(Subject::<NO, i32>::new()).clones();

        src.until(sig).sub(|_: &_| { n.replace(n.get()+1); }, |_: Option<&_>| { n.replace(n.get()+1); } );
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

        s1.until(s2).sub(|_: &_| { n.replace(n.get()+1); }, |_:Option<&_>| { n.replace(n.get()+100); });

        s3.next(1);
        assert_eq!(n.get(), 100);

        s3.complete();
        assert_eq!(n.get(), 100);
    }
}