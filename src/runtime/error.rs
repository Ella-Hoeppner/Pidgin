use std::error::Error;
use std::fmt::{Debug, Display};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum PidginError {
  ArgumentNotNum,
  ArgumentNotInt,
  ArgumentNotList,
  NotYetImplemented,
  CantCastToNum,
  CantApply,
  InvalidArity,
  ExternalError(Rc<dyn Error>),
}
use PidginError::*;

impl Display for PidginError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ArgumentNotNum => write!(f, "argument is not a number"),
      ArgumentNotInt => write!(f, "argument is not an integer"),
      ArgumentNotList => write!(f, "argument is not a list"),
      NotYetImplemented => write!(f, "not yet implemented"),
      CantCastToNum => write!(f, "can't cast to number"),
      CantApply => write!(f, "can't apply"),
      InvalidArity => write!(f, "invalid arity"),
      ExternalError(external_error) => {
        write!(f, "external error: \"{}\"", external_error)
      }
    }
  }
}
impl Error for PidginError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    match self {
      ExternalError(external_error) => Some(&**external_error),
      _ => None,
    }
  }
}
impl From<PidginError> for Rc<dyn Error> {
  fn from(pidgin_error: PidginError) -> Self {
    Rc::new(pidgin_error)
  }
}
pub type PidginResult<T> = std::result::Result<T, PidginError>;