use std::marker::PhantomData;
use std::any::{Any};
use std::rc::Rc;
use std::sync::atomic::AtomicIsize;

use observable::*;
use subref::SubRef;
use std::sync::Arc;
use std::sync::atomic::Ordering;

#[derive(Clone)]
pub struct StartWithOp<'a:'b, 'b, V>
{
    source: Arc<Observable<'a,V>+'b+Send+Sync>,
    v: V,
}

pub trait ObservableStartWith<'a, 'b, V>
{
    fn start_with(self, v: V) -> Arc<Observable<'a,V>+'b+Send+Sync>;
}

impl<'a:'b, 'b, V:'a+Send+Sync+Clone> ObservableStartWith<'a, 'b, V> for Arc<Observable<'a,V>+'b+Send+Sync>
{
    fn start_with(self, v: V) -> Arc<Observable<'a,V>+'b+Send+Sync>
    {
        Arc::new(StartWithOp{ v, source: self  })
    }
}

impl<'a:'b, 'b, V:'a+Send+Sync+Clone> Observable<'a, V> for StartWithOp<'a,'b, V>
{
    #[inline]
    fn sub(&self, dest: Arc<Observer<V> + Send + Sync+'a>) -> SubRef
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
            rxfac::range(1..3).rx().start_with(3).subf(|v| result += v );
        }
        assert_eq!(6, result);
    }


}