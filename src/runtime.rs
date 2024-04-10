use std::{collections::HashMap, fmt::Debug, ops::Index, rc::Rc};

use ordered_float::OrderedFloat;

const U16_CAPCITY: usize = u16::MAX as usize + 1;

type RegisterIndex = u8;
type SymbolIndex = u16;
type ConstIndex = u16;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Error {
  NotYetImplemented,
  CantCastToNum,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

trait SmolIndex {
  fn as_usize(&self) -> usize;
  fn from_usize(x: usize) -> Self;
}
impl SmolIndex for u8 {
  fn as_usize(&self) -> usize {
    *self as usize
  }
  fn from_usize(x: usize) -> Self {
    x as u8
  }
}
impl SmolIndex for u16 {
  fn as_usize(&self) -> usize {
    *self as usize
  }
  fn from_usize(x: usize) -> Self {
    x as u16
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct SmolVec<I: SmolIndex, T: Default + Clone + Debug>(Box<(I, [T; 256])>);
impl<I: SmolIndex, T: Default + Clone + Debug> Index<u8> for SmolVec<I, T> {
  type Output = T;
  fn index(&self, index: u8) -> &Self::Output {
    &self.0 .1[index as usize]
  }
}
impl<I: SmolIndex, T: Default + Clone + Debug> SmolVec<I, T> {
  fn new() -> Self {
    Self(Box::new((
      I::from_usize(0usize),
      core::array::from_fn(|_| T::default()),
    )))
  }
}
impl<I: SmolIndex, T: Default + Clone + Debug> Into<Vec<T>> for SmolVec<I, T> {
  fn into(self) -> Vec<T> {
    self.0 .1.to_vec()
  }
}
impl<I: SmolIndex, T: Default + Clone + Debug> From<Vec<T>> for SmolVec<I, T> {
  fn from(v: Vec<T>) -> Self {
    let len = v.len();
    let mut values = core::array::from_fn(|_| T::default());
    for (i, value) in v.into_iter().enumerate() {
      values[i] = value;
    }
    SmolVec(Box::new((I::from_usize(len), values)))
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Function {
  instructions: SmolVec<u16, Instruction>,
}
impl Function {
  pub fn new(instructions: Vec<Instruction>) -> Self {
    Self {
      instructions: instructions.into(),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Value {
  Nil,
  Bool(bool),
  Num(Num),
  Symbol(SymbolIndex),
  Fn(Function),
}

type X = Vec<Instruction>;

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
      Value::Fn(function) => {
        format!("fn({})", function.instructions.0 .0.as_usize())
      }
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Instruction {
  NoOp,
  Argument(SymbolIndex),
  Return(RegisterIndex),
  Clear(RegisterIndex),
  Const(RegisterIndex, ConstIndex),
  Add(RegisterIndex, RegisterIndex, RegisterIndex),
  Multiply(RegisterIndex, RegisterIndex, RegisterIndex),
  Lookup(RegisterIndex, SymbolIndex),
  Bind(SymbolIndex, RegisterIndex),
  DebugPrint(u8),
}
impl Default for Instruction {
  fn default() -> Self {
    Instruction::NoOp
  }
}

struct EvaluationState {
  stack: [Value; U16_CAPCITY],
  stack_consumption: u16,
  environment: HashMap<SymbolIndex, Value>,
}

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

impl EvaluationState {
  fn new() -> Self {
    const NIL: Value = Value::Nil;
    Self {
      stack: [NIL; U16_CAPCITY],
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
  fn set_register(&mut self, register: RegisterIndex, value: Value) {
    self.stack[register as usize] = value;
    self.stack_consumption = self.stack_consumption.max(register as u16 + 1);
  }
  fn get_register(&self, register: RegisterIndex) -> &Value {
    //debug
    if register as usize >= self.stack_consumption as usize {
      panic!("trying to access register that hasn't been set yet")
    }
    &self.stack[register as usize]
  }
}

pub fn evaluate(program: Program) -> Result<()> {
  let mut state = EvaluationState::new();
  for instruction in program.instructions {
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
      Return(_) => {
        panic!("Instruction::Return called, this should never happen")
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
    }
  }
  Ok(())
}
