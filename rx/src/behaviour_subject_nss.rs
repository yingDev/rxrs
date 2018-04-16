use subject_nss::*;
use std::cell::RefCell;
use observable::Observable;
use std::sync::Arc;
use subref::SubRef;
use observable::Observer;
use std::any::Any;
use util::mss::No;
use util::mss::Mss;
use observable::ArcErr;

pub struct BehaviorSubject<'a, V:Clone+'a>
{
    v: RefCell<Option<V>>,
    subj: Subject<'a, V>
}

impl<'a, V:Clone+'a> BehaviorSubject<'a, V>
{
    pub fn new(value: impl Into<Option<V>>) -> BehaviorSubject<'a, V>
    {
        BehaviorSubject{ subj: Subject::new(), v: RefCell::new(value.into())}
    }

    pub fn value(&self) -> Option<V>
    {
        self.v.borrow().clone()
    }

    pub fn clear_value(&self)
    {
        self.v.replace(None);
    }
}

impl<'a, V:Clone+'a> Observable<'a, V> for BehaviorSubject<'a, V>
{
    #[inline(always)]
    fn sub(&self, o: Mss<No,impl Observer<V>+'a>) -> SubRef<No>
    {
        if o._is_closed() {
            return SubRef::empty();
        }

        {
            let val = self.v.borrow();
            if let Some(ref val) = *val { o.next(val.clone()); }
        }

        if o._is_closed() || self._is_closed() {
            return SubRef::empty();
        }

        self.subj.sub(o)
    }
}

impl<'a, V:Clone+'a> Observer<V> for BehaviorSubject<'a, V>
{
    fn next(&self, v: V)
    {
        if self._is_closed() { return; }

        self.v.replace(Some(v.clone()));
        self.subj.next(v);
    }

    fn err(&self, e: ArcErr)
    {
        self.clear_value();
        self.subj.err(e);
    }

    fn complete(&self)
    {
        self.subj.complete();
    }

    fn _is_closed(&self) -> bool { self.subj._is_closed() }
}


#[cfg(test)]
mod test
{
    use super::*;

    #[test]
    fn value()
    {
        let s = BehaviorSubject::<i32>::new(None);
        assert!(s.value().is_none());

        s.next(1);
        assert_eq!(s.value().unwrap(), 1);

        //s.err(Arc::new(box "error"));
        //assert!(s.value().is_none());

        s.next(2);
        assert!(s.value().is_none());
    }
}