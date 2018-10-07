use std::rc::Rc;
use std::sync::Arc;
use crate::*;

impl<'o, V:Clone, E:Clone, RC: Observable<'o, V, E>> Observable<'o, V, E> for Rc<RC>
{
    #[inline(always)] fn subscribe(&self, observer: impl Observer<V,E>+'o) -> Subscription<'o,NO> { Rc::as_ref(self).subscribe(observer) }
}

impl<V:Clone+Send+Sync, E:Clone+Send+Sync, RC: ObservableSendSync<V, E>> ObservableSendSync<V, E> for Arc<RC>
{
    #[inline(always)] fn subscribe(&self, observer: impl Observer<V,E>+ Send + Sync+'static) -> Subscription<'static, YES>{ Arc::as_ref(self).subscribe(observer) }
}

