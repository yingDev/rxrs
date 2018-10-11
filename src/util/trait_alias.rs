pub trait CSS : Clone+Send+Sync+'static {}
impl<T: Clone+Send+Sync+'static> CSS for T{}

pub trait ObserverSS<V:CSS,E:CSS> : crate::Observer<V,E>+Send+Sync+'static {}
impl<V:CSS, E:CSS, T: crate::Observer<V,E>+Send+Sync+'static> ObserverSS<V,E> for T {}