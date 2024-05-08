use std::{collections::HashMap, rc::Rc};

use crate::{ConstIndex, Instruction, InstructionBlock, Value};

use super::ir_instructions::VirtualRegister;

pub type RegisterLifetimes = HashMap<VirtualRegister, (usize, usize)>;

pub fn calculate_lifetimes<C, M>(
  block: InstructionBlock<VirtualRegister, C, M>,
) -> InstructionBlock<VirtualRegister, C, RegisterLifetimes> {
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

fn inline_constants_inner<R: Clone, M>(
  existing_constants: Vec<Value>,
  instructions: Rc<[Instruction<R, Value>]>,
) -> (Vec<Value>, Vec<Instruction<R, ConstIndex>>) {
  let instructions = (*instructions).to_vec();
  instructions.into_iter().fold(
    (existing_constants, vec![]),
    |(existing_constants, processed_instructions), instruction| {
      match instruction {
        Instruction::Const(_, _) => todo!(),
        _ => todo!(),
      }
    },
  )
}

pub fn inline_constants<R: Clone, M>(
  block: InstructionBlock<R, Value, M>,
) -> (Vec<Value>, InstructionBlock<R, ConstIndex, M>) {
  let metadata = block.metadata;
  let (constants, new_instructions) =
    inline_constants_inner::<R, M>(vec![], block.instructions);
  let new_instruction_block: InstructionBlock<R, ConstIndex, ()> =
    new_instructions.into();
  (constants, new_instruction_block.with_metadata(metadata))
}
