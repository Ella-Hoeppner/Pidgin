pub mod data;
pub mod instructions;
pub mod vm;

use std::fmt::Debug;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Error {
  ArgumentNotNum,
  ArgumentNotInto,
  NotYetImplemented,
  CantCastToNum,
  CantApply,
  InvalidArity,
}
pub type Result<T> = std::result::Result<T, Error>;
