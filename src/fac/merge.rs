use crate::*;
use std::sync::Arc;
use std::cell::UnsafeCell;
use std::sync::atomic::*;

pub struct Merge<'s, 'o, SS:YesNo, By: RefOrVal>
{
    obs: Vec<DynObservable<'s, 'o, SS, By>>
}

impl<'s, 'o, SS:YesNo, By: RefOrVal> Merge<'s, 'o, SS, By>
{
    pub fn new(obs: impl Into<Vec<DynObservable<'s, 'o, SS, By>>>) -> Self
    {
        Merge{ obs: obs.into() }
    }
}

impl<'s, 'o, SS:YesNo, By: RefOrVal+'o>
Observable<'o, SS, By>
for Merge<'s, 'o, SS, By>
{
    fn subscribe(&self, next: impl ActNext<'o, SS, By>, ec: impl ActEc<'o, SS>) -> Unsub<'o, SS> where Self: Sized
    {
        let unsub = Unsub::<SS>::new();
        let next = Arc::new(SSActNextWrap::new(next));
        let state = Arc::new((unsafe{ AnySendSync::new(UnsafeCell::new(Some(ec))) }, AtomicUsize::new(self.obs.len())));
        
        for o in self.obs.iter() {
            let ec = forward_ec((unsub.clone(), SSWrap::new(state.clone())), |(unsub, state), e| {
                unsub.if_not_done(||{
                    if e.is_some() {
                        unsub.unsub();
                        unsafe { &mut *state.0.get() }.take().map_or((), |ec| ec.call_once(e))
                    } else if state.1.fetch_sub(1, Ordering::Relaxed) == 1 {
                        unsub.unsub();
                        unsafe { &mut *state.0.get() }.take().map_or((), |ec| ec.call_once(e))
                    }
                });
            });
            unsub.if_not_done(|| {
                unsub.add(o.subscribe(next.clone(), ec));
            });
        }
        
        unsub
    }
    
    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, By>>, err_or_comp: Box<ActEcBox<'o, SS>>) -> Unsub<'o, SS>
    { self.subscribe(next, err_or_comp) }
}

#[cfg(test)]
mod test
{
    use crate::*;
    use std::rc::Rc;
    use crate::util::clones::Clones;
    use std::cell::RefCell;
    use std::time::Duration;
    use std::sync::Arc;
    use std::sync::atomic::*;
    
    #[test]
    fn smoke()
    {
        let vals = Merge::new(vec![Of::<NO, i32>::value(123).into_dyn(), Of::<NO, i32>::value(456).into_dyn()]);
        vals.subscribe(|v:&_| println!("v={}", *v), |_| println!("complete"));
    }
    
    #[test]
    fn ops()
    {
        let (r1, r2, r3) = Rc::new(RefCell::new(String::new())).clones();
        
        let a = Of::<NO, i32>::value(1).map(|v:&_| *v).into_dyn();
        let b = a.clone().map(|v| v+1);
        let c = b.clone().map(|v| v+1);
        let d = c.clone().take(0);
        
        Merge::new(vec![a, b, c, d]).map(|v| format!("{}", v)).subscribe(move |v:String| {
            r1.borrow_mut().push_str(&v);
        }, move |_| {
            r2.borrow_mut().push_str("ok");
        });
        
        assert_eq!("123ok", r3.borrow().as_str());
    }
    
    #[test]
    fn asynced()
    {
        let (n, n1, n2) = Arc::new(AtomicUsize::new(0)).clones();
        let sch = Arc::new(NewThreadScheduler::new(Arc::new(DefaultThreadFac)));
        
        let a = Of::<YES, usize>::value(1).map(|v:&_| *v).into_dyn();
        let b = Timer::new(Duration::from_millis(10), sch.clone()).take(10).into_dyn();
        let c = Of::<YES, usize>::value(2).map(|v:&_| *v).into_dyn();
        let d = Timer::new(Duration::from_millis(20), sch.clone()).take(10).into_dyn();
    
        Merge::new(vec![a, b, c, d]).subscribe(move |v| {
            n1.fetch_add(v, Ordering::SeqCst);
        }, move |_| {
            n2.store(n2.load(Ordering::SeqCst) * 10, Ordering::SeqCst);
        });
        
        ::std::thread::sleep(Duration::from_millis(250));
        
        assert_eq!(n.load(Ordering::SeqCst), 930);
        
    }
}