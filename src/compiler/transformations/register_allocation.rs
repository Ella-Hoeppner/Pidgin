use std::{
  collections::{HashMap, HashSet},
  error::Error,
  fmt::Display,
};

use crate::{
  blocks::GenericBlock,
  compiler::{SSABlock, SSARegister},
  runtime::{control::Block, vm::Register},
};

use super::lifetimes::Lifetimes;

#[derive(Clone, Debug)]
pub enum RegisterAllocationError {}
impl Display for RegisterAllocationError {
  fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match *self {}
  }
}
impl Error for RegisterAllocationError {}

pub fn allocate_registers(
  block: SSABlock<Lifetimes>,
) -> Result<Block, RegisterAllocationError> {
  block.translate(&|preallocated_registers,
                    instructions,
                    constants,
                    lifetimes| {
    Ok(GenericBlock::new_with_metadata(
      {
        let mut ssa_to_runtime_registers: HashMap<SSARegister, Register> =
          HashMap::new();
        let mut taken_runtime_registers: HashSet<Register> = HashSet::new();
        for preallocated_register in 0..preallocated_registers {
          ssa_to_runtime_registers
            .insert(preallocated_register as usize, preallocated_register);
          taken_runtime_registers.insert(preallocated_register);
        }
        let mut translated_instructions = vec![];
        for (timestamp, instruction) in instructions.iter().enumerate() {
          let timestamp = timestamp as u16;
          let mut finished_ssa_to_runtime_registers: HashMap<
            SSARegister,
            Register,
          > = HashMap::new();
          let finished_ssa_registers: Vec<SSARegister> =
            ssa_to_runtime_registers
              .iter()
              .filter_map(|(ssa_register, _)| {
                let lifetime = lifetimes.get(&ssa_register).unwrap();
                if lifetime.replaced_by.is_none() {
                  lifetime
                    .last_usage()
                    .map(|last_usage| {
                      if last_usage == timestamp {
                        Some(*ssa_register)
                      } else {
                        None
                      }
                    })
                    .flatten()
                } else {
                  None
                }
              })
              .collect();
          for finished_ssa_register in finished_ssa_registers {
            let finised_runtime_register = ssa_to_runtime_registers
              .remove(&finished_ssa_register)
              .unwrap();
            finished_ssa_to_runtime_registers
              .insert(finished_ssa_register, finised_runtime_register);
            let removed =
              taken_runtime_registers.remove(&finised_runtime_register);
            #[cfg(debug_assertions)]
            assert!(removed)
          }
          for (ssa_registser, register_lifetime) in lifetimes.iter() {
            if register_lifetime.creation == Some(timestamp)
              && (timestamp != 0
                || *ssa_registser >= preallocated_registers as usize)
            {
              if let Some(replaced_ssa_registser) = register_lifetime.replacing
              {
                let register = ssa_to_runtime_registers
                  .remove(&replaced_ssa_registser)
                  .expect("Didn't find register when trying to replace");
                ssa_to_runtime_registers.insert(*ssa_registser, register);
              } else {
                let min_unused_register = (0..Register::MAX)
                  .filter(|i| !taken_runtime_registers.contains(i))
                  .next()
                  .expect("Failed to find unused register");
                let replaced_register = ssa_to_runtime_registers
                  .insert(*ssa_registser, min_unused_register);
                #[cfg(debug_assertions)]
                assert!(replaced_register.is_none());
                let register_free =
                  taken_runtime_registers.insert(min_unused_register);
                #[cfg(debug_assertions)]
                assert!(register_free);
              }
            }
          }
          translated_instructions.push(instruction.clone().translate(
            |input: usize| -> u8 {
              *ssa_to_runtime_registers
                .get(&input)
                .or_else(|| finished_ssa_to_runtime_registers.get(&input))
                .unwrap_or_else(|| {
                  panic!(
                    "no current real register found for input ssa register \
                     {input} at timestamp {timestamp}"
                  )
                })
            },
            |output: usize| -> u8 {
              *ssa_to_runtime_registers.get(&output).unwrap_or_else(|| {
                panic!(
                  "no current real register found for output ssa register \
                   {output} at timestamp {timestamp}"
                )
              })
            },
            |(input, output): (usize, usize)| -> u8 {
              *ssa_to_runtime_registers.get(&output).unwrap_or_else(|| {
                panic!(
                  "no current real register found for replacable ssa register \
                   ({input} => {output}) at timestamp {timestamp}\n"
                )
              })
            },
          ));
        }
        translated_instructions
      },
      constants,
      (),
    ))
  })
}
