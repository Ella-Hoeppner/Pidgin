use std::{
  collections::{HashMap, HashSet},
  error::Error,
  fmt::Display,
  rc::Rc,
};

use crate::{
  blocks::GenericBlock, instructions, Block, ConstIndex, GenericValue,
  Instruction, Register, Value,
};

use super::{SSABlock, SSAInstruction, SSARegister, SSAValue};
use Instruction::*;

type InstructionTimestamp = u16;
use crate::runtime::core_functions::CoreFnId;
use GenericValue::*;
use Instruction::*;

#[derive(Clone, Debug)]
pub enum LifetimeError {
  UsedBeforeCreation(SSARegister, InstructionTimestamp),
  OutputToExisting(
    SSARegister,
    Option<InstructionTimestamp>,
    InstructionTimestamp,
  ),
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
         {created_timestamp:?}"
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
    }
  }
}
impl Error for LifetimeError {}

#[derive(Clone, Debug)]
pub struct RegisterLifetime {
  creation: Option<InstructionTimestamp>,
  usages: Vec<InstructionTimestamp>,
  replacing: Option<SSARegister>,
  replaced_by: Option<SSARegister>,
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
  fn last_usage(&self) -> Option<InstructionTimestamp> {
    self.usages.last().cloned()
  }
  fn is_used(&self) -> bool {
    !self.usages.is_empty()
  }
}
type Lifetimes = HashMap<SSARegister, RegisterLifetime>;

fn calculate_register_lifetimes<M>(
  preallocated_registers: u8,
  instructions: &Vec<SSAInstruction>,
  constants: &Vec<SSAValue<M>>,
) -> Result<Lifetimes, LifetimeError> {
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
          return Err(LifetimeError::UsedAfterReplacement(
            input_register,
            timestamp,
            replaced_by,
            lifetime.last_usage().unwrap(),
          ));
        }
        lifetime.usages.push(timestamp);
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
            from_lifetime.last_usage().unwrap(),
          ));
        } else {
          from_lifetime.usages.push(timestamp);
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
  Ok(lifetimes)
}

pub fn track_register_lifetimes<M: Clone>(
  block: SSABlock<M>,
) -> Result<SSABlock<Lifetimes>, LifetimeError> {
  block.translate(&|preallocated_registers, instructions, constants, _| {
    let lifetimes = calculate_register_lifetimes(
      preallocated_registers,
      &instructions,
      &constants,
    )?;
    Ok(GenericBlock::new_with_metadata(
      instructions,
      constants,
      lifetimes,
    ))
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

pub fn inline_core_fn_calls<M: Clone>(
  block: SSABlock<M>,
) -> Result<SSABlock<Lifetimes>, LifetimeError> {
  block.translate(&|preallocated_registers,
                    mut instructions,
                    mut constants,
                    mut lifetimes|
   -> Result<_, LifetimeError> {
    let final_lifetimes = loop {
      let lifetimes = calculate_register_lifetimes(
        preallocated_registers,
        &instructions,
        &constants,
      )?;
      let mut modified = false;
      for (timestamp, instruction) in instructions.iter().enumerate() {
        if let Call(target, f_register, arg_count) = instruction {
          if let Some(f_creation_timestamp) = lifetimes[f_register].creation {
            if let Const(f_register, const_index) =
              instructions[f_creation_timestamp as usize]
            {
              if let CoreFn(fn_id) = constants[const_index as usize] {
                use CoreFnId as F;
                use SSAInstruction as I;
                let args: Vec<_> = ((timestamp + 1)
                  ..(timestamp + 1 + *arg_count as usize))
                  .map(|instruction_index| {
                    match instructions[instruction_index] {
                      CopyArgument(arg) => arg,
                      StealArgument(arg) => arg,
                      _ => panic!(
                        "didn't find CopyArgument or StealArgument after \
                         Call in inline_core_fn_calls"
                      ),
                    }
                  })
                  .collect();
                if let Some(y) = match args.len() {
                  0 => todo!(),
                  1 => {
                    if let Some(nonreplacing_unary_instruction) = match fn_id {
                      F::First => Some(First(*target, args[0])),
                      F::Last => Some(Last(*target, args[0])),
                      F::IsEmpty => Some(IsEmpty(*target, args[0])),
                      _ => None,
                    } {
                      Some(vec![nonreplacing_unary_instruction])
                    } else if let Some(replacing_unary_instruction) =
                      match fn_id {
                        F::Rest => Some(Rest((args[0], *target))),
                        F::ButLast => Some(ButLast((args[0], *target))),
                        _ => None,
                      }
                    {
                      Some(vec![replacing_unary_instruction])
                    } else {
                      None
                    }
                  }
                  2 => {
                    if let Some(nonreplacing_binary_instruction) = match fn_id {
                      F::Add => Some(Add(*target, args[0], args[1])),
                      F::Subtract => Some(Subtract(*target, args[0], args[1])),
                      F::Multiply => Some(Multiply(*target, args[0], args[1])),
                      F::Divide => Some(Divide(*target, args[0], args[1])),
                      _ => None,
                    } {
                      Some(vec![nonreplacing_binary_instruction])
                    } else if let Some(replacing_binary_instruction) =
                      match fn_id {
                        F::Push => Some(Push((args[0], *target), args[1])),
                        F::Cons => Some(Cons((args[0], *target), args[1])),
                        _ => None,
                      }
                    {
                      Some(vec![replacing_binary_instruction])
                    } else {
                      None
                    }
                  }
                  arg_count => {
                    let maybe_instruction_builder: Option<
                      fn(
                        SSARegister,
                        SSARegister,
                        SSARegister,
                      ) -> SSAInstruction,
                    > = match fn_id {
                      F::Add => Some(|a, b, c| Add(a, b, c)),
                      F::Multiply => Some(|a, b, c| Multiply(a, b, c)),
                      _ => None,
                    };
                    if let Some(instruction_builder) = maybe_instruction_builder
                    {
                      let first_free_register =
                        get_max_register(preallocated_registers, &instructions)
                          + 1;
                      let mut new_instructions = vec![instruction_builder(
                        first_free_register,
                        args[0],
                        args[1],
                      )];
                      if arg_count == 3 {
                        new_instructions.push(instruction_builder(
                          *target,
                          args[2],
                          first_free_register,
                        ));
                      } else {
                        for i in (0..(arg_count - 3)) {
                          new_instructions.push(instruction_builder(
                            first_free_register + i + 1,
                            args[i + 2],
                            first_free_register + i,
                          ))
                        }
                        new_instructions.push(instruction_builder(
                          *target,
                          args[arg_count - 1],
                          first_free_register + arg_count - 3,
                        ))
                      }
                      Some(new_instructions)
                    } else {
                      None
                    }
                  }
                } {
                  instructions
                    .splice(timestamp..(timestamp + 1 + *arg_count as usize), y)
                    .collect::<Vec<_>>();
                  modified = true;
                  break;
                }
              }
            }
          }
        }
      }
      if !modified {
        break lifetimes;
      }
    };
    Ok(GenericBlock::new_with_metadata(
      instructions,
      constants,
      final_lifetimes,
    ))
  })
}

pub fn erase_unused_constants<M: Clone>(
  block: SSABlock<M>,
) -> Result<SSABlock<()>, LifetimeError> {
  block.translate(&|preallocated_registers,
                    mut instructions,
                    mut constants,
                    _| {
    let lifetimes = calculate_register_lifetimes(
      preallocated_registers,
      &instructions,
      &constants,
    )?;
    let mut filtered_instructions = vec![];
    let mut filtered_constants = vec![];
    for (timestamp, instruction) in instructions.into_iter().enumerate() {
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
    Ok(GenericBlock::new(filtered_instructions, filtered_constants))
  })
}

mod tests {
  use program_macro::{block, ssa_block};
  use std::fmt::Debug;
  use std::rc::Rc;

  use crate::{
    blocks::GenericBlock,
    compiler::{
      ast_to_ir::expression_ast_to_ir,
      parse::parse_sexp,
      transformations::{
        allocate_registers, erase_unused_constants, inline_core_fn_calls,
        track_register_lifetimes,
      },
      SSABlock,
    },
    runtime::core_functions::CoreFnId,
    Block, EvaluationState,
    GenericValue::{self, *},
    Instruction::*,
    Num::{self, *},
    Value,
  };

  fn debug_string<T: Debug>(x: &T) -> String {
    format!("{:?}", x)
  }

  #[test]
  fn inline_binary_addition() {
    let raw_ir = ssa_block![
      Const(0, 1),
      Const(1, 2),
      Const(2, CoreFn(CoreFnId::Add)),
      Call(3, 2, 2),
      CopyArgument(0),
      CopyArgument(1),
      Return(3)
    ];
    let inlined_ir =
      erase_unused_constants(inline_core_fn_calls(raw_ir).unwrap()).unwrap();
    let expected_inlined_ir =
      ssa_block![Const(0, 1), Const(1, 2), Add(3, 0, 1), Return(3)];
    assert_eq!(
      debug_string(&(inlined_ir.instructions, inlined_ir.constants)),
      debug_string(&(
        expected_inlined_ir.instructions,
        expected_inlined_ir.constants
      ))
    );
  }

  #[test]
  fn inline_trinary_addition() {
    let raw_ir = ssa_block![
      Const(0, 1),
      Const(1, 2),
      Const(2, 3),
      Const(3, CoreFn(CoreFnId::Add)),
      Call(4, 3, 3),
      CopyArgument(0),
      CopyArgument(1),
      CopyArgument(2),
      Return(4)
    ];
    let inlined_ir =
      erase_unused_constants(inline_core_fn_calls(raw_ir).unwrap()).unwrap();
    let expected_inlined_ir = ssa_block![
      Const(0, 1),
      Const(1, 2),
      Const(2, 3),
      Add(5, 0, 1),
      Add(4, 2, 5),
      Return(4)
    ];
    assert_eq!(
      debug_string(&(inlined_ir.instructions, inlined_ir.constants)),
      debug_string(&(
        expected_inlined_ir.instructions,
        expected_inlined_ir.constants
      ))
    );
  }

  #[test]
  fn inline_quaternary_addition() {
    let raw_ir = ssa_block![
      Const(0, 1),
      Const(1, 2),
      Const(2, 3),
      Const(3, 4),
      Const(4, CoreFn(CoreFnId::Add)),
      Call(5, 4, 4),
      CopyArgument(0),
      CopyArgument(1),
      CopyArgument(2),
      CopyArgument(3),
      Return(5)
    ];
    let inlined_ir =
      erase_unused_constants(inline_core_fn_calls(raw_ir).unwrap()).unwrap();
    let expected_inlined_ir = ssa_block![
      Const(0, 1),
      Const(1, 2),
      Const(2, 3),
      Const(3, 4),
      Add(6, 0, 1),
      Add(7, 2, 6),
      Add(5, 3, 7),
      Return(5)
    ];
    assert_eq!(
      debug_string(&(inlined_ir.instructions, inlined_ir.constants)),
      debug_string(&(
        expected_inlined_ir.instructions,
        expected_inlined_ir.constants
      ))
    );
  }

  #[test]
  fn inline_quaternary_multiplication() {
    let raw_ir = ssa_block![
      Const(0, 1),
      Const(1, 2),
      Const(2, 3),
      Const(3, 4),
      Const(4, CoreFn(CoreFnId::Multiply)),
      Call(5, 4, 4),
      CopyArgument(0),
      CopyArgument(1),
      CopyArgument(2),
      CopyArgument(3),
      Return(5)
    ];
    let inlined_ir =
      erase_unused_constants(inline_core_fn_calls(raw_ir).unwrap()).unwrap();
    let expected_inlined_ir = ssa_block![
      Const(0, 1),
      Const(1, 2),
      Const(2, 3),
      Const(3, 4),
      Multiply(6, 0, 1),
      Multiply(7, 2, 6),
      Multiply(5, 3, 7),
      Return(5)
    ];
    assert_eq!(
      debug_string(&(inlined_ir.instructions, inlined_ir.constants)),
      debug_string(&(
        expected_inlined_ir.instructions,
        expected_inlined_ir.constants
      ))
    );
  }

  #[test]
  fn inline_push() {
    let raw_ir = ssa_block![
      EmptyList(0),
      Const(1, 5),
      Const(2, CoreFn(CoreFnId::Push)),
      Call(3, 2, 2),
      CopyArgument(0),
      CopyArgument(1),
      Return(3)
    ];
    let inlined_ir =
      erase_unused_constants(inline_core_fn_calls(raw_ir).unwrap()).unwrap();
    let expected_inlined_ir =
      ssa_block![EmptyList(0), Const(1, 5), Push((0, 3), 1), Return(3)];
    assert_eq!(
      debug_string(&(inlined_ir.instructions, inlined_ir.constants)),
      debug_string(&(
        expected_inlined_ir.instructions,
        expected_inlined_ir.constants
      ))
    );
  }

  #[test]
  fn inline_first() {
    let raw_ir = ssa_block![
      Const(0, List(Rc::new(vec![1.into(), 2.into(), 3.into()]))),
      Const(1, CoreFn(CoreFnId::First)),
      Call(2, 1, 1),
      CopyArgument(0),
      Return(2)
    ];
    let inlined_ir =
      erase_unused_constants(inline_core_fn_calls(raw_ir).unwrap()).unwrap();
    let expected_inlined_ir = ssa_block![
      Const(0, List(Rc::new(vec![1.into(), 2.into(), 3.into()]))),
      First(2, 0),
      Return(2)
    ];
    assert_eq!(
      debug_string(&(inlined_ir.instructions, inlined_ir.constants)),
      debug_string(&(
        expected_inlined_ir.instructions,
        expected_inlined_ir.constants
      ))
    );
  }

  #[test]
  fn inline_rest() {
    let raw_ir = ssa_block![
      Const(0, List(Rc::new(vec![1.into(), 2.into(), 3.into()]))),
      Const(1, CoreFn(CoreFnId::Rest)),
      Call(2, 1, 1),
      CopyArgument(0),
      Return(2)
    ];
    let inlined_ir =
      erase_unused_constants(inline_core_fn_calls(raw_ir).unwrap()).unwrap();
    let expected_inlined_ir = ssa_block![
      Const(0, List(Rc::new(vec![1.into(), 2.into(), 3.into()]))),
      Rest((0, 2)),
      Return(2)
    ];
    assert_eq!(
      debug_string(&(inlined_ir.instructions, inlined_ir.constants)),
      debug_string(&(
        expected_inlined_ir.instructions,
        expected_inlined_ir.constants
      ))
    );
  }
}
