use crate::*;
use std::sync::Arc;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Mutex;

pub struct UntilOp<Src, Sig>
{
    src: Src,
    sig: Arc<Sig>,
}

pub trait ObsUntilOp<'o, SS:YesNo, VBy: RefOrVal, EBy: RefOrVal, Sig: Observable<'o, SS, VBy, EBy>> : Sized
{
    fn until(self, signal: Sig) -> UntilOp<Self, Sig>
    {
        UntilOp{ src: self, sig: Arc::new(signal) }
    }
}

impl<'o, SS:YesNo, VBy: RefOrVal, EBy: RefOrVal, Src: Observable<'o, SS, VBy, EBy>, Sig: Observable<'o, SS, VBy, EBy>>
ObsUntilOp<'o, SS, VBy, EBy, Sig>
for Src {}

impl<'o, VBy: RefOrVal+'o, EBy: RefOrVal+'o, Src: Observable<'o, NO, VBy, EBy>, Sig: Observable<'o, NO, VBy, EBy>>
Observable<'o, NO, VBy, EBy>
for UntilOp<Src, Sig>
{
    fn sub(&self, next: impl ActNext<'o, NO, VBy>, ec: impl ActEc<'o, NO, EBy>) -> Unsub<'o, NO> where Self: Sized
    {
        let (s1, s2, s3, s4) = Unsub::new().clones();
        let (ec1, ec2) = Rc::new(RefCell::new(Some(ec))).clones();

        s1.add_each(self.sig.sub(
            move |_: By<_>        | s2.unsub_then(|| ec1.borrow_mut().take().map_or((), |ec| ec.call_once(None))),
            move |_: Option<By<_>>| s3.unsub()
        ));

        if s4.is_done() { return s4; }
        s4.added_each(self.src.sub(
            next,
            move |e: Option<By<_>>| ec2.borrow_mut().take().map_or((), |ec| ec.call_once(e))
        ))
    }

    fn sub_dyn(&self, next: Box<ActNext<'o, NO, VBy>>, ec: Box<ActEcBox<'o, NO, EBy>>) -> Unsub<'o, NO>
    { self.sub(dyn_to_impl_next(next), dyn_to_impl_ec(ec)) }
}

impl<VBy: RefOrValSSs, EBy: RefOrValSSs, Src: Observable<'static, YES, VBy, EBy>, Sig: Observable<'static, YES, VBy, EBy>>
Observable<'static, YES, VBy, EBy>
for UntilOp<Src, Sig>
{
    fn sub(&self, next: impl ActNext<'static, YES, VBy>, ec: impl ActEc<'static, YES, EBy>) -> Unsub<'static, YES> where Self: Sized
    {
        let (s1, s2, s3, s4, s5) = Unsub::new().clones();
        let (ec1, ec2) = Arc::new(Mutex::new(Some(sendsync_ec(ec)))).clones();
        let next = sendsync_next(next);

        s1.add_each(self.sig.sub(
            move |_: By<_>        | s2.unsub_then(|| ec1.lock().unwrap().take().map_or((), |ec| ec.call_once(None))),
            move |_: Option<By<_>>| s3.unsub()
        ));

        if s4.is_done() { return s4; }
        s4.added_each(self.src.sub(
            move |v: By<_>        | s5.if_not_done(|| next.call(v)),
            move |e: Option<By<_>>| ec2.lock().unwrap().take().map_or((), |ec| ec.call_once(e))
        ))
    }

    fn sub_dyn(&self, next: Box<ActNext<'static, YES, VBy>>, ec: Box<ActEcBox<'static, YES, EBy>>) -> Unsub<'static, YES>
    { self.sub(dyn_to_impl_next_ss(next), dyn_to_impl_ec_ss(ec)) }
}

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

        o.until(sig).sub(|_: By<_>| { n.replace(n.get()+1); }, |_: Option<By<_>>| { n.replace(n.get()+1); } );
        assert_eq!(n.get(), 2);

        n.replace(0);
        sig2.complete();
        o2.until(sig2).sub(|_: By<_>| { n.replace(n.get()+1); }, |_: Option<By<_>>| { n.replace(n.get()+1); } );
        assert_eq!(n.get(), 0);

        Of::value(123).until(Of::value(456)).sub(|_: By<_>| { n.replace(n.get()+1); }, |_: Option<By<_>>| { n.replace(n.get()+1); } );
        assert_eq!(n.get(), 1);

        n.replace(0);
        let (src, src1) = Rc::new(Subject::<NO, i32>::new()).clones();
        let (sig, sig1) = Rc::new(Subject::<NO, i32>::new()).clones();

        src.until(sig).sub(|_: By<_>| { n.replace(n.get()+1); }, |_: Option<By<_>>| { n.replace(n.get()+1); } );
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

        s1.until(s2).sub(|_: By<_>| { n.replace(n.get()+1); }, |_:Option<By<_>>| { n.replace(n.get()+100); });

        s3.next(1);
        assert_eq!(n.get(), 100);

        s3.complete();
        assert_eq!(n.get(), 100);
    }
}