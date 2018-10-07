pub trait YesNo : 'static { const VALUE: bool; }
pub struct YES;
pub struct NO;
impl YesNo for YES { const VALUE: bool = true; }
impl YesNo for NO { const VALUE: bool = false; }