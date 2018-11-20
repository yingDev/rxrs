use crate::*;
use std::sync::Arc;
use std::cell::UnsafeCell;
use std::ptr;
use std::cell::Cell;
use std::ops::Deref;

pub struct ReSpinMutex<SS:YesNo, V>
{
    lock: ReSpinLock<SS>,
    value: RecurCell<V>,
}

pub struct Guard<'a, SS:YesNo, V>
{
    value: &'a RecurCell<V>,
    lock: &'a ReSpinLock<SS>
}

impl<'a, SS:YesNo, V> Drop for Guard<'a, SS, V>
{
    fn drop(&mut self)
    {
        self.lock.exit();
    }
}

impl<'a, SS:YesNo, V> Deref for Guard<'a, SS, V>
{
    type Target = RecurCell<V>;
    fn deref(&self) -> &Self::Target { self.value }
}

impl<SS:YesNo, V> ReSpinMutex<SS, V>
{
    pub fn new(value: V) -> Self
    {
        ReSpinMutex { lock: ReSpinLock::new(), value: RecurCell::new(value) }
    }
    
    pub fn lock(&self) -> Guard<SS, V>
    {
        self.lock.enter();
        Guard{ value: &self.value, lock: &self.lock }
    }
}



//todo: move
pub struct RecurCell<V>
{
    val: UnsafeCell<Option<V>>,
    cur: Cell<*const Option<V>>,
}

impl<V> RecurCell<V>
{
    pub fn new(val: V) -> Self
    {
        RecurCell { val: UnsafeCell::new(Some(val)), cur: Cell::new(ptr::null()) }
    }
    
    pub fn map<U>(&self, f: impl FnOnce(&V) -> U) -> U
    {
        let val = unsafe{ &mut *self.val.get() }.take();
        let u = if val.is_some() {
            assert_eq!(self.cur.get(), ptr::null());
            self.cur.replace(&val);
            f(val.as_ref().unwrap())
        } else {
            assert_ne!(self.cur.get(), ptr::null());
            f(unsafe{ &*self.cur.get() }.as_ref().unwrap())
        };
        
        if self.cur.get() == &val {
            self.cur.replace(ptr::null());
            assert!(unsafe{ &*self.val.get() }.is_none());
            *unsafe{ &mut *self.val.get() } = val;
        }
        
        u
    }
    
    pub fn replace(&self, val: V) -> Option<V>
    {
        self.cur.replace(ptr::null());
        unsafe{ &mut *self.val.get() }.replace(val)
    }
}


pub struct BehaviorSubject<'o, SS:YesNo, V>
{
    //lock: ReSpinLock<SS>,
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
        BehaviorSubject{
            val: ReSpinMutex::new(Some(value)),
            //lock: ReSpinLock::new(),
            subj: Subject::new()
        }
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
                    val.map(|v: &Option<V>| {
                        next.call(v.as_ref().unwrap());
                    });
                }
            });
            sub
        } else {
            self.subj.subscribe( next, ec)
        }
    }

    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, Ref<V>>>, ec: Box<ActEcBox<'o, SS>>) -> Unsub<'o, SS>
    {
        self.subscribe(next, ec)
    }
}


impl<'o, V:'o, SS:YesNo> BehaviorSubject<'o, SS, V>
{
    pub fn next(&self, v:V)
    {
        let cell = self.val.lock();
        cell.map(|val: &Option<V>| {
            if val.is_some() {
                cell.replace(Some(v));
                cell.map(|val: &Option<V>| {
                    self.subj.next_ref(val.as_ref().unwrap());
                });
            }
        })
    }

    pub fn error(&self, e:RxError)
    {
        let cell = self.val.lock();
        cell.map(|val: &Option<V>| {
            if val.is_some() {
                cell.replace(None);
                self.subj.error(e);
            }
        });
    }

    pub fn complete(&self)
    {
        let cell = self.val.lock();
        cell.map(|val: &Option<V>| {
            if val.is_some() {
                cell.replace(None);
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