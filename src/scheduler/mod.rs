use std::boxed::FnBox;
use crate::*;

pub trait Scheduler<SS:YesNo>
{
    fn schedule(&self, due: Option<::std::time::Duration>, act: impl ActOnce<SS, (), Unsub<'static, SS>> + 'static) -> Unsub<'static, SS> where Self: Sized;
}

pub trait SchedulerPeriodic<SS:YesNo> : Scheduler<SS>
{
    fn schedule_periodic(&self, period: ::std::time::Duration, act: impl Act<SS, Ref<Unsub<'static, SS>>> + 'static) -> Unsub<'static, SS> where Self: Sized;
}

pub trait ThreadFactory
{
    fn start(&self, main: impl FnOnce()+Send+Sync+'static) where Self: Sized{ self.start_dyn(box main) }
    fn start_dyn(&self, main: Box<FnBox()+Send+Sync+'static>);
}

impl<SS:YesNo, S: Scheduler<SS>> Scheduler<SS> for Arc<S>
{
    #[inline(always)]
    fn schedule(&self, due: Option<Duration>, act: impl ActOnce<SS, (), Unsub<'static, SS>>+'static) -> Unsub<'static, SS> where Self: Sized {
        Arc::as_ref(self).schedule(due, act)
    }
}

impl<SS:YesNo, S: SchedulerPeriodic<SS>> SchedulerPeriodic<SS> for Arc<S>
{
    #[inline(always)]
    fn schedule_periodic(&self, period: Duration, act: impl Act<SS, Ref<Unsub<'static, SS>>>+'static) -> Unsub<'static, SS> {
        Arc::as_ref(self).schedule_periodic(period, act)
    }
}

pub struct DefaultThreadFac;
impl ThreadFactory for DefaultThreadFac
{
    fn start_dyn(&self, main: Box<FnBox()+Send+Sync+'static>)
    {
        ::std::thread::spawn(move || main.call_box(()));
    }
}

pub use self::event_loop_scheduler::*;
pub use self::new_thread_scheduler::*;
pub use self::current_thread_scheduler::*;
use std::sync::Arc;
use std::time::Duration;

mod event_loop_scheduler;
mod new_thread_scheduler;
mod current_thread_scheduler;
