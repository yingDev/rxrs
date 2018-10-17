use crate::*;

pub struct Of<V>(V);

impl<V> Of<V>
{
    fn value(v: V) -> Self { Of(v) }
    fn value_dyn(v: V) -> Box<Self>  { box Of(v) }
}

impl<'o, V:'o> Observable<'o, NO, Ref<V>, Ref<()>> for Of<V>
{
    fn sub(&self,
           next: impl for<'x> FnNext<NO, By<'x, Ref<V>>>+'o,
           ec: impl for<'x> FnErrComp<NO, By<'x, Ref<()>>>+'o) -> Unsub<'o, NO> where Self: Sized
    {
        next.call(By::r(&self.0));
        ec.call_once(None);

        Unsub::done()
    }

    fn sub_dyn(&self,
               next: Box<for<'x> FnNext<NO, By<'x, Ref<V>>>+'o>,
               ec: Box<for<'x> FnErrCompBox<NO, By<'x, Ref<()>>> +'o>) -> Unsub<'o, NO>
    { self.sub(#[inline(always)] move |v:By<_>| next.call(v), #[inline(always)] move |v: Option<By<_>>| ec.call_box(v)) }
}


#[cfg(test)]
mod test
{
    use crate::*;

    #[test]
    fn smoke()
    {
        let o = Of::value(123);// Of::value(123);

        let n = std::cell::Cell::new(0);
        o.sub(|v:By<_>| { n.replace(*v); }, ());

        assert_eq!(n.get(), 123);

        let o = o.into_dyn(); //Of::value_dyn(123);

        let n = std::cell::Cell::new(0);

        o.sub_dyn(box |v:By<_>| { n.replace(*v + 1); }, box());

        fn get<'o, 'a>() -> Box<dyn Observable<'o, NO, Ref<i32>>+'a>
        {
            box Of::value(123)
        }

        Of::value_dyn(123).sub_dyn(box |v:By<_>| { n.replace(*v + 1); }, box());

        assert_eq!(n.get(), 124);
    }

}