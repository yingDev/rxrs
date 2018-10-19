use std::marker::PhantomData;
use std::sync::Arc;
use crate::*;
use crate::util::alias::SSs;

pub struct MapOp<SS, VBy, Src, F>
{
    f: Arc<F>,
    src: Src,
    PhantomData: PhantomData<(SS, VBy)>
}

pub trait ObservableMapOp<SS, VBy, EBy, VOut, F: Fn(By<VBy>)->VOut> : Sized
{
    fn map(self, f: F) -> MapOp<SS, VBy, Self, F> { MapOp{ f: Arc::new(f), src: self, PhantomData} }
}

impl<'o, VBy: RefOrVal, EBy: RefOrVal, VOut, Src, F, SS:YesNo> ObservableMapOp<SS, VBy,EBy, VOut, F> for Src
    where F: Fn(By<VBy>)->VOut+'o,
          Src: Observable<'o, SS, VBy, EBy>
{}


impl<'o, VOut:'o, VBy: RefOrVal+'o, EBy:RefOrVal+'o, Src, F> Observable<'o, NO, Val<VOut>, EBy> for MapOp<NO, VBy, Src, F>
    where F: Fn(By<VBy>)->VOut+'o,
          Src: Observable<'o, NO, VBy, EBy>
{
    fn sub(&self, next: impl ActNext<'o, NO, Val<VOut>>, ec: impl ActEc<'o, NO, EBy>) -> Unsub<'o, NO> where Self: Sized
    {
        let f = self.f.clone();
        let (s1, s2, s3) = Unsub::new().clones();

        s1.added_each(self.src.sub(
            move |v:By<_>| {
                let v = f(v);
                if !s2.is_done() { next.call(By::v(v)); }
            } ,
            move |e: Option<By<_>>| s3.unsub_then(|| ec.call_once(e))
        ))
    }

    fn sub_dyn(&self, next: Box<ActNext<'o, NO, Val<VOut>>>, ec: Box<ActEcBox<'o, NO, EBy>>) -> Unsub<'o, NO>
    {
        self.sub(move |v:By<_>| next.call(v), move |e: Option<By<_>>| ec.call_box(e))
    }
}

impl<VOut:SSs, VBy: RefOrValSSs, EBy: RefOrValSSs, Src, F> Observable<'static, YES, Val<VOut>, EBy> for MapOp<YES, VBy, Src, F>
    where F: Fn(By<VBy>)->VOut+'static+Send+Sync,
          Src: Observable<'static, YES, VBy, EBy>
{
    fn sub(&self, next: impl ActNext<'static, YES, Val<VOut>>, ec: impl ActEc<'static, YES, EBy>) -> Unsub<'static, YES> where Self: Sized
    {
        let (f, next, ec) = (self.f.clone(), ActSendSync::wrap_next(next), ActSendSync::wrap_ec(ec));
        let (s1, s2, s3) = Unsub::new().clones();

        s1.added_each(self.src.sub(
            move |v:By<_>| {
                let v = f(v);
                s2.if_not_done(|| next.call(By::v(v)));
            },
            move |e: Option<By<_>>| s3.unsub_then(|| ec.into_inner().call_once(e))
        ))
    }

    fn sub_dyn(&self, next: Box<ActNext<'static, YES, Val<VOut>>>, ec: Box<ActEcBox<'static, YES, EBy>>) -> Unsub<'static, YES>
    {
        let (next, ec) = (next.into_ss(), ec.into_ss());
        self.sub(move |v:By<_>| next.call(v), move |e: Option<By<_>>| ec.call_box(e))
    }
}


#[cfg(test)]
mod test
{
    use crate::*;
    use std::cell::RefCell;
    use std::cell::Cell;
    use std::rc::Rc;
    use std::sync::Arc;
    use std::sync::atomic::*;

    #[test]
    fn smoke()
    {
        let n = Cell::new(0);
        let o = Of::value(123);
        o.map(|v: By<Ref<i32>>| *v * 2).sub(|v:By<_>| { n.replace(*v);}, ());
        assert_eq!(n.get(), 246);

        let o = Of::value("B".to_owned());
        let mapped = o.map(|s| format!("A{}", *s)).map(|s| format!("{}C", *s)).into_dyn();

        let result = RefCell::new(String::new());
        mapped.sub_dyn(box |v:By<Val<String>>| result.borrow_mut().push_str(&*v), box());

        assert_eq!(result.borrow().as_str(), "ABC");
    }

    #[test]
    fn unsub()
    {
        let n = Cell::new(0);
        let (i,o) = Rc::new(Subject::<NO, i32>::new()).clones();
        let unsub = o.map(|v| *v+1).sub(|v:By<_>| { n.replace(*v); }, ());

        i.next(1);
        assert_eq!(n.get(), 2);

        unsub();
        i.next(2);
        assert_eq!(n.get(), 2);
    }

    #[test]
    fn boxed()
    {
        let o: Box<Observable<NO, Ref<i32>>> = Of::value_dyn(123);

        let o: Box<Observable<NO, Val<i32>>> = o.map(|v| *v+1).into_dyn();
        o.sub(|v:By<_>| println!("v={}", *v), ());
    }

    #[test]
    fn thread()
    {
        let (n, n1) = Arc::new(AtomicI32::new(0)).clones();
        let (i, o) = Arc::new(Subject::<YES, i32>::new()).clones();

        o.sub(|_: By<_>|{}, ());

        o.map(|v| *v+1).sub(move |v: By<Val<i32>>| {n.store(*v, Ordering::SeqCst); }, ());

        ::std::thread::spawn(move ||{
            i.next(123);
        }).join().ok();

        assert_eq!(n1.load(Ordering::SeqCst), 124);
    }

    #[test]
    fn drops_closure()
    {
        let (r, r1) = Rc::new(0).clones();

        assert_eq!(Rc::strong_count(&r), 2);

        let o = Of::value(123);

        o.map(move |_| Rc::strong_count(&r1)).sub((), ());

        assert_eq!(Rc::strong_count(&r), 1);
    }
}
