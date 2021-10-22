use crate::core::Parser;
use crate::extension::SkipCombinator;
use std::fmt::Debug;
use std::ops::Mul;

impl<'a, I, A, B> Mul<Parser<'a, I, B>> for Parser<'a, I, A>
where
  A: Debug + 'a,
  B: Debug + 'a,
{
  type Output = Parser<'a, I, B>;

  fn mul(self, rhs: Parser<'a, I, B>) -> Self::Output {
    self.skip_left(rhs)
  }
}
