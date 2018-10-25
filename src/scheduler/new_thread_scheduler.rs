use crate::*;
use std::time::Duration;
use std::sync::Arc;

pub struct NewThreadScheduler
{
    ev: EventLoopScheduler
}

impl NewThreadScheduler
{
    pub fn new(fac: Arc<ThreadFactory+Send+Sync+'static>) -> NewThreadScheduler
    {
        NewThreadScheduler{ ev: EventLoopScheduler::new(fac, true) }
    }
}

impl Scheduler<YES> for NewThreadScheduler
{
    fn schedule(&self, due: Option<Duration>, act: impl SchActOnce<YES>) -> Unsub<'static, YES> where Self: Sized
    {
        self.ev.schedule(due, act)
    }
}

impl SchedulerPeriodic<YES> for NewThreadScheduler
{
    fn schedule_periodic(&self, period: Duration, act: impl SchActPeriodic<YES>) -> Unsub<'static, YES> where Self: Sized
    {
        self.ev.schedule_periodic(period, act)
    }
}