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
    pub fn new(obs: impl Into<Vec<DynObservable<'s, 'o, SS, By>>>) -> Merge<'s, 'o, SS, By>
    {
        let obs = obs.into();
        Merge{ obs }
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
        let ec = Arc::new(unsafe{ AnySendSync::new(UnsafeCell::new(Some(ec))) });
        let count = Arc::new(AtomicUsize::new(self.obs.len()));
        
        for o in self.obs.iter() {
            let ec = forward_ec((unsub.clone(), SSWrap::new(ec.clone()), SSWrap::new(count.clone())), |(unsub, ec, count), e| {
                unsub.if_not_done(||{
                    if e.is_some() {
                        unsub.unsub();
                        unsafe { &mut *ec.get() }.take().map_or((), |ec| ec.call_once(e))
                    } else if count.fetch_sub(1, Ordering::Relaxed) == 1 {
                        unsub.unsub();
                        unsafe { &mut *ec.get() }.take().map_or((), |ec| ec.call_once(e))
                    }
                });
            });
            unsub.add(o.subscribe(next.clone(), ec));
        }
        
        unsub
    }
    
    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, By>>, err_or_comp: Box<ActEcBox<'o, SS>>) -> Unsub<'o, SS>
    {
        unimplemented!()
    }
}

#[cfg(test)]
mod test
{
    use crate::*;
    
    #[test]
    fn smoke()
    {
        let vals = Merge::new(vec![Of::value(123).into_dyn(), Of::value(456).into_dyn()]);
        vals.subscribe(|v:&_| println!("v={}", *v), |e| println!("complete"));
    }
}