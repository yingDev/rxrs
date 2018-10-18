use std::marker::PhantomData;
use std::sync::Arc;
use crate::*;

pub struct MapOp<SS:YesNo, VBY: RefOrVal, Src, F>
{
    f: Arc<F>,
    src: Src,
    PhantomData: PhantomData<(SS, VBY)>
}

pub trait ObservableMapOp<SS:YesNo, V, VBY: RefOrVal, EBY: RefOrVal, VOut, F: Fn(&V)->VOut> : Sized
{
    fn map(self, f: F) -> MapOp<SS, VBY, Self, F>
    {
        MapOp{ f: Arc::new(f), src: self, PhantomData}
    }
}

impl<'o, V, VBY: RefOrVal, EBY: RefOrVal, VOut, Src, F> ObservableMapOp<NO, V, VBY,EBY, VOut, F> for Src
    where F: Fn(&V)->VOut+'o,
          Src: Observable<'o, NO, VBY, EBY>
{}

impl<V, VBY: RefOrVal, EBY: RefOrVal, VOut, Src, F> ObservableMapOp<YES, V, VBY,EBY, VOut, F> for Src
    where F: Fn(&V)->VOut+Send+Sync+'static,
          Src: Observable<'static, YES, VBY, EBY>
{}

impl<'o, V:'o, VOut:'o, EBY:RefOrVal+'o, Src, F> Observable<'o, NO, Val<VOut>, EBY> for MapOp<NO, Ref<V>, Src, F>
    where F: Fn(&V)->VOut+'o,
          Src: Observable<'o, NO, Ref<V>, EBY>
{
    fn sub(&self, next: impl ActNext<'o, NO, Val<VOut>>, ec: impl ActEc<'o, NO, EBY>) -> Unsub<'o, NO> where Self: Sized
    {
        let f = self.f.clone();
        self.src.sub(move |v:By<Ref<V>>| next.call(By::v(f(&*v))), ec)
    }

    fn sub_dyn(&self, next: Box<ActNext<'o, NO, Val<VOut>>>, ec: Box<ActEcBox<'o, NO, EBY>>) -> Unsub<'o, NO>
    { self.sub(move |v:By<_>| next.call(v), move |e: Option<By<_>>| ec.call_box(e)) }
}

impl<'o, V:'o, VOut:'o, EBY:RefOrVal+'o, Src, F> Observable<'o, NO, Val<VOut>, EBY> for MapOp<NO, Val<V>, Src, F>
    where F: Fn(&V)->VOut+'o,
          Src: Observable<'o, NO, Val<V>, EBY>
{
    fn sub(&self, next: impl ActNext<'o, NO, Val<VOut>>, ec: impl ActEc<'o, NO, EBY>) -> Unsub<'o, NO> where Self: Sized
    {
        let f = self.f.clone();
        self.src.sub(move |v:By<Val<V>>| next.call(By::v(f(&*v))), ec)
    }

    fn sub_dyn(&self, next: Box<ActNext<'o, NO, Val<VOut>>>, ec: Box<ActEcBox<'o, NO, EBY>>) -> Unsub<'o, NO>
    { self.sub(move |v:By<_>| next.call(v), move |e: Option<By<_>>| ec.call_box(e)) }
}

//=====

impl<V:Send+Sync+'static, VOut:Send+Sync+'static, EBY:RefOrVal+Send+Sync+'static, Src, F> Observable<'static, YES, Val<VOut>, EBY> for MapOp<YES, Ref<V>, Src, F>
    where F: Fn(&V)->VOut+'static+Send+Sync,
          Src: Observable<'static, YES, Ref<V>, EBY>
{
    fn sub_dyn(&self, next: Box<ActNext<'static, YES, Val<VOut>>>, ec: Box<ActEcBox<'static, YES, EBY>>) -> Unsub<'static, YES>
    {
        let (f, next, ec) = (self.f.clone(), next.into_sendsync(), ec.into_sendsync());
        self.src.sub_dyn(box move |v:By<_>| next.call(By::v(f(&*v))), ec)
    }
}

impl<V:Send+Sync+'static, VOut:Send+Sync+'static, EBY:RefOrVal+Send+Sync+'static, Src, F> Observable<'static, YES, Val<VOut>, EBY> for MapOp<YES, Val<V>, Src, F>
    where F: Fn(&V)->VOut+'static+Send+Sync,
          Src: Observable<'static, YES, Val<V>, EBY>
{
    fn sub_dyn(&self, next: Box<ActNext<'static, YES, Val<VOut>>>, ec: Box<ActEcBox<'static, YES, EBY>>) -> Unsub<'static, YES>
    {
        let (f, next, ec) = (self.f.clone(), next.into_sendsync(), ec.into_sendsync());
        self.src.sub_dyn(box move |v:By<_>| next.call(By::v(f(&*v))), ec)
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
        o.map(|v| v*2).sub(|v:By<_>| { n.replace(*v);}, ());
        assert_eq!(n.get(), 246);

        let o = Of::value("B".to_owned());
        let mapped = o.map(|s| format!("A{}", s)).map(|s| format!("{}C", s)).into_dyn();

        let result = RefCell::new(String::new());
        mapped.sub_dyn(box |v:By<Val<String>>| result.borrow_mut().push_str(&*v), box());

        assert_eq!(result.borrow().as_str(), "ABC");
    }

    #[test]
    fn unsub()
    {
        let n = Cell::new(0);
        let (i,o) = Rc::new(Subject::<NO, i32>::new()).clones();
        let unsub = o.map(|v:&i32| v+1).sub(|v:By<_>| { n.replace(*v); }, ());

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

        let o: Box<Observable<NO, Val<i32>>> = o.map(|v| v+1).into_dyn();
        o.sub(|v:By<_>| println!("v={}", *v), ());
    }

    #[test]
    fn thread()
    {
        let (n, n1) = Arc::new(AtomicI32::new(0)).clones();
        let (i, o) = Arc::new(Subject::<YES, i32>::new()).clones();

        o.sub(|v: By<_>|{}, ());

        o.map(|v| v+1).sub(move |v: By<Val<i32>>| { n.store(*v, Ordering::SeqCst); }, ());

        ::std::thread::spawn(move ||{
            i.next(123);
        }).join();

        assert_eq!(n1.load(Ordering::SeqCst), 124);
    }
}
