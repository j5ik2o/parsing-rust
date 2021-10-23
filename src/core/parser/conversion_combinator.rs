use crate::core::Parser;
use crate::extension::{ConversionCombinator, ConversionCombinators};
use crate::internal::ParsersImpl;
use std::fmt::Debug;

impl<'a, I, A> ConversionCombinator<'a> for Parser<'a, I, A> {
  fn convert<B, E, F>(self, f: F) -> Self::P<'a, Self::Input, B>
  where
    F: Fn(Self::Output) -> Result<B, E> + 'a,
    E: Debug,
    Self::Output: Debug + 'a,
    B: Debug + 'a, {
    ParsersImpl::convert(self, f)
  }
}
