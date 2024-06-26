use std::collections::HashMap;

use crate::{
  blocks::GenericBlock,
  compiler::{SSABlock, SSAInstruction, SSARegister},
};

use super::{
  error::{IntermediateCompilationError, IntermediateCompilationResult},
  InstructionTimestamp,
};

#[derive(Clone, Debug)]
pub struct RegisterLifetime {
  pub(crate) creation: Option<InstructionTimestamp>,
  usages: Vec<InstructionTimestamp>,
  pub(crate) replacing: Option<SSARegister>,
  pub(crate) replaced_by: Option<SSARegister>,
}
impl RegisterLifetime {
  fn new_preexisting() -> Self {
    Self {
      creation: None,
      usages: vec![],
      replacing: None,
      replaced_by: None,
    }
  }
  fn new(creation_timestamp: InstructionTimestamp) -> Self {
    Self {
      creation: Some(creation_timestamp),
      usages: vec![],
      replacing: None,
      replaced_by: None,
    }
  }
  fn new_replacing(
    creation_timestamp: InstructionTimestamp,
    replacing: SSARegister,
  ) -> Self {
    Self {
      creation: Some(creation_timestamp),
      usages: vec![],
      replacing: Some(replacing),
      replaced_by: None,
    }
  }
  pub(crate) fn last_usage(&self) -> Option<InstructionTimestamp> {
    self.usages.last().cloned()
  }
  pub(crate) fn is_used(&self) -> bool {
    !self.usages.is_empty()
  }
}
pub(crate) type Lifetimes = HashMap<SSARegister, RegisterLifetime>;

pub(crate) fn calculate_register_lifetimes(
  preallocated_registers: u8,
  instructions: &Vec<SSAInstruction>,
) -> IntermediateCompilationResult<Lifetimes> {
  let mut lifetimes: Lifetimes = Lifetimes::new();
  for preallocated_register in 0..preallocated_registers {
    lifetimes.insert(
      preallocated_register as SSARegister,
      RegisterLifetime::new_preexisting(),
    );
  }
  for (timestamp, instruction) in instructions.iter().enumerate() {
    let timestamp = timestamp as InstructionTimestamp;
    let usages = instruction.usages();
    for input_register in usages.inputs {
      if let Some(lifetime) = lifetimes.get_mut(&input_register) {
        if let Some(replaced_by) = lifetime.replaced_by {
          return Err(IntermediateCompilationError::UsedAfterReplacement(
            input_register,
            timestamp,
            replaced_by,
            lifetime.last_usage().unwrap(),
          ));
        }
        lifetime.usages.push(timestamp);
      } else {
        return Err(IntermediateCompilationError::UsedBeforeCreation(
          input_register,
          timestamp,
        ));
      }
    }
    for output_register in usages.outputs {
      if let Some(existing_lifetime) = lifetimes.get(&output_register) {
        return Err(IntermediateCompilationError::OutputToExisting(
          output_register,
          existing_lifetime.creation,
          timestamp,
        ));
      } else {
        lifetimes.insert(output_register, RegisterLifetime::new(timestamp));
      }
    }
    for (from_register, to_register) in usages.replacements {
      if let Some(from_lifetime) = lifetimes.get_mut(&from_register) {
        if let Some(replaced_by_register) = from_lifetime.replaced_by {
          return Err(IntermediateCompilationError::UsedAfterReplacement(
            from_register,
            timestamp,
            replaced_by_register,
            from_lifetime.last_usage().unwrap(),
          ));
        } else {
          from_lifetime.usages.push(timestamp);
          from_lifetime.replaced_by = Some(to_register);
        }
      } else {
        return Err(IntermediateCompilationError::ReplacingNonexistent(
          from_register,
          timestamp,
        ));
      }
      if let Some(to_lifetime) = lifetimes.get(&to_register) {
        return Err(IntermediateCompilationError::OutputToExisting(
          to_register,
          to_lifetime.creation,
          timestamp,
        ));
      } else {
        lifetimes.insert(
          to_register,
          RegisterLifetime::new_replacing(timestamp, from_register),
        );
      }
    }
  }
  Ok(lifetimes)
}

pub fn track_register_lifetimes<M: Clone>(
  block: SSABlock<M>,
) -> Result<SSABlock<Lifetimes>, IntermediateCompilationError> {
  block.translate(&|preallocated_registers, instructions, constants, _| {
    let lifetimes =
      calculate_register_lifetimes(preallocated_registers, &instructions)?;
    Ok(GenericBlock::new_with_metadata(
      instructions,
      constants,
      lifetimes,
    ))
  })
}
