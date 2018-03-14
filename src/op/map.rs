use std::marker::PhantomData;
use std::any::Any;
use std::rc::Rc;
use subscriber::*;
use observable::*;
use subref::SubRef;
use std::sync::Arc;

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
    fn map<FProj,VOut>(self, proj: FProj) -> MapOp< FProj, V, Src> where FProj : 'a + Send+Sync+Fn(V)->VOut
    {
        MapOp{ proj: proj, source: self, PhantomData }
    }
}

impl<'a, 'f, V:'static+Send+Sync, Src, VOut:'static+Send+Sync, FProj> Observable<'a, VOut> for MapOp<FProj, V, Src> where
    FProj : 'a + Clone+Send+Sync+Fn(V)->VOut,
    Src: Observable<'a, V>,
{
    fn sub(&self, dest: impl Observer<VOut> + Send + Sync+'a) -> SubRef
    {
        let s = Subscriber::new(MapState{ proj: self.proj.clone() }, dest, false);
        s.do_sub(&self.source)
    }
}

impl<'a, V,Dest,VOut,FProj> SubscriberImpl<V,MapState<FProj>> for Subscriber<'a, V, MapState<FProj>,Dest,VOut> where
    FProj : 'a + Send+Sync+Fn(V)->VOut,
    Dest : Observer<VOut>+Send+Sync+'a
{
    fn on_next(&self, v: V)
    {
        self._dest.next( (self._state.proj)(v) );
    }

    fn on_err(&self, e: Arc<Any+Send+Sync>)
    {
        self.do_unsub();
        self._dest.err(e);
    }

    fn on_comp(&self)
    {
        self.do_unsub();
        self._dest.complete();
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