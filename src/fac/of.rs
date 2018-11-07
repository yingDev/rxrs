use crate::*;
use std::marker::PhantomData;

pub struct Of<SS:YesNo, V>
{
    v: Option<V>,
    PhantomData: PhantomData<SS>
}

impl<V, SS:YesNo> Of<SS, V>
{
    pub fn value(v: V) -> Self { Of{ v: Some(v), PhantomData }}
    pub fn empty() -> Self { Of{ v: None, PhantomData } }
}

impl<'o, V:'o, SS:YesNo>
Observable<'o, SS, Ref<V>>
for Of<SS, V>
{
    fn subscribe(&self, next: impl ActNext<'o, SS, Ref<V>>, ec: impl ActEc<'o, SS>+'o) -> Unsub<'o, SS> where Self: Sized
    {
        if ! next.stopped() {
            if let Some(v) = self.v.as_ref() {
                next.call(v);
            }
            if !next.stopped() {
                ec.call_once(None);
            }
        }

        Unsub::done()
    }

    fn subscribe_dyn(&self, next: Box<ActNext<'o, SS, Ref<V>>>, ec: Box<ActEcBox<'o,SS>>) -> Unsub<'o, SS>
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