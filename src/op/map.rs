use std::marker::PhantomData;
use observable::*;
use subref::*;
use observable::RxNoti::*;
use util::mss::*;
use scheduler::get_sync_context;

#[derive(Clone)]
pub struct MapOp<FProj, V, Src, SSO:?Sized, SSS:?Sized>
{
    proj: FProj,
    source: Src,
    PhantomData: PhantomData<(V, *const SSO, *const SSS)>
}

pub trait ObservableOpMap<'a, V, Src, SSO:?Sized, SSS:?Sized>
{
    fn map<FProj,VOut>(self, proj: FProj) -> MapOp< FProj, V, Src, SSO, SSS> where FProj : 'a + Fn(V)->VOut;
}

impl<'a, V, Src, SSO:?Sized, SSS:?Sized> ObservableOpMap<'a, V, Src, SSO, SSS> for Src where Src : Observable<'a, V, SSO, SSS>
{
    #[inline(always)]
    fn map<FProj,VOut>(self, proj: FProj) -> MapOp< FProj, V, Src, SSO, SSS> where FProj : 'a +Fn(V)->VOut
    {
        MapOp{ proj: proj, source: self, PhantomData }
    }
}

macro_rules! fn_sub(
($s:ty, $sss: ty) => {
    #[inline(always)]
    fn sub(&self, o: Mss<$s, impl Observer<VOut> +'a>) -> SubRef<$sss>
    {
        let f = self.proj.clone();
        let sub = InnerSubRef::<$sss>::signal();

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
        }))).into()
    }
});

impl<'a, V:'static+Send+Sync, Src, VOut:'static+Send+Sync, FProj> Observable<'a, VOut, Yes, Yes> for MapOp<FProj, V, Src, Yes, Yes> where
    FProj : 'a + Clone+Send+Sync+Fn(V)->VOut,
    Src: Observable<'a, V, Yes, Yes>,
{
    fn_sub!(Yes, Yes);
}

impl<'a, V:'a, Src, VOut:'static, FProj> Observable<'a, VOut, No, No> for MapOp<FProj, V, Src, No, No> where
    FProj : 'a + Clone + Fn(V)->VOut,
    Src: Observable<'a, V, No, No>
{
    fn_sub!(No, No);
}

impl<'a, V:'a, Src, VOut:'static, FProj> Observable<'a, VOut, No, Yes> for MapOp<FProj, V, Src, No, Yes> where
    FProj : 'a + Clone + Fn(V)->VOut,
    Src: Observable<'a, V, No, Yes>
{
    fn_sub!(No, Yes);
}

impl<'a, V:'a, Src, VOut:'static, FProj> Observable<'a, VOut, Yes, No> for MapOp<FProj, V, Src, Yes, No> where
    FProj : 'a + Clone +Send+Sync+ Fn(V)->VOut,
    Src: Observable<'a, V, Yes, No>
{
    #[inline(always)]
    fn sub(&self, o: Mss<Yes, impl Observer<VOut> +'a>) -> SubRef<No>
    {
        let f = self.proj.clone();
        let sub = InnerSubRef::<No>::signal();
        let inner = get_sync_context().unwrap().create_send(box byclone!(sub => move ||{
            sub.unsub();
        }));
        sub.addss(inner.clone());

        sub.added(self.source.sub_noti(byclone!(inner => move |n| {
            match n {
                Next(v) => {
                    o.next( f(v) );
                    if o._is_closed() {
                        inner.unsub();
                        return IsClosed::True;
                    }
                },
                Err(e) => {
                    inner.unsub();
                    o.err(e);
                },
                Comp => {
                    inner.unsub();
                    o.complete();
                }
            }
                IsClosed::Default
         }))).into()
    }
}


#[cfg(test)]
mod test
{
    use super::*;
    use ::std::sync::atomic::*;
    use test_fixture::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn basic()
    {
        let mut out = 0;
        let src = SimpleObservable;

        src.map(|v| v*2).subf(|v| out += v);
        assert_eq!(out, 12);
    }

    #[test]
    fn threaded()
    {
        let  out = Arc::new(AtomicUsize::new(0));
        let src = ThreadedObservable;

        src.map(|v| v*2).subf(byclone!(out => move |v| out.fetch_add(v as usize, Ordering::SeqCst)));

        thread::sleep(Duration::from_millis(100));
        assert_eq!(out.load(Ordering::SeqCst), 12);
    }
}