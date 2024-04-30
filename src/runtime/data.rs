use std::{
  collections::HashMap, fmt::Debug, hash::Hash, ops::Index, ptr::NonNull,
  rc::Rc,
};

use ordered_float::OrderedFloat;

use crate::Instruction;

use super::{vm::SymbolIndex, Error, Result};

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
pub struct SmolVec<I: SmolIndex, T: Default + Clone + Debug>(
  pub Box<(I, [T; 256])>,
);
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
impl<I: SmolIndex, T: Default + Clone + Debug> From<SmolVec<I, T>> for Vec<T> {
  fn from(v: SmolVec<I, T>) -> Self {
    let (len, values) = *v.0;
    values.into_iter().take(len.as_usize()).collect()
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
  pub instructions: SmolVec<u16, Instruction>,
}
impl Function {
  pub fn new(instructions: Vec<Instruction>) -> Self {
    Self {
      instructions: instructions.into(),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
  Nil,
  Bool(bool),
  Num(Num),
  Symbol(SymbolIndex),
  Fn(Function),
  List(Vec<Value>),
  Map(Box<HashMap<Value, Value>>),
  Str(Rc<str>),
}

impl Hash for Value {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    todo!()
  }
}

impl Value {
  pub fn as_num(&self) -> Result<Num> {
    match self {
      Value::Num(n) => Ok(n.clone()),
      Value::Nil => Ok(Num::Int(0)),
      _ => Err(Error::CantCastToNum),
    }
  }
  pub fn as_bool(&self) -> bool {
    match self {
      Value::Bool(value) => *value,
      Value::Nil => false,
      _ => true,
    }
  }
  pub fn description(&self) -> String {
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
        format!("fn({})", function.instructions.0 .0 as usize)
      }
      Value::Str(s) => format!("\"{}\"", s),
      Value::List(values) => {
        format!(
          "[{}]",
          values
            .iter()
            .map(|v| v.description())
            .collect::<Vec<String>>()
            .join(", ")
        )
      }
      Value::Map(hashmap) => format!(
        "{{{}}}",
        hashmap
          .iter()
          .map(|(key, value)| key.description() + " " + &value.description())
          .collect::<Vec<String>>()
          .join(", ")
      ),
    }
  }
}
