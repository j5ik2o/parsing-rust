use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq)]
pub struct ConfigNumberValue(Decimal);

impl Display for ConfigNumberValue {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl ConfigNumberValue {
  pub fn new(text: &str) -> Self {
    Self(Decimal::from_str_radix(text, 10).unwrap())
  }

  pub fn to_i64(self) -> Option<i64> {
    self.0.to_i64()
  }

  pub fn to_u64(self) -> Option<u64> {
    self.0.to_u64()
  }

  pub fn to_f64(self) -> Option<f64> {
    self.0.to_f64()
  }

  pub fn to_f32(self) -> Option<f32> {
    self.0.to_f32()
  }
}
