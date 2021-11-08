use crate::core::Parser;
use crate::extension::parser::LoggingParser;
use crate::extension::parsers::LoggingParsers;
use crate::internal::ParsersImpl;
use std::fmt::Debug;

impl<'a, I, A> LoggingParser<'a> for Parser<'a, I, A> {
  fn log(self, name: &'a str) -> Self::P<'a, Self::Input, Self::Output>
  where
    Self::Input: Debug,
    Self::Output: Debug + 'a, {
    ParsersImpl::logging(self, name)
  }

  fn name(self, name: &'a str) -> Self::P<'a, Self::Input, Self::Output>
  where
    Self::Input: Debug,
    Self::Output: Debug + 'a, {
    ParsersImpl::name(self, name)
  }
}
