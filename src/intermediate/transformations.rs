use std::{collections::HashMap, rc::Rc};

use crate::{
  ConstIndex, GeneralizedBlock, GeneralizedValue, Instruction, RegisterIndex,
  Value,
};

use super::ir_instructions::{SSABlock, VirtualRegister};

pub type RegisterLifetimes = HashMap<VirtualRegister, (usize, usize)>;

use GeneralizedValue::*;
use Instruction::*;

pub fn calculate_lifetimes<M>(
  block: SSABlock<M>,
) -> SSABlock<RegisterLifetimes> {
  block.replace_metadata(&|instructions, constants, _| {
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
    lifetimes
  })
}
