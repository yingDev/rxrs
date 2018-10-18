use std::marker::PhantomData;

pub trait YesNo : Send+Sync+Clone+ 'static
{
    const SELF: Self;
    const VALUE: bool;
}
#[derive(PartialEq, Copy, Clone)]pub struct YES;
#[derive(PartialEq, Copy, Clone)]pub struct NO;

impl YesNo for YES
{
    const SELF: Self = YES;
    const VALUE: bool = true;
}
impl YesNo for NO
{
    const SELF: Self = NO;
    const VALUE: bool = false;
}

pub unsafe auto trait No {}

impl !No for YES {}