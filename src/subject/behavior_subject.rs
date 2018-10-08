//use std::rc::Rc;
//use std::sync::Arc;
//use std::cell::UnsafeCell;
//use crate::*;
//use crate::{util::trait_alias::CSS};
//
//pub struct BehaviorSubject<'o, V:Clone+'o, E:Clone+'o, SS:YesNo>
//{
//    subj: Subject<'o, V, E, SS>
//}
//
//impl<'o, V:Clone+'o, E:Clone+'o, SS:YesNo> BehaviorSubject<'o, V, E, SS>
//{
//    fn new(value: V) -> BehaviorSubject<'o, V, E, SS>
//    {
//        BehaviorSubject{ subj: Subject::behavior(value) }
//    }
//}
//
//impl<'o, V:Clone+'o, E:Clone+'o> Observable<'o, V, E> for  BehaviorSubject<'o, V, E, NO>
//{
//    #[inline(always)] fn sub(&self, o: impl Observer<V, E> + 'o) -> Unsub<'o, NO> { self.subj.sub(o) }
//}
//
//impl<V:CSS, E:CSS> ObservableSendSync<V, E> for  BehaviorSubject<'static, V, E, YES>
//{
//    #[inline(always)] fn sub(&self, o: impl Observer<V, E> + Send + Sync+ 'static) -> Unsub<'static, YES> { self.subj.sub(o) }
//}
//
//impl<'o, V:Clone+'o, E:Clone+'o, SS:YesNo> Observer<V, E> for BehaviorSubject<'o, V, E, SS>
//{
//    #[inline(always)] fn next(&self, v:V) { self.subj.next(v); }
//    #[inline(always)] fn error(&self, e:E) { self.subj.error(e); }
//    #[inline(always)] fn complete(&self) { self.subj.complete(); }
//}
//
//
//#[cfg(test)]
//mod test
//{
//
//    use std::cell::Cell;
//    use std::sync::Arc;
//    use crate::*;
//
//    #[test]
//    fn shoudl_emit_on_sub()
//    {
//        let n = Cell::new(0);
//        let x = Cell::new(0);
//
//        let s = BehaviorSubject::<i32, (), NO>::new(123);
//
//        s.sub(|v| n.replace(v));
//        assert_eq!(n.get(), 123);
//
//        s.next(456);
//        assert_eq!(n.get(), 456);
//
//        s.next(789);
//
//        s.sub(|v| x.replace(v));
//        assert_eq!(x.get(), 789);
//        assert_eq!(n.get(), 789);
//    }
//}