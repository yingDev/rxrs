#[macro_export]
macro_rules! event_full(

    ($widget: expr,$event: ident, $($it:ident),* => $map: expr, $ret: expr) => {{

        use glib::signal::signal_handler_disconnect;
        use std::cell::RefCell;
        use rx::util::mss::*;

        let weak_widget = $widget.downgrade();
        rx::create_boxed(move |o| {
            if let Some(widget) = weak_widget.upgrade() {

                let id = widget.$event(move |$($it,)*| { o.next($map); $ret } );
                let destroy_id = Rc::new(RefCell::new(None));
                let did = destroy_id.clone();

                let weak_widget = weak_widget.clone();
                let sub= rx::InnerSubRef::<No>::new(byclone!(did=>move || {
                    if did.borrow().is_none() {
                        return;
                    }
                    if let Some(widget) = weak_widget.upgrade() {
                        if let Some(did) = did.borrow_mut().take() {
                            signal_handler_disconnect(&widget, did);
                        }
                        signal_handler_disconnect(&widget, id);
                    }
                }));

                *destroy_id.borrow_mut() = Some(widget.connect_destroy(byclone!(sub, did => move |_| {
                    did.borrow_mut().take();
                    sub.unsub();
                })));

                return sub;
            }
            panic!("the widget has been destroyed!");
    })}};
);

#[macro_export]
macro_rules! event {
    ($widget: expr,$event: ident, true) => { event_full!($widget,$event, it, e => (it.clone(), e.clone()), gtk::Inhibit(true)) };
    ($widget: expr,$event: ident, false) => { event_full!($widget,$event, it, e => (it.clone(), e.clone()), gtk::Inhibit(false)) }
}

#[macro_export]
macro_rules! event_once {
    ($widget: expr,$event: ident, true) => ({ use ::rx::op::*; event!($widget,$event, true).take(1) });
    ($widget: expr,$event: ident, false) => ({ use ::rx::op::*; event!($widget,$event, false).take(1) });
}

#[macro_export]
macro_rules! signal {
    ($widget: expr,$event: ident) => {event_full!($widget,$event, it => it.clone(), ())};
    ($widget: expr,$event: ident, $($it:ident),* => $map: expr) => {event_full!($widget,$event, $($it),* => $map, ())};
}


macro_rules! stateful(
($($var:ident=$e:expr),+ => $closure: expr) => {{
    $(let $var = $e;)*;
    $closure
}});