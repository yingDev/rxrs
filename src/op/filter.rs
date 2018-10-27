use std::marker::PhantomData;
use std::sync::Arc;
use crate::*;

pub struct FilterOp<SS, Src, F>
{
    f: Arc<F>,
    src: Src,
    PhantomData: PhantomData<(SS)>
}

pub trait ObsFilterOp<SS: YesNo, VBy: RefOrVal, EBy: RefOrVal, F: Act<SS, Ref<VBy::RAW>, bool>> : Sized
{
    fn filter(self, f: F) -> FilterOp<SS, Self, F> { FilterOp{ f: Arc::new(f), src: self, PhantomData} }
}

impl<'o, VBy: RefOrVal, EBy: RefOrVal, Src: Observable<'o, SS, VBy, EBy>, F: Act<SS, Ref<VBy::RAW>, bool>+'o, SS:YesNo>
ObsFilterOp<SS, VBy,EBy, F>
for Src {}

impl<'o, SS:YesNo, VBy: RefOrVal+'o, EBy: RefOrVal+'o, Src: Observable<'o, SS, VBy, EBy>, F: Act<SS, Ref<VBy::RAW>, bool>+'o>
Observable<'o, SS, VBy, EBy>
for FilterOp<SS, Src, F>
{
    fn sub(&self, next: impl ActNext<'o, SS, VBy>, ec: impl ActEc<'o, SS, EBy>) -> Unsub<'o, SS> where Self: Sized
    {
        let f = self.f.clone();
        let (s1, s2) = Unsub::new().clones();

        s1.added_each(self.src.sub(
            forward_next(next, move |n,v: VBy| if f.call(v.as_ref()) { s2.if_not_done(|| n.call(v.into_v())); }, |s| s),
            ec
        ))
    }

    fn sub_dyn(&self, next: Box<ActNext<'o, SS, VBy>>, ec: Box<ActEcBox<'o, SS, EBy>>) -> Unsub<'o, SS>
    { self.sub(next, ec) }
}


#[cfg(test)]
mod test
{
    use crate::*;
    use std::cell::Cell;
    use std::sync::atomic::*;
    use std::rc::Rc;
    use std::sync::Arc;

    #[test]
    fn smoke()
    {
        let n = Cell::new(0);
        let (input, output) = Rc::new(Subject::<NO, i32>::new()).clones();

        output.filter(|v:&_| v % 2 == 0).sub(|v:&_| { n.replace(n.get() + *v); }, ());

        for i in 0..10 {
            input.next(i);
        }

        assert_eq!(n.get(), 20);

        let (n, n1) = Arc::new(AtomicUsize::new(0)).clones();
        let (input, output) = Rc::new(Subject::<YES, i32>::new()).clones();
        output.filter(|v:&_| v % 2 == 0).sub(move |v:&_| { n.fetch_add(1, Ordering::SeqCst); }, ());

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
        }).sub(|v:&_| { n.replace(n.get() + *v); }, ());

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

        output.filter(move |_:&_| true).sub(
            move |v:&_| {  n1.replace(n1.get() + *v); },
            move |_:Option<&_>| {  n2.replace(n2.get() + 1);  });

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
            filtered.sub(|i:&_| println!("ok{}",i), ());
        }).join().ok();
    }
}
