use std::marker::PhantomData;
use std::any::Any;
use std::rc::Rc;
use observable::*;
use subref::SubRef;
use std::sync::Arc;
use observable::RxNoti::*;

pub struct MapState<FProj>
{
    proj: FProj
}

#[derive(Clone)]
pub struct MapOp<FProj, V, Src>
{
    proj: FProj,
    source: Src,
    PhantomData: PhantomData<(V)>
}

pub trait ObservableOpMap<'a, V, Src>
{
    fn map<FProj,VOut>(self, proj: FProj) -> MapOp< FProj, V, Src> where FProj : 'a + Send+Sync+Fn(V)->VOut;
}

impl<'a, V, Src> ObservableOpMap<'a, V, Src> for Src where Src : Observable<'a, V>
{
    #[inline(always)]
    fn map<FProj,VOut>(self, proj: FProj) -> MapOp< FProj, V, Src> where FProj : 'a + Send+Sync+Fn(V)->VOut
    {
        MapOp{ proj: proj, source: self, PhantomData }
    }
}

impl<'a, 'f, V:'static+Send+Sync, Src, VOut:'static+Send+Sync, FProj> Observable<'a, VOut> for MapOp<FProj, V, Src> where
    FProj : 'a + Clone+Send+Sync+Fn(V)->VOut,
    Src: Observable<'a, V>,
{
    #[inline(always)]
    fn sub(&self, dest: impl Observer<VOut> + Send + Sync+'a) -> SubRef
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
    use subject::*;
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

        let result = Arc::new(AtomicUsize::new(0));
        let result2 = result.clone();

        let s = Subject::<usize>::new();

        s.rx().map(|v| v*2018 ).subf(move |v| { result2.fetch_add(v, Ordering::SeqCst); }).unsub();

        s.next(1);

        assert_eq!(result.load(Ordering::SeqCst), 0);
    }
}