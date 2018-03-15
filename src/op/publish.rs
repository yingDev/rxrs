use std::marker::PhantomData;
use std::any::Any;
use std::rc::Rc;
use observable::*;
use subref::SubRef;
use std::sync::Arc;
use connectable_observable::*;
use subject::Subject;

pub trait ObservablePublish<'a, V, Src>  where  Src : Observable<'a, V>, V:Clone
{
    fn publish(self) -> ConnectableObservable<'a, V, Src, Subject<'a, V>>;
}

impl<'a, V, Src> ObservablePublish<'a, V, Src> for Src where  Src : Observable<'a, V>, V:Clone
{
    #[inline(always)]
    fn publish(self) -> ConnectableObservable<'a, V, Src, Subject<'a, V>>
    {
        ConnectableObservable::new(self, Subject::new())
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use fac::*;
    use ::std::sync::atomic::*;

    #[test]
    fn basic()
    {
        let result = AtomicIsize::new(0);

        let src = rxfac::range(0..10).publish();
        let sub = src.rx().subf(|v| { result.fetch_add(1, Ordering::SeqCst); } );

        assert_eq!(result.load(Ordering::SeqCst), 0);
        src.connect();
        assert_eq!(result.load(Ordering::SeqCst), 10);
    }

}