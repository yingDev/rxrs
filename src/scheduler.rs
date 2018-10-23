use std::time::Duration;
use crate::*;
use std::thread::*;
use std::sync::Mutex;
use std::collections::VecDeque;
use std::time::SystemTime;
use std::time::Instant;
use std::collections::BinaryHeap;
use std::boxed::FnBox;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::MutexGuard;
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::Weak;

pub trait SchActPeriodic<SS:YesNo, S> : for<'x> Act<SS, &'x S, S> + 'static {}
pub trait SchActOnce<SS:YesNo> : for<'x> ActOnce<SS, &'x Scheduler<SS>, Unsub<'static, SS>> + 'static {}
pub trait SchActBox<SS:YesNo> : for<'x> ActBox<SS, &'x Scheduler<SS>, Unsub<'static, SS>> + 'static {}


impl<SS:YesNo, S, A: for<'x> Act<SS, &'x S, S>+'static>
SchActPeriodic<SS, S>
for A{}

impl<SS:YesNo, A: for<'x> ActOnce<SS, &'x Scheduler<SS>, Unsub<'static, SS>>+'static>
SchActOnce<SS>
for A{}

impl<SS:YesNo, A: for<'x> ActBox<SS, &'x Scheduler<SS>, Unsub<'static, SS>>+'static>
SchActBox<SS>
for A{}

pub trait Scheduler<SS:YesNo>
{
   fn schedule(&self, due: Option<Duration>, act: impl SchActOnce<SS>) -> Unsub<'static, SS> where Self: Sized;
}

pub trait SchedulerPeriodic<SS:YesNo> : Scheduler<SS>
{
    fn schedule_periodic<S>(&self, period: Duration, act: impl SchActPeriodic<SS, S>) -> Unsub<'static, SS> where Self: Sized;
}

struct ActItem
{
    due: Instant,
    act: Box<FnBox(&Scheduler<YES>)+Send+Sync+'static>
}

impl PartialEq<ActItem> for ActItem
{
    fn eq(&self, other: &ActItem) -> bool { self.due == other.due }
}

impl Eq for ActItem {}

impl PartialOrd<ActItem> for ActItem
{
    fn partial_cmp(&self, other: &ActItem) -> Option<::std::cmp::Ordering>
    {
        Some(self.due.cmp(&other.due))
    }
}

impl Ord for ActItem
{
    fn cmp(&self, other: &Self) -> ::std::cmp::Ordering { self.due.cmp(&other.due) }
}

struct ActQueue
{
    timers: BinaryHeap<ActItem>,
    ready: Vec<Box<FnBox(&Scheduler<YES>)+Send+Sync+'static>>
}


struct State
{
    queue: Mutex<ActQueue>,

    has_thread: AtomicBool,
    noti: Condvar,
    fac: Box<ThreadFactory+Send+Sync>
}

pub struct EventLoopScheduler
{
    state: Arc<State>
}

pub trait ThreadFactory
{
    fn start(&self, main: impl FnOnce()+Send+Sync+'static) where Self: Sized{ self.start_dyn(box main) }
    fn start_dyn(&self, main: Box<FnBox()+Send+Sync+'static>);
}

impl State
{
    fn run(state: Arc<State>)
    {
        let selv = Arc::as_ref(&state);

        let mut ready: Vec<Box<FnBox(&Scheduler<YES>)+Send+Sync+'static>> = Vec::new();

        loop {
            let mut queue = state.queue.lock().unwrap();
            if queue.ready.len() == 0 && queue.timers.len() == 0 {
                queue = state.noti.wait(queue).unwrap();
            }
            ready.clear();
            ready.extend(queue.ready.drain(..));

            let now = Instant::now();

            loop {
                if queue.timers.peek().filter(|item| item.due <= now).is_some() {
                    ready.push(queue.timers.pop().unwrap().act);
                } else { break; }
            }

            if ready.len() > 0 {
                drop(queue);

                for act in ready.drain(..) {
                    act.call_box((selv as &Scheduler<YES>, ));
                }
            } else {
                if let Some(next_tick) = queue.timers.peek().map(|item| item.due) {
                    state.noti.wait_timeout(queue, next_tick - now);
                }
            }

        }

    }

    fn ensure_thread(&self)
    {
        if ! self.has_thread.swap(true, Ordering::Release) {
            let selv = unsafe{ Arc::from_raw(self) }.clone();
            self.fac.start_dyn(box move || Self::run(selv));
        }
    }
}

struct AnySendSync<T>(T);
unsafe impl<T> Send for AnySendSync<T>{}
unsafe impl<T> Sync for AnySendSync<T>{}

impl Scheduler<YES> for State
{
    fn schedule(&self, due: Option<Duration>, act: impl SchActOnce<YES>) -> Unsub<'static, YES> where Self: Sized
    {
        let (act1, act2) = Arc::new(AnySendSync(UnsafeCell::new(Some(act)))).clones();
        let (unsub1, unsub2) = Unsub::<YES>::with(move |()| unsafe{ (&mut *act1.0.get()).take();}).clones();
        let act = box move |sch: &Scheduler<YES>| unsub1.if_not_done(|| unsafe{ &mut *act2.0.get()}.take().map_or((), |act| { unsub1.add_each(act.call_once(sch)); }));

        {
            let mut queues = self.queue.lock().unwrap();
            if let Some(due) = due {
                queues.timers.push(ActItem{ due: Instant::now() + due, act });
            } else {
                queues.ready.push(act);
            }

            self.noti.notify_one();
            self.ensure_thread();
        }

        unsub2
    }
}

impl Scheduler<YES> for EventLoopScheduler
{
    fn schedule(&self, due: Option<Duration>, act: impl SchActOnce<YES>) -> Unsub<'static, YES> where Self: Sized
    {
        self.state.schedule(due, act)
    }
}

impl EventLoopScheduler
{
    pub fn new(spawn: impl ThreadFactory+Send+Sync+'static) -> Arc<EventLoopScheduler>
    {
        let state = Arc::new(State{
            queue: Mutex::new(ActQueue{ timers: BinaryHeap::new(), ready: Vec::new() }),
            has_thread: AtomicBool::new(false),
            noti: Condvar::new(),
            fac: box spawn
        });

        Arc::new(EventLoopScheduler{ state })
    }
}


#[cfg(test)]
mod test
{
    use crate::*;
    use ::std::boxed::FnBox;

    #[test]
    fn smoke()
    {

        struct Fac;
        impl ThreadFactory for Fac
        {
            fn start_dyn(&self, main: Box<FnBox()+Send+Sync+'static>)
            {
                ::std::thread::spawn(move || main.call_box(()));
            }
        }

        let sch = EventLoopScheduler::new(Fac);
        sch.schedule(None, |s: &Scheduler<YES>| {
            println!("ok?");
            Unsub::done()
        });

        sch.schedule(Some(::std::time::Duration::from_millis(500)), |s: &Scheduler<YES>| {
            println!("later...");
            Unsub::done()
        });

        ::std::thread::sleep_ms(1000);
    }
}