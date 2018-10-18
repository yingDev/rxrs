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
    fn sub(&self, next: impl ActNext<'o, NO, Val<VOut>>, ec: impl ActEc<'o, NO, EBY>) -> Unsub<'o, NO> where Self: Sized
    {
        let f = self.f.clone();
        self.src.sub(move |v:By<Ref<V>>| next.call(By::v(f(&*v))), ec)
    }

    fn sub_dyn(&self, next: Box<ActNext<'o, NO, Val<VOut>>>, ec: Box<ActEcBox<'o, NO, EBY>>) -> Unsub<'o, NO>
    { self.sub(move |v:By<_>| next.call(v), move |e: Option<By<_>>| ec.call_box(e)) }
}

impl<'o, V:'o, VOut:'o, EBY:RefOrVal+'o, Src, F> Observable<'o, NO, Val<VOut>, EBY> for MapOp<Val<V>, Src, F>
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

impl<V:'static, VOut:'static, EBY:RefOrVal+'static, Src, F> Observable<'static, YES, Val<VOut>, EBY> for MapOp<Ref<V>, Src, F>
    where F: Fn(&V)->VOut+'static+Send+Sync,
          Src: Observable<'static, YES, Ref<V>, EBY>
{
    fn sub_dyn(&self, next: Box<ActNext<'static, YES, Val<VOut>>>, ec: Box<ActEcBox<'static, YES, EBY>>) -> Unsub<'static, YES>
    {
        let (f, next, ec) = (self.f.clone(), next.into_sendsync(), ec.into_sendsync());
        self.src.sub_dyn(box move |v:By<_>| next.call(By::v(f(&*v))), ec)
    }
}

impl<V:'static, VOut:'static, EBY:RefOrVal+'static, Src, F> Observable<'static, YES, Val<VOut>, EBY> for MapOp<Val<V>, Src, F>
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

        let o: Box<Observable<NO, Val<i32>>> = o.map(|v| v+1).into_dyn();
        o.sub(|v:By<Val<i32>>| println!("v={}", *v), ());
    }
}
