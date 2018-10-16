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
        self.sub(move |v: By<_>| next.call(v), move |e: Option<By<_>>| ec.call_box(e))
    }
}
