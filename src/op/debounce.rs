use observable::*;
use subref::SubRef;
use std::sync::Arc;
use std::marker::PhantomData;
use scheduler::Scheduler;
use std::time::Duration;
use std::mem;
use util::mss::*;
use std::sync::Mutex;

pub struct DebounceOp<V, Src, Sch, SSO:?Sized+'static>
{
    source : Src,
    scheduler: Arc<Sch>,
    duration: Duration,

    PhantomData: PhantomData<(V, *const SSO)>
}

pub trait ObservableDebounce<'sa, V, Src, Sch, SSO: ?Sized+'static> where
    Sch: Scheduler+Send+Sync,
    Src : Observable<'sa, V, SSO>,
{
    fn debounce(self, duration: u64, scheduler: Arc<Sch>) -> DebounceOp<V, Src, Sch, SSO>;
}

impl<'sa, V,Src, Sch, SSO:?Sized+'static> ObservableDebounce<'sa, V, Src, Sch, SSO> for Src where
    Sch: Scheduler+Send+Sync+'static,
    Src : Observable<'sa, V, SSO>,
{
    fn debounce(self, duration: u64, scheduler: Arc<Sch>) -> DebounceOp<V, Src, Sch, SSO>
    {
        DebounceOp{ source: self, scheduler, duration: Duration::from_millis(duration), PhantomData }
    }
}

impl<'a, V:'static+Send+Sync, Src, Sch> Observable<'static, V, Yes> for DebounceOp<V, Src, Sch, Yes> where
    Sch: Scheduler+Send+Sync+'static,
    Sch::SSA: Yes,
    Src : Observable<'a, V, Yes>
{
    fn sub(&self, dest: Mss<Yes, impl Observer<V> +'a>) -> SubRef
    {
        use observable::RxNoti::*;

        let sch = self.scheduler.clone();
        let dur = self.duration;
        let val = Arc::new(Mutex::new(None));
        let mut timer = SubRef::empty();
        let dest = Arc::new(dest);

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
    use observable::*;
    use std::sync::atomic::AtomicIsize;
    use scheduler::NewThreadScheduler;
    use scheduler::ImmediateScheduler;

    #[test]
    fn basic()
    {

    }

    #[test]
    fn error()
    {

    }

}