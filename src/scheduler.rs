use std::time::Duration;
use subref::SubRef;
use std::sync::Arc;
use std::sync::Once;
use std::sync::ONCE_INIT;
use util::mss::*;

//todo: facade

pub trait Scheduler
{
    type SSA:?Sized;

    fn schedule(&self, act: Mss<Self::SSA,impl 'static+FnOnce()->SubRef>) -> SubRef;
    fn schedule_after(&self, due: Duration, act: Mss<Self::SSA, impl 'static+FnOnce()->SubRef>) -> SubRef;

    fn schedule_periodic(&self, period: Duration,sigStop: SubRef, act: Mss<Self::SSA, impl 'static+Fn()>) -> SubRef
    {
        unimplemented!()
    }
    fn schedule_long_running(&self, sigStop: SubRef, act: Mss<Self::SSA, impl 'static+FnOnce()>) -> SubRef;
}

pub struct ImmediateScheduler;

impl ImmediateScheduler
{
    pub fn new() -> ImmediateScheduler { ImmediateScheduler }
}

impl Scheduler for ImmediateScheduler
{
    type SSA = No;

    fn schedule(&self, act: Mss<Self::SSA,impl 'static+FnOnce()->SubRef>) -> SubRef
    {
        (act.into_inner())()
    }

    fn schedule_after(&self, due: Duration, act: Mss<Self::SSA,impl 'static+FnOnce()->SubRef>) -> SubRef
    {
        ::std::thread::sleep(due);
        (act.into_inner())()
    }

    fn schedule_long_running(&self, sigStop: SubRef, act: Mss<Self::SSA, impl 'static+FnOnce()>) -> SubRef
    {
        if sigStop.disposed() { return sigStop; }
        let act = act.into_inner();
        self.schedule(Mss::no(||{
            if sigStop.disposed() { return sigStop; }
            act();
            sigStop
        }))
    }

    fn schedule_periodic(&self, period: Duration, sigStop: SubRef, act: Mss<Self::SSA,impl 'static+Fn()>) -> SubRef
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
//
//static mut _NewThreadScheduler: Option<Arc<NewThreadScheduler>> = None;
//static _NewThreadScheduler_INIT: Once = ONCE_INIT;
//
//pub struct NewThreadScheduler
//{
//
//}
//impl NewThreadScheduler
//{
//    pub fn get() -> Arc<NewThreadScheduler> {
//        _NewThreadScheduler_INIT.call_once(|| {
//            unsafe { _NewThreadScheduler = Some(Arc::new(NewThreadScheduler{})); }
//        });
//        unsafe { _NewThreadScheduler.as_ref().unwrap().clone() }
//    }
//}
//
//impl Scheduler for NewThreadScheduler
//{
//    fn schedule(&self, act: impl Send+'static+FnOnce()->SubRef) -> SubRef
//    {
//        let unsub = SubRef::signal();
//        let unsub2 = unsub.clone();
//
//       ::std::thread::spawn(move ||{
//           unsub2.add(act());
//       });
//
//       unsub
//    }
//
//    fn schedule_after(&self, due: Duration, act: impl Send+'static+FnOnce()->SubRef) -> SubRef
//    {
//        let unsub = SubRef::signal();
//        let unsub2 = unsub.clone();
//
//        ::std::thread::spawn(move ||{
//            ::std::thread::sleep(due);
//            if ! unsub2.disposed() { unsub2.add(act()); }
//        });
//
//        unsub
//    }
//
//    fn schedule_periodic(&self, period: Duration, sigStop: SubRef, act: impl Send+'static+Fn()) -> SubRef
//    {
//        let stop = sigStop.clone();
//        ::std::thread::spawn(move ||
//        {
//            while ! stop.disposed(){
//                ::std::thread::sleep(period);
//                if stop.disposed() { break; }
//                act();
//            }
//        });
//        sigStop
//    }
//}