use util::mss::*;
use std::marker::PhantomData;
use observable::*;
use subref::*;

pub struct StartWithOp<Src, V, SSO: ? Sized, SSS:?Sized>
{
    source: Src,
    v: V,
    PhantomData: PhantomData<(*const SSO, *const SSS)>
}

pub trait ObservableStartWith<'a, Src, V, SSO:?Sized, SSS:?Sized> where Src : Observable<'a, V, SSO, SSS>
{
    fn start_with(self, v: V) -> StartWithOp<Src, V, SSO, SSS>;
}

impl<'a, Src, V, SSO:?Sized, SSS:?Sized> ObservableStartWith<'a, Src, V, SSO, SSS> for Src where Src : Observable<'a, V, SSO, SSS>,
{
    fn start_with(self, v: V) -> StartWithOp<Self, V,SSO, SSS>
    {
        StartWithOp{ v, source: self, PhantomData  }
    }
}

impl<'a, Src, V:'a+Clone, SSO:?Sized, SSS:?Sized> Observable<'a, V,SSO,SSS> for StartWithOp<Src, V,SSO,SSS> where Src: Observable<'a, V,SSO,SSS>
{
    #[inline]
    fn sub(&self, o: Mss<SSO, impl Observer<V> +'a>) -> SubRef<SSS>
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
    use test_fixture::*;

    #[test]
    fn basic()
    {
        let mut out = 0;

        let src = SimpleObservable;
        src.start_with(100).subf(|v| out += v);

        assert_eq!(out, 106);
    }


}