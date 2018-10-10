use std::rc::Rc;
use std::sync::Arc;
use std::cell::UnsafeCell;
use crate::*;
use crate::{util::trait_alias::CSS, sync::ReSpinLock};

pub struct BehaviorSubject<'o, V:Clone+'o, E:Clone+'o, SS:YesNo>
{
    lock: ReSpinLock<SS>,
    val: UnsafeCell<Option<V>>,
    subj: Subject<'o, V, E, SS>
}

unsafe impl<'o, V:CSS, E:CSS> Send for BehaviorSubject<'o, V, E, YES>{}
unsafe impl<'o, V:CSS, E:CSS> Sync for BehaviorSubject<'o, V, E, YES>{}

impl<'o, V:Clone+'o, E:Clone+'o, SS:YesNo> BehaviorSubject<'o, V, E, SS>
{
    #[inline(always)]  pub fn new(value: V) -> BehaviorSubject<'o, V, E, SS>
    {
        BehaviorSubject{ lock: ReSpinLock::new(), val: UnsafeCell::new(Some(value)), subj: Subject::new() }
    }

    #[inline(always)] pub fn value(&self) -> Option<V>
    {
        self.lock.enter();
        let val = unsafe{ (&*self.val.get()).clone() };
        self.lock.exit();

        val
    }

    #[inline(never)]
    fn sub_internal(&self, make_sub: impl FnOnce()->Unsub<'o, SS>) -> Unsub<'o, SS>
    {
        self.lock.enter();
        let val = unsafe { &mut *self.val.get() };
        if val.is_none() {
            self.lock.exit();
            return Unsub::done();
        }

        let sub = make_sub();
        self.subj.next(val.as_ref().unwrap().clone());
        self.lock.exit();
        sub
    }
}

impl<'o, V:Clone+'o, E:Clone+'o> Observable<'o, V, E> for  BehaviorSubject<'o, V, E, NO>
{
    #[inline(always)] fn sub(&self, o: impl Observer<V, E> + 'o) -> Unsub<'o, NO>
    {
        self.sub_internal(|| self.subj.sub(o))
    }
}

impl< V:CSS, E:CSS> ObservableSendSync<V, E> for  BehaviorSubject<'static, V, E, YES>
{
    #[inline(always)] fn sub(&self, o: impl Observer<V, E> + Send + Sync + 'static) -> Unsub<'static, YES>
    {
        self.sub_internal(|| self.subj.sub(o))
    }
}

impl<'o, V:Clone+'o, E:Clone+'o, SS:YesNo> Observer<V, E> for BehaviorSubject<'o, V, E, SS>
{
    fn next(&self, v:V)
    {
        let clone = v.clone();
        self.lock.enter();
        let old = if unsafe { &mut *self.val.get() }.is_some() {
            unsafe { &mut *self.val.get() }.replace(clone)
        }else { None };
        self.lock.exit();

        self.subj.next(v);
    }

    fn error(&self, e:E)
    {
        self.lock.enter();
        let old = if unsafe { &mut *self.val.get() }.is_some() {
            unsafe { &mut *self.val.get() }.take()
        } else { None };
        self.lock.exit();

        self.subj.error(e);
    }

    fn complete(&self)
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

        let s = BehaviorSubject::<i32, (), NO>::new(123);

        s.sub(|v| n.replace(v));
        assert_eq!(n.get(), 123);

        s.next(456);
        assert_eq!(n.get(), 456);

        s.next(789);

        s.sub(|v| x.replace(v));
        assert_eq!(x.get(), 789);
        assert_eq!(n.get(), 789);
    }
}