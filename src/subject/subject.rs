use crate::*;
use std::sync::*;
use std::cell::UnsafeCell;
use self::SubjectState::*;
use std::sync::Arc;

enum SubjectState<'o, SS:YesNo, V>
{
    Next(Vec<(Arc<ActNext<'o, SS, Ref<V>>>, UnsafeCell<Option<Box<ActEcBox<'o, SS>>>>, Unsub<'o, SS>)>),
    Error(RxError),
    Complete,
    Drop
}

struct Wrap<'o, SS:YesNo, V>
{
    lock: ReSpinLock<SS>,
    state: RecurCell<UnsafeCell<SubjectState<'o, SS, V>>>
}

unsafe impl <'o, V:Send+Sync+'o> Send for Wrap<'o, YES, V> {}
unsafe impl <'o, V:Send+Sync+'o> Sync for Wrap<'o, YES, V> {}

pub struct Subject<'o, SS:YesNo, V>
{
    state: Arc<Wrap<'o,SS,V>>,
}
unsafe impl<'o, V:Send+Sync+'o> Send for Subject<'o, YES, V> {}
unsafe impl<'o, V:Send+Sync+'o> Sync for Subject<'o, YES, V> {}

impl<'o, V, SS:YesNo> Subject<'o, SS, V>
{
    pub fn new() -> Subject<'o, SS, V>
    {
        Subject{ state: Arc::new(Wrap{lock: ReSpinLock::new(), state: RecurCell::new(UnsafeCell::new(Next(Vec::new()))) })  }
    }

    pub fn new_dyn() -> Box<Self>
    {
        box Subject::new()
    }

    #[inline(never)]
    fn unsub(state: &Weak<Wrap<'o,SS,V>>, observer: &Weak<ActNext<'o, SS, Ref<V>>>)
    {
        if let Some(state) = state.upgrade() {
            if let Some(observer) = observer.upgrade() {
                let Wrap{lock, state} = state.as_ref();
                let recur = lock.enter();

                state.map(|s: &UnsafeCell<_>|{
                    if let Next(obs) = unsafe{ &mut *s.get() } {
                        if recur == 0 {
                            let to_drop = obs.iter().position(|(n,_,_)| Arc::ptr_eq(n, &observer)).map(|i| obs.remove(i))
                                .expect("the observer is expected to be in the vec");
                            lock.exit();
                        } else {
                            let mut vec = Vec::with_capacity(obs.len() - 1 );
                            vec.extend(obs.iter()
                                .filter(|(n,_,_)| ! Arc::ptr_eq(n, &observer))
                                .map(|(n, ec, sub)| (n.clone(), UnsafeCell::new(unsafe{ &mut *ec.get() }.take()), sub.clone() ))
                            );
                            let to_drop = state.replace(UnsafeCell::new(SubjectState::Next(vec)));
                            lock.exit();
                        }
                    } else { lock.exit(); }
                });
            }
        }
    }
}


impl<'s, 'o, V, SS:YesNo> ::std::ops::Drop for Subject<'o,SS,V>
{
    fn drop(&mut self)
    {
        let Wrap{lock, state} = self.state.as_ref();
        let _ = lock.enter();

        state.map(|s: &UnsafeCell<_>| {
            let mut to_drop;
            if let Next(vec) = unsafe{ &mut *s.get() } {
                to_drop = state.replace(UnsafeCell::new(Drop));
                for (_, _, sub) in vec { sub.unsub(); }
            }
            lock.exit();
        });
    }
}

impl<'o, V:'o, SS:YesNo>
Observable<'o, SS, Ref<V>>
for Subject<'o, SS, V>
{
    fn subscribe(&self, next: impl ActNext<'o, SS, Ref<V>>, ec: impl ActEc<'o, SS>) -> Unsub<'o, SS> where Self: Sized
    {
        let Wrap{lock, state} = self.state.as_ref();
        let recur = lock.enter();
    
        state.map(|s: &UnsafeCell<_>|{
            match unsafe { &mut *s.get() } {
                Next(obs) => {
                    let next : Arc<ActNext<'o, SS, Ref<V>>> = Arc::new(next);
                    let ec = Box::new(ec);
                    let weak_state = unsafe{ AnySendSync::new(Arc::downgrade(&self.state)) };
                    let weak_next = unsafe{ AnySendSync::new(Arc::downgrade(&next)) };
            
                    let sub = Unsub::with(forward_act_once((SSWrap::new(weak_state), SSWrap::new(weak_next)), |(weak_state, weak_next), ()| {
                        Self::unsub(&*weak_state, &*weak_next);
                    }));
                    if recur == 0 {
                        obs.push((next, UnsafeCell::new(Some(ec)), sub.clone()));
                    } else {
                        let mut vec = Vec::with_capacity(obs.len() + 1);
                        vec.extend( obs.iter().map(|(n, ec, sub)| (n.clone(), UnsafeCell::new(unsafe{ &mut *ec.get() }.take()), sub.clone())));
                        vec.push((next, UnsafeCell::new(Some(ec)), sub.clone()));
                        
                        state.replace(UnsafeCell::new(Next(vec)));
                    }
                    lock.exit();
                    return sub;
                },
                Error(e) => ec.call_once(Some(e.clone())),
                Complete => ec.call_once(None),
                Drop => {}
            }
            
            lock.exit();
            return Unsub::done()
        })
    }

    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, Ref<V>>>, ec: Box<ActEcBox<'o, SS>>) -> Unsub<'o, SS>
    {
        self.subscribe(next, ec)
    }
}

impl<'o, V:'o+'o, SS:YesNo> Subject<'o, SS, V>
{
    pub fn next(&self, v: V)
    {
        self.next_ref(&v);
    }

    pub fn ec(&self, e: Option<RxError>)
    {
        if e.is_some() {
            self.error(e.unwrap());
        } else {
            self.complete();
        }
    }

    pub fn next_ref(&self, v: &V)
    {
        let Wrap{lock, state} = self.state.as_ref();
        let recur = lock.enter();
        
        state.map(|s: &UnsafeCell<_>|{
            if let Next(vec) = unsafe { & *s.get() } {
                for (n,_,sub) in vec {
                    if ! sub.is_done() { n.call(v,); }
                }
            }
            lock.exit();
        });
    }

    pub fn error(&self, e: RxError)
    {
        let e = e.set_handled();
        let Wrap{lock, state} = self.state.as_ref();
        let recur = lock.enter();

        state.map(|s:&UnsafeCell<_>|{
            if let Next(vec) = unsafe { &*s.get() } {
                let to_drop = state.replace(UnsafeCell::new(Error(e.clone())));
        
                for (_,ec,sub) in vec.iter() {
                    if sub.is_done() { continue; }
                    sub.unsub();
                    unsafe{ &mut *ec.get()}.take().map(|ec| ec.call_box(Some(e.clone())));
                }
            }
            lock.exit();
        });
    }

    pub fn complete(&self)
    {
        let Wrap{lock, state} = self.state.as_ref();
        
        let recur = lock.enter();
        
        state.map(|s:&UnsafeCell<_>|{
            
            if let Next(vec) = unsafe { &*s.get() } {
                let to_drop = state.replace(UnsafeCell::new(Complete));
        
                for (_, ec, sub) in vec.iter() {
                    if sub.is_done() { continue; }
                    sub.unsub();
                    unsafe{ &mut *ec.get() }.take().map(|ec| ec.call_box(None));
                }
            }
    
            lock.exit();
        });
    }
}

#[cfg(test)]
mod tests
{
    //use test::Bencher;
    use crate::*;
    use crate::util::clones::*;

    use std::cell::Cell;
    use std::rc::Rc;
    use std::sync::Arc;
    use std::sync::atomic::*;

    #[test]
    fn smoke()
    {
        let n = Arc::new(AtomicI32::new(0));
        let s = Arc::new(Subject::<YES, i32>::new());

        let nn = n.clone();
        let ss = s.clone();
        s.subscribe(move |v:&_| { nn.store(*v, Ordering::SeqCst); }, ());

        ::std::thread::spawn(move ||{
            ss.next(123);
        }).join().ok();
        assert_eq!(n.load(Ordering::SeqCst), 123);

        s.next(1);
        assert_eq!(n.load(Ordering::SeqCst), 1);

        s.next(2);
        assert_eq!(n.load(Ordering::SeqCst), 2);

        //expects: `temp` does not live long enough
//        let s = Subject::<NO, i32>::new();
//        let temp = Cell::new(0);
//        s.sub(|v:&_| { temp.replace(*v); }, ());



        s.complete();

        let n = Cell::new(0);
        let ss = Subject::<NO, i32>::new_dyn();
        ss.subscribe_dyn(box |v:&_| { n.replace(*v); }, box());
    }

    #[test]
    fn next_after_complete()
    {
        let s = Subject::<NO, i32>::new();
        s.subscribe(|_:&_| assert!(false, "shouldn't call"), ());

        s.complete();
        s.next(1);
    }

    #[test]
    fn unsub()
    {
        let s = Subject::<NO, i32>::new();
        let unsub = s.subscribe(|_: &_| assert!(false, "shouldn't call"), ());
        unsub();

        s.next(1);
    }

    #[test]
    fn unsub_in_next()
    {
        let (sub, sub2) = Unsub::new().clones();
        let s = Subject::<NO, i32>::new();
        s.subscribe(move |_: &_| sub.unsub(), ());
        sub2.add(s.subscribe(move |_: &_| assert!(false, "should not happen"), ()));

        s.next(1);
    }

    #[test]
    fn should_complete_only_once()
    {
//        let n = Arc::new(AtomicI32::new(0));
//        let s = Arc::new(Subject::<i32, (), YES>::new());
//
//        let nn = n.clone();
//        s.sub(((), (), move |()|{ nn.fetch_add(1, Ordering::SeqCst); }));
//
//        let mut threads = vec![];
//        for i in 0..8 {
//            let ss = s.clone();
//            threads.push(::std::thread::spawn(move ||{
//
//                for j in 0..10 {
//                    ss.complete();
//                }
//
//            }));
//        }
//
//        for t in threads { t.join(); }
//
//        assert_eq!(n.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn drop_should_unsub()
    {
        let n = Rc::new(Cell::new(0));
        let s = Subject::<NO, i32>::new();

        for _ in 0..10{
            let nn = n.clone();
            s.subscribe(|_:&_|{}, ()).add(Unsub::<NO>::with(move || { nn.replace(nn.get() + 1); }));
        }

        //s.complete();
        drop(s);

        assert_eq!(n.get(), 10);
    }

    #[test]
    fn as_observer()
    {
        let src = Of::value(123);

        let n = std::cell::Cell::new(0);
        let s = Subject::<NO, i32>::new();

        s.subscribe(|v:&_| { n.replace(*v); }, ());

        src.subscribe(|v:&_| s.next(*v), |e| { s.ec(e) } );

        assert_eq!(n.get(), 123);
    }

}