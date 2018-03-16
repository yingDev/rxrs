use std::marker::PhantomData;
use std::any::Any;
use std::rc::Rc;
use observable::*;
use subref::SubRef;
use std::sync::Arc;
use observable::RxNoti::*;

#[derive(Clone)]
pub struct MapOp<'a:'b, 'b,V, VOut, F >
{
    proj: F,
    source: Arc<Observable<'a,V>+'b+Send+Sync>,
    PhantomData: PhantomData<(VOut)>
}

pub trait ObservableOpMap<'a:'b, 'b, V, VOut, F> where F : 'a + Clone+Send+Sync+Fn(V)->VOut
{
    fn map(self, proj: F) -> Arc<Observable<'a,VOut>+'b+Send+Sync> ;
}

impl<'a:'b, 'b, V: 'a+Send+Sync, VOut: 'a+Send+Sync, F> ObservableOpMap<'a, 'b, V, VOut, F> for Arc<Observable<'a, V>+'b+Send+Sync> where
    F : 'a + Clone+Send+Sync+Fn(V)->VOut
{
    fn map(self, proj: F) -> Arc<Observable<'a,VOut>+'b+Send+Sync>
    {
        Arc::new(MapOp{ proj: proj, source: self, PhantomData })
    }
}

impl<'a:'b, 'b, V:'a+Send+Sync, VOut:'a+Send+Sync, F> Observable<'a, VOut> for MapOp<'a,'b,V,VOut,F> where
    F : 'a + Clone+Send+Sync+Fn(V)->VOut,
{
    fn sub(&self, dest: Arc<Observer<VOut> + Send + Sync+'a>) -> SubRef
    {
        let f = self.proj.clone();
        self.source.sub_noti(move |n| {
            match n {
                Next(v) => {
                    dest.next( f(v) );
                    if dest._is_closed() { return IsClosed::True; }
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
    use ::std::sync::atomic::*;
    use fac::rxfac;

    #[test]
    fn basic()
    {
        let mut r = 0;

        {
            let x = 2018;
            let src = rxfac::range(1..2);
            src.map(|v| v*x ).subf(|v| r += v );
        }

        assert_eq!(r, 2018);
    }

    #[test]
    fn unsub()
    {

//        let result = Arc::new(AtomicUsize::new(0));
//        let result2 = result.clone();
//
//        let s = Subject::<usize>::new();
//
//        s.rx().map(|v| v*2018 ).subf(move |v| { result2.fetch_add(v, Ordering::SeqCst); }).unsub();
//
//        s.next(1);
//
//        assert_eq!(result.load(Ordering::SeqCst), 0);
    }
}