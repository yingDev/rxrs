use std::marker::PhantomData;
use std::any::{Any};
use std::rc::Rc;
use std::sync::atomic::AtomicIsize;

use observable::*;
use subref::SubRef;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use scheduler::Scheduler;
use util::ArcCell;
use util::AtomicOption;
use std::sync::Weak;
use std::collections::VecDeque;
use std::sync::Condvar;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use observable::*;
use observable::RxNoti::*;

pub struct ObserveOn<'a, 'b, V, Sch> //where Src: Observable<'a, V> + Send + Sync, Sch: Scheduler + Send + Sync
{
    source: Arc<Observable<'a,V>+'b+Send+Sync>,
    scheduler: Arc<Sch>,
}

pub trait ObservableObserveOn<'b, V, Sch>
{
    fn observe_on(self, scheduler: Arc<Sch>) -> Arc<Observable<'static, V>+'b+Send+Sync>;
}

impl<'a:'b, 'b, V:'static+Send+Sync, Sch> ObservableObserveOn<'b, V, Sch> for Arc<Observable<'a, V> + 'b+Send+Sync> where Sch: 'static + Scheduler + Send + Sync
{
    fn observe_on(self, scheduler: Arc<Sch>) -> Arc<Observable<'static, V>+'b+Send+Sync>
    {
        Arc::new(ObserveOn { scheduler, source: self })
    }
}

impl<'a:'b,'b, V: 'static + Send + Sync, Sch> Observable<'static, V> for ObserveOn<'a, 'b, V, Sch> where Sch: Scheduler + Send + Sync + 'static
{
    #[inline(never)]
    fn sub(&self, dest: Arc<Observer<V> + Send + Sync+'static>) -> SubRef
    {
        let scheduler = self.scheduler.clone();
        let queue =  Arc::new((Condvar::new(), Mutex::new(VecDeque::new()), AtomicOption::new()));
        let stopped = Arc::new(AtomicBool::new(false));

        let q = queue.clone();
        let sub = SubRef::signal();
        let sub2 = sub.clone();

        let stopped2 = stopped.clone();
        sub.add(self.scheduler.schedule_long_running(sub.clone(), move || {
            dispatch(dest, q, scheduler, stopped2);
        }));

        sub.add(self.source.sub_noti(move |n| {
            let &(ref cond, ref q, ref err) = &*queue;

            match n {
                Next(v) => {
                    q.lock().unwrap().push_back(v);
                },
                Err(e) => {
                    if stopped.compare_and_swap(false, true, Ordering::Acquire) {
                        q.lock().unwrap();
                        sub2.unsub();
                        err.swap(e, Ordering::SeqCst);
                    }
                },
                Comp => {
                    if stopped.compare_and_swap(false, true, Ordering::Acquire) {
                        sub2.unsub();
                    }
                }
            }
            cond.notify_one();
            if stopped.load(Ordering::Acquire) {
                return IsClosed::True;
            }
            return IsClosed::Default;
        }));

        sub
    }
}

#[inline(never)]
fn dispatch<V, Sch>(dest: Arc<Observer<V>>, queue: Arc<(Condvar, Mutex<VecDeque<V>>, AtomicOption<Arc<Any+Send+Sync>>)>, scheduler: Arc<Sch>, stopped: Arc<AtomicBool>) where
    Sch: Scheduler,
{
    loop {
        let &(ref cond, ref lock, ref err) = &*queue;

        while let Some(v) = lock.lock().unwrap().pop_front() {
            if dest._is_closed() { break; }
            dest.next(v);
        }

        if stopped.load(Ordering::Acquire) {
            if let Some(e) = err.take(Ordering::Acquire) {
                dest.err(e);
            } else {
                dest.complete();
            }
            return;
        }

        cond.wait(lock.lock().unwrap());
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use subject::*;
    use fac::*;
    use op::*;
    use scheduler::*;
    use std::thread;
    use std::time::Duration;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn basic()
    {
        rxfac::range(0..10).rx().take(3).map(|v| format!("*{}*", v)).observe_on(NewThreadScheduler::get())
            .subf(( |v| println!("{} on thread {:?}", v, thread::current().id()), (),
                  || println!("complete on thread {:?}", thread::current().id())));

        thread::sleep(::std::time::Duration::from_millis(1000));
    }

    #[test]
    fn timer()
    {
        let out = Arc::new(Mutex::new(String::new()));
        let (out1, out2) = (out.clone(), out.clone());

        let x = 5;
        let toStr = |s:i32| format!("{}", s+x);
        rxfac::range(0..10).rx().filter(|v| v < &x).take(3).map(toStr).subf(|v| println!("scoped: {}-{}",v,x));

        let src = rxfac::timer(0, Some(10), NewThreadScheduler::get()).rx()
            .skip(3)
            .filter(|i| i % 2 == 0)
            .take(3)
            .map(|v| format!("{}",v))
            .observe_on(NewThreadScheduler::get());

        src.subf((
                move |v:String| out1.lock().unwrap().push_str(&v),
                (),
                move | | out2.lock().unwrap().push_str("ok")
            ));

        thread::sleep(::std::time::Duration::from_millis(2000));

        assert_eq!(*out.lock().unwrap(), "468ok");
    }
}