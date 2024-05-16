pub mod cleanup;
pub mod core_inlining;
pub mod lifetimes;
pub mod register_allocation;

use super::{SSAInstruction, SSARegister};

type InstructionTimestamp = u16;

fn get_max_register(
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
