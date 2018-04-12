use subject::*;
use std::sync::Mutex;
use observable::Observable;
use std::sync::Arc;
use subref::SubRef;
use observable::Observer;
use std::any::Any;
use util::mss::Yes;
use util::mss::Mss;
use observable::ArcErr;

pub struct BehaviorSubject<'a, V:Clone+'static>
{
    v: Mutex<Option<V>>,
    subj: Subject<'a, V>
}

impl<'a, V:Clone+'static> BehaviorSubject<'a, V>
{
    pub fn new(value: Option<V>) -> BehaviorSubject<'a, V>
    {
        BehaviorSubject{ subj: Subject::new(), v: Mutex::new(value)}
    }

    pub fn value(&self) -> Option<V>
    {
        let guard = self.v.lock().unwrap();
        guard.as_ref().map(|v| v.clone())
    }

    pub fn clear_value(&self)
    {
        let mut guard = self.v.lock().unwrap();
        *guard = None;
    }
}

impl<'a, V:Clone+'static> Observable<'a, V, Yes, Yes> for BehaviorSubject<'a, V>
{
    #[inline(always)]
    fn sub(&self, o: Mss<Yes,impl Observer<V>+'a>) -> SubRef<Yes>
    {
        if o._is_closed() {
            return SubRef::empty();
        }

        {
            let guard = self.v.lock().unwrap();
            if let Some(ref val) = *guard { o.next(val.clone()); }
        }

        if o._is_closed() || self._is_closed() {
            return SubRef::empty();
        }

        self.subj.sub(o)
    }
}

impl<'a, V:Clone+'static> Observer<V> for BehaviorSubject<'a, V>
{
    fn next(&self, v: V)
    {
        if self._is_closed() { return; }

        {
            let mut guard = self.v.lock().unwrap();
            *guard = Some(v.clone());
        }

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

        s.err(Arc::new(box "error"));
        assert!(s.value().is_none());

        s.next(2);
        assert!(s.value().is_none());
    }
}