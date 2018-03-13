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
use subref::SubRef;
use std::sync::Arc;
use observable::Observable;
use observable::Observer;
use subref::IntoSubRef;


pub fn create<'a, V, Sub, TRet>(sub:Sub) -> impl Observable<'a, V> where Sub : Fn(&(Observer<V>+Send+Sync+'a))->TRet, TRet: IntoSubRef
{
    CreatedObservable{ sub: move |o| { sub(o).into() }, PhantomData  }
}

pub fn create_static<V, Sub, TRet>(sub:Sub) -> impl Observable<'static, V> where Sub : Fn(Arc<Observer<V>+Send+Sync+'static>)->TRet, TRet: IntoSubRef
{
    StaticCreatedObservable{ sub: move |o| { sub(o).into() }, PhantomData  }
}

impl<'a,V,F, TRet> Observable<'a,V> for F where F : Fn(&(Observer<V>+Send+Sync+'a))->TRet, TRet: IntoSubRef
{
    fn sub(&self, o: impl Observer<V> + Send + Sync + 'a) -> SubRef
    {
        Observable::sub(&create(self), o)
    }
}



//
//pub fn create_clonable<'a, V, Sub, O>(sub:Sub) -> impl Clone+Observable<'a, V> where Sub : Clone+Fn(O)->SubRef, O : Observer<V>+Send+Sync+'a
//{
//    CreatedObservable{ sub:sub, PhantomData  }
//}
//
//pub fn range<'a, V:'static>(range: Range<V>) -> impl Clone+Observable<'a,V> where V : Step
//{
//    create_clonable(move |o|
//    {
//        for i in range.clone()
//        {
//            if o._is_closed() { return SubRef::empty() }
//            o.next(i);
//        }
//        o.complete();
//        SubRef::empty()
//    })
//}
//
//pub fn of<'a, V:Clone+'static>(v:V) -> impl Clone+Observable<'a, V>
//{
//    create_clonable(move |o|
//    {
//        o.next(v.clone());
//        o.complete();
//
//        SubRef::empty()
//    })
//}
//
////fixme: semantics on `drop`: complete or just abort ?
pub struct CreatedObservable<'a, V, Sub> where Sub : Fn(&(Observer<V>+Send+Sync+'a))->SubRef
{
    sub: Sub,
    PhantomData: PhantomData<(*const V, &'a())>
}

impl<'a, V,Sub> Clone for CreatedObservable<'a, V,Sub> where Sub : Clone+Fn(&(Observer<V>+Send+Sync+'a))->SubRef
{
    fn clone(&self) -> CreatedObservable<'a, V,Sub>
    {
        CreatedObservable{ sub: self.sub.clone(), PhantomData}
    }
}

impl<'a, V,Sub> Observable<'a, V> for CreatedObservable<'a, V,Sub> where Sub : Fn(&(Observer<V>+Send+Sync+'a))->SubRef
{
    fn sub(&self, dest: impl Observer<V> + Send + Sync+'a) -> SubRef
    {
        (self.sub)(&dest)
    }
}

pub struct StaticCreatedObservable<V, Sub> where Sub : Fn(Arc<Observer<V>+Send+Sync+'static>)->SubRef
{
    sub: Sub,
    PhantomData: PhantomData<(*const V)>
}
impl<V,Sub> Observable<'static, V> for StaticCreatedObservable<V,Sub> where Sub : Fn(Arc<Observer<V>+Send+Sync+'static>)->SubRef
{
    fn sub(&self, dest: impl Observer<V> + Send + Sync+'static) -> SubRef
    {
        (self.sub)(Arc::new(dest))
    }
}

#[cfg(test)]
mod test
{
    use super::*;

    #[test]
    fn gen()
    {
        let i = 9;

        let s = create_static(|o|{
            o.next(1);
            ::std::thread::spawn(move || o.next(2)).join();
        }).subf(|v| println!("{}", v));

        let mut j = 0;

        let src = create(|o|{
            for i in 0..10 {
                o.next(i);
            }
            o.complete();
        });

        src.rx().subf(|v| { println!("xx {}", { j+=v; v}) });
        src.subf(|v| { println!("yy {}", v) });

        src.subf(( |v| { println!("yy {}", v) }, (), ||println!("complete!") ));
    }
}