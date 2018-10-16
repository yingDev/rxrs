use crate::*;

pub struct Of<V> { v: V }

impl<V> Of<V>
{
    pub fn value(v: V) -> Of<V> { Of{ v } }
}

impl<'o, V: 'o> Observable<'o, NO, Ref<V>, Ref<()>> for Of<V>
{
    fn sub(&self,
           next: impl FnNext<NO, Ref<V>> + 'o,
           ec: impl FnErrComp<NO, Ref<()>> + 'o) -> Unsub<'o, NO> where Self: Sized
    {
        next.call(By::r(&self.v));
        ec.call_once(None);

        Unsub::done()
    }

    fn sub_dyn(&self,
               next: Box<FnNext<NO, Ref<V>> + 'o>,
               ec: Box<FnErrCompBox<NO, Ref<()>> + 'o>) -> Unsub<'o, NO>
    {
        self.sub(next, ec)
    }
}


#[cfg(test)]
mod test
{
    use crate::*;

    #[test]
    fn smoke()
    {
        let n = std::cell::Cell::new(0);
        let o = Of::value(123);

        o.sub(|v:By<_>| { n.replace(*v); }, ());
        assert_eq!(n.get(), 123);

        let o = o.into_dyn();
        o.sub_dyn(box |v:By<_>| { n.replace(*v+1); }, box());
        assert_eq!(n.get(), 124);
    }
}