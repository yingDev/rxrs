use std::marker::PhantomData;
use std::sync::Arc;
use crate::*;

pub struct MapOp<VBY: RefOrVal, Src, F>
{
    f: Arc<F>,
    src: Src,
    PhantomData: PhantomData<VBY>
}

pub trait ObservableMapOp<V, VBY: RefOrVal, EBY: RefOrVal, VOut, F> : Sized
    where F: Fn(&V)->VOut
{
    fn map(self, f: F) -> MapOp<VBY, Self, F>
    { MapOp{ f: Arc::new(f), src: self, PhantomData} }
}

impl<'o, V, VBY: RefOrVal, EBY: RefOrVal, VOut, Src, F> ObservableMapOp<V, VBY,EBY, VOut, F> for Src
    where F: Fn(&V)->VOut+'o,
          Src: Observable<'o, NO, VBY, EBY>
{}


impl<'o, V:'o, VOut:'o, EBY:RefOrVal+'o, Src, F> Observable<'o, NO, Val<VOut>, EBY> for MapOp<Ref<V>, Src, F>
    where F: Fn(&V)->VOut+'o,
          Src: Observable<'o, NO, Ref<V>, EBY>
{
    fn sub(&self, next: impl for<'x> Act<NO, By<'x, Val<VOut>>>+'o, ec: impl for<'x> ActOnce<NO, Option<By<'x, EBY>>>+'o) -> Unsub<'o, NO> where Self: Sized
    {
        let f = self.f.clone();
        self.src.sub(move |v:By<Ref<V>>| next.call(By::v(f(&*v))), move |e: Option<By<_>>| ec.call_once(e))
    }

    fn sub_dyn(&self, next: Box<for<'x> Act<NO, By<'x, Val<VOut>>>+'o>, ec: Box<for<'x> ActBox<NO, Option<By<'x, EBY>>> +'o>) -> Unsub<'o, NO>
    { self.sub(move |v:By<_>| next.call(v), move |e: Option<By<_>>| ec.call_box(e)) }
}

impl<'o, V:'o, VOut:'o, EBY:RefOrVal+'o, Src, F> Observable<'o, NO, Val<VOut>, EBY> for MapOp<Val<V>, Src, F>
    where F: Fn(&V)->VOut+'o,
          Src: Observable<'o, NO, Val<V>, EBY>
{
    fn sub(&self, next: impl for<'x> Act<NO, By<'x, Val<VOut>>>+'o, ec: impl for<'x> ActOnce<NO, Option<By<'x, EBY>>>+'o) -> Unsub<'o, NO> where Self: Sized
    {
        let f = self.f.clone();
        self.src.sub(move |v:By<Val<V>>| next.call(By::v(f(&*v))), move |e: Option<By<_>>| ec.call_once(e))
    }

    fn sub_dyn(&self, next: Box<for<'x> Act<NO, By<'x, Val<VOut>>>+'o>, ec: Box<for<'x> ActBox<NO, Option<By<'x, EBY>>> +'o>) -> Unsub<'o, NO>
    { self.sub(move |v:By<_>| next.call(v), move |e: Option<By<_>>| ec.call_box(e)) }
}

#[cfg(test)]
mod test
{
    use crate::*;
    use std::cell::RefCell;
    use std::cell::Cell;
    use std::rc::Rc;

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
        let unsub = o.map(|v| v+1).sub(|v:By<_>| { n.replace(*v); }, ());

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

        o.map(|v| v+1).sub(|v:By<_>| println!("v={}", *v), ());
    }
}
