use std::marker::PhantomData;
use std::any::Any;
use std::rc::Rc;
use subscriber::*;
use observable::*;
use unsub_ref::UnsubRef;
use std::sync::Arc;

pub struct MapState<FProj>
{
    proj: Arc<FProj>
}

#[derive(Clone)]
pub struct MapOp<FProj, V, Src>
{
    proj: Arc<FProj>,
    source: Src,
    PhantomData: PhantomData<V>
}

pub trait ObservableOpMap<V, Src>
{
    fn map<FProj,VOut>(self, proj: FProj) -> MapOp< FProj, V, Src> where FProj : 'static + Fn(V)->VOut;
}

impl<V, Src> ObservableOpMap<V, Src> for Src where Src : Observable<  V>
{
    fn map<FProj,VOut>(self, proj: FProj) -> MapOp<FProj, V, Src> where FProj : 'static + Fn(V)->VOut
    {
        MapOp{ proj: Arc::new(proj), source: self, PhantomData }
    }
}

impl<V:'static+Send+Sync, Src, VOut:'static+Send+Sync, FProj> Observable<VOut> for MapOp<FProj, V, Src> where FProj : Send+Sync+'static + Fn(V)->VOut, Src: Observable<V>
{
    fn sub(&self, dest: Arc<Observer<VOut>+Send+Sync>) -> UnsubRef<'static>
    {
        let s = Arc::new(Subscriber::new(MapState{ proj: self.proj.clone() }, dest, false));
        let sub = self.source.sub(s.clone());
        s.set_unsub(&sub);

        sub
    }
}

impl<V,VOut,FProj> SubscriberImpl<V,MapState<FProj>> for Subscriber<V,MapState<FProj>,VOut> where FProj: Fn(V)->VOut
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

    #[test]
    fn basic()
    {

        let result = Arc::new(AtomicUsize::new(0));
        let result2 = result.clone();

        let s = Subject::<usize>::new();

        s.rx().map(|v| v*2018 ).subn(move |v| { result2.fetch_add(v, Ordering::SeqCst); });

        s.next(1);

        assert_eq!(result.load(Ordering::SeqCst), 2018);
    }

    #[test]
    fn unsub()
    {

        let result = Arc::new(AtomicUsize::new(0));
        let result2 = result.clone();

        let s = Subject::<usize>::new();

        s.rx().map(|v| v*2018 ).subn(move |v| { result2.fetch_add(v, Ordering::SeqCst); }).unsub();

        s.next(1);

        assert_eq!(result.load(Ordering::SeqCst), 0);
    }
}