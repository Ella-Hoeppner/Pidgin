use crate::{
  blocks::GenericBlock,
  compiler::{
    intermediate::get_max_ssa_register, SSABlock, SSAInstruction, SSARegister,
  },
  instructions::GenericInstruction::*,
  runtime::{core_functions::CoreFnId, data::GenericValue::*},
};

use super::{
  error::IntermediateCompilationError,
  lifetimes::{calculate_register_lifetimes, Lifetimes},
};

pub fn inline_core_fn_calls<M: Clone>(
  block: SSABlock<M>,
) -> Result<SSABlock<Lifetimes>, IntermediateCompilationError> {
  block.translate(&|preallocated_registers,
                    mut instructions,
                    constants,
                    _|
   -> Result<_, IntermediateCompilationError> {
    let final_lifetimes = loop {
      let lifetimes =
        calculate_register_lifetimes(preallocated_registers, &instructions)?;
      let mut modified = false;
      for (timestamp, instruction) in instructions.iter().enumerate() {
        if let Call(target, f_register, arg_count) = instruction {
          if let Some(f_creation_timestamp) = lifetimes[f_register].creation {
            if let Const(_, const_index) =
              instructions[f_creation_timestamp as usize]
            {
              if let CoreFn(fn_id) = constants[const_index as usize] {
                use CoreFnId as F;
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
                if let Some(replacement_instructions) = match fn_id {
                  F::CreateList => Some(if *arg_count == 0 {
                    vec![EmptyList(*target)]
                  } else {
                    let max_register = get_max_ssa_register(
                      preallocated_registers,
                      &instructions,
                    );
                    let mut list_instructions =
                      vec![EmptyList(max_register + 1)];
                    for i in 0..*arg_count as usize - 1 {
                      list_instructions.push(Push(
                        (max_register + i + 1, max_register + i + 2),
                        args[i],
                      ));
                    }
                    list_instructions.push(Push(
                      (max_register + *arg_count as usize, *target),
                      args[*arg_count as usize - 1],
                    ));
                    list_instructions
                  }),
                  _ => match args.len() {
                    0 => {
                      if let Some(nullary_instruction) = match fn_id {
                        F::CreateList => Some(EmptyList(*target)),
                        _ => None,
                      } {
                        Some(vec![nullary_instruction])
                      } else {
                        None
                      }
                    }
                    1 => {
                      if let Some(nonreplacing_unary_instruction) = match fn_id
                      {
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
                      if let Some(nonreplacing_binary_instruction) = match fn_id
                      {
                        F::Add => Some(Add(*target, args[0], args[1])),
                        F::Subtract => {
                          Some(Subtract(*target, args[0], args[1]))
                        }
                        F::Multiply => {
                          Some(Multiply(*target, args[0], args[1]))
                        }
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
                      if let Some(instruction_builder) =
                        maybe_instruction_builder
                      {
                        let first_free_register = get_max_ssa_register(
                          preallocated_registers,
                          &instructions,
                        ) + 1;
                        let mut new_instructions = vec![instruction_builder(
                          first_free_register,
                          args[0],
                          args[1],
                        )];
                        if arg_count == 3 {
                          new_instructions.push(instruction_builder(
                            *target,
                            first_free_register,
                            args[2],
                          ));
                        } else {
                          for i in 0..(arg_count - 3) {
                            new_instructions.push(instruction_builder(
                              first_free_register + i + 1,
                              first_free_register + i,
                              args[i + 2],
                            ))
                          }
                          new_instructions.push(instruction_builder(
                            *target,
                            first_free_register + arg_count - 3,
                            args[arg_count - 1],
                          ))
                        }
                        Some(new_instructions)
                      } else {
                        None
                      }
                    }
                  },
                } {
                  let _ = instructions
                    .splice(
                      timestamp..(timestamp + 1 + *arg_count as usize),
                      replacement_instructions,
                    )
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

#[cfg(test)]
mod tests {
  use block_macros::ssa_block;
  use std::fmt::Debug;
  use std::rc::Rc;

  use crate::{
    compiler::{
      intermediate::{
        cleanup::erase_unused_constants, core_inlining::inline_core_fn_calls,
      },
      SSABlock,
    },
    instructions::GenericInstruction::*,
    runtime::core_functions::CoreFnId,
    runtime::data::GenericValue::*,
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
      Add(4, 5, 2),
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
      Add(7, 6, 2),
      Add(5, 7, 3),
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
      Multiply(7, 6, 2),
      Multiply(5, 7, 3),
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
