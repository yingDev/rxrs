pub trait CSS : Clone+Send+Sync+'static {}
impl<T: Clone+Send+Sync+'static> CSS for T{}