use crate::*;

#[derive(Copy, Clone)]
pub struct Of<V: Clone, SS: YesNo>(V, SS);

#[inline(always)]
pub fn of<V:Clone, SS: YesNo>(v:V, s:SS) -> Of<V, SS> { Of(v, s) }

impl<V> !Send for Of<V, NO> {}
impl<V> !Sync for Of<V, NO> {}

impl<V:Clone, SS:YesNo> Of<V,SS>
{
    #[inline(always)]
    fn sub_internal<'o>(&self, observer: impl Observer<V,()>+'o) -> Unsub<'o, SS>
    {
        observer.next(self.0.clone());
        observer.complete();
        Unsub::done()
    }
}

impl<'o, V: Clone+'o> Observable<'o, V, ()> for Of<V, NO>
{
    #[inline(always)] fn sub(&self, observer: impl Observer<V,()>+'o) -> Unsub<'o, NO> { self.sub_internal(observer) }

}

impl<V: Clone+Send+Sync+'static> ObservableSendSync<V, ()> for Of<V, YES>
{
    #[inline(always)] fn sub(&self, observer: impl Observer<V,()>+Send+Sync+'static) -> Unsub<'static, YES> { self.sub_internal(observer) }

}


#[cfg(test)]
mod test
{
    use crate::*;
    use std::sync::atomic::*;

    #[test]
    fn smoke()
    {
        let o = of(123, NO);
        o.sub(|v| println!("it works: {}", v));

        let o = of(456, YES);
        o.sub(|v| println!("it works: {}", v));

        ::std::thread::spawn(move ||{
            o.sub(|v| println!("it works: {}", v));
        }).join();
    }

    #[test]
    fn side_effects()
    {
        let cell = ::std::cell::Cell::new(123);
        let o = of(456, NO);
        o.sub(|v| { cell.replace(v); });
        assert_eq!(cell.get(), 456);

        let arc = ::std::sync::Arc::new(AtomicI32::new(123));
        let o = of(456, YES);
        let arclone = arc.clone();
        ::std::thread::spawn(move || {
            o.sub(move |v| { arclone.store(v, Ordering::SeqCst); });
        }).join();
        assert_eq!(arc.load(Ordering::SeqCst), 456);
    }

    #[test]
    fn complete()
    {
        let o = of(123, NO);
        let sub = o.sub(|v|{});

        assert!(sub.is_done());
    }
}