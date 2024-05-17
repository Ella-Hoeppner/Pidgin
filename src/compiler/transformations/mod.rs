pub mod cleanup;
pub mod core_inlining;
pub mod lifetimes;
pub mod register_allocation;

use crate::runtime::control::Block;

use self::{
  cleanup::erase_unused_constants, core_inlining::inline_core_fn_calls,
  lifetimes::track_register_lifetimes, register_allocation::allocate_registers,
};

use super::{error::CompilationError, SSABlock, SSAInstruction, SSARegister};

pub(crate) type InstructionTimestamp = u16;

fn get_max_ssa_register(
  preallocated_registers: u8,
  instructions: &Vec<SSAInstruction>,
) -> SSARegister {
  let mut max_register =
    preallocated_registers.checked_sub(1).unwrap_or(0) as SSARegister;
  for usage in instructions.iter().map(|instruction| instruction.usages()) {
    for input in usage.inputs {
      max_register = max_register.max(input)
    }
    for output in usage.outputs {
      max_register = max_register.max(output)
    }
    for (old, new) in usage.replacements {
      max_register = max_register.max(old).max(new)
    }
  }
  max_register
}

pub(crate) fn raw_ir_to_bytecode(
  raw_ir: SSABlock<()>,
) -> Result<Block, CompilationError> {
  allocate_registers(track_register_lifetimes(erase_unused_constants(
    inline_core_fn_calls(raw_ir)?,
  )?)?)
}
