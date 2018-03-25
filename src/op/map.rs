use std::marker::PhantomData;
use observable::*;
use subref::SubRef;
use observable::RxNoti::*;
use util::mss::*;

#[derive(Clone)]
pub struct MapOp<FProj, V, Src, SSO:?Sized>
{
    proj: FProj,
    source: Src,
    PhantomData: PhantomData<(V, SSO)>
}

pub trait ObservableOpMap<'a, V, Src, SSO:?Sized>
{
    fn map<FProj,VOut>(self, proj: FProj) -> MapOp< FProj, V, Src, SSO> where FProj : 'a + Fn(V)->VOut;
}

impl<'a, V, Src, SSO:?Sized> ObservableOpMap<'a, V, Src, SSO> for Src where Src : Observable<'a, V, SSO>
{
    #[inline(always)]
    fn map<FProj,VOut>(self, proj: FProj) -> MapOp< FProj, V, Src, SSO> where FProj : 'a +Fn(V)->VOut
    {
        MapOp{ proj: proj, source: self, PhantomData }
    }
}

macro_rules! fn_sub(
($s:ty) => {
    #[inline(always)]
    fn sub(&self, o: Mss<$s, impl Observer<VOut> +'a>) -> SubRef
    {
        let f = self.proj.clone();
        let sub = SubRef::signal();

        sub.clone().added(self.source.sub_noti(byclone!(sub => move |n| {
        match n {
            Next(v) => {
                o.next( f(v) );
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
        })).added(sub))
    }
});

impl<'a, V:'static+Send+Sync, Src, VOut:'static+Send+Sync, FProj> Observable<'a, VOut, Yes> for MapOp<FProj, V, Src, Yes> where
    FProj : 'a + Clone+Send+Sync+Fn(V)->VOut,
    Src: Observable<'a, V, Yes>,
{
    fn_sub!(Yes);
}

impl<'a, V:'a, Src, VOut:'static, FProj> Observable<'a, VOut, No> for MapOp<FProj, V, Src, No> where
    FProj : 'a + Clone + Fn(V)->VOut,
    Src: Observable<'a, V, No>
{
    fn_sub!(No);
}


#[cfg(test)]
mod test
{
//    use super::*;
//    use ::std::sync::atomic::*;
//    use fac::rxfac;
//
//    #[test]
//    fn basic()
//    {
//        let mut r = 0;
//
//        {
//            let x = 2018;
//            let src = rxfac::range(1..2);
//            src.map(|v| v*x ).subf(|v| r += v );
//        }
//
//        assert_eq!(r, 2018);
//    }
}