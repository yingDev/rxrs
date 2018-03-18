use std::time::Duration;
use subref::SubRef;
use std::sync::Arc;
use std::sync::Once;
use std::sync::ONCE_INIT;
use util::traicks::*;

//todo: facade


pub struct MaySend<IsSend, T> where IsSend : YesNo
{
    t: T,
    isSend: IsSend
}

impl<IsSend,T> MaySend<IsSend, T> where IsSend : YesNo
{
    pub fn into_inner(self) -> T
    {
        self.t
    }
}
unsafe impl<T> Send for MaySend<Yes, T> {}

impl<T:Send> MaySend<Yes, T>
{
    fn new(t: T) -> MaySend<Yes, T> { MaySend{ t, isSend: Yes } }
}

impl<T> MaySend<No, T>
{
    fn new(t: T) -> MaySend<No, T> { MaySend{ t, isSend: No } }
}


pub trait Scheduler
{
    type RequireSend: YesNo;

    fn schedule(&self, act: MaySend<Self::RequireSend, impl 'static+FnOnce()->SubRef>) -> SubRef;
    fn schedule_after(&self, due: Duration, act: MaySend<Self::RequireSend, impl 'static+FnOnce()->SubRef>) -> SubRef;

    fn schedule_periodic(&self, period: Duration,sigStop: SubRef, act: MaySend<Self::RequireSend, impl 'static+Fn()>) -> SubRef
    {
        unimplemented!()
    }
    fn schedule_long_running(&self, sigStop: SubRef, act: MaySend<Self::RequireSend, impl 'static+FnOnce()>) -> SubRef
    {
        unimplemented!()
    }
}

pub struct ImmediateScheduler;

impl ImmediateScheduler
{
    pub fn new() -> ImmediateScheduler { ImmediateScheduler }
}

impl Scheduler for ImmediateScheduler
{
    type RequireSend = No;

    fn schedule(&self, act: MaySend<Self::RequireSend, impl 'static+FnOnce()->SubRef>) -> SubRef
    {
        (act.into_inner())()
    }

    fn schedule_after(&self, due: Duration, act: MaySend<Self::RequireSend, impl 'static+FnOnce()->SubRef>) -> SubRef
    {
        ::std::thread::sleep(due);
        (act.into_inner())()
    }

    fn schedule_periodic(&self, period: Duration,sigStop: SubRef, act: MaySend<Self::RequireSend, impl 'static+Fn()>) -> SubRef
    {
        let act = act.into_inner();
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

    fn schedule_long_running(&self, sigStop: SubRef, act: MaySend<Self::RequireSend, impl 'static+FnOnce()>) -> SubRef
    {
        if sigStop.disposed() { return sigStop; }
        self.schedule( MaySend::<No, _>::new(move ||{
            if sigStop.disposed() { return sigStop; }
            (act.into_inner())();
            sigStop
        }))
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
    type RequireSend = Yes;

    fn schedule(&self, act: MaySend<Self::RequireSend, impl 'static+FnOnce()->SubRef>) -> SubRef
    {
        let unsub = SubRef::signal();
        let unsub2 = unsub.clone();

       ::std::thread::spawn(move ||{
           unsub2.add((act.into_inner())());
       });

       unsub
    }

    fn schedule_after(&self, due: Duration, act: MaySend<Self::RequireSend, impl 'static+FnOnce()->SubRef>) -> SubRef
    {
        let unsub = SubRef::signal();
        let unsub2 = unsub.clone();

        ::std::thread::spawn(move ||{
            ::std::thread::sleep(due);
            if ! unsub2.disposed() { unsub2.add((act.into_inner())()); }
        });

        unsub
    }

    fn schedule_periodic(&self, period: Duration,sigStop: SubRef, act: MaySend<Self::RequireSend, impl 'static+Fn()>) -> SubRef
    {
        let stop = sigStop.clone();
        ::std::thread::spawn(move ||
        {
            let act = act.into_inner();
            while ! stop.disposed(){
                ::std::thread::sleep(period);
                if stop.disposed() { break; }
                act();
            }
        });
        sigStop
    }

    fn schedule_long_running(&self, sigStop: SubRef, act: MaySend<Self::RequireSend, impl 'static+FnOnce()>) -> SubRef
    {
        if sigStop.disposed() { return sigStop; }
        let unsub = SubRef::signal();
        let unsub2 = unsub.clone();

        ::std::thread::spawn(move ||{
            (act.into_inner())();
        });

        unsub
    }
}