use util::mss::*;
use std::marker::PhantomData;
use observable::*;
use subref::*;

pub struct StartWithOp<Src, V, SSO: ? Sized>
{
    source: Src,
    v: V,
    PhantomData: PhantomData<SSO>
}

pub trait ObservableStartWith<'a, Src, V, SSO:?Sized> where Src : Observable<'a, V, SSO>
{
    fn start_with(self, v: V) -> StartWithOp<Src, V, SSO>;
}

impl<'a, Src, V, SSO:?Sized> ObservableStartWith<'a, Src, V, SSO> for Src where Src : Observable<'a, V, SSO>,
{
    fn start_with(self, v: V) -> StartWithOp<Self, V,SSO>
    {
        StartWithOp{ v, source: self, PhantomData  }
    }
}

impl<'a, Src, V:'a+Send+Sync+Clone, SSO:?Sized> Observable<'a, V,SSO> for StartWithOp<Src, V,SSO> where Src: Observable<'a, V,SSO>
{
    #[inline]
    fn sub(&self, o: Mss<SSO, impl Observer<V> +'a>) -> SubRef
    {
        o.next(self.v.clone());
        if o._is_closed() {
            return SubRef::empty();
        }
        self.source.sub(o)
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use fac::*;
    use observable::RxNoti::*;

    #[test]
    fn basic()
    {
        //let mut result = 0;
        //{
        //    rxfac::range(1..3).start_with(3).subf(|v| result += v );
        //}
        //assert_eq!(6, result);
    }


}