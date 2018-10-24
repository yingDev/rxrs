use crate::*;
use crate::any_send_sync::AnySendSync;
use std::boxed::FnBox;
use std::cell::UnsafeCell;
use std::collections::BinaryHeap;
use std::mem::forget;
use std::sync::Arc;
use std::sync::atomic::*;
use std::sync::Condvar;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;

pub struct EventLoopScheduler
{
    state: Arc<Inner>
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

struct ActQueue
{
    timers: BinaryHeap<ActItem>,
    ready: Vec<Box<FnBox(&Scheduler<YES>)+Send+Sync+'static>>
}

struct ActItem
{
    due: Instant,
    act: Box<FnBox(&Scheduler<YES>)+Send+Sync+'static>
}

//todo: extract
pub trait ThreadFactory
{
    fn start(&self, main: impl FnOnce()+Send+Sync+'static) where Self: Sized{ self.start_dyn(box main) }
    fn start_dyn(&self, main: Box<FnBox()+Send+Sync+'static>);
}

impl Inner
{
    fn run(state: Arc<Inner>)
    {
        let mut ready: Vec<Box<FnBox(&Scheduler<YES>)+Send+Sync+'static>> = Vec::new();
        let mut queue = state.queue.lock().unwrap();

        while ! state.disposed.load(Ordering::Relaxed) {

            if queue.ready.len() == 0 && queue.timers.len() == 0 {
                if state.exit_if_empty {
                    state.has_thread.store(false, Ordering::Relaxed);
                    break;
                }
                queue = state.noti.wait(queue).unwrap();
            }

            ready.extend(queue.ready.drain(..));
            let now = Instant::now();
            while queue.timers.peek().filter(|item| item.due <= now).is_some() {
                ready.push(queue.timers.pop().unwrap().act);
            }

            if ready.len() == 0 {
                if let Some(next_tick) = queue.timers.peek().map(|item| item.due) {
                    queue = state.noti.wait_timeout(queue, next_tick - now).unwrap().0;
                }
                continue;
            }

            drop(queue);
            for act in ready.drain(..) {
                act.call_box((Arc::as_ref(&state) as &Scheduler<YES>, ));
            }
            queue = state.queue.lock().unwrap();
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

impl Drop for Inner
{
    fn drop(&mut self)
    {
        let mut queue = self.queue.lock().unwrap();
        self.disposed.store(true, Ordering::Release);
        queue.timers.clear();
        queue.ready.clear();

        self.noti.notify_one();
    }
}

impl Scheduler<YES> for Inner
{
    fn schedule(&self, due: Option<Duration>, act: impl SchActOnce<YES>) -> Unsub<'static, YES> where Self: Sized
    {
        if self.disposed.load(Ordering::Acquire) {
            return Unsub::done();
        }

        let (act1, act2) = Arc::new(unsafe{ AnySendSync::new(UnsafeCell::new(Some(act))) }).clones();
        let (sub1, sub2) = Unsub::<YES>::with(move |()| unsafe{ (&mut *act1.get()).take(); }).clones();
        let act = box move |sch: &Scheduler<YES>| sub1.if_not_done(|| unsafe{ &mut *act2.get()}.take().map_or((), |act| { sub1.add_each(act.call_once(sch)); }));

        let mut queues = self.queue.lock().unwrap();
        if let Some(due) = due {
            queues.timers.push(ActItem{ due: Instant::now() + due, act });
        } else {
            queues.ready.push(act);
        }

        self.noti.notify_one();
        self.ensure_thread();

        sub2
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
    use ::std::boxed::FnBox;
    use crate::*;

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
        sch.schedule(Some(::std::time::Duration::from_millis(4)), |s: &Scheduler<YES>| {
            println!("later...4");
            Unsub::done()
        });


        sch.schedule(Some(::std::time::Duration::from_millis(3)), |s: &Scheduler<YES>| {
            println!("later...3");
            Unsub::done()
        });
        sch.schedule(Some(::std::time::Duration::from_millis(2)), |s: &Scheduler<YES>| {
            println!("later... 2");
            Unsub::done()
        });
        sch.schedule(Some(::std::time::Duration::from_millis(1)), |s: &Scheduler<YES>| {
            println!("later... 1");
            Unsub::done()
        });

        ::std::thread::sleep_ms(1000);
    }
}