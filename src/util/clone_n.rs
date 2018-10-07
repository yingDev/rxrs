pub trait CloneN : Clone
{
    #[inline(always)] fn clone2(&self) -> (Self, Self) { (self.clone(), self.clone()) }
    #[inline(always)] fn clone3(&self) -> (Self, Self, Self) { (self.clone(), self.clone(), self.clone()) }
    #[inline(always)] fn clone4(&self) -> (Self, Self, Self, Self) { (self.clone(), self.clone(), self.clone(), self.clone()) }
    #[inline(always)] fn clone5(&self) -> (Self, Self, Self, Self, Self) { (self.clone(), self.clone(), self.clone(), self.clone(), self.clone()) }

    #[inline(always)] fn cloned2(self) -> (Self, Self) { (self.clone(), self) }
    #[inline(always)] fn cloned3(self) -> (Self, Self, Self) { (self.clone(), self.clone(), self) }
    #[inline(always)] fn cloned4(self) -> (Self, Self, Self, Self) { (self.clone(), self.clone(), self.clone(), self) }
    #[inline(always)] fn cloned5(self) -> (Self, Self, Self, Self, Self) { (self.clone(), self.clone(), self.clone(), self.clone(), self) }
}

impl<C: Clone> CloneN for C {}