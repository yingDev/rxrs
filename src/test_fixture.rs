use observable::*;
use subref::*;
use std::rc::Rc;
use std::cell::Cell;
use std::cell::RefCell;
use util::mss::*;
use std::thread;
use std::time::Duration;

#[derive(Clone)]
pub struct SimpleObservable;
impl<'a> Observable<'a, i32, No, No> for SimpleObservable
{
    fn sub(&self, o: Mss<No, impl Observer<i32>+'a>) -> SubRef<No>
    {
       o.next(1);
       o.next(2);
       o.next(3);
       o.complete();

       SubRef::empty()
    }
}

pub struct StoreObserverObservable<'a>
{
    o: RefCell<Option<Box<Observer<i32>+'a>>>
}
impl<'a> StoreObserverObservable<'a>
{
    pub fn new() -> StoreObserverObservable<'a>
    {
        StoreObserverObservable{ o: RefCell::new(None) }
    }

    pub fn next(&self, v:i32)
    {
        self.o.borrow().as_ref().unwrap().next(v);
    }
}
impl<'a> Observable<'a, i32, No, Yes> for StoreObserverObservable<'a>
{
    fn sub(&self, o: Mss<No, impl Observer<i32>+'a>) -> SubRef<Yes>
    {
        *self.o.borrow_mut() = Some(Box::new(o.into_inner()));
        SubRef::empty()
    }
}

#[derive(Clone)]
pub struct ThreadedObservable;
impl Observable<'static, i32, Yes, Yes> for ThreadedObservable
{
    fn sub(&self, o: Mss<Yes, impl Observer<i32>+'static>) -> SubRef<Yes>
    {
        let sub = InnerSubRef::signal();

        ::std::thread::spawn( byclone!(sub =>move ||{
            o.next(1);
            thread::sleep(Duration::from_millis(10));
            o.next(2);
            thread::sleep(Duration::from_millis(10));
            o.next(3);
            thread::sleep(Duration::from_millis(10));
            o.complete();

            sub.unsub();
        }));

        sub.into_subref()
    }
}

pub struct SimpleObserver;
impl Observer<i32> for SimpleObserver
{
    fn next(&self, v:i32)
    {
        println!("v={}", v);
    }
}

pub struct LocalObserver<'a>(pub &'a i32);
impl<'a> Observer<i32> for LocalObserver<'a>
{
    fn next(&self, v:i32)
    {
        println!("v");
    }
}

pub struct NonSendObserver(pub Rc<i32>);
impl Observer<i32> for NonSendObserver{}
