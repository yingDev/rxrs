#![feature(fn_traits)]
#![feature(box_syntax)]
#![feature(conservative_impl_trait)]
#![allow(non_snake_case)]
#![feature(associated_type_defaults)]
#![feature(unboxed_closures)]
#![feature(non_ascii_idents)]
#![feature(universal_impl_trait)]
#![feature(generators, generator_trait, step_trait)]
#![feature(fnbox)]

pub mod observable;
pub mod subject;
pub mod behaviour_subject;
pub mod subscriber;
pub mod op;
pub mod fac;
pub mod unsub_ref;
pub mod util;
pub mod scheduler;

#[cfg(test)]
mod test
{
    use super::*;
    use fac::rxfac;
    use observable::*;
    use unsub_ref::*;
    use op::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn hello_world()
    {
        let mut result = String::new();

        let src = rxfac::create(|o|
        {
            o.next("hello");
            o.next("world");
            o.complete();
            UnsubRef::empty()
        });

        src.rx().take(1).map(|s| s.to_uppercase()).sub_scoped(|s:String| result.push_str(&s));
        src.rx().skip(1).sub_scoped(|s| result.push_str(s));

        assert_eq!(result, "HELLOworld");
    }
}