#![feature(fn_traits, unboxed_closures, integer_atomics, optin_builtin_traits, fnbox, test, cell_update, box_syntax, impl_trait_in_bindings)]
#![allow(non_snake_case)]

pub use crate::observable::*;
pub use crate::observer::*;
pub use crate::by::*;
pub use crate::unsub::*;
pub use crate::op::*;
pub use crate::sync::*;
pub use crate::fac::*;
pub use crate::scheduler::*;
pub use crate::subject::*;
pub use crate::error::*;

pub mod util;

mod observable;
mod observer;
mod error;
mod sync;
mod op;
mod subject;
mod unsub;
mod fac;
mod scheduler;
mod by;
