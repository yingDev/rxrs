//use std::time::Duration;
//use crate::*;
//use std::thread::*;
//use std::sync::Mutex;
//use std::collections::VecDeque;
//use std::time::SystemTime;
//use std::time::Instant;
//use std::collections::BinaryHeap;
//
//trait SchAct<SS:YesNo> : for<'x> Act<SS, &'x Scheduler<SS>, Unsub<'static, SS>>{}
//trait SchActOnce<SS:YesNo> : for<'x> ActOnce<SS, &'x Scheduler<SS>, Unsub<'static, SS>>{}
//
//impl<SS:YesNo, A: for<'x> Act<SS, &'x Scheduler<SS>, Unsub<'static, SS>>> SchAct<SS> for A{}
//impl<SS:YesNo, A: for<'x> ActOnce<SS, &'x Scheduler<SS>, Unsub<'static, SS>>> SchActOnce<SS> for A{}
//
//pub trait Scheduler<SS:YesNo>
//{
//    fn schedule(&self, act: impl SchAct<SS>+'static) -> Unsub<'static, SS> where Self: Sized
//    {
//        self.schedule_after(None, act)
//    }
//    fn schedule_after(&self, dur: impl Into<Option<Duration>>, act: impl SchActOnce<SS>+'static) -> Unsub<'static, SS> where Self: Sized;
//    fn schedule_periodic(&self, dur: Duration, delay: Duration, act: impl SchAct<SS>+'static) -> Unsub<'static, SS> where Self: Sized;
//
//
//    //fn schedule_dyn(&self, act: Box<FnOnce()>) -> Unsub<'static, SS>;
//}
//
//struct TimerRec<SS: YesNo>
//{
//    due: Instant,
//    act: Box<SchAct<SS>>,
//}
//
//struct State<SS: YesNo>
//{
//    timers: BinaryHeap< TimerRec<SS> >,
//    queue: VecDeque< Box<SchAct<SS>> >
//}
//
//pub struct EventLoopScheduler<SS:YesNo>
//{
//
//}
//
//
