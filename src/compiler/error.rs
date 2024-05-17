use std::{error::Error, fmt::Display};

use crate::compiler::SSARegister;

use super::transformations::InstructionTimestamp;

#[derive(Clone, Debug)]
pub enum CompilationError {
  UsedBeforeCreation(SSARegister, InstructionTimestamp),
  OutputToExisting(
    SSARegister,
    Option<InstructionTimestamp>,
    InstructionTimestamp,
  ),
  ReplacingNonexistent(SSARegister, InstructionTimestamp),
  UsedAfterReplacement(
    SSARegister,
    InstructionTimestamp,
    SSARegister,
    InstructionTimestamp,
  ),
}
impl Display for CompilationError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use CompilationError::*;
    match *self {
      UsedBeforeCreation(register, timestamp) => write!(
        f,
        "attempted to use register {register} before creation at \
         timestamp {timestamp}"
      ),
      OutputToExisting(register, created_timestamp, new_timestamp) => write!(
        f,
        "attempted to output to register {register} at timestamp \
         {new_timestamp}, when register was already created at timestamp \
         {created_timestamp:?}"
      ),
      ReplacingNonexistent(register, timestamp) => write!(
        f,
        "attempting to replace register {register} at timestamp {timestamp}, \
         but the register does not exist"
      ),
      UsedAfterReplacement(
        register,
        timestamp,
        already_replaced_register,
        already_replaced_timestamp,
      ) => write!(
        f,
        "attempting to use register {register} at timestamp {timestamp}, \
         but the register as already replaced by {already_replaced_register} at
         timestamp {already_replaced_timestamp}"
      ),
    }
  }
}
impl Error for CompilationError {}
