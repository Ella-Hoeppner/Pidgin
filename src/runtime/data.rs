use minivec::MiniVec;
use std::{
  collections::{HashMap, HashSet},
  fmt::Debug,
  hash::Hash,
  ops::{Add, Div, Mul, Neg, Sub},
  rc::Rc,
};

use ordered_float::OrderedFloat;

use crate::{CoreFnIndex, Instruction};

use super::{vm::SymbolIndex, Error, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Num {
  Int(i64),
  Float(OrderedFloat<f64>),
}
use Num::*;

impl Num {
  pub fn floor(&self) -> i64 {
    match self {
      Int(i) => *i,
      Float(f) => f.floor() as i64,
    }
  }
  pub fn ceil(&self) -> i64 {
    match self {
      Int(i) => *i,
      Float(f) => f.ceil() as i64,
    }
  }
  pub fn min(a: Num, b: &Num) -> Num {
    match (a, b) {
      (Int(a), Int(b)) => Int(a.min(*b)),
      (Float(a), Float(b)) => Float(a.min(*b)),
      (Int(a), Float(b)) => {
        let b_derefed = *b;
        if (a as f64) <= f64::from(b_derefed) {
          Int(a)
        } else {
          Float(b_derefed)
        }
      }
      (Float(a), Int(b)) => {
        let b_derefed = *b;
        if (b_derefed as f64) <= f64::from(a) {
          Int(b_derefed)
        } else {
          Float(a)
        }
      }
    }
  }
  pub fn max(a: Num, b: &Num) -> Num {
    match (a, b) {
      (Int(a), Int(b)) => Int(a.max(*b)),
      (Float(a), Float(b)) => Float(a.max(*b)),
      (Int(a), Float(b)) => {
        let b_derefed = *b;
        if (a as f64) >= f64::from(b_derefed) {
          Int(a)
        } else {
          Float(b_derefed)
        }
      }
      (Float(a), Int(b)) => {
        let b_derefed = *b;
        if (b_derefed as f64) >= f64::from(a) {
          Int(b_derefed)
        } else {
          Float(a)
        }
      }
    }
  }
  pub fn as_float(&self) -> OrderedFloat<f64> {
    match self {
      Int(i) => OrderedFloat::from(*i as f64),
      Float(f) => *f,
    }
  }
}

impl Add for Num {
  type Output = Num;
  fn add(self, other: Num) -> Num {
    match (self, other) {
      (Int(a), Int(b)) => (a + b).into(),
      (Float(a), Float(b)) => (a + b).into(),
      (Int(a), Float(b)) => ((a as f64) + *b).into(),
      (Float(a), Int(b)) => (a + (b as f64)).into(),
    }
  }
}

impl Sub for Num {
  type Output = Num;
  fn sub(self, other: Num) -> Num {
    match (self, other) {
      (Int(a), Int(b)) => (a - b).into(),
      (Float(a), Float(b)) => (a - b).into(),
      (Int(a), Float(b)) => ((a as f64) - *b).into(),
      (Float(a), Int(b)) => (*a - (b as f64)).into(),
    }
  }
}

impl Neg for Num {
  type Output = Num;
  fn neg(self) -> Num {
    match self {
      Int(i) => Int(-i),
      Float(f) => Float(-f),
    }
  }
}

impl Mul for Num {
  type Output = Num;
  fn mul(self, other: Num) -> Num {
    match (self, other) {
      (Int(a), Int(b)) => (a * b).into(),
      (Float(a), Float(b)) => (a * b).into(),
      (Int(a), Float(b)) => ((a as f64) * *b).into(),
      (Float(a), Int(b)) => (a * (b as f64)).into(),
    }
  }
}

impl Div for Num {
  type Output = Num;
  fn div(self, other: Num) -> Num {
    (self.as_float() / other.as_float()).into()
  }
}

impl From<f64> for Num {
  fn from(f: f64) -> Self {
    Float(OrderedFloat::from(f))
  }
}

impl From<OrderedFloat<f64>> for Num {
  fn from(f: OrderedFloat<f64>) -> Self {
    Float(f)
  }
}

impl From<i64> for Num {
  fn from(i: i64) -> Self {
    Int(i)
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
  Nil,
  Bool(bool),
  Char(char),
  Num(Num),
  Symbol(SymbolIndex),
  Str(Rc<String>),
  List(Rc<Vec<Value>>),
  Map(Rc<HashMap<Value, Value>>),
  Set(Rc<HashSet<Value>>),
  CoreFn(CoreFnIndex),
  CompositeFn(MiniVec<Instruction>),
}

impl Hash for Value {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    todo!()
  }
}

impl Value {
  pub fn as_num(&self) -> Result<&Num> {
    match self {
      Value::Num(n) => Ok(n),
      Value::Nil => Ok(&Int(0)),
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
      Value::Char(c) => format!("'{}'", c),
      Value::Num(n) => match n {
        Int(i) => i.to_string(),
        Float(f) => {
          let mut s = f.to_string();
          if !s.contains('.') {
            s += ".";
          }
          s
        }
      },
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
      Value::Set(hashset) => format!(
        "#{{{}}}",
        hashset
          .iter()
          .map(|value| value.description())
          .collect::<Vec<String>>()
          .join(", ")
      ),
      Value::Symbol(index) => format!("symbol {}", index),
      Value::Str(s) => format!("\"{}\"", s),
      Value::CompositeFn(instructions) => {
        format!("fn({})", instructions.len())
      }
      Value::CoreFn(_) => todo!(),
    }
  }
}

impl From<Num> for Value {
  fn from(n: Num) -> Self {
    Value::Num(n)
  }
}
impl From<i64> for Value {
  fn from(i: i64) -> Self {
    Value::Num(i.into())
  }
}
impl From<f64> for Value {
  fn from(f: f64) -> Self {
    Value::Num(f.into())
  }
}
impl From<OrderedFloat<f64>> for Value {
  fn from(f: OrderedFloat<f64>) -> Self {
    Value::Num(f.into())
  }
}
impl From<bool> for Value {
  fn from(b: bool) -> Self {
    Value::Bool(b)
  }
}
impl From<char> for Value {
  fn from(c: char) -> Self {
    Value::Char(c)
  }
}
impl From<String> for Value {
  fn from(s: String) -> Self {
    Value::Str(Rc::new(s))
  }
}
