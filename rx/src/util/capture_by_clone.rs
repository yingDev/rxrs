#[macro_export]
macro_rules! byclone(
($($var:ident),* => $closure: expr) => {{
    $(let $var = $var.clone();)*;
    $closure
}});
