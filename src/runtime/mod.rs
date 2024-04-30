pub mod data;
pub mod vm;

use std::fmt::Debug;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Error {
  NotYetImplemented,
  CantCastToNum,
  CantApply,
}
pub type Result<T> = std::result::Result<T, Error>;
