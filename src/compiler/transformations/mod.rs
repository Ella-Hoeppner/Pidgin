use std::{
  collections::{HashMap, HashSet},
  error::Error,
  fmt::Display,
  rc::Rc,
};

pub mod cleanup;
pub mod core_inlining;
pub mod lifetimes;
pub mod register_allocation;

use crate::{
  blocks::GenericBlock, instructions, Block, ConstIndex, GenericValue,
  Instruction, Register, Value,
};

use self::lifetimes::{LifetimeError, Lifetimes, RegisterLifetime};

use super::{SSABlock, SSAInstruction, SSARegister, SSAValue};
use Instruction::*;

use lifetimes::calculate_register_lifetimes;

type InstructionTimestamp = u16;
use crate::runtime::core_functions::CoreFnId;
use GenericValue::*;

fn get_max_register(
  preallocated_registers: u8,
  instructions: &Vec<SSAInstruction>,
) -> SSARegister {
  let mut max_register: SSARegister = 0;
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
