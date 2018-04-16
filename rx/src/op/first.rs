use std::marker::PhantomData;
use observable::*;
use subref::*;
use observable::RxNoti::*;
use util::mss::*;
use std::sync::Arc;
//use scheduler::*;

pub struct FirstOpError
{

}

#[derive(Clone)]
pub struct FirstOp<'o, Src, V, F, SSO: ?Sized, SSS: ?Sized> where F:'o+Clone+Fn(&V)->bool
{
    source: Src,
    pred: F,
    PhantomData: PhantomData<(&'o(), *const V, *const SSO, *const SSS)>,
}

pub trait ObservableFirst<'o, Src, V, SSO: ? Sized, SSS: ?Sized> where Src: Observable<'o, V, SSO, SSS>
{
    fn first(self) -> FirstOp<'o, Src, V, fn(&V)->bool, SSO, SSS>;
}

impl<'o, Src, V, SSO: ? Sized, SSS: ? Sized> ObservableFirst<'o, Src, V, SSO, SSS> for Src where Src: Observable<'o, V, SSO, SSS>,
{
    #[inline(always)]
    fn first(self) -> FirstOp<'o, Self, V, fn(&V)->bool, SSO, SSS>
    {
        fn _true<V>(_:&V)->bool{ true }
        FirstOp { pred:_true, PhantomData, source: self }
    }
}

macro_rules! fn_sub (
($s:ty, $sss:ty) => (fn_sub!($s, $sss, $sss););

($s: ty, $sss: ty, $inner_sss: ty)=>{
    #[inline(always)]
    fn sub(&self, o: Mss<$s, impl Observer<V> +'o>) -> SubRef<$sss>
    {
        let sub = InnerSubRef::<$inner_sss>::signal();
        let pred = self.pred.clone();

        sub.add(self.source.sub_noti(byclone!(sub,pred => move |n| {
            match n {
                Next(v) => {
                    if pred(&v) {
                        sub.unsub();
                        o.next(v);
                        o.complete();
                        return IsClosed::True;
                    }else if o._is_closed() {
                        sub.unsub();
                        return IsClosed::True;
                    }
                },
                Err(e) => {
                    sub.unsub();
                    o.err(e);
                },
                Comp => {
                    if ! sub.disposed() {
                        sub.unsub();
                        o.err(Arc::new(box FirstOpError{}));
                    }
                }
            }
            IsClosed::Default
        })));

        sub.into_subref()
    }
});

impl<'o, Src, V:'o, F> Observable<'o, V, Yes, Yes> for FirstOp<'o, Src, V, F, Yes, Yes> where
    Src: Observable<'o, V, Yes, Yes>,F:'o+Clone+Send+Sync+Fn(&V)->bool
{
    fn_sub!(Yes, Yes);
}

impl<'o, Src, V:'o, F> Observable<'o, V, No, No> for FirstOp<'o, Src, V, F, No, No> where
    Src: Observable<'o, V, No, No>,F:'o+Clone+Fn(&V)->bool
{
    fn_sub!(No, No);
}

impl<'o, Src, V:'o, F> Observable<'o, V, No, Yes> for FirstOp<'o, Src, V, F, No, Yes> where
    Src: Observable<'o, V, No, Yes>,F:'o+Clone+Fn(&V)->bool
{
    fn_sub!(No, Yes);
}
//
//impl<'a, 'me, Src, V> Observable<'a, 'me, V, Yes, No> for FirstOp<'me, Src, V, Yes, No> where Src: Observable<'a, 'me, V, Yes, No>
//{
//    //fn_sub!(Yes, No, Yes);
//    fn sub(&'me self, o: Mss<Yes, impl Observer<V> +'a>) -> SubRef<No>
//    {
//        if self.total <= 0 {
//            o.complete();
//            return SubRef::<No>::empty();
//        }
//
//        let sub = InnerSubRef::<No>::signal();
//        let inner = get_sync_context().unwrap().create_send(box byclone!(sub => move ||{
//            sub.unsub();
//        }));
//        sub.addss(inner.clone());
//
//        let mut count = self.total;
//
//        sub.added(self.source.sub_noti(byclone!(inner => move |n| {
//            match n {
//                Next(v) => {
//                    count -= 1;
//                    if count > 0 {
//                    o.next(v);
//                    if o._is_closed() {
//                        inner.unsub();
//                        return IsClosed::True;
//                    }
//                }else {
//                    o.next(v);
//                    inner.unsub();
//                    o.complete();
//                    return IsClosed::True;
//                }
//                },
//                Err(e) => {
//                    inner.unsub();
//                    o.err(e);
//                },
//                Comp => {
//                    inner.unsub();
//                    o.complete()
//                }
//            }
//
//            IsClosed::Default
//        }))).into_subref()
//    }
//}

#[cfg(test)]
mod test
{
    use super::*;
    use observable::RxNoti::*;
    use test_fixture::*;
    use std::rc::Rc;
    use std::cell::Cell;
    use std::cell::{RefCell, Ref};
    use fac;

    #[test]
    fn lifetime()
    {
        //let s = SimpleObservable;
        fac::empty::<i32>().first().subf((
            |v| println!("v={}", v),
            |e| print!("error!"),
            || println!("complete")
        ));
    }
}