use observable::*;
use subref::SubRef;
use std::sync::Arc;
use std::marker::PhantomData;
use scheduler::SchedulerPeriodic;
use std::time::Duration;
use std::mem;
use util::mss::*;
use std::sync::Mutex;
use std::rc::Rc;
use std::cell::Cell;

pub struct DebounceOp<V, Src, Sch, SSO:?Sized+'static>
{
    source : Src,
    scheduler: Arc<Sch>,
    duration: Duration,

    PhantomData: PhantomData<(V, *const SSO)>
}

pub trait ObservableDebounce<'sa, V, Src, SSA:?Sized+'static, Sch, SSO: ?Sized+'static> where
    Sch: SchedulerPeriodic<SSA>+Send+Sync,
    Src : Observable<'sa, V, SSO>
{
    fn debounce(self, duration: u64, scheduler: Arc<Sch>) -> DebounceOp<V, Src, Sch, SSA>;
}

impl<'sa, V, Src, SSA:?Sized+'static, Sch, SSO:?Sized+'static> ObservableDebounce<'sa, V, Src, SSA, Sch, SSO> for Src where
    Sch: SchedulerPeriodic<SSA>+Send+Sync+'static,
    Src : Observable<'sa, V, SSO>
{
    fn debounce(self, duration: u64, scheduler: Arc<Sch>) -> DebounceOp<V, Src, Sch, SSA>
    {
        DebounceOp{ source: self, scheduler, duration: Duration::from_millis(duration), PhantomData }
    }
}

impl<'a, V:'static+Send, Src, Sch> Observable<'static, V, Yes> for DebounceOp<V, Src, Sch, Yes> where
    Sch: SchedulerPeriodic<Yes>+Send+Sync+'static,
    Src : Observable<'a, V, Yes>
{
    fn sub(&self, o: Mss<Yes, impl Observer<V> +'static>) -> SubRef
    {
        use observable::RxNoti::*;

        let sch = self.scheduler.clone();
        let dur = self.duration;
        let val : Arc<Mutex<Option<V>>> = Arc::new(Mutex::new(None));
        let mut timer = SubRef::empty();
        let o: Arc<Mss<Yes, _>> = Arc::new(o);

        let sub = SubRef::signal().added(timer.clone());

        sub.add(self.source.sub_noti(byclone!(sub => move |n| {
            timer.unsub();
            match n {
                Next(v) => {
                    *val.lock().unwrap() = Some(v);
                    timer = sch.schedule_after(dur, Mss::<Yes,_>::new(byclone!(o, val, sub => move ||{
                        let mut val = val.lock().unwrap();
                        if val.is_some(){
                            o.next(val.take().unwrap());
                        }
                        if o._is_closed() {
                            sub.unsub();
                        }
                        SubRef::empty()
                    })));

                    if o._is_closed() {
                       sub.unsub();
                       return IsClosed::True;
                    }
                },

                Err(e) => {
                    sub.unsub();
                    let mut val = val.lock().unwrap();
                    if val.is_some() {
                        o.next(val.take().unwrap());
                    }
                    o.err(e);
                },

                Comp => {
                    sub.unsub();
                    let mut val = val.lock().unwrap();
                    if val.is_some() {
                        o.next(val.take().unwrap());
                    }
                    o.complete();
                }
            }

            IsClosed::Default
        })).added(sub.clone()));

        sub
    }
}


impl<'a, V:'static, Src, Sch> Observable<'static, V, No> for DebounceOp<V, Src, Sch, No> where
    Sch: SchedulerPeriodic<No>+'static,
    Src : Observable<'a, V, No>
{
    fn sub(&self, o: Mss<No, impl Observer<V> +'static>) -> SubRef
    {
        use observable::RxNoti::*;

        let sch = self.scheduler.clone();
        let dur = self.duration;
        let val : Rc<Cell<Option<V>>> = Rc::new(Cell::new(None));
        let mut timer = SubRef::empty();

        let sub = SubRef::signal().added(timer.clone());

        let o = Rc::new(o);
        sub.add(self.source.sub_noti(byclone!(sub => move |n| {
            timer.unsub();
            match n {
                Next(v) => {
                    val.replace(Some(v));
                    timer = sch.schedule_after(dur, Mss::<No,_>::new(byclone!(o, val, sub => move ||{
                        let mut val = val.replace(None);
                        if val.is_some(){
                            o.next(val.take().unwrap());
                        }
                        if o._is_closed() {
                            sub.unsub();
                        }
                        SubRef::empty()
                    })));

                    if o._is_closed() {
                       sub.unsub();
                       return IsClosed::True;
                    }
                },

                Err(e) => {
                    sub.unsub();
                    let mut val = val.replace(None);
                    if val.is_some() {
                        o.next(val.take().unwrap());
                    }
                    o.err(e);
                },

                Comp => {
                    sub.unsub();
                    let mut val = val.replace(None);
                    if val.is_some() {
                        o.next(val.take().unwrap());
                    }
                    o.complete();
                }
            }

            IsClosed::Default
        })).added(sub.clone()));

        sub
    }
}


#[cfg(test)]
mod test
{
    use super::*;
    use observable::*;
    use std::sync::atomic::AtomicIsize;
    use scheduler::NewThreadScheduler;
    use scheduler::ImmediateScheduler;
    use test_fixture::*;
    use std::sync::atomic::Ordering;
    use std::thread;
    use std::cell::RefCell;

    #[test]
    fn basic()
    {
        let src = ThreadedObservable; //1,2,3
        let out = Arc::new(AtomicIsize::new(0));
        src.debounce(100, NewThreadScheduler::get()).subf(byclone!(out => move |v| { println!("{}",v); out.fetch_add(v as isize, Ordering::SeqCst); }));

        thread::sleep(Duration::from_millis(300));
        assert_eq!(out.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn im()
    {
        let src = SimpleObservable; //1,2,3
        let out = Rc::new(RefCell::new(0));
        src.debounce(100, ImmediateScheduler::get()).subf(byclone!(out => move |v| { println!("{}",v); *out.borrow_mut() += v; }));

        //todo: as expected ?
        assert_eq!(*out.borrow(), 6);
    }
}