use std::cell::UnsafeCell;
use std::collections::BinaryHeap;
use std::mem::forget;
use std::sync::{Condvar, Mutex, Arc, atomic::*};
use std::time::{Duration, Instant};
use crate::*;
use crate::util::clones::*;

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
    fac: Arc<ThreadFactory+Send+Sync+'static>
}

struct ActQueue
{
    timers: BinaryHeap<ActItem>,
    ready: Vec<ActItem>,

    tmp_for_remove: Option<BinaryHeap<ActItem>>
}

type ArcActFn = Arc<Fn()+Send+Sync+'static>;

struct ActItem
{
    due: Instant,
    period: Option<Duration>,
    unsub: Unsub<'static, YES>,
    act: ArcActFn
}

impl Scheduler<YES> for Inner
{
    fn schedule(&self, due: Option<Duration>, act: impl ActOnce<YES, (), Unsub<'static, YES>> + 'static) -> Unsub<'static, YES> where Self: Sized
    {
        if self.disposed.load(Ordering::Acquire) { return Unsub::done(); }

        let (sub, sub1) = Unsub::new().clones();
        let act = unsafe{ AnySendSync::new(UnsafeCell::new(Some(act))) };
        self.schedule_internal(due.unwrap_or(Duration::new(0,0)), None, Arc::new(move ||
            unsafe{ &mut *act.get()}.take().map_or((), |a| { sub1.add_each(a.call_once(())); })
        ), sub)
    }
}

impl SchedulerPeriodic<YES> for Inner
{
    fn schedule_periodic(&self, period: Duration, act: impl Act<YES, Ref<Unsub<'static, YES>>> + 'static) -> Unsub<'static, YES> where Self: Sized
    {
        if self.disposed.load(Ordering::Acquire) { return Unsub::done(); }

        let (sub, sub1) = Unsub::new().clones();
        let act = unsafe{ AnySendSync::new(UnsafeCell::new(Some(act))) };
        self.schedule_internal(period, Some(period), Arc::new(move ||
            unsafe{ &*act.get()}.as_ref().map_or((), |a| a.call(&sub1))
        ), sub)
    }
}

impl Inner
{
    fn run(state: Arc<Inner>)
    {
        let mut ready: Vec<ActItem> = Vec::new();
        let mut re_schedules: Vec<ActItem> = Vec::new();
        let mut queue = state.queue.lock().unwrap();

        while ! state.disposed.load(Ordering::Relaxed) {

            if ready.len() == 0 && queue.ready.len() == 0 && queue.timers.len() == 0 {
                if state.exit_if_empty {
                    state.has_thread.store(false, Ordering::Relaxed);
                    break;
                }
                queue = state.noti.wait(queue).unwrap();
            }

            ready.extend(queue.ready.drain(..));
            let now = Instant::now();
            while queue.timers.peek().filter(|item| item.due <= now).is_some() {
                ready.push(queue.timers.pop().unwrap());
            }

            if ready.len() == 0 {
                if let Some(next_tick) = queue.timers.peek().map(|item| item.due) {
                    queue = state.noti.wait_timeout(queue, next_tick - now).unwrap().0;
                }
                continue;
            }

            drop(queue);
            for mut act in ready.drain(..).filter(|a| !a.unsub.is_done()) {
                act.act.call(());
                if act.unsub.is_done() { continue; }

                if let Some(period) = act.period {
                    act.due += period;
                    re_schedules.push(act);
                }
            }

            queue = state.queue.lock().unwrap();
            let now = Instant::now();
            for a in re_schedules.drain(..) {
                if a.due <= now || a.unsub.is_done() {
                    ready.push(a);
                } else {
                    queue.timers.push(a);
                }
            }
        }

    }

    fn ensure_thread(&self)
    {
        if ! self.has_thread.swap(true, Ordering::Release) {
            let selv = self.get_arc_self();
            self.fac.start_dyn(box move || Self::run(selv));
        }
    }

    fn remove(&self, act: &ArcActFn)
    {
        //since BinaryHeap has no `remove()`, we use a ~O(n) way to remove the target item
        let mut queue = self.queue.lock().unwrap();
        let mut tmp = queue.tmp_for_remove.take().unwrap();

        tmp.append(&mut queue.timers);
        for a in tmp.drain() {
            if ! Arc::ptr_eq(act, &a.act) {
                queue.timers.push(a);
            }
        }
        queue.tmp_for_remove.replace(tmp);
    }

    fn get_arc_self(&self) -> Arc<Self>
    {
        let (arc, ret) = unsafe{ Arc::from_raw(self) }.clones();
        forget(arc);
        ret
    }

    fn schedule_internal(&self, due: Duration, period: Option<Duration>, act: ArcActFn, sub: Unsub<'static, YES>) -> Unsub<'static, YES>
    {
        let mut queues = self.queue.lock().unwrap();
        let item = ActItem{ due: Instant::now() + due, act: act.clone(), period, unsub:  sub.clone()};
        if due == Duration::new(0, 0) {
            queues.ready.push(item);
        } else {
            queues.timers.push(item);
        }

        let selv = Arc::downgrade(&self.get_arc_self());
        sub.add(Unsub::<YES>::with(move || selv.upgrade().map_or((), |arc| arc.remove(&act) )));

        self.noti.notify_one();
        self.ensure_thread();

        sub
    }
}

impl Scheduler<YES> for EventLoopScheduler
{
    fn schedule(&self, due: Option<Duration>, act: impl ActOnce<YES, (), Unsub<'static, YES>> + 'static) -> Unsub<'static, YES> where Self: Sized
    {
        self.state.schedule(due, act)
    }
}

impl SchedulerPeriodic<YES> for EventLoopScheduler
{
    fn schedule_periodic(&self, period: Duration, act: impl Act<YES, Ref<Unsub<'static, YES>>> + 'static) -> Unsub<'static, YES> where Self: Sized
    {
        self.state.schedule_periodic(period, act)
    }
}

impl Drop for EventLoopScheduler
{
    fn drop(&mut self)
    {
        self.state.disposed.store(true, Ordering::Release);
        self.state.noti.notify_one();
    }
}

impl EventLoopScheduler
{
    pub fn new(fac: Arc<ThreadFactory+Send+Sync+'static>, exit_if_empty: bool) -> EventLoopScheduler
    {
        let state = Arc::new(Inner {
            queue: Mutex::new(ActQueue{ timers: BinaryHeap::new(), tmp_for_remove: Some(BinaryHeap::new()), ready: Vec::new() }),
            has_thread: AtomicBool::new(false),
            disposed: AtomicBool::new(false),
            exit_if_empty,
            noti: Condvar::new(),
            fac
        });

        EventLoopScheduler{ state }
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
    use ::std::time::Duration;
    use std::sync::Arc;

    #[test]
    fn smoke()
    {
        let sch = EventLoopScheduler::new(Arc::new(DefaultThreadFac), true);

        let sub = sch.schedule_periodic(Duration::from_millis(33), |_:&Unsub<'static, YES>| println!("shit"));
        ::std::thread::spawn(move ||{
            ::std::thread::sleep(Duration::from_millis(700));
            sub.unsub();
        });


        sch.schedule(None, || {
            println!("ok? a");
            Unsub::done()
        });
        sch.schedule(None, || {
            println!("ok? b");
            Unsub::done()
        });

        sch.schedule(None, || {
            println!("ok? c");
            Unsub::done()
        });

        sch.schedule(Some(::std::time::Duration::from_millis(4)), || {
            println!("later...4");
            Unsub::done()
        });


        sch.schedule(Some(::std::time::Duration::from_millis(3)), || {
            println!("later...3");
            ::std::thread::sleep(Duration::from_millis(200));
            Unsub::done()
        });
        sch.schedule(Some(::std::time::Duration::from_millis(2)), || {
            println!("later... 2");
            ::std::thread::sleep(Duration::from_millis(200));

            Unsub::done()
        });
        sch.schedule(Some(::std::time::Duration::from_millis(1)), || {
            println!("later... 1");
            ::std::thread::sleep(Duration::from_millis(200));

            Unsub::done()
        });

        ::std::thread::sleep(Duration::from_millis(2000));
    }
}