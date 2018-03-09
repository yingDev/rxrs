use std::marker::PhantomData;
use std::cell::RefCell;
use std::ops::Range;
use std::iter::Step;
use std::any::{Any, TypeId};

use observable::*;
use subscriber::SubscriberImpl;
use subscriber::Subscriber;
use std::rc::Rc;
use std::fmt::Debug;
use subject::*;
use observable::*;
use unsub_ref::UnsubRef;
use std::sync::Arc;
use observable::Observable;
use observable::Observer;

pub fn create<'a, V, Sub>(sub:Sub) -> impl Clone+Observable<'a, V> where Sub : Clone+Fn(Arc<Observer<V>+Send+Sync+'a>)->UnsubRef
{
    CreatedObservable{ sub:sub, PhantomData  }
}

pub fn range<'a, V:'static>(range: Range<V>) -> impl Clone+Observable<'a,V> where V : Step
{
    create(move |o|
    {
        for i in range.clone()
        {
            if o._is_closed() { return UnsubRef::empty() }
            o.next(i);
        }
        o.complete();
        UnsubRef::empty()
    })
}

pub fn of<'a, V:Clone+'static>(v:V) -> impl Clone+Observable<'a, V>
{
    create(move |o|
    {
        o.next(v.clone());
        o.complete();

        UnsubRef::empty()
    })
}

//fixme: semantics on `drop`: complete or just abort ?
pub struct CreatedObservable<V, Sub>
{
    sub: Sub,
    PhantomData: PhantomData<V>
}

impl<V,Sub> Clone for CreatedObservable<V,Sub> where Sub:Clone
{
    fn clone(&self) -> CreatedObservable<V,Sub>
    {
        CreatedObservable{ sub: self.sub.clone(), PhantomData}
    }
}

impl<'a, V,Sub> Observable<'a, V> for CreatedObservable<V,Sub> where Sub : Fn(Arc<Observer<V>+Send+Sync+'a>)->UnsubRef
{
    fn sub(&self, dest: Arc<Observer<V>+Send+Sync+'a>) -> UnsubRef
    {
        (self.sub)(dest)
    }
}
