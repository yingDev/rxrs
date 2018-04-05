use std::marker::PhantomData;
use observable::*;
use subref::*;
use observable::RxNoti::*;
use util::mss::*;
use scheduler::*;

#[derive(Clone)]
pub struct TakeOp<Src, V, SSO: ? Sized, SSS: ? Sized>
{
    source: Src,
    total: isize,
    PhantomData: PhantomData<(V, *const SSO, *const SSS)>,
}

pub trait ObservableTake<'a, Src, V, SSO: ? Sized, SSS: ? Sized> where Src: Observable<'a, V, SSO, SSS>
{
    fn take(self, total: isize) -> TakeOp<Src, V, SSO, SSS>;
}

impl<'a, Src, V, SSO: ? Sized, SSS: ? Sized> ObservableTake<'a, Src, V, SSO, SSS> for Src where Src: Observable<'a, V, SSO, SSS>,
{
    #[inline(always)]
    fn take(self, total: isize) -> TakeOp<Self, V, SSO, SSS>
    {
        TakeOp { total, PhantomData, source: self }
    }
}

macro_rules! fn_sub (
($s:ty, $sss:ty) => (fn_sub!($s, $sss, $sss););

($s: ty, $sss: ty, $inner_sss: ty)=>{
    #[inline(always)]
    fn sub(&self, o: Mss<$s, impl Observer<V> +'a>) -> SubRef<$sss>
    {
        if self.total <= 0 {
            o.complete();
            return SubRef::<$sss>::empty();
        }

        let sub = InnerSubRef::<$inner_sss>::signal();
        let mut count = self.total;

        sub.add(self.source.sub_noti(byclone!(sub => move |n| {
            match n {
                Next(v) => {
                    count -= 1;
                    if count > 0 {
                        o.next(v);
                        if o._is_closed() {
                           sub.unsub();
                           return IsClosed::True;
                        }
                    }else {
                        o.next(v);
                        sub.unsub();
                        o.complete();
                        return IsClosed::True;
                    }
                },
                Err(e) => {
                    sub.unsub();
                    o.err(e);
                },
                Comp => {
                    sub.unsub();
                    o.complete()
                }
            }
            IsClosed::Default
        })));

        sub.into_subref()
    }
});

impl<'a, Src, V: 'a> Observable<'a, V, Yes, Yes> for TakeOp<Src, V, Yes, Yes> where Src: Observable<'a, V, Yes, Yes>
{
    fn_sub!(Yes, Yes);
}

impl<'a, Src, V: 'a> Observable<'a, V, No, No> for TakeOp<Src, V, No, No> where Src: Observable<'a, V, No, No>
{
    fn_sub!(No, No);
}

impl<'a, Src, V: 'a> Observable<'a, V, No, Yes> for TakeOp<Src, V, No, Yes> where Src: Observable<'a, V, No, Yes>
{
    fn_sub!(No, Yes);
}

impl<'a, Src, V: 'a> Observable<'a, V, Yes, No> for TakeOp<Src, V, Yes, No> where Src: Observable<'a, V, Yes, No>
{
    //fn_sub!(Yes, No, Yes);
    fn sub(&self, o: Mss<Yes, impl Observer<V> +'a>) -> SubRef<No>
    {
        if self.total <= 0 {
            o.complete();
            return SubRef::<No>::empty();
        }

        let sub = InnerSubRef::<No>::signal();
        let inner = get_sync_context().unwrap().create_send(box byclone!(sub => move ||{
            sub.unsub();
        }));
        sub.addss(inner.clone());

        let mut count = self.total;

        sub.added(self.source.sub_noti(byclone!(inner => move |n| {
            match n {
                Next(v) => {
                    count -= 1;
                    if count > 0 {
                    o.next(v);
                    if o._is_closed() {
                        inner.unsub();
                        return IsClosed::True;
                    }
                }else {
                    o.next(v);
                    inner.unsub();
                    o.complete();
                    return IsClosed::True;
                }
                },
                Err(e) => {
                    inner.unsub();
                    o.err(e);
                },
                Comp => {
                    inner.unsub();
                    o.complete()
                }
            }

            IsClosed::Default
        }))).into_subref()
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use observable::RxNoti::*;
    use test_fixture::*;
    use std::rc::Rc;

    #[test]
    fn src_sso()
    {
        //SSO: No
        let s = SimpleObservable;

        let o = NonSendObserver(Rc::new(1));
        s.sub(Mss::new(o));
        //should not compile
        //s.sub(o);

        let o = NonSendObserver(Rc::new(2));
        s.rx().take(1).sub(Mss::new(o));

        let o = SimpleObserver;
        s.rx().take(1).sub(Mss::new(o));

        let s = ThreadedObservable;
        let o = NonSendObserver(Rc::new(3));
        //should not compile
        //s.sub_nss(o);
        let o = SimpleObserver;
        //s.sub(o);
        s.rx().take(1).sub(Mss::new(o));

        let i = 1;
        let o = LocalObserver(&i);
        let s = ThreadedObservable;
        //should not compile
        //s.sub(o);
        //s.rx().take(1).sub(o);

        let i = 1;
        let s = SimpleObservable;
        let o = LocalObserver(&i);
        //s.sub(o);
        s.rx().take(1).sub(Mss::new(o));
    }
}