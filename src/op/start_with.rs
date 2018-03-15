use std::marker::PhantomData;
use std::any::{Any};
use std::rc::Rc;
use std::sync::atomic::AtomicIsize;

use observable::*;
use subscriber::*;
use subref::SubRef;
use std::sync::Arc;
use std::sync::atomic::Ordering;

#[derive(Clone)]
pub struct StartWithOp<Src, V>
{
    source: Src,
    v: V,
}

pub trait ObservableStartWith<'a, Src, V> where Src : Observable<'a, V>
{
    fn start_with(self, v: V) -> StartWithOp<Src, V>;
}

impl<'a, Src, V> ObservableStartWith<'a, Src, V> for Src where Src : Observable<'a, V>,
{
    fn start_with(self, v: V) -> StartWithOp<Self, V>
    {
        StartWithOp{ v, source: self  }
    }
}

impl<'a, Src, V:'a+Send+Sync+Clone> Observable<'a, V> for StartWithOp<Src, V> where Src: Observable<'a, V>
{
    #[inline]
    fn sub(&self, dest: impl Observer<V> + Send + Sync+'a) -> SubRef
    {
        dest.next(self.v.clone());
        if dest._is_closed() {
            return SubRef::empty();
        }
        self.source.sub(dest)
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use subject::*;
    use fac::*;
    use observable::RxNoti::*;

    #[test]
    fn basic()
    {
        let mut result = 0;
        {
            rxfac::range(1..3).start_with(3).subf(|v| result += v );
        }
        assert_eq!(6, result);
    }


}