use observable::Observable;
use subref::SubRef;
use std::sync::Arc;
use observable::Observer;
use subject::Subject;
use std::marker::PhantomData;
use std::any::Any;

pub struct ConnectableObservable<'a:'b,'b, V, Subj> where Subj: Observer<V>+Observable<'a, V>+Send+Sync+'a
{
    source: Arc<Observable<'a,V>+'b+Send+Sync>,
    subject: Arc<Subj>,
}

impl<'a:'b, 'b, V, Subj> ConnectableObservable<'a,'b,V, Subj>  where Subj : Observer<V>+Observable<'a, V>+Send+Sync+'a
{
    //todo: ? allow call multi times ?
    pub fn connect(&self) -> SubRef
    {
        self.source.sub(self.subject.clone())
    }

    #[inline(always)]
    pub fn new(source: Arc<Observable<'a,V>+'b+Send+Sync>, subject: Subj) -> ConnectableObservable<'a, 'b, V, Subj>
    {
        ConnectableObservable{ source, subject: Arc::new(subject) }
    }
}

impl<'a:'b,'b, V, Subj> Observable<'a, V> for ConnectableObservable<'a, 'b, V, Subj>  where  Subj : Observer<V>+Observable<'a, V>+Send+Sync+'a
{
    fn sub(&self, dest: Arc<Observer<V> + Send + Sync+'a>) -> SubRef
    {
        self.subject.sub(dest)
    }
}
//
//#[cfg(test)]
//mod test
//{
//    use super::*;
//    use fac::*;
//    use observable::*;
//    use op::*;
//
//    #[test]
//    fn basic()
//    {
//        let src = rxfac::range(0..10);
//        let multi = ConnectableObservable::new(src.clone(), Subject::new());
//
//        multi.subf(|v| println!("{}", v), (), || println!("comp"));
//        multi.connect();
//    }
//}