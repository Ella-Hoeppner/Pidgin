use std::error::Error;
use std::fmt::{Debug, Display};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum RuntimeError {
  ArgumentNotNum,
  ArgumentNotInt,
  ArgumentNotList,
  NotYetImplemented,
  CantCastToNum(Value),
  CantApply(Value),
  InvalidArity,
  CantCreateCoroutine(String),
  DeadCoroutine,
  CoroutineAlreadyRunning,
  IsntCoroutine,
  ExternalError(Rc<dyn Error>),
}
impl PartialEq for RuntimeError {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::ExternalError(l0), Self::ExternalError(r0)) => Rc::ptr_eq(l0, r0),
      _ => core::mem::discriminant(self) == core::mem::discriminant(other),
    }
  }
}
use RuntimeError::*;

use super::data::Value;

impl Display for RuntimeError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ArgumentNotNum => write!(f, "argument is not a number"),
      ArgumentNotInt => write!(f, "argument is not an integer"),
      ArgumentNotList => write!(f, "argument is not a list"),
      NotYetImplemented => write!(f, "not yet implemented"),
      CantCastToNum(value) => write!(f, "can't cast value {value} to number"),
      CantApply(value) => write!(f, "can't apply value {value}"),
      InvalidArity => write!(f, "invalid arity"),
      CantCreateCoroutine(s) => write!(f, "{}", s),
      CoroutineAlreadyRunning => {
        write!(f, "attempt to run coroutine that is already running")
      }
      IsntCoroutine => write!(f, "argument is not a coroutine"),
      DeadCoroutine => write!(f, "attempt to run dead coroutine"),
      ExternalError(external_error) => {
        write!(f, "external error: \"{}\"", external_error)
      }
    }
  }
}
impl Error for RuntimeError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    match self {
      ExternalError(external_error) => Some(&**external_error),
      _ => None,
    }
  }
}
impl From<RuntimeError> for Rc<dyn Error> {
  fn from(pidgin_error: RuntimeError) -> Self {
    Rc::new(pidgin_error)
  }
}
pub type RuntimeResult<T> = std::result::Result<T, RuntimeError>;
