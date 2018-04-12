use std::marker::PhantomData;
use std::any::Any;
use std::rc::Rc;
use observable::*;
use subref::SubRef;
use std::sync::Arc;
use connectable_observable::*;
use util::mss::*;

pub trait ObservableMulticast<'a,  V, Src, Subj, SSO:?Sized, SSS:?Sized>
{
    fn multicast(self, subject: Subj) -> ConnectableObservable<'a, V, Src, Subj, SSO, SSS>;
}

impl<'a, V, Src, Subj, SSS:?Sized> ObservableMulticast<'a, V, Src, Subj, Yes, SSS> for Src where  Src : Observable<'a, V, Yes, SSS>, Subj : Observer<V>+Observable<'a, V, Yes, SSS>+Send+Sync+'a, V:Clone
{
    #[inline(always)]
    fn multicast(self, subject: Subj) -> ConnectableObservable<'a, V, Src, Subj, Yes, SSS>
    {
        ConnectableObservable::<'a,V,Src,Subj, Yes,SSS>::new(self, subject)
    }
}

impl<'a, V, Src, Subj, SSS:?Sized> ObservableMulticast<'a, V, Src, Subj, No, SSS> for Src where  Src : Observable<'a, V, No, SSS>, Subj : Observer<V>+Observable<'a, V, No, SSS>+'a, V:Clone
{
    #[inline(always)]
    fn multicast(self, subject: Subj) -> ConnectableObservable<'a, V, Src, Subj, No, SSS>
    {
        ConnectableObservable::<'a,V,Src,Subj, No,SSS>::new(self, subject)
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use test_fixture::*;
    use std::cell::Cell;

    #[test]
    fn basic()
    {
        use subject_nss::*;

        let mut out = 0;
        let src = SimpleObservable;

        {
            let con = src.multicast(Subject::new());
            con.subf(|v| out += v);

            //assert_eq!(out, 0);
            con.connect();
        }

        assert_eq!(out, 6);
    }

    #[test]
    fn threaded()
    {
        use subject::*;
        use std::sync::atomic::{ AtomicIsize, Ordering };
        use std::{ thread, time::Duration };

        let mut out = Arc::new(AtomicIsize::new(0));
        let src = ThreadedObservable;

        let con = src.multicast(Subject::new());
        con.subf(byclone!(out => move |v| out.fetch_add(v as isize,Ordering::SeqCst)));

        assert_eq!(out.load(Ordering::SeqCst), 0);
        thread::spawn(move ||{
            con.connect();
        });
        thread::sleep(Duration::from_millis(100));
        assert_eq!(out.load(Ordering::SeqCst), 6);
    }

}