use std::{
  collections::{HashMap, HashSet},
  error::Error,
  fmt::Display,
  rc::Rc,
};

use crate::{
  blocks::GenericBlock, Block, ConstIndex, GenericValue, Instruction, Register,
  Value,
};

use super::{SSABlock, SSAInstruction, SSARegister};

type InstructionTimestamp = u16;
use GenericValue::*;
use Instruction::*;

#[derive(Clone, Debug)]
pub enum LifetimeError {
  UsedBeforeCreation(SSARegister, InstructionTimestamp),
  OutputToExisting(SSARegister, InstructionTimestamp, InstructionTimestamp),
  ReplacingNonexistent(SSARegister, InstructionTimestamp),
  UsedAfterReplacement(
    SSARegister,
    InstructionTimestamp,
    SSARegister,
    InstructionTimestamp,
  ),
  ReplacingAfterReplacement(
    SSARegister,
    InstructionTimestamp,
    SSARegister,
    InstructionTimestamp,
  ),
  Unused(SSARegister, InstructionTimestamp),
}
impl Display for LifetimeError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use LifetimeError::*;
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
         {created_timestamp}"
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
      ReplacingAfterReplacement(
        register,
        timestamp,
        already_replaced_register,
        already_replaced_timestamp,
      ) => write!(
        f,
        "attempting to replace register {register} at timestamp {timestamp}, \
         but the register as already replaced by {already_replaced_register} at
         timestamp {already_replaced_timestamp}"
      ),
      Unused(register, timestamp) => {
        write!(
          f,
          "register {register}, created at timestamp {timestamp}, is never used"
        )
      }
    }
  }
}
impl Error for LifetimeError {}

#[derive(Clone, Debug)]
pub struct RegisterLifetime {
  creation: InstructionTimestamp,
  last_usage: Option<InstructionTimestamp>,
  replacing: Option<SSARegister>,
  replaced_by: Option<SSARegister>,
}
impl RegisterLifetime {
  fn new(creation_timestamp: InstructionTimestamp) -> Self {
    Self {
      creation: creation_timestamp,
      last_usage: None,
      replacing: None,
      replaced_by: None,
    }
  }
  fn new_replacing(
    creation_timestamp: InstructionTimestamp,
    replacing: SSARegister,
  ) -> Self {
    Self {
      creation: creation_timestamp,
      last_usage: None,
      replacing: Some(replacing),
      replaced_by: None,
    }
  }
}

pub fn track_register_lifetimes<M>(
  block: SSABlock<M>,
) -> Result<SSABlock<HashMap<SSARegister, RegisterLifetime>>, LifetimeError> {
  block.replace_metadata(0, &|preallocated_registers, instructions, _, _| {
    let mut lifetimes: HashMap<SSARegister, RegisterLifetime> = HashMap::new();
    for preallocated_register in 0..preallocated_registers {
      lifetimes.insert(
        preallocated_register as SSARegister,
        RegisterLifetime::new(0),
      );
    }
    for (timestamp, instruction) in instructions.iter().enumerate() {
      let timestamp = timestamp as InstructionTimestamp;
      let usages = instruction.register_lifetime_constraints();
      for input_register in usages.inputs {
        if let Some(lifetime) = lifetimes.get_mut(&input_register) {
          if let Some(replaced_by) = lifetime.replaced_by {
            return Err(LifetimeError::UsedAfterReplacement(
              input_register,
              timestamp,
              replaced_by,
              lifetime.last_usage.unwrap(),
            ));
          }
          lifetime.last_usage = Some(timestamp);
        } else {
          return Err(LifetimeError::UsedBeforeCreation(
            input_register,
            timestamp,
          ));
        }
      }
      for output_register in usages.outputs {
        if let Some(existing_lifetime) = lifetimes.get(&output_register) {
          return Err(LifetimeError::OutputToExisting(
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
            return Err(LifetimeError::UsedAfterReplacement(
              from_register,
              timestamp,
              replaced_by_register,
              from_lifetime.last_usage.unwrap(),
            ));
          } else {
            from_lifetime.last_usage = Some(timestamp);
            from_lifetime.replaced_by = Some(to_register);
          }
        } else {
          return Err(LifetimeError::ReplacingNonexistent(
            from_register,
            timestamp,
          ));
        }
        if let Some(to_lifetime) = lifetimes.get(&to_register) {
          return Err(LifetimeError::OutputToExisting(
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
    for (register, lifetime) in lifetimes.iter() {
      if lifetime.last_usage.is_none() {
        return Err(LifetimeError::Unused(*register, lifetime.creation));
      }
    }
    Ok(lifetimes)
  })
}

#[derive(Clone, Debug)]
pub enum RegisterAllocationError {}
impl Display for RegisterAllocationError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match *self {}
  }
}
impl Error for RegisterAllocationError {}

pub fn allocate_registers(
  block: SSABlock<HashMap<SSARegister, RegisterLifetime>>,
) -> Result<Block, RegisterAllocationError> {
  block.translate_instructions(
    0,
    &|preallocated_registers, instructions, lifetimes| {
      Ok((
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
                      .last_usage
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
              if register_lifetime.creation == timestamp
                && (timestamp != 0
                  || *ssa_registser >= preallocated_registers as usize)
              {
                if let Some(replaced_ssa_registser) =
                  register_lifetime.replacing
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
        (),
      ))
    },
  )
}

#[derive(Clone, Debug)]
pub enum CompilationError {
  Lifetime(LifetimeError),
  RegisterAllocation(RegisterAllocationError),
}
impl Display for CompilationError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      CompilationError::Lifetime(e) => write!(f, "lifetime error: {}", e),
      CompilationError::RegisterAllocation(e) => {
        write!(f, "register allocation error: {}", e)
      }
    }
  }
}
impl Error for CompilationError {}

pub fn ir_to_bytecode<M>(ir: SSABlock<M>) -> Result<Block, CompilationError> {
  let lifetime_ir =
    track_register_lifetimes(ir).map_err(|e| CompilationError::Lifetime(e))?;
  allocate_registers(lifetime_ir)
    .map_err(|e| CompilationError::RegisterAllocation(e))
}
