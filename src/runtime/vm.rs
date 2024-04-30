use std::collections::HashMap;

use crate::Value;

use crate::runtime::instructions::Instruction;

use super::{data::Num, Error, Result};

const U16_CAPCITY: usize = u16::MAX as usize + 1;

pub type RegisterIndex = u8;
pub type SymbolIndex = u16;
pub type ConstIndex = u16;

pub struct Program {
  instructions: Vec<Instruction>,
  constants: Vec<Value>,
}
impl Program {
  pub fn new(instructions: Vec<Instruction>, constants: Vec<Value>) -> Self {
    Self {
      instructions,
      constants,
    }
  }
}

struct StackFrame {
  stack_start: u16,
  result_register: RegisterIndex,
}

struct EvaluationState {
  stack: [Value; U16_CAPCITY],
  stack_frames: Vec<StackFrame>,
  stack_consumption: u16,
  environment: HashMap<SymbolIndex, Value>,
}

impl EvaluationState {
  fn new() -> Self {
    const NIL: Value = Value::Nil;
    Self {
      stack: [NIL; U16_CAPCITY],
      stack_frames: vec![],
      stack_consumption: 0,
      environment: HashMap::new(),
    }
  }
  fn display_stack(&self) -> String {
    self
      .stack
      .iter()
      .take(self.stack_consumption as usize)
      .enumerate()
      .map(|(i, value)| format!("{}: {}", i, value.description()))
      .reduce(|a, b| a + "\n" + &b)
      .unwrap_or("".to_string())
  }
  fn display_environment(&self) -> String {
    let mut bindings: Vec<_> = self.environment.iter().collect();
    bindings.sort_by_key(|(symbol_index, _value_pointer)| **symbol_index);
    bindings
      .into_iter()
      .map(|(symbol_index, value)| {
        format!(
          "symbol_index: {}, value: {}",
          symbol_index,
          value.description()
        )
      })
      .reduce(|a, b| a + "\n" + &b)
      .unwrap_or("".to_string())
  }
  fn stack_index(&self, register: RegisterIndex) -> u16 {
    *self
      .stack_frames
      .last()
      .map(|stack_frame| &stack_frame.stack_start)
      .unwrap_or(&0)
      + register as u16
  }
  fn set_register(&mut self, register: RegisterIndex, value: Value) {
    let stack_index = self.stack_index(register);
    self.stack[stack_index as usize] = value;
    self.stack_consumption = self.stack_consumption.max(stack_index + 1);
  }
  fn get_register(&self, register: RegisterIndex) -> &Value {
    //debug
    if register as usize >= self.stack_consumption as usize {
      panic!("trying to access register that hasn't been set yet")
    }
    //
    &self.stack[self.stack_index(register) as usize]
  }
}

pub fn evaluate(program: Program) -> Result<()> {
  let mut state = EvaluationState::new();
  let mut instruction_stack = program.instructions.clone();
  instruction_stack.reverse();
  while let Some(instruction) = instruction_stack.pop() {
    use Instruction::*;
    match instruction {
      NoOp => {
        println!(
          "Instruction::NoOp called! this probably shouldn't be happening :)"
        )
      }
      Argument(_) => {
        panic!("Instruction::Argument called, this should never happen")
      }
      Clear(register_index) => state.set_register(register_index, Value::Nil),
      Const(register_index, const_index) => {
        state.set_register(
          register_index,
          program.constants[const_index as usize].clone(),
        );
      }
      Add(
        sum_register_index,
        input_register_index_1,
        input_register_index_2,
      ) => {
        let addend_1 = state.get_register(input_register_index_1);
        let addend_2 = state.get_register(input_register_index_2);
        let sum = Num::add(addend_1.as_num()?, &addend_2.as_num()?);
        state.set_register(sum_register_index, Value::Num(sum));
      }
      Multiply(
        product_register_index,
        input_register_index_1,
        input_register_index_2,
      ) => {
        let multiplicand_1 = state.get_register(input_register_index_1);
        let multiplicand_2 = state.get_register(input_register_index_2);
        let product =
          Num::multiply(multiplicand_1.as_num()?, &multiplicand_2.as_num()?);
        state.set_register(product_register_index, Value::Num(product));
      }
      Bind(symbol_index, register) => {
        state
          .environment
          .insert(symbol_index, state.get_register(register).clone());
      }
      Lookup(register, symbol_index) => {
        state.set_register(register, state.environment[&symbol_index].clone());
      }
      Apply(result_register, fn_register, args_register) => {
        let f = state.get_register(fn_register).clone();
        let arg = state.get_register(args_register).clone();
        state.stack_frames.push(StackFrame {
          stack_start: state.stack_consumption,
          result_register,
        });
        match f {
          Value::Fn(instructions) => {
            let mut x = instructions.into_iter().peekable();
            while let Some(Argument(symbol_index)) = x.peek() {
              state.environment.insert(*symbol_index, arg.clone());
              x.next();
            }
            for instruction in x.rev() {
              instruction_stack.push(instruction);
            }
          }
          _ => {
            return Err(Error::CantApply);
          }
        }
      }
      Return(return_value_register) => {
        let return_value = state.get_register(return_value_register).clone();
        let stack_frame = state.stack_frames.pop().unwrap();
        for i in stack_frame.stack_start..state.stack_consumption {
          state.stack[i as usize] = Value::Nil;
        }
        state.stack_consumption = stack_frame.stack_start;
        state.set_register(stack_frame.result_register, return_value);
      }
      DebugPrint(id) => {
        println!("DEBUG {}", id);
        println!("--------------------");
        println!(
          "stack ({}):\n{}\n",
          state.stack_consumption,
          state.display_stack()
        );
        println!("environment:\n{}", state.display_environment());
        println!("--------------------\n");
      }
      _ => todo!(),
    }
  }
  Ok(())
}
