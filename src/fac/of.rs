use crate::*;

pub struct Of<V>(Option<V>);

impl<V> Of<V>
{
    pub fn value(v: V) -> Self { Of(Some(v)) }
    pub fn value_clone<'o>(v: V) -> impl Observable<'o, NO, Val<V>> where V: Clone+'o
    {
        Self::value(v).map(|v:&V| v.clone())
    }
    pub fn empty() -> Self { Of(None) }
}

impl<'o, V:'o>
Observable<'o, NO, Ref<V>>
for Of<V>
{
    fn subscribe(&self, next: impl ActNext<'o, NO, Ref<V>>, ec: impl ActEc<'o, NO>+'o) -> Unsub<'o, NO> where Self: Sized
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

    fn subscribe_dyn(&self, next: Box<ActNext<'o, NO, Ref<V>>>, ec: Box<ActEcBox<'o,NO>>) -> Unsub<'o, NO>
    { self.subscribe(next, ec) }
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
        o.subscribe(|v:&_| { n.replace(*v); }, ());

        assert_eq!(n.get(), 123);

        let n = std::cell::Cell::new(0);
        let o = o.into_dyn(); //Of::value_dyn(123);


        o.subscribe_dyn(box |v:&_| { n.replace(*v + 1); }, box ());

        Of::value(123).into_dyn().subscribe_dyn(box |v:&_| { n.replace(*v + 1); }, box());

        assert_eq!(n.get(), 124);
    }

    #[test]
    fn empty()
    {
        let n = Cell::new(0);
        let o = Of::empty();
        o.subscribe(|_v:&()| assert!(false, "shouldn't happend"), |_e| { n.replace(1); } );

        assert_eq!(n.get(), 1);
    }

}