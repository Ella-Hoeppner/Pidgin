use std::{collections::HashMap, rc::Rc};

use ordered_float::OrderedFloat;

const U16_CAPCITY: usize = u16::MAX as usize + 1;
const DEFAULT_MEMORY_SIZE: usize = 1000;

type RegisterIndex = u16;
type SymbolIndex = u32;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Error {
  NotYetImplemented,
  CantCastToNum,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Num {
  Int(i64),
  Float(OrderedFloat<f64>),
}

impl Num {
  pub fn floor(&self) -> i64 {
    match self {
      Num::Int(i) => *i,
      Num::Float(f) => f64::from(*f) as i64,
    }
  }
  pub fn add(a: Num, b: &Num) -> Num {
    match (a, b) {
      (Num::Int(a), Num::Int(b)) => Num::Int(a + b),
      (Num::Float(a), Num::Float(b)) => Num::Float(a + b),
      (Num::Int(a), Num::Float(b)) => {
        Num::Float(OrderedFloat::from(a as f64) + b)
      }
      (Num::Float(a), Num::Int(b)) => Num::Float(a + (*b as f64)),
    }
  }
  pub fn multiply(a: Num, b: &Num) -> Num {
    match (a, b) {
      (Num::Int(a), Num::Int(b)) => Num::Int(a * b),
      (Num::Float(a), Num::Float(b)) => Num::Float(a * b),
      (Num::Int(a), Num::Float(b)) => {
        Num::Float(OrderedFloat::from(a as f64) * b)
      }
      (Num::Float(a), Num::Int(b)) => Num::Float(a * (*b as f64)),
    }
  }
  pub fn min(a: Num, b: &Num) -> Num {
    match (a, b) {
      (Num::Int(a), Num::Int(b)) => Num::Int(a.min(*b)),
      (Num::Float(a), Num::Float(b)) => Num::Float(a.min(*b)),
      (Num::Int(a), Num::Float(b)) => {
        let b_derefed = *b;
        if (a as f64) <= f64::from(b_derefed) {
          Num::Int(a)
        } else {
          Num::Float(b_derefed)
        }
      }
      (Num::Float(a), Num::Int(b)) => {
        let b_derefed = *b;
        if (b_derefed as f64) <= f64::from(a) {
          Num::Int(b_derefed)
        } else {
          Num::Float(a)
        }
      }
    }
  }
  pub fn max(a: Num, b: &Num) -> Num {
    match (a, b) {
      (Num::Int(a), Num::Int(b)) => Num::Int(a.max(*b)),
      (Num::Float(a), Num::Float(b)) => Num::Float(a.max(*b)),
      (Num::Int(a), Num::Float(b)) => {
        let b_derefed = *b;
        if (a as f64) >= f64::from(b_derefed) {
          Num::Int(a)
        } else {
          Num::Float(b_derefed)
        }
      }
      (Num::Float(a), Num::Int(b)) => {
        let b_derefed = *b;
        if (b_derefed as f64) >= f64::from(a) {
          Num::Int(b_derefed)
        } else {
          Num::Float(a)
        }
      }
    }
  }
  pub fn as_float(&self) -> OrderedFloat<f64> {
    match self {
      Num::Int(i) => OrderedFloat::from(*i as f64),
      Num::Float(f) => *f,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Value {
  Nil,
  Bool(bool),
  Num(Num),
  Symbol(SymbolIndex),
}

impl Value {
  fn as_num(&self) -> Result<Num> {
    match self {
      Value::Num(n) => Ok(n.clone()),
      Value::Nil => Ok(Num::Int(0)),
      _ => Err(Error::CantCastToNum),
    }
  }
  fn as_bool(&self) -> bool {
    match self {
      Value::Bool(value) => *value,
      Value::Nil => false,
      _ => true,
    }
  }
  fn description(&self) -> String {
    match self {
      Value::Nil => "nil".to_string(),
      Value::Bool(b) => b.to_string(),
      Value::Num(n) => match n {
        Num::Int(i) => i.to_string(),
        Num::Float(f) => {
          let mut s = f.to_string();
          if !s.contains('.') {
            s += ".";
          }
          s
        }
      },
      Value::Symbol(index) => format!("symbol {}", index),
    }
  }
}

pub enum Instruction {
  Clear(RegisterIndex),
  Const(RegisterIndex, Value),
  Add(RegisterIndex, RegisterIndex, RegisterIndex),
  Multiply(RegisterIndex, RegisterIndex, RegisterIndex),
  Lookup(RegisterIndex, SymbolIndex),
  Bind(SymbolIndex, RegisterIndex),
  DebugPrint(Option<String>),
}

struct EvaluationState {
  memory_size: usize,
  memory: [Value; DEFAULT_MEMORY_SIZE],
  registers: [*const Value; U16_CAPCITY],
  environment: HashMap<SymbolIndex, *const Value>,
}

impl EvaluationState {
  fn new() -> Self {
    const NIL: Value = Value::Nil;
    Self {
      memory_size: 0,
      memory: [NIL; DEFAULT_MEMORY_SIZE],
      registers: [std::ptr::null(); U16_CAPCITY],
      environment: HashMap::new(),
    }
  }
  fn display_memory(&self) -> String {
    (0..self.memory_size)
      .map(|i| format!("{}: {}", i, self.memory[i].description()))
      .reduce(|a, b| a + "\n" + &b)
      .unwrap_or("".to_string())
  }
  fn display_environment(&self) -> String {
    let mut bindings: Vec<_> = self.environment.iter().collect();
    bindings.sort_by_key(|(symbol_index, _value_pointer)| **symbol_index);
    bindings
      .into_iter()
      .map(|(symbol_index, value_pointer)| {
        let value = unsafe { &**value_pointer };
        format!(
          "symbol_index: {}, value: {}",
          symbol_index,
          value.description()
        )
      })
      .reduce(|a, b| a + "\n" + &b)
      .unwrap_or("".to_string())
  }
  fn display_registers(&self) -> String {
    let memory_pointer = self.memory.as_ptr();
    (0..U16_CAPCITY)
      .into_iter()
      .filter_map(|i| {
        let value_pointer = self.registers[i];
        if !value_pointer.is_null() {
          let value = unsafe { &*value_pointer };
          Some(format!(
            "{}: ({}) {}",
            i,
            unsafe { value_pointer.offset_from(memory_pointer) },
            value.description()
          ))
        } else {
          None
        }
      })
      .reduce(|a, b| a + "\n" + &b)
      .unwrap_or("".to_string())
  }
  fn store_value(&mut self, value: Value) -> *const Value {
    if self.memory_size == self.memory.len() {
      panic!("out of memory!!");
    }
    self.memory[self.memory_size] = value;
    self.memory_size += 1;
    &self.memory[self.memory_size - 1] as *const Value
  }
}

pub fn evaluate(instructions: Vec<Instruction>) -> Result<()> {
  let mut state = EvaluationState::new();
  for instruction in instructions {
    match instruction {
      Instruction::Clear(register_index) => {
        state.registers[register_index as usize] = std::ptr::null()
      }
      Instruction::Const(register_index, value) => {
        state.registers[register_index as usize] = state.store_value(value);
      }
      Instruction::Add(
        sum_register_index,
        input_register_index_1,
        input_register_index_2,
      ) => {
        let addend_1 =
          unsafe { &*state.registers[input_register_index_1 as usize] };
        let addend_2 =
          unsafe { &*state.registers[input_register_index_2 as usize] };
        let sum = Num::add(addend_1.as_num()?, &addend_2.as_num()?);
        state.registers[sum_register_index as usize] =
          state.store_value(Value::Num(sum));
      }
      Instruction::Multiply(
        product_register_index,
        input_register_index_1,
        input_register_index_2,
      ) => unsafe {
        let multiplicand_1 = &*state.registers[input_register_index_1 as usize];
        let multiplicand_2 = &*state.registers[input_register_index_2 as usize];
        let product =
          Num::multiply(multiplicand_1.as_num()?, &multiplicand_2.as_num()?);
        state.registers[product_register_index as usize] =
          state.store_value(Value::Num(product));
      },
      Instruction::Bind(symbol_index, register) => {
        state
          .environment
          .insert(symbol_index, state.registers[register as usize].clone());
      }
      Instruction::Lookup(register, symbol_index) => {
        state.registers[register as usize] = state.environment[&symbol_index];
      }
      Instruction::DebugPrint(maybe_message) => {
        if let Some(message) = maybe_message {
          println!("{}", message);
        }
        println!("--------------------");
        println!(
          "memory ({}):\n{}\n",
          state.memory_size,
          state.display_memory()
        );
        println!("registers:\n{}\n", state.display_registers());
        println!("environment:\n{}", state.display_environment());
        println!("--------------------\n");
      }
    }
  }
  Ok(())
}
