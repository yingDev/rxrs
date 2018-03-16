use std::marker::PhantomData;
use std::cell::RefCell;
use std::ops::Range;
use std::iter::Step;
use std::any::{Any, TypeId};

use observable::*;
use std::rc::Rc;
use std::fmt::Debug;
use observable::*;
use subref::SubRef;
use std::sync::Arc;
use observable::Observable;
use observable::Observer;
use subref::IntoSubRef;


#[inline(always)]
pub fn create<'a:'b, 'b, V:'a+Send+Sync, Sub, TRet>(sub:Sub) -> impl Observable<'a,V>+'b+Send+Sync where
    Sub : Send+Sync+'a+Fn(Arc<Observer<V>+Send+Sync+'a>)->TRet,
    TRet: IntoSubRef
{
    CreatedObservable{ sub, PhantomData }
}

#[inline(always)]
pub fn range<'a:'b, 'b, V:'static+Send+Sync>(range: Range<V>) -> impl Observable<'a,V>+'b+Send+Sync where V : Step
{
    create(move |o| {
        for i in range.clone() {
            if o._is_closed() { return; }
            o.next(i);
        }
        o.complete();
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
pub struct CreatedObservable<'a, V, Sub>
{
    sub: Sub,
    PhantomData: PhantomData<(V, &'a())>
}
unsafe impl<'a,V,Sub> Send for CreatedObservable<'a,V,Sub> where Sub : 'a+Send{}
unsafe impl<'a,V,Sub> Sync for CreatedObservable<'a,V,Sub> where Sub : 'a+Sync{}

impl<'a, V,Sub> Clone for CreatedObservable<'a, V,Sub> where Sub : Clone+Fn(Arc<Observer<V>+Send+Sync+'a>)->SubRef
{
    #[inline(always)]
    fn clone(&self) -> CreatedObservable<'a, V,Sub>
    {
        CreatedObservable{ sub: self.sub.clone(), PhantomData}
    }
}

impl<'a, V,Sub, TRet> Observable<'a, V> for CreatedObservable<'a, V,Sub> where Sub : Fn(Arc<Observer<V>+Send+Sync+'a>)->TRet, TRet: IntoSubRef
{
    #[inline(always)]
    fn sub(&self, o: Arc<Observer<V>+'a+Send+Sync>) -> SubRef
    {
        (self.sub)(o).into()
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use observable::RxNoti::*;
    use op::*;
    use fac::timer::*;

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

            src.rx().subf(|v| sum += v);
        }

       // assert_eq!(sum, 3);
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

            src.rx().take(1).subf(|v| sum += v);
        }

        //assert_eq!(sum, 1);
    }
}