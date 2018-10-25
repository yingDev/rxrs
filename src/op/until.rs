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

impl<'o, VBy: RefOrVal+'o, EBy: RefOrVal+'o, SVBy: RefOrVal+'o, SEBy: RefOrVal+'o, Src: Observable<'o, NO, VBy, EBy>, Sig: Observable<'o, NO, SVBy, SEBy>>
Observable<'o, NO, VBy, EBy>
for UntilOp<Src, SVBy, SEBy, Sig>
{
    fn sub(&self, next: impl ActNext<'o, NO, VBy>, ec: impl ActEc<'o, NO, EBy>) -> Unsub<'o, NO> where Self: Sized
    {
        let (s1, s2, s3) = Unsub::new().clones();
        let (ec1, ec2) = Rc::new(RefCell::new(Some(ec))).clones();

        s1.add_each(self.sig.sub(
            move |_: By<_>| s2.unsub_then(|| ec1.borrow_mut().take().map_or((), |ec| ec.call_once(None))), ()
        ));

        if s3.is_done() { return s3; }
        s3.added_each(self.src.sub(
            next,
            move |e: Option<By<_>>| ec2.borrow_mut().take().map_or((), |ec| ec.call_once(e))
        ))
    }

    fn sub_dyn(&self, next: Box<ActNext<'o, NO, VBy>>, ec: Box<ActEcBox<'o, NO, EBy>>) -> Unsub<'o, NO>
    { self.sub(dyn_to_impl_next(next), dyn_to_impl_ec(ec)) }
}

impl<VBy: RefOrValSSs, EBy: RefOrValSSs,  SVBy: RefOrValSSs, SEBy: RefOrValSSs, Src: Observable<'static, YES, VBy, EBy>, Sig: Observable<'static, YES, SVBy, SEBy>>
Observable<'static, YES, VBy, EBy>
for UntilOp<Src, SVBy, SEBy, Sig>
{
    fn sub(&self, next: impl ActNext<'static, YES, VBy>, ec: impl ActEc<'static, YES, EBy>) -> Unsub<'static, YES> where Self: Sized
    {
        let (s1, s2, s3, s4, s5) = Unsub::new().clones();
        //no mutex here because access is protected by Unsub's internal lock
        let (ec1, ec2) = Arc::new(AnySendSync(UnsafeCell::new(Some(ec)))).clones();
        let next = sendsync_next(next);

        s1.add_each(self.sig.sub(
            move |_: By<_>| s2.unsub_then(|| unsafe{ &mut *ec1.0.get() }.take().map_or((), |ec| ec.call_once(None))), ()
        ));

        if s3.is_done() { return s3; }
        s3.added_each(self.src.sub(
            move |v: By<_>        | s4.if_not_done(|| next.call(v)),
            move |e: Option<By<_>>| s5.if_not_done(||unsafe{ &mut *ec2.0.get() }.take().map_or((), |ec| ec.call_once(e)))
        ))
    }

    fn sub_dyn(&self, next: Box<ActNext<'static, YES, VBy>>, ec: Box<ActEcBox<'static, YES, EBy>>) -> Unsub<'static, YES>
    { self.sub(dyn_to_impl_next_ss(next), dyn_to_impl_ec_ss(ec)) }
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