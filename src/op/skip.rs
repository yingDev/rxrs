use std::marker::PhantomData;
use std::any::{Any};
use std::rc::Rc;
use std::sync::atomic::AtomicIsize;

use observable::*;
use subref::SubRef;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use observable::*;
use observable::RxNoti::*;

#[derive(Clone)]
pub struct SkipOp<'a:'b, 'b, V>
{
    source: Arc<Observable<'a,V>+'b+Send+Sync>,
    total: usize,
}

pub trait ObservableSkip<'a:'b, 'b, V>
{
    fn skip(self, total: usize) -> Arc<Observable<'a,V>+'b+Send+Sync>;
}

impl<'a:'b,'b, V:'a+Send+Sync> ObservableSkip<'a,'b, V> for Arc<Observable<'a, V>+'b+Send+Sync>
{
    #[inline(always)]
    fn skip(self, total: usize) -> Arc<Observable<'a,V>+'b+Send+Sync>
    {
        Arc::new(SkipOp{ total, source: self })
    }
}

impl<'a:'b,'b, V:'a+Send+Sync> Observable<'a,V> for SkipOp<'a, 'b, V>
{
    #[inline(always)]
    fn sub(&self, dest: Arc<Observer<V> + Send + Sync+'a>) -> SubRef
    {
        let mut count = self.total;
        if count == 0 {
            return self.source.sub(dest);
        }

        self.source.sub_noti(move |n|{
            match n {
                Next(v) => {
                    if count == 0 {
                        dest.next(v);
                        if dest._is_closed() { return IsClosed::True; }
                    } else { count -= 1; }
                },
                Err(e) => dest.err(e),
                Comp => dest.complete()
            }
            IsClosed::Default
        })
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use subject::*;
    use observable::RxNoti::*;

    #[test]
    fn basic()
    {
        let mut result = 0;
        {
            let s = Subject::anew();

            s.rx().skip(1).sub_noti(|n| match n {
               Next(v) => result += v ,
               Comp => result += 100,
                _=> {}
            });
            s.next(1);
            s.next(2);
            s.complete();
        }

        assert_eq!(result, 102);
    }
}