#![feature(fn_traits)]
#![feature(box_syntax)]
#![feature(fn_box)]
#![feature(unboxed_closures)]
#![feature(fnbox)]
#![feature(thread_local_state)]
#![feature(use_extern_macros)]
#![feature(concat_idents)]
#![feature(optin_builtin_traits)]
#![type_length_limit="2097152"]
#![feature(option_filter)]

#[macro_use]
extern crate rxrs as rx;
extern crate gtk;
extern crate gdk;
extern crate glib;
extern crate owning_ref;
#[macro_use]
extern crate rx_gtk;

mod todo_models;
mod todo_view_model;
mod todo_view;

use gtk::prelude::*;
use std::rc::Rc;
use gdk::DisplayExt;

use rx_gtk::*;
use todo_view_model::*;
use todo_view::*;
use rx::observable::*;

fn main()
{
    if let Err(e) = gtk::init(){ panic!("{}", e); }
    rx::scheduler::set_sync_context(Some(GtkScheduler::<rx::util::mss::No>::get()));
    load_style();

    let builder = gtk::Builder::new_from_string(include_str!("win.glade"));
    let window: gtk::Window = builder.get_object("win").unwrap();

    event_once!(window, connect_delete_event, true).subf(|_| gtk::main_quit());

    let vm = Rc::new(TodoViewModel::new());
    vm.add_item("hello world".to_owned());

    let view = TodoView::new(&builder);
    view.bind(vm.clone());

    window.show_all();
    gtk::main();
}

fn load_style()
{
    let provider = gtk::CssProvider::new();
    if let Err(e) = provider.load_from_data(include_str!("style.css").as_bytes()) {
        panic!("{}", e);
    }

    gtk::StyleContext::add_provider_for_screen(&gdk::Display::get_default().unwrap().get_default_screen(),
                                               &provider,
                                               gtk::STYLE_PROVIDER_PRIORITY_USER);
}