//use std::marker::PhantomData;
//use crate::*;
//
//pub struct Of<V>(V);
//
//pub fn of<V>(v:V) -> Of<V> { Of(v) }
//
//impl<'s, 'o, V> Observable<'s, 'o, &'s V, ()> for Of<V>
//{
//    fn sub(&'s self, o: impl Observer<&'s V, ()>+'o) -> Unsub<'o, NO>
//    {
//        o.next(&self.0);
//        o.complete();
//        Unsub{ PhantomData }
//    }
//}