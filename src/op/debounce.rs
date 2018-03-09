use std::rc::Rc;
use std::any::Any;
use subscriber::*;
use observable::*;
use unsub_ref::UnsubRef;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use util::AtomicOption;
use util::ArcCell;
use std::marker::PhantomData;
use scheduler::Scheduler;
use std::time::Duration;
use std::sync::Mutex;

pub struct DebounceState<V, Sch>
{
    scheduler: Arc<Sch>,
    dur: usize,
    timer: Mutex<UnsubRef<'static>>,
    val: Arc<Mutex<Option<V>>>
}

impl<V,Sch> DebounceState<V,Sch>
{
    fn emmit_last(&self, dest: &Arc<Observer<V>+Send+Sync>)
    {
        let mut timerLock = self.timer.lock().unwrap();
        let timer = ::std::mem::replace(&mut *timerLock, UnsubRef::empty());
        if ! timer.disposed() {
            timer.unsub();
            ::std::mem::drop(timerLock);

            let mut vlock = self.val.lock().unwrap();
            if let Some(v) = ::std::mem::replace(&mut *vlock, None) {
                ::std::mem::drop(vlock);
                dest.next(v);
            }
        }
    }
}

#[derive(Clone)]
pub struct DebounceOp<V, Src, Sch>
{
    source : Src,
    scheduler: Arc<Sch>,
    duration: usize,

    PhantomData: PhantomData<V>
}

pub trait ObservableDebounce<V, Src, Sch> where
    Sch: Scheduler+Send+Sync,
    Src : Observable<V>,
    Self: Sized
{
    fn debounce(self, duration: usize, scheduler: Arc<Sch>) -> DebounceOp<V, Src, Sch>;
}

impl<V,Src, Sch> ObservableDebounce<V, Src, Sch> for Src where
    Sch: Scheduler+Send+Sync+'static,
    Src : Observable<V>,
    Self: Sized
{
    fn debounce(self, duration: usize, scheduler: Arc<Sch>) -> DebounceOp<V, Src, Sch>
    {
        DebounceOp{ source: self, scheduler, duration, PhantomData }
    }
}

impl<V:'static+Send+Sync, Src, Sch> Observable<V> for DebounceOp<V, Src, Sch> where
    Sch: Scheduler+Send+Sync+'static,
    Src : Observable<V>
{
    fn sub(&self, dest: Arc<Observer<V>+Send+Sync>) -> UnsubRef<'static>
    {
        let s = Arc::new(Subscriber::new(DebounceState{
            scheduler: self.scheduler.clone(),
            dur: self.duration,
            timer: Mutex::new(UnsubRef::empty()),
            val: Arc::new(Mutex::new(None))
        }, dest, false));

        s.set_unsub(&self.source.sub(s.clone()));

        UnsubRef::empty()
    }
}

impl<V, Sch> SubscriberImpl<V, DebounceState<V, Sch>> for Subscriber<V, DebounceState<V, Sch>> where
    V: Send+Sync+'static,
    Sch: Scheduler
{
    fn on_next(&self, v: V)
    {
        let s = &self._state;
        let dest = self._dest.clone();

        let mut val = s.val.lock().unwrap();
        let oldVal = ::std::mem::replace(&mut *val, Some(v));
        if oldVal.is_some() {
            s.timer.lock().unwrap().unsub();
        }
        ::std::mem::drop(val);

        let val2 = Arc::clone(&s.val);

        let timer = s.scheduler.schedule_after(Duration::from_millis(s.dur as u64), move ||{
            let mut vlock = val2.lock().unwrap();
            if let Some(v) = ::std::mem::replace(&mut *vlock, None) {
                ::std::mem::drop(vlock);
                if !dest._is_closed() {
                    dest.next(v);
                }
            }
            UnsubRef::empty()
        });

        if !timer.disposed(){
            *s.timer.lock().unwrap() = timer;
        }
    }

    fn on_err(&self, e: Arc<Any+Send+Sync>)
    {
        self._state.emmit_last(&self._dest);
        self.do_unsub();
        self._dest.err(e);
    }

    fn on_comp(&self)
    {
        self._state.emmit_last(&self._dest);
        self.do_unsub();
        self._dest.complete();
    }
}


#[cfg(test)]
mod test
{
    use super::*;
    use subject::*;
    use fac::*;
    use op::*;
    use observable::*;
    use std::sync::atomic::AtomicIsize;
    use scheduler::NewThreadScheduler;
    use scheduler::ImmediateScheduler;

    #[test]
    fn basic()
    {
        fn sleep(ms: u64){ ::std::thread::sleep(::std::time::Duration::from_millis(ms)) }

        let r = Arc::new(Mutex::new(vec![]));
        let (r2, r3) = (r.clone(), r.clone());

        rxfac::create(|o|{
            ::std::thread::spawn(move ||{
                o.next(1);sleep(10);
                o.next(2);sleep(110);
                o.next(3);sleep(10);
                o.next(4);sleep(10);
                o.next(5);sleep(10);
                o.next(6);sleep(200);
                o.next(7);
                o.complete();
            });
            UnsubRef::empty()
        }).debounce(100, NewThreadScheduler::get())
            .subf(move |v| r2.lock().unwrap().push(v),
                  (),
                  move ||{ r3.lock().unwrap().push(100) }
            );

        ::std::thread::sleep(Duration::from_secs(2));

        assert_eq!(&*r.lock().unwrap(), &[2,6,7,100]);
    }

    #[test]
    fn error()
    {
        fn sleep(ms: u64){ ::std::thread::sleep(::std::time::Duration::from_millis(ms)) }

        let r = Arc::new(Mutex::new(vec![]));
        let (r2, r3, r4) = (r.clone(), r.clone(), r.clone());

        rxfac::create(|o|{
            ::std::thread::spawn(move ||{
                o.next(1);sleep(10);
                o.next(2);o.err(Arc::new(123));
                //o.next(3);sleep(10);
                //o.next(4);sleep(10);
                //o.next(5);sleep(10);
                //o.next(6);sleep(200);
                //o.next(7);
                //o.complete();
            });
            UnsubRef::empty()
        }).debounce(100, NewThreadScheduler::get())
            .subf(move |v| r2.lock().unwrap().push(v),
                  move |e| { r4.lock().unwrap().push(1000)  },
                  move | |{ r3.lock().unwrap().push(100) }
            );

        ::std::thread::sleep(Duration::from_secs(2));

        assert_eq!(&*r.lock().unwrap(), &[2,1000]);
    }

}