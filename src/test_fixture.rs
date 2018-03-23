use observable::*;
use subref::SubRef;
use std::rc::Rc;
use std::cell::Cell;
use std::cell::RefCell;
use util::mss::*;

pub struct SimpleObservable;
impl<'a> Observable<'a, i32, No> for SimpleObservable
{
    fn sub(&self, o: Mss<No, impl Observer<i32>+'a>) -> SubRef
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
impl<'a> Observable<'a, i32, No> for StoreObserverObservable<'a>
{
    fn sub(&self, o: Mss<No, impl Observer<i32>+'a>) -> SubRef
    {
        *self.o.borrow_mut() = Some(Box::new(o.into_inner()));
        SubRef::empty()
    }
}


pub struct ThreadedObservable;
impl Observable<'static, i32, Yes> for ThreadedObservable
{
    fn sub(&self, o: Mss<Yes, impl Observer<i32>+'static>) -> SubRef
    {
        ::std::thread::spawn(move ||{
            o.next(1);
            o.complete();
        });

        SubRef::empty()
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
