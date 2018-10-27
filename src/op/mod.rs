mod map;
mod filter;
//mod until;
//mod take;

pub use self::map::*;
pub use self::filter::*;
//pub use self::until::*;
//pub use self::take::*;

#[cfg(test)]
mod test
{
//    use crate::*;
//    use std::rc::Rc;
//    use std::cell::RefCell;
//
//    #[test]
//    fn filter_map()
//    {
//        let s = RefCell::new(String::new());
//        let (s1, s2) = Rc::new(Subject::<NO, i32>::new()).clones();
//
//        s1.filter(|i| **i % 2 == 0).map(|v| format!("{}", *v)).sub(|v:By<Val<String>>| s.borrow_mut().push_str(&*v), ());
//
//        for i in 0..10 {
//            s2.next(i);
//        }
//
//        assert_eq!(s.borrow().as_str(), "02468");
//
//    }
}