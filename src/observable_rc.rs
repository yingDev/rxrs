use std::rc::Rc;
use std::sync::Arc;
use crate::*;

impl<'s, 'o, V:Clone, E:Clone, RC: Observable<'s, 'o, V, E>> Observable<'s, 'o, V, E> for Rc<RC>
{
    #[inline(always)] fn subscribe(&'s self, observer: impl Observer<V,E>+'o) -> Subscription<'o,NO> { Rc::as_ref(self).subscribe(observer) }
}

impl<'s, V:Clone+Send+Sync, E:Clone+Send+Sync, RC: ObservableSendSync<'s, V, E>> ObservableSendSync<'s, V, E> for Arc<RC>
{
    #[inline(always)] fn subscribe(&'s self, observer: impl Observer<V,E>+ Send + Sync+'static) -> Subscription<'static, YES>{ Arc::as_ref(self).subscribe(observer) }
}

