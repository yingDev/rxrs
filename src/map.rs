//use std::marker::PhantomData;
//use std::sync::Arc;
//use crate::*;
//
//pub struct MapOp<V, Src, F>
//{
//    f: F,
//    src: Src,
//    PhantomData: PhantomData<V>
//}
//
//struct MapOpSubscriber<'s, O, F>
//{
//    f: &'s F,
//    o: O,
//}
//
//pub trait ObservableMapOp<V, E, VOut, F> : Sized
//    where F: Fn(V)->VOut
//{
//    fn map(self, f: F) -> MapOp<V, Self, F>
//    {
//        MapOp{ f, src: self, PhantomData}
//    }
//}
//
//impl<'s, 'o, V, E, VOut, Src, F> ObservableMapOp<V,E, VOut, F> for Src
//    where F: Fn(V)->VOut,
//          Src: Observable<'s, 'o, V, E>
//{}
//
//impl<'s, 'o, V, E, VOut, Src, F> ObservableMapOp<V,E, VOut, F> for Src
//    where F: Fn(&V)->VOut,
//          Src: ByRefObservable<'s, 'o, V, E>
//{}
//
//impl<'s, V, E, VOut, O, F> Observer<V, E> for MapOpSubscriber<'s, O, F>
//    where O: Observer<VOut, E>,
//          F: Fn(V)->VOut,
//{
//    fn next(&self, v:V) { self.o.next(self.f.call((v,))) }
//    fn error(&self, e:E){ self.o.error(e) }
//    fn complete(&self)  { self.o.complete() }
//}
//
//impl<'s: 'o, 'o, V, VOut, E, Src, F> Observable<'s, 'o, VOut, E> for MapOp<V, Src, F>
//    where F: Fn(V)->VOut,
//          Src: Observable<'s, 'o, V, E>
//{
//    fn sub(&'s self, o: impl Observer<VOut, E> + 'o) -> Unsub<'o, NO>
//    {
//        self.src.sub(MapOpSubscriber{ f: &self.f, o })
//    }
//
//}
//
