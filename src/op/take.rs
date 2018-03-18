use std::marker::PhantomData;
use std::any::{Any};
use std::rc::Rc;
use std::sync::atomic::AtomicIsize;

use observable::*;
use subref::SubRef;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use observable::RxNoti::*;
use std::mem;
use util::traicks::*;
use util::sarc::Sarc;
use util::sarc::SendSyncInfo;

#[derive(Clone)]
pub struct TakeOp<'a, 'b, V, S:YesNo>
{
    source: Sarc<S, Observable<'a,V=V,SSO=S>+'b>,
    total: usize,
}

pub trait ObservableTake<'a, 'b, V, S:YesNo>
{
    fn take(self, total: usize) -> Sarc<S, TakeOp<'a, 'b, V, S>>;
}

impl<'a: 'b, 'b, V: 'a, S: YesNo+'static, O> ObservableTake<'a, 'b, V, S> for Sarc<S, O> where O: Observable<'a,V=V,SSO=S>+'b
{
    fn take(self, total: usize) -> Sarc<S, TakeOp<'a, 'b, V, S>>
    {
        Sarc::new(TakeOp { total, source: self })
    }
}

impl<'a: 'b, 'b, V: 'a, S:YesNo+'static> Observable<'a> for TakeOp<'a, 'b, V, S>
{
    type V = V;
    type SSO = S;

    fn sub(&self, o: Sarc<Self::SSO, Observer<Self::V>+'a>) -> SubRef
    {
        if self.total == 0 {
            o.complete();
            return SubRef::empty();
        }

        let sub = SubRef::signal();
        let sub2 = sub.clone();
        let mut count = self.total;

        sub.add(self.source.sub_noti(move |n| {
            match n {
                Next(v) => {
                    count -= 1;
                    if count > 0 {
                        o.next(v);
                        if o._is_closed() { return IsClosed::True; }
                    }else {
                        o.next(v);
                        sub2.unsub();
                        o.complete();
                        return IsClosed::True;
                    }
                },
                Err(e) => {
                    sub2.unsub();
                    o.err(e);
                },
                Comp => {
                    sub2.unsub();
                    o.complete()
                }
            }
            IsClosed::Default
        }));

        sub
    }
}

#[cfg(test)]
mod test
{
    use util::sarc;

    use super::*;
    use std::sync::atomic::*;
    use std::thread;
    use std::time::Duration;
    use op::*;
    use util::sarc::SendSyncInfo;


    #[test]
    fn basic()
    {
        {
            let mut out = 0;

            {
                let src = Src::sarc();
                src.take(2).subf(|v| out += v);
            }

            assert_eq!(out, 3);
        }

        {
            let out = Arc::new(AtomicIsize::new(0));
            let out2 = out.clone();
            let src = ThreadedSrc::sarc();

            //extect compile fail
            //src.take(2).subf(|v| { out.fetch_add(v as isize, Ordering::SeqCst); });

            src.take(2).subf(move |v| { out2.fetch_add(v as isize, Ordering::SeqCst); });
            thread::sleep(Duration::from_millis(500));

            assert_eq!(out.load(Ordering::SeqCst), 3);
        }
    }

    struct Src<'a>(PhantomData<&'a ()>);

    impl<'a> Src<'a>
    {
        fn sarc() -> Sarc<No, Src<'a>>{ Sarc::new(Src(PhantomData))}
    }

    impl<'a> Observable<'a> for Src<'a>
    {
        type V = i32;
        type SSO = No;

        fn sub(&self, o: Sarc<Self::SSO, Observer<Self::V>+'a>) -> SubRef
        {
            o.next(1);
            o.next(2);
            o.next(3);
            o.complete();

            SubRef::empty()
        }
    }


    struct ThreadedSrc;
    impl ThreadedSrc
    {
        fn sarc() -> Sarc<Yes, ThreadedSrc> { Sarc::new(ThreadedSrc) }
    }

    impl Observable<'static> for ThreadedSrc
    {
        type V = i32;
        type SSO = Yes;

        fn sub(&self, o: Sarc<Self::SSO, Observer<Self::V>+'static>) -> SubRef
        {
            let u = SubRef::signal();
            let u2 = u.clone();

            ::std::thread::spawn(move || {
                for i in 1..4 {
                    if u2.disposed() { break; }
                    o.next(i);
                }
                o.complete();
            });

            u
        }
    }
}