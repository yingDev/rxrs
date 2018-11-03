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
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::fmt::Display;
use std::fmt::Formatter;
use std::sync::Arc;
use std::error::Error;

pub mod util;

mod observable;
mod observer;
mod sync;
mod op;
mod subject;
mod unsub;
mod fac;
mod scheduler;
mod by;


#[derive(Debug)]
pub struct RxError
{
    err: Arc<Error>,
    handled: bool,
}


impl Display for RxError
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result
    { write!(f, "RxError: {:}; handled={}", self.err, self.handled) }
}

impl Clone for RxError
{
    fn clone(&self) -> Self { RxError{ err: self.err.clone(), handled: false } }
}

impl RxError
{
    fn new(e: impl Error+'static) -> Self
    {
        RxError{ err: Arc::new(e), handled: false }
    }

    fn set_handled(self) -> Self
    {
        let mut s = self;
        s.handled = true;
        s
    }

    fn simple(source: Option<Arc<Error+'static>>, msg: impl Into<String>) -> Self
    {
        #[derive(Debug)]
        struct StrError(Option<Arc<Error+'static>>, String);
        impl Display for StrError
        {
            fn fmt(&self, f: &mut Formatter) -> std::fmt::Result
            { write!(f, "RxError: {}; source={:?}", self.1, self.0) }
        }
        impl Error for StrError
        {
            fn source(&self) -> Option<&(dyn Error+'static)> {
                match &self.0 {
                    Some(arc) => Some(Arc::as_ref(arc)),
                    None => None
                }
            }
        }

        Self::new(StrError(source, msg.into()))

    }

    fn handle(mut self, f: impl Fn(&Error) -> Option<RxError>) -> Option<RxError>
    {
        let out = f(self.err.as_ref());
        self.handled = true;
        out
    }
}

impl Drop for RxError
{
    fn drop(&mut self)
    {
        if ! self.handled {
            panic!("Unhandled Error: {}", self.err )
        }
    }
}
