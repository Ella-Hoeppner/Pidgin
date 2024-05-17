use crate::{compiler::SSABlock, instructions::GenericInstruction::*};

use super::{error::IntermediateCompilationError, lifetimes::calculate_register_lifetimes};

pub fn erase_unused_constants<M: Clone>(
  block: SSABlock<M>,
) -> Result<SSABlock<()>, IntermediateCompilationError> {
  block.translate(&|preallocated_registers, instructions, constants, _| {
    let lifetimes =
      calculate_register_lifetimes(preallocated_registers, &instructions)?;
    let mut filtered_instructions = vec![];
    let mut filtered_constants = vec![];
    for instruction in instructions.into_iter() {
      if let Const(target, const_index) = instruction {
        if lifetimes[&target].is_used() {
          filtered_instructions
            .push(Const(target, filtered_constants.len() as u16));
          filtered_constants.push(constants[const_index as usize].clone());
        }
      } else {
        filtered_instructions.push(instruction)
      }
    }
    Ok(SSABlock::new(filtered_instructions, filtered_constants))
  })
}
