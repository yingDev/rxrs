use std::marker::PhantomData;
use std::sync::Arc;
use crate::*;
use std::error::Error;

pub struct FilterOp<SS, Src, F>
{
    f: Arc<F>,
    src: Src,
    PhantomData: PhantomData<(SS)>
}

pub trait ObsFilterOp<SS: YesNo, VBy: RefOrVal, F: Act<SS, Ref<VBy::RAW>, bool>> : Sized
{
    fn filter(self, f: F) -> FilterOp<SS, Self, F> { FilterOp{ f: Arc::new(f), src: self, PhantomData} }
}

impl<'o, SS:YesNo, VBy: RefOrVal, Src: Observable<'o, SS, VBy>, F: Act<SS, Ref<VBy::RAW>, bool>+'o>
ObsFilterOp<SS, VBy, F>
for Src {}

pub trait DynObsFilterOp<'o, SS: YesNo, VBy: RefOrVal+'o, F: Act<SS, Ref<VBy::RAW>, bool>+'o>
{
    fn filter(self, f: F) -> Self;
}

impl<'o, SS:YesNo, VBy: RefOrVal+'o, F: Act<SS, Ref<VBy::RAW>, bool>+'o>
DynObsFilterOp<'o, SS, VBy, F>
for DynObservable<'o, 'o, SS, VBy>
{
    fn filter(self, f: F) -> Self
    { FilterOp{ f: Arc::new(f), src: self.src, PhantomData }.into_dyn() }
}

impl<'o, SS:YesNo, VBy: RefOrVal+'o, Src: Observable<'o, SS, VBy>, F: Act<SS, Ref<VBy::RAW>, bool>+'o>
Observable<'o, SS, VBy>
for FilterOp<SS, Src, F>
{
    fn subscribe(&self, next: impl ActNext<'o, SS, VBy>, ec: impl ActEc<'o, SS>) -> Unsub<'o, SS> where Self: Sized
    {
        let next = SSActNextWrap::new(next);
        let f = act_sendsync(self.f.clone());
        let sub = Unsub::<SS>::new();

        sub.clone().added_each(self.src.subscribe(
            forward_next(next, (f, sub), |n, (f, sub), v: VBy| {
                if f.call(v.as_ref()) {
                    sub.if_not_done(|| n.call(v.into_v()));
                }
            }, |s, _| s.stopped()),
            ec
        ))
    }

    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, VBy>>, ec: Box<ActEcBox<'o, SS>>) -> Unsub<'o, SS>
    { self.subscribe(next, ec) }
}


#[cfg(test)]
mod test
{
    use crate::*;
    use crate::util::clones::*;

    use std::cell::Cell;
    use std::sync::atomic::*;
    use std::rc::Rc;
    use std::sync::Arc;

    #[test]
    fn smoke()
    {
        let n = Cell::new(0);
        let (input, output) = Rc::new(Subject::<NO, i32>::new()).clones();

        output.filter(|v:&_| v % 2 == 0).subscribe(|v:&_| { n.replace(n.get() + *v); }, ());

        for i in 0..10 {
            input.next(i);
        }

        assert_eq!(n.get(), 20);

        let (n, n1) = Arc::new(AtomicUsize::new(0)).clones();
        let (input, output) = Rc::new(Subject::<YES, i32>::new()).clones();
        output.into_dyn().filter(|v:&_| v % 2 == 0).subscribe_dyn(box move |_v:&_| { n.fetch_add(1, Ordering::SeqCst); }, box());

        input.next(1);
        input.next(2);
        input.next(3);

        assert_eq!(n1.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn cb_safe()
    {
        let n = Cell::new(0);
        let (input, output, side_effect) = Rc::new(Subject::<NO, i32>::new()).clones();

        output.filter(move |v:&_| {
            side_effect.complete();
            v % 2 == 0
        }).subscribe(|v:&_| { n.replace(n.get() + *v); }, ());

        for i in 0..10 {
            input.next(i);
        }

        assert_eq!(n.get(), 0);
    }

    #[test]
    fn should_complete()
    {
        let (n1, n2, n3) = Rc::new(Cell::new(0)).clones();
        let (input, output) = Rc::new(Subject::<NO, i32>::new()).clones();

        output.filter(move |_:&_| true).subscribe(
            move |v:&_| {  n1.replace(n1.get() + *v); },
            move |_| {  n2.replace(n2.get() + 1);  });

        input.next(1);
        input.next(2);

        assert_eq!(n3.get(), 3);

        input.complete();

        assert_eq!(n3.get(), 4);
    }

    #[test]
    fn thread()
    {
        let s = Subject::<YES, i32>::new();
        let filtered = s.filter(|i:&_| i % 2 == 0 );

        ::std::thread::spawn(move ||{
            filtered.subscribe(|i:&_| println!("ok{}",i), ());
        }).join().ok();
    }
}
