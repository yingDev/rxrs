use std::marker::PhantomData;
use std::any::Any;
use std::rc::Rc;
use subscriber::*;
use observable::*;
use unsub_ref::UnsubRef;
use std::sync::Arc;
use connectable_observable::*;

pub trait ObservableMulticast<'a, V, Src, Subj>  where  Src : Observable<'a, V>, Subj : Observer<V>+Observable<'a, V>+Send+Sync+'static,V:Clone
{
    fn multicast(self, subject: Subj) -> ConnectableObservable<'a, V, Src, Subj>;
}

impl<'a, V, Src, Subj> ObservableMulticast<'a, V, Src, Subj> for Src where  Src : Observable<'a, V>, Subj : Observer<V>+Observable<'a, V>+Send+Sync+'static, V:Clone
{
    fn multicast(self, subject: Subj) -> ConnectableObservable<'a, V, Src, Subj>
    {
        ConnectableObservable::new(self, subject)
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use subject::*;
    use ::std::sync::atomic::*;

    #[test]
    fn basic()
    {
    }

}