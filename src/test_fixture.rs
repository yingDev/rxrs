use observable::Observable;
use observable::No;
use observable::Observer;
use observable::Yes;
use subref::SubRef;
use std::rc::Rc;

pub struct SimpleObservable;
impl<'a> Observable<'a, i32, No> for SimpleObservable
{
    fn sub(&self, o: impl Observer<i32>+'a) -> SubRef
    {
        o.next(1);
        o.next(2);
        o.next(3);
        o.complete();

        SubRef::empty()
    }
}

pub struct ThreadedObservable;
impl Observable<'static, i32, Yes> for ThreadedObservable
{
    fn sub(&self, o: impl Observer<i32>+Send+Sync+'static) -> SubRef
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
