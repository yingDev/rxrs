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
use std::sync::atomic::*;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::Weak;
use std::mem::forget;

pub struct EventLoopScheduler
{
    state: Arc<Inner>
}

struct ActItem
{
    due: Instant,
    act: Box<FnBox(&Scheduler<YES>)+Send+Sync+'static>
}

struct ActQueue
{
    timers: BinaryHeap<ActItem>,
    ready: Vec<Box<FnBox(&Scheduler<YES>)+Send+Sync+'static>>
}

struct Inner
{
    queue: Mutex<ActQueue>,

    has_thread: AtomicBool,
    disposed: AtomicBool,
    exit_if_empty: bool,

    noti: Condvar,
    fac: Box<ThreadFactory+Send+Sync>
}



pub trait ThreadFactory
{
    fn start(&self, main: impl FnOnce()+Send+Sync+'static) where Self: Sized{ self.start_dyn(box main) }
    fn start_dyn(&self, main: Box<FnBox()+Send+Sync+'static>);
}

impl Inner
{
    fn run(state: Arc<Inner>)
    {
        let selv = Arc::as_ref(&state);

        let mut ready: Vec<Box<FnBox(&Scheduler<YES>)+Send+Sync+'static>> = Vec::new();

        loop {
            let mut queue = state.queue.lock().unwrap();
            if queue.ready.len() == 0 && queue.timers.len() == 0 {
                if state.exit_if_empty {
                    state.disposed.store(true, Ordering::Release);
                    break;
                }
                queue = state.noti.wait(queue).unwrap();
            }

            if state.disposed.load(Ordering::Acquire) {
                break;
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
            let selv = unsafe{
                let arc = Arc::from_raw(self);
                let ret = arc.clone();
                forget(arc);
                ret
            };
            self.fac.start_dyn(box move || Self::run(selv));
        }
    }
}

struct AnySendSync<T>(T);
unsafe impl<T> Send for AnySendSync<T>{}
unsafe impl<T> Sync for AnySendSync<T>{}

impl Scheduler<YES> for Inner
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

//todo: drop
impl EventLoopScheduler
{
    pub fn new(fac: impl ThreadFactory+Send+Sync+'static, exit_if_empty: bool) -> Arc<EventLoopScheduler>
    {
        let state = Arc::new(Inner {
            queue: Mutex::new(ActQueue{ timers: BinaryHeap::new(), ready: Vec::new() }),
            has_thread: AtomicBool::new(false),
            disposed: AtomicBool::new(false),
            exit_if_empty,
            noti: Condvar::new(),
            fac: box fac
        });

        Arc::new(EventLoopScheduler{ state })
    }
}

impl PartialEq<ActItem> for ActItem
{
    fn eq(&self, other: &ActItem) -> bool { self.due == other.due }
}

impl Eq for ActItem {}

impl PartialOrd<ActItem> for ActItem
{
    fn partial_cmp(&self, other: &ActItem) -> Option<::std::cmp::Ordering> { Some(other.due.cmp(&self.due)) }
}

impl Ord for ActItem
{
    fn cmp(&self, other: &Self) -> ::std::cmp::Ordering { other.due.cmp(&self.due) }
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

        let sch = EventLoopScheduler::new(Fac, true);
        sch.schedule(None, |s: &Scheduler<YES>| {
            println!("ok? a");
            Unsub::done()
        });
        sch.schedule(None, |s: &Scheduler<YES>| {
            println!("ok? b");
            Unsub::done()
        });        sch.schedule(None, |s: &Scheduler<YES>| {
        println!("ok? c");
        Unsub::done()
    });
        sch.schedule(Some(::std::time::Duration::from_millis(500)), |s: &Scheduler<YES>| {
            println!("later...500");
            Unsub::done()
        });


        sch.schedule(Some(::std::time::Duration::from_millis(200)), |s: &Scheduler<YES>| {
            println!("later...200");
            Unsub::done()
        });
        sch.schedule(Some(::std::time::Duration::from_millis(540)), |s: &Scheduler<YES>| {
            println!("later... 540");
            Unsub::done()
        });
        sch.schedule(Some(::std::time::Duration::from_millis(50)), |s: &Scheduler<YES>| {
            println!("later... 50");
            Unsub::done()
        });

        ::std::thread::sleep_ms(1000);
    }
}