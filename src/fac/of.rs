use crate::*;

pub struct Of<V>(Option<V>);

impl<V> Of<V>
{
    pub fn value(v: V) -> Self { Of(Some(v)) }
    pub fn value_dyn(v: V) -> Box<Self>  { box Of(Some(v)) }
    pub fn empty() -> Self { Of(None) }
}

impl<'o, V:'o> Observable<'o, NO, Ref<V>> for Of<V>
{
    fn sub(&self, next: impl ActNext<'o, NO, Ref<V>>, ec: impl ActEc<'o, NO, Ref<()>>+'o) -> Unsub<'o, NO> where Self: Sized
    {
        if ! next.stopped() {
            if let Some(v) = self.0.as_ref() {
                next.call(v);
            }
            if !next.stopped() {
                ec.call_once(None);
            }
        }

        Unsub::done()
    }

    fn sub_dyn(&self, next: Box<ActNext<'o, NO, Ref<V>>>, ec: Box<ActEcBox<'o,NO, Ref<()>>>) -> Unsub<'o, NO>
    { self.sub(next, ec) }
}


#[cfg(test)]
mod test
{
    use crate::*;
    use std::cell::Cell;

    #[test]
    fn smoke()
    {
        let o = Of::value(123);// Of::value(123);

        let n = std::cell::Cell::new(0);
        o.sub(|v:&_| { n.replace(*v); }, ());

        assert_eq!(n.get(), 123);

        let o = o.into_dyn(); //Of::value_dyn(123);
        let n = std::cell::Cell::new(0);


        o.sub_dyn(box |v:&_| { n.replace(*v + 1); }, box ());

        Of::value_dyn(123).sub_dyn(box |v:&_| { n.replace(*v + 1); }, box());

        assert_eq!(n.get(), 124);
    }

    #[test]
    fn empty()
    {
        let n = Cell::new(0);
        let o = Of::empty();
        o.sub(|v:&()| assert!(false, "shouldn't happend"), |e:Option<&_>| { n.replace(1); } );

        assert_eq!(n.get(), 1);
    }

}