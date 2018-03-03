use std::time::Duration;

pub trait Scheduler
{
    fn schedule(&self, act: impl FnOnce()+Send+'static);
    fn schedule_after(&self, due: Duration, act: impl FnOnce()+Send+'static);
}

pub struct ImScheduler;

impl ImScheduler
{
    pub fn new() -> ImScheduler { ImScheduler }
}

impl Scheduler for ImScheduler
{
    fn schedule(&self, act: impl FnOnce()+Send+'static)
    {
        act();
    }

    fn schedule_after(&self, due: Duration, act: impl FnOnce()+Send+'static)
    {
        ::std::thread::sleep(due);
        act();
    }
}

pub struct NewThreadScheduler;
impl NewThreadScheduler
{
    pub fn new() -> NewThreadScheduler { NewThreadScheduler }
}

impl Scheduler for NewThreadScheduler
{
    fn schedule(&self, act: impl FnOnce()+Send+'static)
    {
       ::std::thread::spawn(move ||{
           act();
       });
    }

    fn schedule_after(&self, due: Duration, act: impl FnOnce()+Send+'static)
    {
        ::std::thread::spawn(move ||{
            ::std::thread::sleep(due);
            act();
        });

    }
}