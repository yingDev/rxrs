use crate::*;
use std::time::Duration;
use std::sync::Arc;
use std::collections::BinaryHeap;
use std::time::Instant;
use std::rc::Rc;
use std::cell::RefCell;
use std::cell::Cell;


type RcActFn = Rc<Fn(())+'static>;

struct ActItem
{
    due: Instant,
    period: Option<Duration>,
    unsub: Unsub<'static, NO>,
    act: RcActFn
}

pub struct CurrentThreadScheduler
{
    running: Cell<bool>,
    queue: RefCell<BinaryHeap<ActItem>>
}

impl CurrentThreadScheduler
{
    pub fn new() -> CurrentThreadScheduler
    {
        CurrentThreadScheduler{ running: Cell::new(false), queue: RefCell::new(BinaryHeap::new()) }
    }

    fn run(&self)
    {
        loop {
            let act = self.queue.borrow_mut().pop();

            if let Some(mut act) = act {
                let now = Instant::now();
                if act.due > now {
                    ::std::thread::sleep(act.due - now);
                }
                if ! act.unsub.is_done() {
                    act.act.call(((), ));
                }
                if ! act.unsub.is_done() {
                    if let Some(period) = act.period {
                        act.due += period;
                        self.queue.borrow_mut().push(act);
                    }
                }
            } else { break; }


        }
    }
}

impl Scheduler<NO> for CurrentThreadScheduler
{
    fn schedule(&self, due: Option<Duration>, act: impl SchActOnce<NO>) -> Unsub<'static, NO> where Self: Sized
    {
        if !self.running.get() {
            self.running.replace(true);
            due.map(::std::thread::sleep);
            let unsub = act.call_once(());

            self.run();

            self.running.replace(false);
            return unsub;
        }

        let (act, act1) = Rc::new(RefCell::new(Some(act))).clones();
        let (sub, sub1) = Unsub::<NO>::with(move|()| { act1.borrow_mut().take(); }).clones();
        let (act, act1) = Rc::new(move |()| {
            let act = act.borrow_mut().take();
            act.map_or((), |a| { sub.add_each(a.call_once(())); })
        }).clones();

        self.queue.borrow_mut().push(ActItem{
            due: Instant::now() + due.unwrap_or_else(|| Duration::new(0,0)),
            period: None,
            unsub: sub1.clone(),
            act
        });

        sub1
    }
}

impl SchedulerPeriodic<NO> for CurrentThreadScheduler
{
    fn schedule_periodic(&self, period: Duration, act: impl SchActPeriodic<NO>) -> Unsub<'static, NO> where Self: Sized
    {
        let (act, act1) = Rc::new(act).clones();
        let (sub, sub1, sub2) = Unsub::<NO>::new().clones();
        let (act, act1) = Rc::new(move |()| {
            act.call(&sub2);
        }).clones();

        self.queue.borrow_mut().push(ActItem{
            due: Instant::now() + period,
            period: Some(period),
            unsub: sub1.clone(),
            act
        });

        if !self.running.get() {
            self.running.replace(true);
            self.run();
            self.running.replace(false);
        }

        sub1
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
    use std::time::Duration;
    use std::cell::Cell;
    use std::rc::Rc;
    use std::sync::Arc;

    #[test]
    fn smoke()
    {
        let (n, n1) = Rc::new(Cell::new(0)).clones();
        let s = CurrentThreadScheduler::new();

        s.schedule(Some(Duration::from_millis(100)), move |()| {
            n.replace(n.get() + 1);
            Unsub::done()
        });

        assert_eq!(n1.get(), 1);
    }

    #[test]
    fn recurse()
    {
        let (n, n1) = Rc::new(Cell::new(0)).clones();
        let (s, s1, s2) = Arc::new(CurrentThreadScheduler::new()).clones();

        s.schedule(Some(Duration::from_millis(100)), move |()| {
            println!("a");
            let (n, n1, n2) = n.clones();
            n.replace(n.get() + 1);

            s1.schedule(Some(Duration::from_millis(800)), move |()| {
                println!("b");
                n.replace(n.get() + 1);
                Unsub::done()
            } );

            s2.schedule(Some(Duration::from_millis(100)), move |()| {
                println!("c");
                n1.replace(n1.get() + 1);
                Unsub::done()
            } );

            Unsub::done()
        });
    }

    #[test]
    fn periododic()
    {
        let (n, n1) = Rc::new(Cell::new(0)).clones();
        let (s, s1, s2) = Arc::new(CurrentThreadScheduler::new()).clones();

        s.schedule_periodic(Duration::from_millis(10), move |unsub: &Unsub<NO>| {
            if n.replace(n.get() + 1) == 9 {
                unsub.unsub();
            }
        });

        assert_eq!(n1.get(), 10);
    }
}