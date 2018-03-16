use std::rc::Rc;
use std::any::Any;
use observable::*;
use subref::SubRef;
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
use observable::RxNoti::*;
use std::mem;


#[derive(Clone)]
pub struct DebounceOp<'a, 'b, V, Sch>
{
    source : Arc<Observable<'a,V>+'b+Send+Sync>,
    scheduler: Arc<Sch>,
    duration: Duration,
}

pub trait ObservableDebounce<'b, V, Sch> where Sch: Scheduler+Send+Sync
{
    fn debounce(self, duration: u64, scheduler: Arc<Sch>) -> Arc<Observable<'static, V> + 'b+Send+Sync>;
}

impl<'a:'b, 'b, V: 'static+Send+Sync, Sch> ObservableDebounce<'b, V, Sch> for Arc<Observable<'a,V>+'b+Send+Sync> where Sch: Scheduler+Send+Sync+'static
{
    fn debounce(self, duration: u64, scheduler: Arc<Sch>) -> Arc<Observable<'static, V> + 'b+Send+Sync>
    {
        Arc::new(DebounceOp{ source: self, scheduler, duration: Duration::from_millis(duration) })
    }
}

impl<'a:'b, 'b, V:'static+Send+Sync, Sch> Observable<'static, V> for DebounceOp<'a,'b, V, Sch> where
    Sch: Scheduler+Send+Sync+'static
{
    fn sub(&self, dest: Arc<Observer<V> + Send + Sync+'static>) -> SubRef
    {
        let sch = self.scheduler.clone();
        let dur = self.duration;
        let val = Arc::new(Mutex::new(None));
        let mut timer = SubRef::empty();

        let sub = SubRef::signal();
        let sub2 = sub.clone();

        sub.add(self.source.sub_noti(move |n| {
            timer.unsub();
            if sub2.disposed() {
                return IsClosed::True;
            }
            match n {
                Next(v) => {
                    *val.lock().unwrap() = Some(v);
                    let dest = dest.clone();
                    let val = val.clone();
                    timer = sch.schedule_after(dur, move ||{
                        let mut val = val.lock().unwrap();
                        if val.is_some(){
                            dest.next(mem::replace(&mut *val, None).unwrap());
                        }
                        SubRef::empty()
                    });
                },

                Err(e) => {
                    sub2.unsub();
                    let mut val = val.lock().unwrap();
                    if val.is_some() {
                        dest.next(mem::replace(&mut *val, None).unwrap());
                    }
                    dest.err(e);
                },

                Comp => {
                    sub2.unsub();
                    let mut val = val.lock().unwrap();
                    if val.is_some() {
                        dest.next(mem::replace(&mut *val, None).unwrap());
                    }
                    dest.complete();
                }
            }

            IsClosed::Default
        }));

        sub
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
        }).rx().debounce(100, NewThreadScheduler::get())
            .subf(( move |v| r2.lock().unwrap().push(v),
                  (),
                  move ||{ r3.lock().unwrap().push(100) }
            ));

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
            SubRef::empty()
        }).rx().debounce(100, NewThreadScheduler::get())
            .subf(( move |v| r2.lock().unwrap().push(v),
                    move |e| { r4.lock().unwrap().push(1000)  },
                    move | |{ r3.lock().unwrap().push(100) }
            ));

        ::std::thread::sleep(Duration::from_secs(2));

        assert_eq!(&*r.lock().unwrap(), &[2,1000]);
    }

}