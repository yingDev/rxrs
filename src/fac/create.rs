use std::marker::PhantomData;
use subref::SubRef;
use observable::Observable;
use observable::Observer;
use subref::IntoSubRef;
use util::mss::*;
use std::cell::UnsafeCell;
use std::cell::RefCell;
use std::rc::Rc;

//todo: refactor

pub struct EmptyObservable<'a,V, SSS:?Sized>(PhantomData<(&'a(), *const V, *const SSS)>);
impl<'a,V, SSS:?Sized> Clone for EmptyObservable<'a,V,SSS>
{
    fn clone(&self) -> EmptyObservable<'a,V,SSS>
    {
        EmptyObservable(PhantomData)
    }
}
unsafe impl<'a,V,SSS:?Sized> Send for EmptyObservable<'a,V,SSS>{}
unsafe impl<'a,V,SSS:?Sized> Sync for EmptyObservable<'a,V,SSS>{}

impl<'a, V, SSS:Sized> Observable<'a, V, No, SSS> for EmptyObservable<'a,V, SSS>
{
    fn sub(&self, o: Mss<No, impl Observer<V>+'a>) -> SubRef<SSS>
    {
        o.complete();
        SubRef::empty()
    }
}

pub fn empty<'a, V>() -> EmptyObservable<'a,V, Yes>
{
    EmptyObservable(PhantomData)
}


pub struct JustObservable<V:Clone>(V);
impl<'a,V:Clone> Observable<'a,V, No, Yes> for JustObservable<V>
{
    fn sub(&self, o: Mss<No, impl Observer<V>+'a>) -> SubRef<Yes>
    {
        o.next(self.0.clone());
        o.complete();
        SubRef::empty()
    }
}
unsafe impl<V:Clone> Send for JustObservable<V> where V:Send{}
unsafe impl<V:Clone> Sync for JustObservable<V> {}
impl<V:Clone> Clone for JustObservable<V>
{
    fn clone(&self) -> JustObservable<V>
    {
        JustObservable(self.0.clone())
    }
}

pub fn just<V:Clone>(v:V) -> JustObservable<V>
{
    JustObservable(v)
}

pub struct LocalObservable<'a, V, F, R>(RefCell<F>, PhantomData<(&'a(), *const R, *const V)>) where F: 'a+FnMut(Mss<No, &(Observer<V>+'a)>) -> R, R: IntoSubRef<No>+'static;
impl<'a:'b,'b,V, F, R> Observable<'a, V, No, No> for LocalObservable<'a, V, F, R> where F: 'a+FnMut(Mss<No, &(Observer<V>+'a)>) -> R, R: IntoSubRef<No>+'static
{
    fn sub(&self, o: Mss<No, impl Observer<V>+'a>) -> SubRef<No>
    {
        let mut f = self.0.borrow_mut();
        let sub = f.call_mut((Mss::no(&o.into_inner()),));

        IntoSubRef::into(sub)
    }
}
impl<'a,V,F,R> Clone for LocalObservable<'a,V,F,R> where F: Clone+'a+FnMut(Mss<No, &(Observer<V>+'a)>) -> R, R: IntoSubRef<No>+'static
{
    fn clone(&self) -> Self
    {
        LocalObservable(self.0.clone(), PhantomData)
    }
}

pub fn create<'a:'b, 'b, V:'a, F, R>(sub: F) -> LocalObservable<'a,V,F,R> where F: FnMut(Mss<No, &(Observer<V>+'a)>) -> R, R: IntoSubRef<No>+'static
{
    LocalObservable(RefCell::new(sub), PhantomData)
}

#[derive(Clone)]
pub struct LocalBoxedObservable<'a, V, F, R, SSS:?Sized>(RefCell<F>, PhantomData<(&'a(), R, V, *const SSS)>) where F: 'a+FnMut(Mss<No, Box<Observer<V>+'a>>) -> R, R: IntoSubRef<SSS>+'static;
impl<'a:'b, 'b, V:'a, F, R, SSS:?Sized> Observable<'a, V, No, SSS> for LocalBoxedObservable<'a, V, F, R, SSS> where F: 'a+FnMut(Mss<No, Box<Observer<V>+'a>>) -> R, R: IntoSubRef<SSS>+'static
{
    fn sub(&self, o: Mss<No, impl Observer<V>+'a>) -> SubRef<SSS>
    {
        let mut f = self.0.borrow_mut();
        let sub = f.call_mut((o.into_boxed(),));
        IntoSubRef::into(sub)
    }
}

pub fn create_boxed<'a:'b, 'b, V:'a, F, R, SSS:?Sized>(sub: F) -> LocalBoxedObservable<'a,V,F,R,SSS> where F: FnMut(Mss<No, Box<Observer<V>+'a>>) -> R, R: IntoSubRef<SSS>+'static
{
    LocalBoxedObservable(RefCell::new(sub), PhantomData)
}

#[derive(Clone)]
pub struct SendSyncObserverObservable<V, F, R>(F, PhantomData<(R, V)>) where F: Send+Sync+Fn(Mss<Yes, Box<Observer<V>+'static>>) -> R, R: IntoSubRef<Yes>;
impl<V, F, R> Observable<'static, V, Yes, Yes> for SendSyncObserverObservable<V, F, R> where F: Send+Sync+Fn(Mss<Yes, Box<Observer<V>+'static>>) -> R, R: IntoSubRef<Yes>
{
    fn sub(&self, o: Mss<Yes, impl Observer<V>+'static>) -> SubRef<Yes>
    {
        let sub = (self.0)(o.into_boxed());
        IntoSubRef::into(sub)
    }
}
pub fn create_sso<'a, V, F, R>(sub: F) -> SendSyncObserverObservable<V,F,R> where F: Send+Sync+Fn(Mss<Yes, Box<Observer<V>+'static>>) -> R, R: IntoSubRef<Yes>
{
    SendSyncObserverObservable(sub, PhantomData)
}

#[derive(Copy,Clone)]
pub struct RangeObservable
{
    start: i32,
    end: i32,
}
impl<'a> Observable<'a, i32, No, No> for RangeObservable
{
    fn sub(&self, o: Mss<No, impl Observer<i32>+'a>) -> SubRef<No>
    {
        let mut i = self.start;
        let end = self.end;

        while i != end {
            if o._is_closed() { return SubRef::empty(); }
            o.next(i);
            i += 1;
        }
        if o._is_closed() { return SubRef::empty(); }
        o.complete();

        SubRef::empty()
    }
}
pub fn range<'a>(start:i32, len: usize) -> RangeObservable
{
    RangeObservable{ start, end: start + (len as i32) }
}



#[cfg(test)]
mod test
{
    use super::*;
    use observable::*;
    use std::sync::atomic::AtomicIsize;
    use std::sync::atomic::Ordering;
    use std::time::Duration;
    use std::sync::Arc;
    use test_fixture::SimpleObservable;

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
    fn just_()
    {
        just(123).subf(|v| println!("{}", v));
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

    #[test]
    fn range_test()
    {

        let i = range(0, 3);
        let mut count = 0;
        i.subf(|v| count += 1);


        assert_eq!(count, 3);
    }
}