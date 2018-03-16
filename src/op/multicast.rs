use std::marker::PhantomData;
use std::any::Any;
use std::rc::Rc;
use observable::*;
use subref::SubRef;
use std::sync::Arc;
use connectable_observable::*;

pub trait ObservableMulticast<'a, 'b, V, Subj> where Subj : Observer<V>+Observable<'a, V>+Send+Sync+'a, V:Clone
{
    fn multicast(self, subject: Subj) -> ConnectableObservable<'a, 'b, V, Subj>;
}

impl<'a:'b, 'b, V, Subj> ObservableMulticast<'a, 'b, V, Subj> for Arc<Observable<'a, V>+'b+Send+Sync> where Subj : Observer<V>+Observable<'a, V>+Send+Sync+'a, V:Clone
{
    fn multicast(self, subject: Subj) -> ConnectableObservable<'a, 'b, V, Subj>
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