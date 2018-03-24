use std::marker::PhantomData;
use std::cell::RefCell;
use std::ops::Range;
use std::iter::Step;
use std::any::{Any, TypeId};

use observable::*;
use std::rc::Rc;
use std::fmt::Debug;
use observable::*;
use subref::SubRef;
use std::sync::Arc;
use observable::Observable;
use observable::Observer;
use subref::IntoSubRef;
use util::mss::*;
use std::cell::UnsafeCell;
use std::mem;

pub fn create<'a:'b, 'b, V:'a, F, R>(sub: F) -> impl Observable<'a, V, No+'static>+'b where F: FnMut(Mss<No, &(Observer<V>+'a)>) -> R, R: IntoSubRef+'static
{
    struct LocalObservable<'a, V, F, R>(F, PhantomData<(&'a(), *const R, *const V)>) where F: 'a+Fn(Mss<No, &(Observer<V>+'a)>) -> R, R: IntoSubRef+'static;
    impl<'a:'b,'b,V, F, R> Observable<'a, V, No> for LocalObservable<'a, V, F, R> where F: 'a+Fn(Mss<No, &(Observer<V>+'a)>) -> R, R: IntoSubRef+'static
    {
        fn sub(&self, o: Mss<No, impl Observer<V>+'a>) -> SubRef
        {
            let sub = (self.0)(Mss::no(&o.into_inner()));
            IntoSubRef::into(sub)
        }
    }
    let cell = UnsafeCell::new(sub);
    unsafe {
        LocalObservable(move |o| (*cell.get())(o), PhantomData)
    }
}

pub fn create_boxed<'a:'b, 'b, V:'a, F, R>(sub: F) -> impl Observable<'a, V, No+'static>+'b where F: FnMut(Mss<No, Box<Observer<V>+'a>>) -> R, R: IntoSubRef+'static
{
    struct LocalObservable<'a, V, F, R>(F, PhantomData<(&'a(), R, V)>) where F: 'a+Fn(Mss<No, Box<Observer<V>+'a>>) -> R, R: IntoSubRef+'static;
    impl<'a:'b, 'b, V:'a, F, R> Observable<'a, V, No> for LocalObservable<'a, V, F, R> where F: 'a+Fn(Mss<No, Box<Observer<V>+'a>>) -> R, R: IntoSubRef+'static
    {
        fn sub(&self, o: Mss<No, impl Observer<V>+'a>) -> SubRef
        {
            let sub = (self.0)(o.into_boxed());
            IntoSubRef::into(sub)
        }
    }

    let cell = UnsafeCell::new(sub);
    unsafe { LocalObservable(move |o| (*cell.get())(o),PhantomData) }
}

pub fn create_sso<'a, V, F, R>(sub: F) -> impl Observable<'static, V, Yes> where F: Send+Sync+Fn(Mss<Yes, Box<Observer<V>+'static>>) -> R, R: IntoSubRef
{
    struct SendSyncObserverObservable<V, F, R>(F, PhantomData<(R, V)>) where F: Send+Sync+Fn(Mss<Yes, Box<Observer<V>+'static>>) -> R, R: IntoSubRef;
    impl<V, F, R> Observable<'static, V, Yes> for SendSyncObserverObservable<V, F, R> where F: Send+Sync+Fn(Mss<Yes, Box<Observer<V>+'static>>) -> R, R: IntoSubRef
    {
        fn sub(&self, o: Mss<Yes, impl Observer<V>+'static>) -> SubRef
        {
            let sub = (self.0)(o.into_boxed());
            IntoSubRef::into(sub)
        }
    }
    SendSyncObserverObservable(sub, PhantomData)
}


#[cfg(test)]
mod test
{
    use super::*;
    use std::sync::atomic::AtomicIsize;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    #[test]
    fn basic()
    {
        let mut i = 100;

        {
            let j = 0;
            let src = create(|o| {
                o.next(1+i);
                o.next(2);
                o.complete();
                i+=1;
            });
            src.subf(|v| println!("{}", v+j));
        }

        assert_eq!(i, 101);
    }

    #[test]
    fn send_sync()
    {
        let src = create_sso(|o| {
            o.next(1);
            ::std::thread::spawn(move ||{
                o.next(2);
                o.complete();
            });
        });

        let i = Arc::new(AtomicIsize::new(0));
        let i2 = i.clone();
        src.subf(move |v| i2.fetch_add(v, Ordering::SeqCst));

        ::std::thread::sleep(Duration::from_millis(100));
        assert_eq!(i.load(Ordering::SeqCst), 3);
    }
}