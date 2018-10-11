
pub fn unmut<V, R, F:FnMut(V)->R>(f: F) -> impl Fn(V)->R
{
    let cell = ::std::cell::UnsafeCell::new(f);
    move |v| unsafe{ &mut *cell.get() }(v)
}
