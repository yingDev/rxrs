use crate::*;
use std::sync::Arc;
use std::cell::UnsafeCell;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ptr;
use std::cell::Cell;

pub struct MutexSlim<SS:YesNo, V>
{
    lock: ReSpinLock<SS>,
    value: UnsafeCell<V>,
}

pub struct Guard<'a, SS:YesNo, V>
{
    lock: &'a ReSpinLock<SS>,
    val: &'a mut V,
}

impl<'a, SS:YesNo, V> Deref for Guard<'a, SS, V>
{
    type Target = V;
    
    fn deref(&self) -> &V
    {
        self.val
    }
}

impl<'a, SS:YesNo, V> DerefMut for Guard<'a, SS, V>
{
    fn deref_mut(&mut self) -> &mut V
    {
        self.val
    }
}

impl<'a, SS:YesNo, V> Drop for Guard<'a, SS, V>
{
    fn drop(&mut self)
    {
        self.lock.exit();
    }
}


impl<SS:YesNo, V> MutexSlim<SS, V>
{
    pub fn new(value: V) -> Self
    {
        MutexSlim { lock: ReSpinLock::new(), value: UnsafeCell::new(value) }
    }
    
    pub fn lock(&self) -> Guard<SS, V>
    {
        let recur = self.lock.enter();
        if recur != 0 {
            panic!("MutexSlim is not reentrant; please drop the guard first.");
        }
        Guard{ lock: &self.lock, val: unsafe{ &mut *self.value.get() } }
    }
    
    pub fn is_recur(&self) -> bool { self.lock.recur() != 0 }
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
    lock: ReSpinLock<SS>,
    val: RecurCell<Option<V>>,
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
            val: RecurCell::new(Some(value)),
            lock: ReSpinLock::new(),
            subj: Subject::new()
        }
    }

    pub fn value<U>(&self, map: impl FnOnce(&Option<V>) -> U) -> U
    {
        self.lock.enter();
        let u = self.val.map(map);
        self.lock.exit();
        
        u
    }
}

impl<'o, V:'o, SS:YesNo>
Observable<'o, SS, Ref<V>>
for BehaviorSubject<'o, SS, V>
{
    fn subscribe(&self, next: impl ActNext<'o, SS, Ref<V>>, ec: impl ActEc<'o, SS>) -> Unsub<'o, SS> where Self: Sized
    {
        self.lock.enter();
        
        let sub = if self.val.map(|v:&_| v.is_some()) {
            let next = Arc::new(SSActNextWrap::new(next));
            let sub = self.subj.subscribe( next.clone(), ec);
            sub.if_not_done(||{
                if ! next.stopped() {
                    self.val.map(|v: &Option<V>| {
                        next.call(v.as_ref().unwrap());
                    });
                }
            });
            sub
        } else {
            self.subj.subscribe( next, ec)
        };
        
        self.lock.exit();
        
        sub
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
        self.lock.enter();
        
        self.val.map(|val: &Option<V>| {
            if val.is_some() {
                self.val.replace(Some(v));
                self.val.map(|val: &Option<V>| {
                    self.subj.next_ref(val.as_ref().unwrap());
                });
            }
        });
        
        self.lock.exit();
    }

    pub fn error(&self, e:RxError)
    {
        self.lock.enter();
        
        self.val.map(|val: &Option<V>| {
            if val.is_some() {
                self.val.replace(None);
                self.subj.error(e);
            }
        });
        
        self.lock.exit();
    }

    pub fn complete(&self)
    {
        self.lock.enter();
    
        self.val.map(|val: &Option<V>| {
            if val.is_some() {
                self.val.replace(None);
                self.subj.complete();
            }
        });
        
        self.lock.exit();
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