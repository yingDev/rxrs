use crate::*;
use std::time::Duration;

struct NewThreadScheduler
{

}

impl Scheduler<YES> for NewThreadScheduler
{
    fn schedule(&self, due: Option<Duration>, act: impl SchActOnce<YES>) -> Unsub<'static, YES> where Self: Sized
    {
        unimplemented!()
    }
}