use observable::Observable;
use unsub_ref::UnsubRef;
use std::sync::Arc;
use observable::Observer;
use subject::Subject;
use std::marker::PhantomData;
use subscriber::SubscriberImpl;
use subscriber::Subscriber;
use std::any::Any;

pub struct ConnectableObservable<V, Src, Subj> where Src: Observable<V>+Send+Sync, Subj : Observer<V>+Observable<V>+Send+Sync+'static
{
    source: Src,
    subject: Arc<Subj>,

    PhantomData: PhantomData<V>
}

impl<V, Src, Subj> ConnectableObservable<V, Src, Subj>  where Src: Observable<V>+Send+Sync, Subj : Observer<V>+Observable<V>+Send+Sync+'static
{
    pub fn connect(&self) -> UnsubRef<'static>
    {
        self.source.sub(self.subject.clone())
    }

    pub fn new(source: Src, subject: Subj) -> ConnectableObservable<V, Src, Subj>
    {
        ConnectableObservable{ source, subject: Arc::new(subject), PhantomData }
    }
}

impl<V, Src, Subj> Observable<V> for ConnectableObservable<V, Src, Subj>  where Src: Observable<V>+Send+Sync, Subj : Observer<V>+Observable<V>+Send+Sync+'static
{
    fn sub(&self, dest: Arc<Observer<V> + Send + Sync>) -> UnsubRef<'static>
    {
        self.subject.sub(dest)
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use fac::*;
    use observable::*;
    use op::*;

    #[test]
    fn basic()
    {
        let src = rxfac::range(0..10);
        let multi = ConnectableObservable::new(src.clone(), Subject::new());

        multi.subf(|v| println!("{}", v), (), || println!("comp"));
        multi.connect();
    }
}