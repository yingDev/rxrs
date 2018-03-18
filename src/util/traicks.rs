pub trait YesNo : Clone+Default
{
    fn value(&self) -> bool;
}
pub trait YesType{}
pub trait NoType{}

#[derive(Copy, Clone)]
pub struct Yes;
#[derive(Copy, Clone)]
pub struct No;

impl YesNo for Yes
{
    fn value(&self) -> bool{ true }
}
impl YesNo for No
{
    fn value(&self) -> bool{ false }
}

impl YesType for Yes{}
impl NoType for No{}

impl Default for Yes {
    fn default() -> Yes { Yes }
}
impl Default for No {
    fn default() -> No { No }
}