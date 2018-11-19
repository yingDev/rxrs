use crate::*;
use std::sync::Arc;
use std::cell::UnsafeCell;
use std::sync::atomic::*;
use std::ops::Deref;
use std::ops::DerefMut;
use std::collections::VecDeque;

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


pub struct BehaviorSubject<'o, SS:YesNo, V>
{
    lock: ReSpinLock<SS>,
    val: UnsafeCell<Arc<Option<V>>>,
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
            val: UnsafeCell::new(Arc::new(Some(value))),
            lock: ReSpinLock::new(),
            subj: Subject::new()
        }
    }

    pub fn value<U>(&self, map: impl FnOnce(&Option<V>) -> U) -> U
    {
        self.lock.enter();
        let val = unsafe { (&*self.val.get()) }.clone();
        self.lock.exit();
        map(Arc::as_ref(&val))
        
    }

    #[inline(never)]
    fn sub_internal(&self, next: Arc<ActNext<'o, SS, Ref<V>>>, make_sub: impl FnOnce()->Unsub<'o, SS>) -> Unsub<'o, SS>
    {
        self.lock.enter();
        
        let sub = make_sub();
        if ! sub.is_done() {
            let val = unsafe { &*self.val.get() }.clone();
            next.call(Option::as_ref(&val).unwrap());
        }

        self.lock.exit();
        
        sub
    }
}

impl<'o, V:'o>
Observable<'o, NO, Ref<V>>
for BehaviorSubject<'o, NO, V>
{
    fn subscribe(&self, next: impl ActNext<'o, NO, Ref<V>>, ec: impl ActEc<'o, NO>) -> Unsub<'o, NO> where Self: Sized
    {
        self.subscribe_dyn(box next, box ec)
    }

    fn subscribe_dyn(&self, next: Box<ActNext<'o, NO, Ref<V>>>, ec: Box<ActEcBox<'o, NO>>) -> Unsub<'o, NO>
    {
        let next: Arc<ActNext<'o, NO, Ref<V>>> = next.into();
        //fixme: forward
        self.sub_internal(next.clone(),  move || self.subj.subscribe_dyn(box move |v:&_| next.call(v), ec))
    }
}

impl<V:Clone+'static+Send+Sync>
Observable<'static, YES, Ref<V>>
for BehaviorSubject<'static, YES, V>
{
    fn subscribe(&self, next: impl ActNext<'static, YES, Ref<V>>, ec: impl ActEc<'static, YES>) -> Unsub<'static, YES> where Self: Sized
    {
        self.subscribe_dyn(box next, box ec)
    }

    fn subscribe_dyn(&self, next: Box<ActNext<'static, YES, Ref<V>>>, ec: Box<ActEcBox<'static, YES>>) -> Unsub<'static, YES>
    {
        let next: Box<ActNext<'static, YES, Ref<V>>+Send+Sync> = unsafe{ ::std::mem::transmute(next) };
        let next: Arc<ActNext<'static, YES, Ref<V>>+Send+Sync> = next.into();
        self.sub_internal(next.clone(),  move || self.subj.subscribe_dyn(box move |v:&_| next.call(v), ec))
    }
}


impl<'o, V:'o, SS:YesNo> BehaviorSubject<'o, SS, V>
{
    pub fn next(&self, v:V)
    {
        self.lock.enter();
        
        let val = unsafe{ &mut *self.val.get() };
        
        if val.is_some() {
            if let Some(val) = Arc::get_mut(val) {
                *val = Some(v);
            } else {
                *val = Arc::new(Some(v));
            }
            let val = val.clone();
            self.lock.exit();
            self.subj.next_ref(Option::as_ref(&val).unwrap());
            return;
        }
        
        self.lock.exit();
    }

    pub fn error(&self, e:RxError)
    {
        self.lock.enter();
    
        let val = unsafe{ &mut *self.val.get() };
        let to_drop;
        if val.is_some() {
            if let Some(val) = Arc::get_mut(val) {
                to_drop = val.take();
            } else {
                *val = Arc::new(None);
            }
        }
        self.lock.exit();
        
        self.subj.error(e);
    }

    pub fn complete(&self)
    {
        self.lock.enter();
    
        let val = unsafe{ &mut *self.val.get() };
        let to_drop;
        if val.is_some() {
            if let Some(val) = Arc::get_mut(val) {
                to_drop = val.take();
            } else {
                *val = Arc::new(None);
            }
        }
        self.lock.exit();
    
        self.subj.complete();
    }
}


#[cfg(test)]
mod test
{

    use std::cell::Cell;
    use crate::*;
    use std::rc::Rc;
    
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
            }
            n.replace(n.get() + v);
        }, ());
        
        assert_eq!(n1.get(), 3);
    }
    
    
}