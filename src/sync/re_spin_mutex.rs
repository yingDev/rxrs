use crate::*;
use std::ops::Deref;

pub struct ReSpinMutex<SS:YesNo, V>
{
    lock: ReSpinLock<SS>,
    value: RecurCell<V>,
}

pub struct Guard<'a, SS:YesNo, V>
{
    value: &'a RecurCell<V>,
    lock: &'a ReSpinLock<SS>
}

impl<'a, SS:YesNo, V> Drop for Guard<'a, SS, V>
{
    fn drop(&mut self)
    {
        self.lock.exit();
    }
}

impl<'a, SS:YesNo, V> Deref for Guard<'a, SS, V>
{
    type Target = RecurCell<V>;
    fn deref(&self) -> &Self::Target { self.value }
}

impl<SS:YesNo, V> ReSpinMutex<SS, V>
{
    pub fn new(value: V) -> Self
    {
        ReSpinMutex { lock: ReSpinLock::new(), value: RecurCell::new(value) }
    }
    
    pub fn lock(&self) -> Guard<SS, V>
    {
        self.lock.enter();
        Guard{ value: &self.value, lock: &self.lock }
    }
}