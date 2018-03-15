use std::marker::PhantomData;
use std::cell::RefCell;
use std::ops::Range;
use std::iter::Step;
use std::any::{Any, TypeId};

use observable::*;
use std::rc::Rc;
use std::fmt::Debug;
use subject::*;
use observable::*;
use subref::SubRef;
use std::sync::Arc;
use observable::Observable;
use observable::Observer;
use subref::IntoSubRef;


#[inline(always)]
pub fn create<'a, V, Sub, TRet>(sub:Sub) -> impl Observable<'a, V> where Sub : Fn(&(Observer<V>+Send+Sync+'a))->TRet, TRet: IntoSubRef
{
    CreatedObservable{ sub: move |o| { sub(o).into() }, PhantomData  }
}

#[inline(always)]
pub fn create_clonable<'a, V, Sub, TRet>(sub:Sub) -> impl Clone+Observable<'a, V> where Sub : Clone+Fn(&(Observer<V>+Send+Sync+'a))->TRet, TRet: IntoSubRef
{
    CreatedObservable{ sub: move |o| { sub(o).into() }, PhantomData  }
}

#[inline(always)]
pub fn create_static<V, Sub, TRet>(sub:Sub) -> impl Observable<'static, V> where Sub : Fn(Arc<Observer<V>+Send+Sync+'static>)->TRet, TRet: IntoSubRef
{
    StaticCreatedObservable{ sub: move |o| { sub(o).into() }, PhantomData  }
}

#[inline(always)]
pub fn create_clonable_static< V:Clone, Sub, TRet>(sub:Sub) -> impl Clone+Observable<'static, V> where Sub : Clone+Fn(Arc<Observer<V>+Send+Sync+'static>)->TRet, TRet: IntoSubRef
{
    StaticCreatedObservable{ sub: move |o| { sub(o).into() }, PhantomData  }
}

#[inline(always)]
pub fn range<'a, V:'static>(range: Range<V>) -> impl Clone+Observable<'a,V> where V : Step
{
    create_clonable(move |o| {
        for i in range.clone() {
            if o._is_closed() { return SubRef::empty() }
            o.next(i);
        }
        o.complete();
        SubRef::empty()
    })
}

//
//pub fn create_clonable<'a, V, Sub, O>(sub:Sub) -> impl Clone+Observable<'a, V> where Sub : Clone+Fn(O)->SubRef, O : Observer<V>+Send+Sync+'a
//{
//    CreatedObservable{ sub:sub, PhantomData  }
//}
//

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
    PhantomData: PhantomData<(V, &'a())>
}

impl<'a, V,Sub> Clone for CreatedObservable<'a, V,Sub> where Sub : Clone+Fn(&(Observer<V>+Send+Sync+'a))->SubRef
{
    #[inline(always)]
    fn clone(&self) -> CreatedObservable<'a, V,Sub>
    {
        CreatedObservable{ sub: self.sub.clone(), PhantomData}
    }
}

impl<'a, V,Sub> Observable<'a, V> for CreatedObservable<'a, V,Sub> where Sub : Fn(&(Observer<V>+Send+Sync+'a))->SubRef
{

    #[inline(always)]
    fn sub(&self, o: impl Observer<V>+'a+Send+Sync) -> SubRef
    {
        (self.sub)(&o)
    }
}

#[derive(Clone)]
pub struct StaticCreatedObservable<V, Sub> where Sub : Fn(Arc<Observer<V>+Send+Sync+'static>)->SubRef
{
    sub: Sub,
    PhantomData: PhantomData<V>
}


impl<V,Sub> Observable<'static, V> for StaticCreatedObservable<V,Sub> where Sub : Fn(Arc<Observer<V>+Send+Sync+'static>)->SubRef
{
    #[inline(always)]
    fn sub(&self, o: impl Observer<V>+'static+Send+Sync) -> SubRef
    {
        (self.sub)(Arc::new(o))
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use observable::RxNoti::*;

    #[test]
    fn gen()
    {
        let mut sum = 0;

        {
            let src = create(|o|{
                for i in 0..3 {
                    o.next(i);
                }
                o.complete();
            });

            src.subf(|v| sum += v);
        }

        assert_eq!(sum, 3);
    }

    #[test]
    fn dest_closed()
    {
        let mut sum = 0;

        {
            let src = create(|o|{
                for i in 1..3 {
                    o.next(i);
                }
                o.complete();
            });

            src.sub_noti(|n| match n {
                Next(v) => { sum += v; IsClosed::True }
                _ => { IsClosed::Default }
            });
        }

        assert_eq!(sum, 1);
    }
}