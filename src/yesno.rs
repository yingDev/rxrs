pub trait YesNo : Send+Sync+Clone+ 'static { const VALUE: bool; }

#[derive(Copy, Clone)]
pub struct YES;
#[derive(Copy, Clone)]
pub struct NO;

impl YesNo for YES { const VALUE: bool = true; }
impl YesNo for NO { const VALUE: bool = false; }