use std::marker::PhantomData;
use std::any::Any;
use std::rc::Rc;
use observable::*;
use subref::SubRef;
use std::sync::Arc;
use observable::*;
use observable::RxNoti::*;
use util::mss::*;


#[derive(Clone)]
pub struct FilterOp<'a:'b, 'b, Src:'b, V, F, SSO:?Sized+'static>
{
    source: Src,
    pred: F,
    PhantomData: PhantomData<(*const V,&'a(), &'b (), &'static SSO)>
}

pub trait ObservableFilter<'a, 'b, Src, V, F, SSO:?Sized> where
    Src : Observable<'a, V, SSO>+'b
{
    fn filter(self, pred: F) -> FilterOp<'a, 'b, Src, V, F, SSO>;
}

impl<'b, Src, V:'static+Send+Sync, F:'static> ObservableFilter<'static, 'b, Src, V, F, Yes> for Src where
    Src : Observable<'static, V, Yes>+'b, F: 'static+Clone+Send+Sync+Fn(&V)->bool
{
    #[inline(always)]
    fn filter(self, pred: F) -> FilterOp<'static, 'b, Src, V, F, Yes>
    {
        FilterOp { source: self, pred: pred, PhantomData }
    }
}

impl<'a:'b, 'b, Src, V, F> ObservableFilter<'a, 'b, Src, V, F, No> for Src where Src : Observable<'a, V, No>+'b, F: 'a+Clone+Fn(&V)->bool
{
    #[inline(always)]
    fn filter(self, pred: F) -> FilterOp<'a, 'b, Src, V, F, No>
    {
        FilterOp { source: self, pred: pred, PhantomData }
    }
}

macro_rules! fn_sub(($s: ty) => {
    fn sub(&self, o: Mss<$s, impl Observer<V> +'a>) -> SubRef
    {
        if o._is_closed() {
            return SubRef::empty();
        }

        let f = self.pred.clone();
        let sub = SubRef::signal();

        sub.add(self.source.sub_noti(byclone!(sub => move |n| {
            match n {
                Next(v) => {
                    if f(&v) { o.next(v) };
                    if o._is_closed() {
                        sub.unsub();
                        return IsClosed::True;
                    }
                },
                Err(e) => {
                    sub.unsub();
                    o.err(e);
                },
                Comp => {
                    sub.unsub();
                    o.complete();
                }
            }
            IsClosed::Default
        })).added(sub.clone()));

        sub
    }
});

impl<'a:'static, 'b, Src, V:'static+Send+Sync, F> Observable<'a, V, Yes> for FilterOp<'a, 'b, Src, V, F, Yes>  where
    Src : Observable<'static, V, Yes>+'b, F: 'static+Clone+Send+Sync+Fn(&V)->bool
{
    fn_sub!(Yes);
}
impl<'a:'b, 'b, Src, V:'a, F> Observable<'a, V, No> for FilterOp<'a, 'b,  Src, V, F, No>  where
    Src : Observable<'a, V, No>+'b, F: 'a+Clone+Fn(&V)->bool
{
    fn_sub!(No);
}

#[cfg(test)]
mod test
{
    use super::*;
    use test_fixture::*;

    #[test]
    fn basic()
    {
        let s = SimpleObservable;
        s.filter(|v:&_| v> &1).subf(|v| println!("v={}", v));

        let s = ThreadedObservable;
        s.rx().filter(|v:&_| v>&1).subf(|v| println!("{}", v));
    }
}