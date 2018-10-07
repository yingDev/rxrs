use std::rc::Rc;
use std::sync::Arc;
use crate::*;

impl<'o, V:Clone+'o, E:Clone+'o, Src: Observable<'o, V, E>> Observable<'o, V, E> for Box<Src>
{
    #[inline(always)] fn subscribe(&self, observer: impl Observer<V,E>+'o) -> Subscription<'o,NO> { Box::as_ref(self).subscribe(observer) }
}

impl<'o, V:Clone+'o, E:Clone+'o, Src: Observable<'o, V, E>> Observable<'o, V, E> for Rc<Src>
{
    #[inline(always)] fn subscribe(&self, observer: impl Observer<V,E>+'o) -> Subscription<'o,NO> { Rc::as_ref(self).subscribe(observer) }
}


impl<V:Clone+Send+Sync+'static, E:Clone+Send+Sync+'static, Src: ObservableSendSync<V, E>> ObservableSendSync<V, E> for Box<Src>
{
    #[inline(always)] fn subscribe(&self, observer: impl Observer<V,E>+ Send + Sync+'static) -> Subscription<'static, YES>{ Box::as_ref(self).subscribe(observer) }
}

impl<V:Clone+Send+Sync+'static, E:Clone+Send+Sync+'static, Src: ObservableSendSync<V, E>> ObservableSendSync<V, E> for Arc<Src>
{
    #[inline(always)] fn subscribe(&self, observer: impl Observer<V,E>+ Send + Sync+'static) -> Subscription<'static, YES>{ Arc::as_ref(self).subscribe(observer) }
}
