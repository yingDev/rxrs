use crate::*;
use std::sync::Arc;

pub struct BehaviorSubject<'o, SS:YesNo, V>
{
    val: ReSpinMutex<SS, Option<V>>,
    subj: Subject<'o, SS, V>,
}

unsafe impl<'o, V:Send+Sync+'o> Send for BehaviorSubject<'o, YES, V>{}
unsafe impl<'o, V:Send+Sync+'o> Sync for BehaviorSubject<'o, YES, V>{}

impl<'o, V:'o, SS:YesNo> BehaviorSubject<'o, SS, V>
{
    #[inline(always)]
    pub fn new(value: V) -> BehaviorSubject<'o, SS, V>
    {
        BehaviorSubject{ val: ReSpinMutex::new(Some(value)), subj: Subject::new() }
    }

    pub fn value<U>(&self, map: impl FnOnce(&Option<V>) -> U) -> U
    {
        self.val.lock().map(map)
    }
}

impl<'o, V:'o, SS:YesNo>
Observable<'o, SS, Ref<V>>
for BehaviorSubject<'o, SS, V>
{
    fn subscribe(&self, next: impl ActNext<'o, SS, Ref<V>>, ec: impl ActEc<'o, SS>) -> Unsub<'o, SS> where Self: Sized
    {
        let val = self.val.lock();
        if val.map(|v:&_| v.is_some()) {
            let next = Arc::new(SSActNextWrap::new(next));
            let sub = self.subj.subscribe( next.clone(), ec);
            sub.if_not_done(||{
                if ! next.stopped() {
                    val.map(|v: &Option<V>| next.call(v.as_ref().unwrap()));
                }
            });
            sub
        } else {
            self.subj.subscribe(next, ec)
        }
    }

    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, Ref<V>>>, ec: Box<ActEcBox<'o, SS>>) -> Unsub<'o, SS>
    { self.subscribe(next, ec) }
}


impl<'o, V:'o, SS:YesNo> BehaviorSubject<'o, SS, V>
{
    pub fn next(&self, v:V)
    {
        let cell = self.val.lock();
        cell.map(|val: &Option<V>| {
            if val.is_some() {
                let _drop = cell.replace(Some(v));
                cell.map(|val: &Option<V>| self.subj.next_ref(val.as_ref().unwrap()));
            }
        })
    }

    pub fn error(&self, e:RxError)
    {
        let cell = self.val.lock();
        cell.map(|val: &Option<V>| {
            if val.is_some() {
                let _drop = cell.replace(None);
                self.subj.error(e);
            }
        });
    }

    pub fn complete(&self)
    {
        let cell = self.val.lock();
        cell.map(|val: &Option<V>| {
            if val.is_some() {
                let _drop = cell.replace(None);
                self.subj.complete();
            }
        });
        
    }
}


#[cfg(test)]
mod test
{

    use std::cell::Cell;
    use crate::*;
    use std::rc::Rc;
    use crate::util::clones::Clones;
    
    #[test]
    fn shoudl_emit_on_sub()
    {
        let n = Cell::new(0);
        let x = Cell::new(0);

        let s = BehaviorSubject::<NO, i32>::new(123);

        s.subscribe(|v:&_| { n.replace(*v); }, ());
        assert_eq!(n.get(), 123);

        s.next(456);
        assert_eq!(n.get(), 456);

        s.next(789);

        s.subscribe(|v:&_| { x.replace(*v); }, ());
        assert_eq!(x.get(), 789);
        assert_eq!(n.get(), 789);
    }
    
    #[test]
    fn recurse()
    {
        let (n, n1) = Rc::new(Cell::new(0)).clones();
        
        let s = Rc::new(BehaviorSubject::<NO, i32>::new(1));
        s.clone().subscribe(move |v: &i32| {
            if *v == 1 {
                s.next(2);
                let n = n.clone();
                s.subscribe(move |v:&_| { n.replace(n.get() + v); }, ());
            }
            n.replace(n.get() + v);
        }, ());
        
        assert_eq!(n1.get(), 1+2+2);
    }
    
    
}