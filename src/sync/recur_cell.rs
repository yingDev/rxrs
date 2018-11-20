use std::cell::UnsafeCell;
use std::cell::Cell;
use std::ptr;

pub struct RecurCell<V>
{
    val: UnsafeCell<Option<V>>,
    cur: Cell<*const Option<V>>,
}

impl<V> RecurCell<V>
{
    pub fn new(val: V) -> Self
    {
        RecurCell { val: UnsafeCell::new(Some(val)), cur: Cell::new(ptr::null()) }
    }
    
    pub fn map<U>(&self, f: impl FnOnce(&V) -> U) -> U
    {
        let val = unsafe{ &mut *self.val.get() }.take();
        let u = if val.is_some() {
            assert_eq!(self.cur.get(), ptr::null());
            self.cur.replace(&val);
            f(val.as_ref().unwrap())
        } else {
            assert_ne!(self.cur.get(), ptr::null());
            f(unsafe{ &*self.cur.get() }.as_ref().unwrap())
        };
        
        if self.cur.get() == &val {
            self.cur.replace(ptr::null());
            assert!(unsafe{ &*self.val.get() }.is_none());
            *unsafe{ &mut *self.val.get() } = val;
        }
        
        u
    }
    
    pub fn replace(&self, val: V) -> Option<V>
    {
        self.cur.replace(ptr::null());
        unsafe{ &mut *self.val.get() }.replace(val)
    }
}