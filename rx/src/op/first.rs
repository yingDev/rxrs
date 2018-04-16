use std::marker::PhantomData;
use observable::*;
use subref::*;
use observable::RxNoti::*;
use util::mss::*;
use std::sync::Arc;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt;
//use scheduler::*;

#[derive(Debug)]
pub struct FirstOpError
{

}

impl Display for FirstOpError
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Error for FirstOpError
{
    fn description(&self) -> &str
    {
        "Source completed with no item"
    }
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
pub trait ObservableFirstIf<'o, Src, V, F, SSO: ? Sized, SSS: ?Sized> where Src: Observable<'o, V, SSO, SSS>, F:'o+Clone+Fn(&V)->bool
{
    fn first_if(self, pred: F) -> FirstOp<'o, Src, V, F, SSO, SSS>;
}

impl<'o, Src, V,  SSO: ? Sized, SSS: ? Sized> ObservableFirst<'o, Src, V, SSO, SSS> for Src where Src: Observable<'o, V, SSO, SSS>
{
    #[inline(always)]
    fn first(self) -> FirstOp<'o, Self, V, fn(&V)->bool, SSO, SSS>
    {
        fn _true<V>(_:&V)->bool{ true }
        FirstOp { pred:_true, PhantomData, source: self }
    }
}
impl<'o, Src, V, F, SSO: ? Sized, SSS: ? Sized> ObservableFirstIf<'o, Src, V, F, SSO, SSS> for Src where Src: Observable<'o, V, SSO, SSS>,F:'o+Clone+Fn(&V)->bool
{
    #[inline(always)]
    fn first_if(self, f:F) -> FirstOp<'o, Self, V, F, SSO, SSS>
    {
        FirstOp { pred:f, PhantomData, source: self }
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
    fn empty()
    {
        let mut r = Cell::new(0);
        //let s = SimpleObservable;
        fac::empty::<i32>().first().subf((
            |v| r.replace(r.get()+1),
            |e: ArcErr| {
                if let Some(ref e) = e.downcast_ref::<FirstOpError>() {
                    println!("{}", e);
                    r.replace(r.get() + 2);
                }
            },
            || r.replace(r.get()+3)
        ));

        assert_eq!(2, r.get());
    }

    #[test]
    fn pred()
    {
        fac::range(0,3).first_if(|i:&_| *i>1).subf(|v| println!("v={}",v));
        fac::range(0,3).first_if(|i:&_| *i== -1).subf(((), |e| println!("error")));
    }
}