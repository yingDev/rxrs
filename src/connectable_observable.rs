use observable::Observable;
use subref::SubRef;
use std::sync::Arc;
use observable::Observer;
use subject::Subject;
use std::marker::PhantomData;
use std::any::Any;
use observable::Yes;
use observable::Mss;

pub struct ConnectableObservable<'a, V, Src, Subj> where Src: Observable<'a, V, Yes>, Subj : Observer<V>+Observable<'a, V, Yes>+Send+Sync+'a
{
    source: Src,
    subject: Arc<Subj>,

    PhantomData: PhantomData<(V,&'a ())>
}

impl<'a, V, Src, Subj> ConnectableObservable<'a, V, Src, Subj>  where Src: Observable<'a, V, Yes>, Subj : Observer<V>+Observable<'a, V, Yes>+Send+Sync+'a
{
    pub fn connect(&self) -> SubRef
    {
        self.source.sub(self.subject.clone())
    }

    #[inline(always)]
    pub fn new(source: Src, subject: Subj) -> ConnectableObservable<'a, V, Src, Subj>
    {
        ConnectableObservable{ source, subject: Arc::new(subject), PhantomData }
    }
}

impl<'a, V, Src, Subj> Observable<'a, V, Yes> for ConnectableObservable<'a, V, Src, Subj>  where Src: Observable<'a, V, Yes>+Send+Sync, Subj : Observer<V>+Observable<'a, V, Yes>+Send+Sync+'a
{
    #[inline(always)]
    fn sub(&self, dest: Mss<Yes, impl Observer<V> +'a>) -> SubRef
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