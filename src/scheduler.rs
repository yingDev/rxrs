use std::time::Duration;
use unsub_ref::UnsubRef;
use std::sync::Arc;
use std::sync::Once;
use std::sync::ONCE_INIT;

//todo: facade

pub trait Scheduler
{
    fn schedule(&self, act: impl Send+'static+FnOnce()->UnsubRef<'static>) -> UnsubRef<'static>;
    fn schedule_after(&self, due: Duration, act: impl Send+'static+FnOnce()->UnsubRef<'static>) -> UnsubRef<'static>;

    fn schedule_periodic(&self, period: Duration,sigStop: UnsubRef<'static>, act: impl Send+'static+Fn()) -> UnsubRef<'static>
    {
        unimplemented!()
    }
    fn schedule_long_running(&self, sigStop: UnsubRef<'static>, act: impl Send+'static+FnOnce()) -> UnsubRef<'static>
    {
        if sigStop.disposed() { return sigStop; }
        self.schedule(||{
            if sigStop.disposed() { return sigStop; }
            act();
            sigStop
        })
    }
}

pub struct ImmediateScheduler;

impl ImmediateScheduler
{
    pub fn new() -> ImmediateScheduler { ImmediateScheduler }
}

impl Scheduler for ImmediateScheduler
{
    fn schedule(&self, act: impl Send+'static+FnOnce()->UnsubRef<'static>) -> UnsubRef<'static>
    {
        act()
    }

    fn schedule_after(&self, due: Duration, act: impl Send+'static+FnOnce()->UnsubRef<'static>) -> UnsubRef<'static>
    {
        ::std::thread::sleep(due);
        act()
    }

    fn schedule_periodic(&self, period: Duration, sigStop: UnsubRef<'static>, act: impl Send+'static+Fn()) -> UnsubRef<'static>
    {
        while ! sigStop.disposed()
        {
            ::std::thread::sleep(period);
            if sigStop.disposed() {
                break;
            }
            act();
        }

        sigStop
    }
}

static mut _NewThreadScheduler: Option<Arc<NewThreadScheduler>> = None;
static _NewThreadScheduler_INIT: Once = ONCE_INIT;

pub struct NewThreadScheduler
{

}
impl NewThreadScheduler
{
    pub fn get() -> Arc<NewThreadScheduler> {
        _NewThreadScheduler_INIT.call_once(|| {
            unsafe { _NewThreadScheduler = Some(Arc::new(NewThreadScheduler{})); }
        });
        unsafe { _NewThreadScheduler.as_ref().unwrap().clone() }
    }
}

impl Scheduler for NewThreadScheduler
{
    fn schedule(&self, act: impl Send+'static+FnOnce()->UnsubRef<'static>) -> UnsubRef<'static>
    {
        let unsub = UnsubRef::signal();
        let unsub2 = unsub.clone();

       ::std::thread::spawn(move ||{
           unsub2.add(act());
       });

       unsub
    }

    fn schedule_after(&self, due: Duration, act: impl Send+'static+FnOnce()->UnsubRef<'static>) -> UnsubRef<'static>
    {
        let unsub = UnsubRef::signal();
        let unsub2 = unsub.clone();

        ::std::thread::spawn(move ||{
            ::std::thread::sleep(due);
            if ! unsub2.disposed() { unsub2.add(act()); }
        });

        unsub
    }

    fn schedule_periodic(&self, period: Duration, sigStop: UnsubRef<'static>, act: impl Send+'static+Fn()) -> UnsubRef<'static>
    {
        let stop = sigStop.clone();
        ::std::thread::spawn(move ||
        {
            while ! stop.disposed(){
                ::std::thread::sleep(period);
                if stop.disposed() { break; }
                act();
            }
        });
        sigStop
    }
}