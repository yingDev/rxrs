#![feature(fn_traits)]
#![feature(box_syntax)]
#![allow(non_snake_case)]
#![feature(associated_type_defaults)]
#![feature(unboxed_closures)]
#![feature(non_ascii_idents)]
#![feature(generators, generator_trait, step_trait)]
#![feature(fnbox)]
#![feature(get_type_id)]
#![feature(generic_param_attrs)]
#![feature(dropck_eyepatch)]
#![feature(nested_impl_trait)]
#![type_length_limit="33554432"]
#![feature(specialization)]
#![feature(coerce_unsized)]
#![feature(unsize)]
#![allow(non_snake_case)]
#![feature(optin_builtin_traits)]
#![feature(core_intrinsics)]

#[macro_use] pub mod util;

pub mod observable;
pub mod subject;
pub mod subject_nss;
pub mod behaviour_subject;
pub mod behaviour_subject_nss;
pub mod op;
pub mod scheduler;
mod fac;
mod subref;
pub mod connectable_observable;
//
pub use fac::*;
pub use subref::*;


#[cfg(test)]
mod test_fixture;

#[cfg(test)]
mod test
{

}