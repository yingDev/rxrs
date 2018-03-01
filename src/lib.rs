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
