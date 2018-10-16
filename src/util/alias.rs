pub trait SSs: Send + Sync + 'static {}
impl<T: Send+Sync+'static> SSs for T{}