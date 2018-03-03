use std::time::Duration;

pub trait Scheduler
{
    fn schedule(&self, act: impl FnOnce());
    fn schedule_after(&self, due: Duration, act: impl FnOnce());
}

pub struct ImScheduler;

impl ImScheduler
{
    fn new() -> ImScheduler { ImScheduler }
}

impl Scheduler for ImScheduler
{
    fn schedule(&self, act: impl FnOnce())
    {
        act();
    }

    fn schedule_after(&self, due: Duration, act: impl FnOnce())
    {
        ::std::thread::sleep(due);
        act();
    }
}