use crate::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::rc::Rc;

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
        let next = Arc::new(SSActNextWrap::new(next));
        
        let ec = Arc::new(Mutex::new(Some(ec)));
        
        let unsub = Unsub::<SS>::new();
        for obs in self.obs.iter() {
            unsub.if_not_done(|| {
                let ec = ec.clone();
                let unsub2 = unsub.clone();

//                unsub.add_each(obs.subscribe(next.clone(), move |e:Option<RxError>| {
//                    unsub2.unsub_then(|| ec.lock().unwrap().take().map_or((), |ec| ec.call_once(e)))
//                }));
            });
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