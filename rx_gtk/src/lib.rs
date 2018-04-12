#![feature(box_syntax)]
#![feature(fn_box)]
#![feature(unboxed_closures)]
#![feature(fnbox)]
#![feature(thread_local_state)]
#![feature(use_extern_macros)]

extern crate rxrs as rx;
extern crate gtk;
extern crate gdk;
extern crate glib;

mod macros;
mod gtk_scheduler;

pub use macros::*;
pub use gtk_scheduler::*;