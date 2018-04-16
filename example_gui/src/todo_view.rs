use todo_view_model::*;
use std::rc::Rc;
use std::collections::HashMap;
use gtk;
use rx;
use gtk::prelude::*;
use rx::observable::*;
use rx::op::*;
use todo_models::*;
use rx::behaviour_subject_nss::*;
use rx::subject_nss::Subject;


const FILTER_BTNS: &'static [(Filter, &'static str)] = {
    use ::todo_models::Filter::*;
    &[(Done, "btn_done"), (Todo, "btn_todo"), (All, "btn_all")]
};

pub struct TodoView
{
    state: Rc<State>
}

struct State
{
    input: gtk::Entry,
    list: gtk::ListBox,
    btn_clear_completed: gtk::Button,
    lb_items_left: gtk::Label,
    lb_empty: gtk::Label,

    filter_btns: Rc<HashMap<Filter, gtk::RadioButton>>,
    sig_reset: Rc<Subject<'static,()>>
}


impl TodoView
{
    pub fn new(builder: &gtk::Builder) -> TodoView
    {
        let input: gtk::Entry = builder.get_object("input").unwrap();
        let list : gtk::ListBox = builder.get_object("list").unwrap();
        let lb_items_left : gtk::Label = builder.get_object("lb_items_left").unwrap();
        let btn_clear_completed : gtk::Button = builder.get_object("btn_clear_completed").unwrap();
        let lb_empty : gtk::Label = builder.get_object("lb_empty").unwrap();

        let filter_btns = Rc::new(FILTER_BTNS.iter()
            .map(|(f,s)| (*f, builder.get_object::<gtk::RadioButton>(s).unwrap()))
            .collect::<HashMap<_,_>>());

        TodoView{ state: Rc::new(State{ input, list, filter_btns, lb_items_left, btn_clear_completed, lb_empty, sig_reset: Rc::new(Subject::new()) }) }
    }

    pub fn bind(&self, vm: impl Into<Option<Rc<TodoViewModel>>>)
    {
        let state = self.state.clone();
        state.sig_reset.next(());

        if let Some(vm) = vm.into() {
            let sig_reset = Rc::new(state.sig_reset.clone().first());

            state.filter_btns.iter().for_each(|(f, btn)| {
                signal!(btn,connect_toggled).take_until(sig_reset.clone()).subf(byclone!(vm, f, btn => move |_| {
                    if btn.get_active() {
                        vm.set_filter(f);
                    }
                }));
            });

            signal!(state.btn_clear_completed,connect_clicked).take_until(sig_reset.clone()).subf(byclone!(vm => move |_| {
                vm.clear_completed();
            }));

            vm.filter().take_until(sig_reset.clone()).subf(byclone!(state=> move |f| {
                state.filter_btns[&f].set_active(true);
            }));

            vm.items_left().take_until(sig_reset.clone()).subf(byclone!(state => move |i| {
                state.lb_items_left.set_markup(&format!("{}", i));
            }));

            vm.changes().take_until(sig_reset.clone()).subf(byclone!(vm, state, sig_reset => move |_|
            {
                state.list.get_children().iter().for_each(|w| state.list.remove(w));
                let mut n=0;
                vm.items().subf(|it: ItemRef|
                {
                    let row = gtk::ListBoxRow::new();
                    let item = gtk::CheckButton::new_with_label(&it.title);
                    item.set_active(it.completed);
                    row.add(&item);
                    if let Some(ctx) = row.get_style_context() {
                        ctx.add_class("todo_item");
                    }
                    state.list.prepend(&row);
                    n += 1;
                    signal!(item, connect_toggled).take_until(sig_reset.clone()).subf(byclone!(vm,it => move |_| vm.toggle_item(it.id)));
                });

                state.lb_empty.set_visible(n == 0);
                state.list.show_all();
            }));

            signal!(state.input, connect_activate, it => it.get_text())
                .take_until(sig_reset.clone())
                .filter(|v:&Option<String>| v.as_ref().filter(|s| s.len() > 0).is_some())
                .map(|v| v.unwrap())
                .subf(byclone!(state, vm => move |v: String|
            {
                state.input.set_text("");
                vm.add_item(v);
            }));
        }

    }
}