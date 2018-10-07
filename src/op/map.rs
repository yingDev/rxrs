use std::marker::PhantomData;
use crate::*;

pub trait Mapper<V, VOut, E, EOut>
{
    fn next(&self, v:V) -> VOut;
    fn error(&self, e:E) -> EOut;
}

impl<V, VOut, E, FN> Mapper<V, VOut, E, E> for FN where FN: Fn(V)->VOut
{
    #[inline(always)] fn next(&self, v:V) -> VOut { self.call((v,)) }
    #[inline(always)] fn error(&self, e:E) -> E { e }
}

impl<V, E, EOut, FE> Mapper<V, V, E, EOut> for ((), FE) where FE: Fn(E)->EOut
{
    #[inline(always)] fn next(&self, v:V) -> V { v }
    #[inline(always)] fn error(&self, e:E) -> EOut { self.1.call((e,)) }
}

impl<V, VOut, E, EOut, FN, FE> Mapper<V, VOut, E, EOut> for (FN, FE) where FN: Fn(V)->VOut, FE: Fn(E)->EOut
{
    #[inline(always)] fn next(&self, v:V) -> VOut { self.0.call((v,)) }
    #[inline(always)] fn error(&self, e:E) -> EOut { self.1.call((e,)) }
}



pub trait ObservableOpMap<V, VOut, E, EOut, SS> : Sized
{
    fn map<F: Mapper<V, VOut, E, EOut>>(self,f: F) -> OpMap<V, VOut, E, EOut, Self, F, SS>
    {
        OpMap { src: self, f, PhantomData }
    }
}

impl<'s, 'o, V:Clone,VOut:Clone, E:Clone, EOut:Clone, Src> ObservableOpMap<V, VOut, E, EOut, NO> for Src where Src : Observable<'s, 'o, V, E> {}
impl<'s, V:Clone+Send+Sync,VOut:Clone+Send+Sync, E:Clone+Send+Sync, EOut:Clone+Send+Sync, Src> ObservableOpMap<V, VOut, E, EOut, YES> for Src where Src : ObservableSendSync<'s, V, E> {}

pub struct OpMap<V, VOut, E, EOut, Src, F, SS> where F : Mapper<V,VOut,E,EOut>
{
    src: Src,
    f: F,
    PhantomData: PhantomData<(V, VOut, E, EOut, SS)>
}

impl<'s, 'o, V:Clone+'o, E:Clone+'o, VOut:Clone+'o, EOut:Clone+'o, Src: Observable<'s, 'o, V, E>, F: Mapper<V,VOut, E, EOut>+Clone+'o> Observable<'s, 'o, VOut, EOut> for OpMap<V, VOut, E, EOut, Src, F, NO>
{
    fn subscribe(&'s self, observer: impl Observer<VOut,EOut>+'o) -> Subscription<'o, NO>
    {
        self.src.subscribe( OpMapSubscriber { observer, f: self.f.clone(), PhantomData })
    }
}

impl<'s, V:Clone+Send+Sync+'static, E:Clone+Send+Sync+'static, EOut: Clone+Send+Sync+'static, VOut:Clone+Send+Sync+'static, Src: ObservableSendSync<'s, V, E>, F: Mapper<V,VOut, E, EOut>+Send+Sync+Clone+'static> ObservableSendSync<'s, VOut, EOut> for OpMap<V, VOut, E, EOut, Src, F, YES>
{
    fn subscribe(&'s self, observer: impl Observer<VOut,EOut>+Send+Sync+'static) -> Subscription<'static, YES>
    {
        self.src.subscribe( OpMapSubscriber { observer, f: self.f.clone(), PhantomData})
    }
}

struct OpMapSubscriber<V, VOut, E, EOut, Dest, F: Mapper<V,VOut, E, EOut>>
{
    observer: Dest,
    f: F,
    PhantomData: PhantomData<(V, VOut, E, EOut)>
}

impl<'a, V:Clone, VOut:Clone, E:Clone, EOut:Clone, Dest: Observer<VOut,EOut>, F: Mapper<V,VOut, E, EOut>> Observer<V,E> for OpMapSubscriber<V, VOut, E, EOut, Dest, F>
{
    #[inline(always)] fn next(&self, v:V) { self.observer.next(self.f.next(v)) }
    #[inline(always)] fn error(&self, e: E) { self.observer.error(self.f.error(e)) }
    #[inline(always)] fn complete(&self) { self.observer.complete() }
}

#[cfg(test)]
mod test
{
    use crate::*;
    use super::*;
    use std::sync::atomic::*;

    #[test]
    fn basic()
    {
        let n = ::std::cell::Cell::new(0);
        let o = of(123, NO);
        o.map(|v| v+1).subscribe(|v| { n.replace(v); } );

        assert_eq!(n.get(), 124);
    }

    #[test]
    fn thread()
    {
        let n = ::std::sync::Arc::new(AtomicI32::new(0));
        let o = of(123, YES);
        let nn = n.clone();

        let mapped = o.map(|v| v + 1);

        ::std::thread::spawn(move ||{
            mapped.subscribe(move |v| { nn.store(v, Ordering::SeqCst); });
        }).join();

        assert_eq!(n.load(Ordering::SeqCst), 124);
    }

    #[test]
    fn multiple()
    {
        let n = ::std::cell::Cell::new(0);
        let o = of(0, NO);
        o.map(|v| v+1).map(|v| v+1).map(|v| v+1).map(|v| v+1).subscribe(|v| { n.replace(v); } );

        assert_eq!(n.get(), 4);
    }

    #[test]
    fn src_rc()
    {
        let n = ::std::cell::Cell::new(0);
        let o = ::std::rc::Rc::new(of(123, NO));
        let a = o.clone().map(|v| v+1);
        let b = o.clone().map(|v| v+2);

        a.subscribe(|v| { n.replace(v); } );
        assert_eq!(n.get(), 124);

        b.subscribe(|v| { n.replace(v); });
        assert_eq!(n.get(), 125);
    }
}