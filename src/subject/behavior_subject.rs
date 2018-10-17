use std::rc::Rc;
use std::sync::Arc;
use std::cell::UnsafeCell;
use crate::*;
use crate::{util::alias::SSs, sync::ReSpinLock};

pub struct BehaviorSubject<'o, SS:YesNo, V, E:Clone=()>
{
    lock: ReSpinLock<SS>,
    val: UnsafeCell<Option<V>>,
    subj: Subject<'o, SS, V, E>
}

unsafe impl<'o, V:SSs, E:SSs+Clone> Send for BehaviorSubject<'o, YES, V, E>{}
unsafe impl<'o, V:SSs, E:SSs+Clone> Sync for BehaviorSubject<'o, YES, V, E>{}

impl<'o, V, E:Clone, SS:YesNo> BehaviorSubject<'o, SS, V, E>
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
    fn sub_internal(&self, next: Arc<for<'x> FnNext<NO, By<'x,Ref<V>>>+'o>, make_sub: impl FnOnce()->Unsub<'o, SS>) -> Unsub<'o, SS>
    {
        self.lock.enter();
        let val = unsafe { &mut *self.val.get() };
        if val.is_none() {
            self.lock.exit();
            return Unsub::done();
        }

        let sub = make_sub();
        next.call(By::r(val.as_ref().unwrap()));

        self.lock.exit();
        sub
    }
}

impl<'o, V:Clone+'o, E:Clone+'o> Observable<'o, NO, Ref<V>, Ref<E>> for  BehaviorSubject<'o, NO, V, E>
{
    fn sub(&self, next: impl for<'x> FnNext<NO, By<'x, Ref<V>>>+'o, ec: impl for<'x> FnErrComp<NO, By<'x, Ref<E>>>+'o) -> Unsub<'o, NO> where Self: Sized
    {
        self.sub_dyn(box next, box ec)
    }

    fn sub_dyn(&self, next: Box<for<'x> FnNext<NO, By<'x, Ref<V>>>+'o>, ec: Box<for<'x> FnErrCompBox<NO, By<'x, Ref<E>>> +'o>) -> Unsub<'o, NO>
    {
        let next: Arc<for<'x> FnNext<NO, By<'x, Ref<V>>>+'o> = next.into();
        self.sub_internal(next.clone(),  move || self.subj.sub_dyn(box move |v:By<_>| next.call(v), ec))
    }
}

//impl< V:SSs, E:SSs> ObservableSendSync<V, E> for  BehaviorSubject<'static, V, E, YES>
//{
//    #[inline(always)] fn sub(&self, o: impl Observer<V, E> + Send + Sync + 'static) -> Unsub<'static, YES>
//    {
//        self.sub_internal(|| self.subj.sub(o))
//    }
//}

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
        let old = if unsafe { &mut *self.val.get() }.is_some() {
            unsafe { &mut *self.val.get() }.take()
        } else { None };
        self.lock.exit();

        self.subj.error(e);
    }

    pub fn complete(&self)
    {
        self.lock.enter();
        let old = if unsafe { &mut *self.val.get() }.is_some() {
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
    use std::sync::Arc;
    use crate::*;

    #[test]
    fn shoudl_emit_on_sub()
    {
        let n = Cell::new(0);
        let x = Cell::new(0);

        let s = BehaviorSubject::<NO, i32>::new(123);

        s.sub(|v:By<_>| { n.replace(*v); }, ());
        assert_eq!(n.get(), 123);

        s.next(456);
        assert_eq!(n.get(), 456);

        s.next(789);

        s.sub(|v:By<_>| { x.replace(*v); }, ());
        assert_eq!(x.get(), 789);
        assert_eq!(n.get(), 789);
    }
}