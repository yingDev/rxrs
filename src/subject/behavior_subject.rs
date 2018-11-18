use crate::*;
use std::sync::Arc;
use std::cell::UnsafeCell;

pub struct BehaviorSubject<'o, SS:YesNo, V>
{
    lock: ReSpinLock<SS>,
    val: UnsafeCell<Option<V>>,
    subj: Subject<'o, SS, V>
}

unsafe impl<'o, V:Send+Sync+'o> Send for BehaviorSubject<'o, YES, V>{}
unsafe impl<'o, V:Send+Sync+'o> Sync for BehaviorSubject<'o, YES, V>{}

impl<'o, V:'o, SS:YesNo> BehaviorSubject<'o, SS, V>
{
    #[inline(always)]
    pub fn new(value: V) -> BehaviorSubject<'o, SS, V>
    {
        BehaviorSubject{ lock: ReSpinLock::new(), val: UnsafeCell::new(Some(value)), subj: Subject::new() }
    }

    #[inline(always)]
    pub fn value(&self, f: impl FnOnce(Option<&V>))
    {
        self.lock.enter();
        let val = unsafe{ (&*self.val.get()) }.as_ref();
        f(val);
        self.lock.exit();
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
        if unsafe { &*self.val.get() }.is_some() {
            unsafe { &mut *self.val.get() }.replace(v);
            self.subj.next_ref(unsafe { &*self.val.get() }.as_ref().unwrap());
        }
        self.lock.exit();

    }

    pub fn error(&self, e:RxError)
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