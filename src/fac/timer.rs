//use std::time::Duration;
//use crate::*;
//use std::marker::PhantomData;
//
//
//struct Timer<SS: YesNo, Sch: Scheduler<SS>>
//{
//    dur: Duration,
//    sch: Sch,
//    PhantomData: PhantomData<SS>
//}
//
//impl<Sch: Scheduler<NO>> Observable<'static, NO, Val<usize>> for Timer<NO, Sch>
//{
//    fn sub(&self, next: impl ActNext<'static, NO, Val<usize>>, ec: impl ActEc<'static, NO>) -> Unsub<'static, NO>
//    {
//        Unsub::done()
//    }
//
//    fn sub_dyn(&self, next: Box<ActNext<'static, NO, Val<usize>>>, ec: Box<ActEcBox<'static, NO>>) -> Unsub<'static, NO>
//    {
//        Unsub::done()
//    }
//}
//
//impl<Sch: Scheduler<YES>> Observable<'static, YES, Val<usize>> for Timer<YES, Sch>
//{
//    fn sub(&self, next: impl ActNext<'static, YES, Val<usize>>, ec: impl ActEc<'static, YES>) -> Unsub<'static, YES>
//    {
//        Unsub::done()
//    }
//
//    fn sub_dyn(&self, next: Box<ActNext<'static, YES, Val<usize>>>, ec: Box<ActEcBox<'static, YES>>) -> Unsub<'static, YES>
//    {
//        Unsub::done()
//    }
//}
//
//impl<Sch: Scheduler<NO>> Observable<'static, YES, Val<usize>> for Timer<NO, Sch>
//{
//    fn sub(&self, next: impl ActNext<'static, YES, Val<usize>>, ec: impl ActEc<'static, YES>) -> Unsub<'static, YES>
//    {
//        Unsub::done()
//    }
//
//    fn sub_dyn(&self, next: Box<ActNext<'static, YES, Val<usize>>>, ec: Box<ActEcBox<'static, YES>>) -> Unsub<'static, YES>
//    {
//        Unsub::done()
//    }
//}
