use std::collections::HashMap;

use crate::{
  Instruction, InstructionBlock, IntermediateInstructionBlock, Value,
};

pub type VirtualRegister = usize;

pub type RegisterLifetimes = HashMap<VirtualRegister, (usize, usize)>;

pub fn calculate_lifetimes<M>(
  block: IntermediateInstructionBlock<M>,
) -> IntermediateInstructionBlock<RegisterLifetimes> {
  let mut lifetimes = RegisterLifetimes::new();
  for (i, instruction) in block.instructions.iter().enumerate() {
    let (inputs, outputs) = instruction.input_and_output_registers();
    for input in inputs {
      lifetimes.entry(input).or_insert((i, i));
    }
    for output in outputs {
      lifetimes
        .entry(output)
        .and_modify(|span| span.1 = i)
        .or_insert((i, i));
    }
  }
  block.replace_metadata(lifetimes)
}
