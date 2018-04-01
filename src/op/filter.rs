use std::marker::PhantomData;
use observable::*;
use subref::SubRef;
use observable::RxNoti::*;
use util::mss::*;


#[derive(Clone)]
pub struct FilterOp<'a:'b, 'b, Src:'b, V, F, SSO:?Sized, SSS:?Sized>
{
    source: Src,
    pred: F,
    PhantomData: PhantomData<(*const V,&'a(), &'b (), *const SSO, *const SSS)>
}

pub trait ObservableFilter<'a, 'b, Src, V, F, SSO:?Sized, SSS:?Sized>
{
    fn filter(self, pred: F) -> FilterOp<'a, 'b, Src, V, F, SSO, SSS>;
}

impl<'a:'b, 'b, Src, V:'a+Send+Sync, F:'a, SSS:?Sized> ObservableFilter<'a, 'b, Src, V, F, Yes, SSS> for Src where
    Src : Observable<'a, V, Yes, SSS>+'b,
    F: 'a+Clone+Send+Sync+Fn(&V)->bool
{
    #[inline(always)]
    fn filter(self, pred: F) -> FilterOp<'a, 'b, Src, V, F, Yes, SSS>
    {
        FilterOp { source: self, pred, PhantomData }
    }
}

impl<'a:'b, 'b, Src, V, F, SSS:?Sized> ObservableFilter<'a, 'b, Src, V, F, No, SSS> for Src where
    Src : Observable<'a, V, No, SSS>+'b,
    F: 'a+Clone+Fn(&V)->bool
{
    #[inline(always)]
    fn filter(self, pred: F) -> FilterOp<'a, 'b, Src, V, F, No, SSS>
    {
        FilterOp { source: self, pred, PhantomData }
    }
}


macro_rules! fn_sub(($s: ty, $sss: ty) => {
    fn sub(&self, o: Mss<$s, impl Observer<V> +'a>) -> SubRef<$sss>
    {
        if o._is_closed() {
            return SubRef::empty();
        }

        let f = self.pred.clone();
        let sub = SubRef::<$sss>::signal();

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

impl<'a:'b, 'b, Src, V:'static+Send+Sync, F> Observable<'a, V, Yes, Yes> for FilterOp<'a, 'b, Src, V, F, Yes, Yes>  where
    Src : Observable<'a, V, Yes, Yes>+'b, F: 'a+Clone+Send+Sync+Fn(&V)->bool
{
    fn_sub!(Yes, Yes);
}
impl<'a:'b, 'b, Src, V:'a, F> Observable<'a, V, No, Yes> for FilterOp<'a, 'b,  Src, V, F, No, Yes>  where
    Src : Observable<'a, V, No, Yes>+'b, F: 'a+Clone+Fn(&V)->bool
{
    fn_sub!(No, Yes);
}

impl<'a:'b, 'b, Src, V:'a, F> Observable<'a, V, No, No> for FilterOp<'a, 'b,  Src, V, F, No, No>  where
    Src : Observable<'a, V, No, No>+'b, F: 'a+Clone+Fn(&V)->bool
{
    fn_sub!(No, No);
}



#[cfg(test)]
mod test
{
    use super::*;
    use test_fixture::*;

    #[test]
    fn basic()
    {
        let i = 0;
        let s = SimpleObservable;
        s.filter(|v:&_| *v> i).subf(|v| println!("v={}", v));

        let s = ThreadedObservable;
        s.rx().filter(|v:&_| *v>3).subf(|v| println!("{}", v));
    }
}