use std::marker::PhantomData;
use std::any::Any;
use std::rc::Rc;
use observable::*;
use subref::SubRef;
use std::sync::Arc;
use connectable_observable::*;
use subject::Subject;
use subject_nss::Subject as SubjectNss;
use util::mss::*;

pub trait ObservablePublish<'a, V, Src, Subj, SSO:?Sized, SSS:?Sized>
{
    fn publish(self) -> ConnectableObservable<'a, V, Src, Subj, SSO, SSS>;
}

impl<'a, V, Src> ObservablePublish<'a, V, Src, Subject<'a,V>, Yes, Yes> for Src where  Src : Observable<'a, V, Yes, Yes>, V:Clone
{
    #[inline(always)]
    fn publish(self) -> ConnectableObservable<'a, V, Src, Subject<'a, V>, Yes, Yes>
    {
        ConnectableObservable::<'a, V, Src, Subject<'a, V>, Yes, Yes>::new(self, Subject::new())
    }
}


impl<'a, V, Src> ObservablePublish<'a, V, Src, SubjectNss<'a,V>, No, No> for Src where  Src : Observable<'a, V, No, No>, V:Clone
{
    #[inline(always)]
    fn publish(self) -> ConnectableObservable<'a, V, Src, SubjectNss<'a, V>, No, No>
    {
        ConnectableObservable::<'a, V, Src, SubjectNss<'a, V>, No, No>::new(self, SubjectNss::new())
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use test_fixture::*;
    use ::std::sync::atomic::*;

    #[test]
    fn basic()
    {
        let mut out = 0;
        let src = SimpleObservable;
        {
            let con = src.publish();
            con.subf(|v| out += v);
            con.connect();
        }

        assert_eq!(out, 6);
    }

}