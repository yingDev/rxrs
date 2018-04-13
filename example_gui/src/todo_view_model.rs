use rx::subject_nss::Subject;
use rx::observable::*;
use rx::op::*;
use rx::util::mss::*;
use rx::scheduler::*;
use rx;
use rx::behaviour_subject_nss::*;

use owning_ref::RefRef;
use std::cell::RefCell;
use std::cell::Cell;
use std::sync::Arc;
use std::rc::Rc;

#[derive(Clone)]
pub struct TodoItem
{
    pub id: i32,
    pub completed: bool,
    pub title: String
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Filter
{
    Done, Todo, All
}

pub type ItemRef<'i> = RefRef<'i, Vec<TodoItem>, TodoItem>;

pub struct TodoViewModel
{
    cur_id: Cell<i32>,
    items: Rc<RefCell<Vec<TodoItem>>>,
    filter: BehaviorSubject<'static, Filter>,

    changes: Subject<'static, ()>,
}

impl TodoViewModel
{
    pub fn new() -> TodoViewModel
    {
        TodoViewModel{
            cur_id: Cell::new(0),
            items: Rc::new(RefCell::new(vec![])),
            filter: BehaviorSubject::new(Filter::Todo),
            changes: Subject::new()
        }
    }

    pub fn set_filter(&self, f: Filter)
    {
        if self.filter.value().unwrap() != f {
            self.filter.next(f);
            self.changes.next(());
        }
    }

    pub fn toggle_item(&self, id: i32)
    {
        let new = self.items.borrow().iter().map(|item| {
            let mut new = item.clone();
            if item.id == id { new.completed = !new.completed; }
            new
        }).collect();
        self.items.replace(new);

        self.changes.next(());
    }

    pub fn add_item(&self, title: String)
    {
        self.items.borrow_mut().push(TodoItem{ id: self.cur_id.get(), completed: false, title });
        self.cur_id.replace(self.cur_id.get() + 1);

        //if we are viewing 'done' items, jump to 'todo' tab
        if self.filter.value().unwrap() == Filter::Done {
            self.filter.next(Filter::Todo);
        }

        self.changes.next(());
    }

    pub fn clear_completed(&self)
    {
        let new = self.items.borrow().iter().filter(|i| ! i.completed).map(|i| i.clone()).collect();
        self.items.replace(new);

        //if we are viewing 'done' items, jump to 'todo' tab
        if self.filter.value().unwrap() == Filter::Done {
            self.filter.next(Filter::Todo);
        }

        self.changes.next(());
    }

    pub fn filter<'a>(&'a self) -> impl Observable<'static, Filter>+'a
    {
        self.filter.rx()
    }

    pub fn items_left<'a>(&'a self) -> impl Observable<'static, usize>+'a
    {
        let items = self.items.clone();
        self.changes().map(move |_| items.borrow().iter().filter(|i| !i.completed).count())
    }

    pub fn changes<'a>(&'a self) -> impl Observable<'static, ()>+'a
    {
        self.changes.rx().start_with(&())
    }

    pub fn items<'i>(&'i self) -> impl Observable<ItemRef<'i>>
    {
        let filter = self.filter.value().unwrap();

        rx::create(move |o| {
            let r = self.items.borrow();
            let mut i =0;
            while !o._is_closed() && i < r.len() {
                o.next(RefRef::new(self.items.borrow()).map(|v| &v[i]));
                i += 1;
            }
            if !o._is_closed() { o.complete(); }
        }).filter(move |r:&ItemRef| match filter {
            Filter::Done => r.completed,
            Filter::Todo => !r.completed,
            Filter::All => true
        })
    }
}