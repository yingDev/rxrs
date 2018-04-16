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