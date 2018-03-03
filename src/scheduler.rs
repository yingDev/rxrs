use std::time::Duration;
use unsub_ref::UnsubRef;

pub trait Scheduler
{
    fn schedule(&self, act: impl Send+'static+FnOnce()->UnsubRef<'static>) -> UnsubRef<'static>;
    fn schedule_after(&self, due: Duration, act: impl Send+'static+FnOnce()->UnsubRef<'static>) -> UnsubRef<'static>;
}


pub struct ImScheduler;

impl ImScheduler
{
    pub fn new() -> ImScheduler { ImScheduler }
}

impl Scheduler for ImScheduler
{
    fn schedule(&self, act: impl Send+'static+FnOnce()->UnsubRef<'static>) -> UnsubRef<'static>
    {
        act()
    }

    fn schedule_after(&self, due: Duration, act: impl Send+'static+FnOnce()->UnsubRef<'static>) -> UnsubRef<'static>
    {
        ::std::thread::sleep(due);
        act()
    }
}

pub struct NewThreadScheduler;
impl NewThreadScheduler
{
    pub fn new() -> NewThreadScheduler { NewThreadScheduler }
}

impl Scheduler for NewThreadScheduler
{
    fn schedule(&self, act: impl Send+'static+FnOnce()->UnsubRef<'static>) -> UnsubRef<'static>
    {
        let unsub = UnsubRef::signal();
        let unsub2 = unsub.clone();

       ::std::thread::spawn(move ||{
           unsub2.add(act());
       });

        unsub
    }

    fn schedule_after(&self, due: Duration, act: impl Send+'static+FnOnce()->UnsubRef<'static>) -> UnsubRef<'static>
    {
        let unsub = UnsubRef::signal();
        let unsub2 = unsub.clone();

        ::std::thread::spawn(move ||{
            ::std::thread::sleep(due);
            if ! unsub2.disposed() { unsub2.add(act()); }
        });

        unsub
    }
}