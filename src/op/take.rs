use std::marker::PhantomData;
use observable::*;
use subref::SubRef;
use observable::RxNoti::*;
use util::mss::*;

#[derive(Clone)]
pub struct TakeOp<Src, V, SSO:?Sized>
{
    source: Src,
    total: isize,
    PhantomData: PhantomData<(V, SSO)>
}

pub trait ObservableTake<'a, Src, V, SSO:?Sized> where Src : Observable<'a, V, SSO>
{
    fn take(self, total: isize) -> TakeOp<Src, V, SSO>;
}

impl<'a, Src, V, SSO:?Sized> ObservableTake<'a, Src, V, SSO> for Src where Src : Observable<'a, V, SSO>,
{
    #[inline(always)]
    fn take(self, total: isize) -> TakeOp<Self, V, SSO>
    {
        TakeOp{ total, PhantomData, source: self  }
    }
}

macro_rules! fn_sub(
($s: ty)=>{
    #[inline(always)]
    fn sub(&self, dest: Mss<$s, impl Observer<V> +'a>) -> SubRef
    {
        if self.total <= 0 {
            dest.complete();
            return SubRef::empty();
        }

        let sub = SubRef::signal();
        let mut count = self.total;

        sub.add(self.source.sub_noti(byclone!(sub => move |n| {
            match n {
                Next(v) => {
                    count -= 1;
                    if count > 0 {
                        dest.next(v);
                        if dest._is_closed() {
                           sub.unsub();
                           return IsClosed::True;
                        }
                    }else {
                        dest.next(v);
                        sub.unsub();
                        dest.complete();
                        return IsClosed::True;
                    }
                },
                Err(e) => {
                    sub.unsub();
                    dest.err(e);
                },
                Comp => {
                    sub.unsub();
                    dest.complete()
                }
            }
            IsClosed::Default
        })));

        sub
    }
});

impl<'a, Src, V:'a> Observable<'a, V, Yes> for TakeOp<Src, V, Yes> where Src: Observable<'a, V, Yes>
{
    fn_sub!(Yes);
}
impl<'a, Src, V:'a> Observable<'a, V, No> for TakeOp<Src, V, No> where Src: Observable<'a, V, No>
{
    fn_sub!(No);
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