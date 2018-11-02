use std::sync::Arc;
use std::cell::UnsafeCell;
use crate::*;

pub struct BehaviorSubject<'o, SS:YesNo, V, E:Clone=()>
{
    lock: ReSpinLock<SS>,
    val: UnsafeCell<Option<V>>,
    subj: Subject<'o, SS, V, E>
}

unsafe impl<'o, V:Send+Sync+'o, E:Send+Sync+Clone+'o> Send for BehaviorSubject<'o, YES, V, E>{}
unsafe impl<'o, V:Send+Sync+'o, E:Send+Sync+Clone+'o> Sync for BehaviorSubject<'o, YES, V, E>{}

impl<'o, V:'o, E:Clone, SS:YesNo> BehaviorSubject<'o, SS, V, E>
{
    #[inline(always)]
    pub fn new(value: V) -> BehaviorSubject<'o, SS, V, E>
    {
        BehaviorSubject{ lock: ReSpinLock::new(), val: UnsafeCell::new(Some(value)), subj: Subject::new() }
    }

    #[inline(always)]
    pub fn value(&self) -> Option<&V>
    {
        self.lock.enter();
        let val = unsafe{ (&*self.val.get()) }.as_ref();
        self.lock.exit();

        val
    }

    #[inline(never)]
    fn sub_internal(&self, next: Arc<ActNext<'o, SS, Ref<V>>>, make_sub: impl FnOnce()->Unsub<'o, SS>) -> Unsub<'o, SS>
    {
        self.lock.enter();
        let val = unsafe { &mut *self.val.get() };
        if val.is_none() {
            self.lock.exit();
            return Unsub::done();
        }

        let sub = make_sub();
        next.call(val.as_ref().unwrap());

        self.lock.exit();
        sub
    }
}

impl<'o, V:'o, E:Clone+'o>
Observable<'o, NO, Ref<V>, Ref<E>>
for BehaviorSubject<'o, NO, V, E>
{
    fn subscribe(&self, next: impl ActNext<'o, NO, Ref<V>>, ec: impl ActEc<'o, NO, Ref<E>>) -> Unsub<'o, NO> where Self: Sized
    {
        self.subscribe_dyn(box next, box ec)
    }

    fn subscribe_dyn(&self, next: Box<ActNext<'o, NO, Ref<V>>>, ec: Box<ActEcBox<'o, NO, Ref<E>>>) -> Unsub<'o, NO>
    {
        let next: Arc<ActNext<'o, NO, Ref<V>>> = next.into();
        self.sub_internal(next.clone(),  move || self.subj.subscribe_dyn(box move |v:&_| next.call(v), ec))
    }
}

impl<V:Clone+'static+Send+Sync, E:Clone+'static+Send+Sync>
Observable<'static, YES, Ref<V>, Ref<E>>
for BehaviorSubject<'static, YES, V, E>
{
    fn subscribe(&self, next: impl ActNext<'static, YES, Ref<V>>, ec: impl ActEc<'static, YES, Ref<E>>) -> Unsub<'static, YES> where Self: Sized
    {
        self.subscribe_dyn(box next, box ec)
    }

    fn subscribe_dyn(&self, next: Box<ActNext<'static, YES, Ref<V>>>, ec: Box<ActEcBox<'static, YES, Ref<E>>>) -> Unsub<'static, YES>
    {
        let next: Box<ActNext<'static, YES, Ref<V>>+Send+Sync> = unsafe{ ::std::mem::transmute(next) };
        let next: Arc<ActNext<'static, YES, Ref<V>>+Send+Sync> = next.into();
        self.sub_internal(next.clone(),  move || self.subj.subscribe_dyn(box move |v:&_| next.call(v), ec))
    }
}


impl<'o, V:'o, E:Clone+'o, SS:YesNo> BehaviorSubject<'o, SS, V, E>
{
    pub fn next(&self, v:V)
    {
        self.lock.enter();
        if unsafe { &*self.val.get() }.is_some() {
            unsafe { &mut *self.val.get() }.replace(v);
            self.subj.next_ref(unsafe { &*self.val.get() }.as_ref().unwrap());
        }
        self.lock.exit();

    }

    pub fn error(&self, e:E)
    {
        self.lock.enter();
        let _ = if unsafe { &mut *self.val.get() }.is_some() {
            unsafe { &mut *self.val.get() }.take()
        } else { None };
        self.lock.exit();

        self.subj.error(e);
    }

    pub fn complete(&self)
    {
        self.lock.enter();
        let _ = if unsafe { &mut *self.val.get() }.is_some() {
            unsafe { &mut *self.val.get()}.take()
        }else { None };
        self.lock.exit();

        self.subj.complete();
    }
}


#[cfg(test)]
mod test
{

    use std::cell::Cell;
    use crate::*;

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
}