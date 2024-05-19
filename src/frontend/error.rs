use std::{error::Error, fmt::Display};

use crate::{
  compiler::{
    ast::error::{ASTError, ASTResult},
    intermediate::error::IntermediateCompilationError,
  },
  runtime::error::RuntimeError,
};

#[derive(Debug, Clone, PartialEq)]
pub enum PidginError {
  AST(ASTError),
  Compiler(IntermediateCompilationError),
  Runtime(RuntimeError),
}

impl From<ASTError> for PidginError {
  fn from(err: ASTError) -> Self {
    Self::AST(err)
  }
}
impl From<IntermediateCompilationError> for PidginError {
  fn from(err: IntermediateCompilationError) -> Self {
    Self::Compiler(err)
  }
}
impl From<RuntimeError> for PidginError {
  fn from(err: RuntimeError) -> Self {
    Self::Runtime(err)
  }
}

impl Display for PidginError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      PidginError::AST(err) => write!(f, "ast error: {err}"),
      PidginError::Compiler(err) => write!(f, "compiler error: {err}"),
      PidginError::Runtime(err) => write!(f, "runtime error: {err}"),
    }
  }
}

impl Error for PidginError {}

pub type PidginResult<T> = Result<T, PidginError>;
pub(crate) fn to_pidgin_result<T, E: Into<PidginError>>(
  result: Result<T, E>,
) -> PidginResult<T> {
  result.map_err(|err| err.into())
}
