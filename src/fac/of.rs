use crate::*;

pub struct Of<V>(V);

impl<V> Of<V>
{
    pub fn value(v: V) -> Self { Of(v) }
    pub fn value_dyn(v: V) -> Box<Self>  { box Of(v) }
}

impl<'o, V:'o> Observable<'o, NO> for Of<V>
{
    type By = Ref<V>;
    type EBy = Ref<()>;

    fn sub(&self, next: impl ActNext<'o, NO, Self::By>, ec: impl ActEc<'o, NO, Self::EBy>+'o) -> Unsub<'o, NO> where Self: Sized
    {
        if ! next.stopped() {
            next.call(&self.0);
            if !next.stopped() {
                ec.call_once(None);
            }
        }

        Unsub::done()
    }

    fn sub_dyn(&self, next: Box<ActNext<'o, NO, Self::By>>, ec: Box<ActEcBox<'o,NO, Self::EBy>>) -> Unsub<'o, NO>
    { self.sub(next, ec) }
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
        o.sub(|v:&_| { n.replace(*v); }, ());

        assert_eq!(n.get(), 123);

        let o = o.into_dyn(); //Of::value_dyn(123);

        let n = std::cell::Cell::new(0);

        o.sub_dyn(box |v:&_| { n.replace(*v + 1); }, box ());

        Of::value_dyn(123).sub_dyn(box |v:&_| { n.replace(*v + 1); }, box());

        assert_eq!(n.get(), 124);
    }

}