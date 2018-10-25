use std::time::Duration;
use crate::*;
use std::marker::PhantomData;
use std::sync::Arc;

//todo:

pub struct Timer<SS: YesNo, Sch: SchedulerPeriodic<SS>>
{
    period: Duration,
    scheduler: Sch,
    PhantomData: PhantomData<SS>
}

impl<Sch: SchedulerPeriodic<YES>> Timer<YES, Sch>
{
    //todo: default should be a DefaultScheduler ...
    pub fn new(period: Duration, scheduler: Sch) -> Self
    {
        Timer{ period, scheduler, PhantomData }
    }
}


impl<Sch: SchedulerPeriodic<YES>>
Observable<'static, YES, Val<usize>>
for Timer<YES, Sch>
{
    fn sub(&self, next: impl ActNext<'static, YES, Val<usize>>, ec: impl ActEc<'static, YES>) -> Unsub<'static, YES>
    {
        Unsub::done()
    }

    fn sub_dyn(&self, next: Box<ActNext<'static, YES, Val<usize>>>, ec: Box<ActEcBox<'static, YES>>) -> Unsub<'static, YES>
    {
        Unsub::done()
    }
}