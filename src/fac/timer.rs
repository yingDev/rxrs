use std::time::Duration;
use crate::*;


struct Timer<Sch: Scheduler>
{
    dur: Duration,
    sch: Sch,
}

impl<Sch: Scheduler> Observable<'static, Sch::SS, Val<usize>> for Timer<Sch>
{
    fn sub(&self, next: impl ActNext<'static, Sch::SS, Val<usize>>, ec: impl ActEc<'static, Sch::SS>) -> Unsub<'static, Sch::SS>
    {
        Unsub::done()
    }

    fn sub_dyn(&self, next: Box<ActNext<'static, Sch::SS, Val<usize>>>, ec: Box<ActEcBox<'static, Sch::SS>>) -> Unsub<'static, Sch::SS>
    {
        Unsub::done()
    }
}
