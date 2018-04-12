use std::sync::Arc;
use std::sync::{Once, ONCE_INIT};
use std::marker::PhantomData;
use std::sync::Mutex;
use std::collections::{VecDeque, HashMap};
use std::cell::{Cell, RefCell};

use rx::observable::*;
use rx::scheduler::*;
use rx::util::mss::*;
use rx::*;

use std::time::Duration;
use gtk::{idle_add, timeout_add,is_initialized_main_thread};
use std::boxed::FnBox;
use std::sync::atomic::{AtomicBool, Ordering};
use std::rc::Rc;

macro_rules! ensure_main_thread(()=>{
    if !::gtk::is_initialized_main_thread() {
        panic!("this method can only be called from main thread");
    }
});

pub struct GtkScheduler<SSA: ? Sized + 'static>
{
    PhantomData: PhantomData<(&'static SSA)>
}


impl GtkScheduler<No>
{
    pub fn get() -> Arc<GtkScheduler<No>>
    {
        ensure_main_thread!();

        static mut VALUE: Option<Arc<GtkScheduler<No>>> = None;
        static VALUE_INIT: Once = ONCE_INIT;
        VALUE_INIT.call_once(|| {
            unsafe { VALUE = Some(Arc::new(GtkScheduler { PhantomData })) }
        });
        unsafe { VALUE.as_ref().unwrap().clone() }
    }
}

impl SychronizationContext for GtkScheduler<No>
{
    fn create_send(&self, act: Box<'static+FnBox()>)->SubRef<Yes>
    {
        thread_local!(
            static CBS: RefCell<HashMap<u64, Box<'static+FnBox()>>> = RefCell::new(HashMap::new());
            static ID : Cell<u64> = Cell::new(0);
        );

        let id = ID.with(|it| it.replace(it.get() + 1));
        CBS.with(|it| it.borrow_mut().insert(id, act));

        InnerSubRef::<Yes>::new(move || {
            ::glib::idle_add(move ||{
                CBS.with(|it| it.borrow_mut().remove(&id)).unwrap().call_box(());
                ::gtk::Continue(false)
            });
        }).into_subref()
    }
}

impl GtkScheduler<Yes>
{
    pub fn get_ss() -> Arc<GtkScheduler<Yes>>
    {
        static mut VALUE: Option<Arc<GtkScheduler<Yes>>> = None;
        static VALUE_INIT: Once = ONCE_INIT;
        VALUE_INIT.call_once(|| {
            unsafe { VALUE = Some(Arc::new(GtkScheduler { PhantomData })) }
        });
        unsafe { VALUE.as_ref().unwrap().clone() }
    }
}

impl Scheduler<No, No> for GtkScheduler<No>
{
    //todo: cancel ?????
    fn schedule(&self, act: Mss<No, impl 'static + FnOnce() -> SubRef<No>>) -> SubRef<No>
    {
        ensure_main_thread!();

        let mut opt = Some(act);
        let mut src = idle_add(move || {
            if let Some(cb) = ::std::mem::replace(&mut opt, None) {
                (cb.into_inner())();
            }
            ::gtk::Continue(false)
        });

        InnerSubRef::<No>::new(move || {
            ::glib::source_remove(src);
        }).into_subref()
    }

    fn schedule_after(&self, due: Duration, act: Mss<No, impl 'static + FnOnce() -> SubRef<No>>) -> SubRef<No>
    {
        ensure_main_thread!();

        let mut opt = Some(act);

        let sig = InnerSubRef::<No>::signal();
        let done = Arc::new(AtomicBool::new(false));

        let id = timeout_add(dur2millis(due), byclone!(sig,done => move || {
            if !done.compare_and_swap(false, true, Ordering::AcqRel) {
                if let Some(cb) = opt.take() {
                    (cb.into_inner())();
                }
                sig.unsub();
            }

            ::gtk::Continue(false)
        }));

        sig.addedf(move || {
           ::glib::source_remove(id );
        }).into_subref()
    }
}

impl Scheduler<Yes, Yes> for GtkScheduler<Yes>
{
    fn schedule(&self, act: Mss<Yes, impl 'static + FnOnce() -> SubRef<Yes>>) -> SubRef<Yes>
    {
        let mut act: Option<Mss<Yes, _>> = Some(act);
        let mut id = Some(::glib::idle_add(move || {
            ((act.take().unwrap()).into_inner())();
            ::gtk::Continue(false)
        }));

        InnerSubRef::<Yes>::new(move || ::glib::source_remove(id.take().unwrap()) ).into_subref()
    }

    fn schedule_after(&self, due: Duration, act: Mss<Yes, impl 'static + FnOnce() -> SubRef<Yes>>) -> SubRef<Yes>
    {
        let sig = InnerSubRef::<Yes>::signal();

        let mut act: Option<Mss<Yes, _>> = Some(act);
        let mut id = Some(::glib::timeout_add(dur2millis(due), byclone!(sig => move || {
            ((act.take().unwrap()).into_inner())();
            sig.unsub();
            ::gtk::Continue(false)
        })));

        sig.addedf(move || {
            ::glib::source_remove(id.take().unwrap());
        }).into_subref()
    }
}


impl Scheduler<Yes, No> for GtkScheduler<Yes>
{
    fn schedule(&self, act: Mss<Yes, impl 'static + FnOnce() -> SubRef<No>>) -> SubRef<No>
    {
        let mut act: Option<Mss<Yes, _>> = Some(act);
        let mut id = ::glib::idle_add(move || {
            ((act.take().unwrap()).into_inner())();
            ::gtk::Continue(false)
        });

        InnerSubRef::<No>::new(move || ::glib::source_remove(id) ).into_subref()
    }

    fn schedule_after(&self, due: Duration, act: Mss<Yes, impl 'static + FnOnce() -> SubRef<No>>) -> SubRef<No>
    {
        let sig = InnerSubRef::<Yes>::signal();

        let mut act: Option<Mss<Yes, _>> = Some(act);
        let mut id = ::glib::timeout_add(dur2millis(due), byclone!(sig => move || {
            ((act.take().unwrap()).into_inner())();
            sig.unsub();
            ::gtk::Continue(false)
        }));

        sig.addf(move || {
            ::glib::source_remove(id);
        });

        InnerSubRef::<No>::signal().added(sig.into_subref()).into_subref()
    }
}

impl SchedulerPeriodic<No, No> for GtkScheduler<No>
{
    fn schedule_periodic(&self, period: Duration, sigStop: InnerSubRef<No>, act: Mss<No, impl 'static + Fn()>) -> SubRef<No>
    {
        ensure_main_thread!();

        let act = act.into_inner();

        let mut id = Some(timeout_add(dur2millis(period), byclone!(sigStop =>  move || {
            if sigStop.disposed() {
                return ::gtk::Continue(false);
            }
            act();
            ::gtk::Continue(!sigStop.disposed())
        })));

        sigStop.addedf(move || {
            if is_initialized_main_thread() {
                ::glib::source_remove(id.take().unwrap());
            } else {
                ::glib::idle_add(move || {
                    ::glib::source_remove(id.take().unwrap());
                    ::gtk::Continue(false)
                });
             }
        }).into_subref()
    }
}

impl SchedulerPeriodic<Yes, Yes> for GtkScheduler<Yes>
{
    fn schedule_periodic(&self, period: Duration, sigStop: InnerSubRef<Yes>, act: Mss<Yes, impl 'static + Fn()>) -> SubRef<Yes>
    {
        let act : Arc<Mss<Yes, _>> = Arc::new(act);
        let mut id = Some(::glib::timeout_add(dur2millis(period), byclone!(sigStop =>  move || {
            if sigStop.disposed() {
                return ::gtk::Continue(false);
            }
            act();

            ::gtk::Continue(!sigStop.disposed())
        })));

        sigStop.addedf(move || { ::glib::source_remove(id.take().unwrap()); }).into_subref()
    }
}

impl<'sa, V: 'static + Send + Sync, Src> SchedulerObserveOn<'sa, V, Src, No, No, No, No> for GtkScheduler<No>
    where Src: Observable<'sa, V, No, No>
{
    type ObserveOn = ObserveOnGtk<'sa, Src, No, No>;
    fn observe_on(&self, source: Src) -> Self::ObserveOn { ObserveOnGtk { source, PhantomData } }
}

//impl<'sa, V: 'static + Send + Sync, Src> SchedulerObserveOn<'sa, V, Src, No, Yes, No, No> for GtkScheduler<No>
//    where Src: Observable<'sa, V, No, Yes>
//{
//    type ObserveOn = ObserveOnGtk<'sa, Src, No, Yes>;
//    fn observe_on(&self, source: Src) -> Self::ObserveOn { ObserveOnGtk { source, PhantomData } }
//}

impl<'sa, V: 'static + Send + Sync, Src> SchedulerObserveOn<'sa, V, Src, Yes, No, No, No> for GtkScheduler<No>
    where Src: Observable<'sa, V, Yes, No>
{
    type ObserveOn = ObserveOnGtk<'sa, Src, Yes, No>;
    fn observe_on(&self, source: Src) -> Self::ObserveOn { ObserveOnGtk { source, PhantomData } }
}


impl<'sa, V: 'static + Send + Sync, Src> SchedulerObserveOn<'sa, V, Src, Yes, Yes,  Yes, Yes> for GtkScheduler<Yes>
    where Src: Observable<'sa, V, Yes, Yes>
{
    type ObserveOn = ObserveOnGtkSS<'sa, Src, Yes>;

    fn observe_on(&self, source: Src) -> Self::ObserveOn { ObserveOnGtkSS { source, PhantomData } }
}

//impl<'sa, V: 'static + Send + Sync, Src> SchedulerObserveOn<'sa, V, Src, Yes, No, Yes, No> for GtkScheduler<Yes>
//    where Src: Observable<'sa, V, Yes, No>
//{
//    type ObserveOn = ObserveOnGtkSS<'sa, Src, Yes>;
//    fn observe_on(&self, source: Src) -> Self::ObserveOn { ObserveOnGtkSS { source, PhantomData } }
//}


pub struct ObserveOnGtk<'sa, Src, SrcSSO: ? Sized , SrcSSS:?Sized>
{
    source: Src,
    PhantomData: PhantomData<(&'sa (), *const SrcSSO, *const SrcSSS)>
}

pub struct ObserveOnGtkSS<'sa, Src, SrcSSO: ? Sized + 'static>
{
    source: Src,
    PhantomData: PhantomData<(&'sa (), *const SrcSSO)>
}

macro_rules! fn_sub(($sss:ty)=>{
    fn sub(&self, o: Mss<No, impl Observer<V> + 'static>) -> SubRef<$sss>
    {
        use rx::observable::RxNoti::*;

        ensure_main_thread!();


        thread_local!(
            static DISPATCHERS : RefCell<HashMap<u64,Box<Fn()>>> = RefCell::new(HashMap::new());
            static ID : Cell<u64> = Cell::new(0);
        );

        if o._is_closed() {
            return SubRef::empty();
        }

        let queue = Arc::new(Mutex::new(VecDeque::new()));
        let id = ID.with(|i| i.replace(i.get() + 1));

        let sub = InnerSubRef::<$sss>::new(move || {
            println!("ObserveOnGtk: unsub");
            ::glib::idle_add(move || {
                DISPATCHERS.with(|d| d.borrow_mut().remove(&id));
                ::gtk::Continue(false)
            });
        });

        DISPATCHERS.with(byclone!(queue, sub => move |d| {
            d.borrow_mut().insert(id, box move || {
                let mut q = queue.lock().unwrap();
                while let Some(noti) = q.pop_front() {
                    match noti {
                        Next(v) => {
                            if o._is_closed() { sub.unsub(); return; }
                            o.next(v);
                            if o._is_closed() { sub.unsub(); }
                        },
                        Err(e) => { sub.unsub(); o.err(e);},
                        Comp => { sub.unsub(); o.complete(); },
                    }
                }
            });
        }));

        sub.add(self.source.sub_noti(byclone!(sub, queue => move |n| {
            if sub.disposed() {
                return IsClosed::True;
            }
            queue.lock().unwrap().push_back(n); //todo

            ::glib::idle_add(move || {
                DISPATCHERS.with(|d| {
                    if let Some(cb) = d.borrow().get(&id) {
                        cb();
                    }
                });
                ::gtk::Continue(false)
            });
            return IsClosed::Default;
        })));

        sub.into_subref()
    }
});

impl<'sa, V: 'static + Send + Sync, Src> Observable<'static, V, No, Yes> for ObserveOnGtk<'sa, Src, No, Yes> where Src: Observable<'sa, V, No, Yes> //todo
{
    fn_sub!(Yes);
}
//
//impl<'sa, V: 'static + Send + Sync, Src> Observable<'static, V, No, No> for ObserveOnGtk<'sa, Src, No, Yes> where Src: Observable<'sa, V, No, Yes> //todo
//{
//    fn_sub!(No);
//}

impl<'sa, V: 'static + Send + Sync, Src> Observable<'static, V, No, No> for ObserveOnGtk<'sa, Src, Yes, No> where Src: Observable<'sa, V, Yes, No> //todo
{
    //fn_sub!(No);
    fn sub(&self, o: Mss<No, impl Observer<V> + 'static>) -> SubRef<No>
    {
        use rx::observable::RxNoti::*;
        ensure_main_thread!();

        thread_local!(
            static DISPATCHERS : RefCell<HashMap<u64,Box<Fn()>>> = RefCell::new(HashMap::new());
            static ID : Cell<u64> = Cell::new(0);
        );

        if o._is_closed() {
            return SubRef::empty();
        }

        let queue = Arc::new(Mutex::new(VecDeque::new()));
        let id = ID.with(|i| i.replace(i.get() + 1));

        let sub = InnerSubRef::<No>::new(move || {
            println!("ObserveOnGtk: unsub");
            ::glib::idle_add(move || {
                DISPATCHERS.with(|d| d.borrow_mut().remove(&id));
                ::gtk::Continue(false)
            });
        });

        DISPATCHERS.with(byclone!(queue, sub => move |d| {
            d.borrow_mut().insert(id, box move || {
                let mut q = queue.lock().unwrap();
                while let Some(noti) = q.pop_front() {
                    match noti {
                        Next(v) => {
                        if o._is_closed() { sub.unsub(); return; }
                        o.next(v);
                            if o._is_closed() { sub.unsub(); }
                        },
                        Err(e) => { sub.unsub(); o.err(e);},
                            Comp => { sub.unsub(); o.complete(); },
                        }
                }
            });
        }));

        sub.added(self.source.sub_noti(byclone!(queue => move |n| {

            queue.lock().unwrap().push_back(n); //todo

            ::glib::idle_add(move || {
                DISPATCHERS.with(|d| {
                    if let Some(cb) = d.borrow().get(&id) {
                        cb();
                    }
                });
                ::gtk::Continue(false)
            });

            return IsClosed::Default;
        }))).into_subref()
    }
}

impl<'sa, V: 'static + Send + Sync, Src> Observable<'static, V, No, No> for ObserveOnGtk<'sa, Src, No, No> where Src: Observable<'sa, V, No, No> //todo
{
    fn_sub!(No);
}

macro_rules! fn_sub_ss(($sss:ty)=>{
    fn sub(&self, o: Mss<Yes, impl Observer<V> + 'static>) -> SubRef<$sss>
    {
        use rx::observable::RxNoti::*;

        if o._is_closed() {
            return SubRef::empty();
        }
        let sub = InnerSubRef::<$sss>::signal();
        let o : Arc<Mss<Yes,_>> = Arc::new(o);

        sub.add(self.source.sub_noti(byclone!(o,sub => move |n| {
            if o._is_closed() {
                sub.unsub();
                return IsClosed::True;
            }
            let id = match n {
                Next(v) => {
                    let mut v = Some(v);
                    ::glib::idle_add(byclone!(sub, o => move ||{
                        o.next((v.take().unwrap()));
                        if o._is_closed() { sub.unsub(); }
                        ::gtk::Continue(false)
                    }))
                },

                Err(e) => {
                    let mut e = Some(e);
                    ::glib::idle_add(byclone!(sub, o=> move ||{
                        sub.unsub();
                        o.err(e.take().unwrap());
                        ::gtk::Continue(false)
                    }))
                },

                Comp => ::glib::idle_add(byclone!(sub, o => move || {
                    sub.unsub();
                    o.complete();
                    ::gtk::Continue(false)
                }))
            };
            return IsClosed::Default;
        })));

        sub.into_subref()
    }
});

impl<'sa, V: 'static + Send + Sync, Src> Observable<'static, V, Yes, Yes> for ObserveOnGtkSS<'sa, Src, Yes> where Src: Observable<'sa, V, Yes, Yes> //todo
{
    fn_sub_ss!(Yes);
}

//impl<'sa, V: 'static + Send + Sync, Src> Observable<'static, V, Yes, No> for ObserveOnGtkSS<'sa, Src, Yes> where Src: Observable<'sa, V, Yes, No> //todo
//{
//    fn_sub_ss!(No);
//}

fn dur2millis(dur: Duration) -> u32
{
    let nanos = dur.subsec_nanos() as u64;
    let ms = (1000 * 1000 * 1000 * dur.as_secs() + nanos) / (1000 * 1000);
    ms as u32
}
