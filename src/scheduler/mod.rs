use std::boxed::FnBox;
use crate::*;

pub trait Scheduler<SS:YesNo>
{
    fn schedule(&self, due: Option<::std::time::Duration>, act: impl SchActOnce<SS>) -> Unsub<'static, SS> where Self: Sized;
}

pub trait SchedulerPeriodic<SS:YesNo> : Scheduler<SS>
{
    fn schedule_periodic(&self, period: ::std::time::Duration, act: impl SchActPeriodic<SS>) -> Unsub<'static, SS> where Self: Sized;
}

pub trait ThreadFactory
{
    fn start(&self, main: impl FnOnce()+Send+Sync+'static) where Self: Sized{ self.start_dyn(box main) }
    fn start_dyn(&self, main: Box<FnBox()+Send+Sync+'static>);
}

pub unsafe trait SchActPeriodic<SS:YesNo> : for<'x> Act<SS, &'x Unsub<'static, SS>> + 'static {}
pub unsafe trait SchActOnce<SS:YesNo> : ActOnce<SS, (), Unsub<'static, SS>> + 'static {}
pub unsafe trait SchActBox<SS:YesNo> : ActBox<SS, (), Unsub<'static, SS>> + 'static {}

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
mod event_loop_scheduler;
mod new_thread_scheduler;
mod current_thread_scheduler;
