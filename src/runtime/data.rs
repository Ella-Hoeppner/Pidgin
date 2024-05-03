use minivec::MiniVec;
use std::{
  collections::{HashMap, HashSet},
  fmt::{Debug, Display},
  hash::Hash,
  ops::{Add, Div, Mul, Neg, Sub},
  rc::Rc,
};

use ordered_float::OrderedFloat;

use crate::{CoreFnIndex, Instruction};

use super::{core_functions::CoreFnId, vm::SymbolIndex, Error, Result};

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
  pub fn as_int_lossless(&self) -> Result<i64> {
    match self {
      Int(i) => Ok(*i),
      Float(f) => {
        let i = f.into_inner() as i64;
        if i as f64 == **f {
          Ok(i)
        } else {
          Err(Error::ArgumentNotInt)
        }
      }
    }
  }
  pub fn numerical_equal(&self, other: &Num) -> bool {
    match (self, other) {
      (Int(a), Int(b)) => a == b,
      (Float(a), Float(b)) => a == b,
      (Int(a), Float(b)) => (*a as f64) == **b,
      (Float(a), Int(b)) => *a == (*b as f64),
    }
  }
  pub fn inc(&self) -> Num {
    match self {
      Int(i) => Int(i + 1),
      Float(f) => Float(f + 1.0),
    }
  }
  pub fn dec(&self) -> Num {
    match self {
      Int(i) => Int(i - 1),
      Float(f) => Float(f - 1.0),
    }
  }
  pub fn abs(&self) -> Num {
    match self {
      Int(i) => Int(i.abs()),
      Float(f) => Float(f.abs().into()),
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

pub type InstructionBlock = Rc<[Instruction]>;

#[derive(Clone, Debug)]
pub struct CompositeFunction {
  pub arg_count: u8,
  pub instructions: InstructionBlock,
}
impl CompositeFunction {
  pub fn new<T: Into<InstructionBlock>>(
    arg_count: u8,
    instructions: T,
  ) -> Self {
    Self {
      arg_count,
      instructions: instructions.into(),
    }
  }
}

#[derive(Clone, Debug)]
pub enum Value {
  Nil,
  Bool(bool),
  Char(char),
  Number(Num),
  Symbol(SymbolIndex),
  Str(Rc<String>),
  List(Rc<Vec<Value>>),
  Hashmap(Rc<HashMap<Value, Value>>),
  Hashset(Rc<HashSet<Value>>),
  CoreFn(CoreFnId),
  CompositeFn(Rc<CompositeFunction>),
  RawVec(MiniVec<Value>),
}
use Value::*;

impl PartialEq for Value {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Nil, Self::Nil) => true,
      (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
      (Self::Char(l0), Self::Char(r0)) => l0 == r0,
      (Self::Number(l0), Self::Number(r0)) => l0 == r0,
      (Self::Symbol(l0), Self::Symbol(r0)) => l0 == r0,
      (Self::Str(l0), Self::Str(r0)) => l0 == r0,
      (Self::List(l0), Self::List(r0)) => l0 == r0,
      (Self::Hashmap(l0), Self::Hashmap(r0)) => l0 == r0,
      (Self::Hashset(l0), Self::Hashset(r0)) => l0 == r0,
      (Self::CoreFn(l0), Self::CoreFn(r0)) => l0 == r0,
      (Self::CompositeFn(l0), Self::CompositeFn(r0)) => Rc::ptr_eq(l0, r0),
      (Self::RawVec(l0), Self::RawVec(r0)) => l0 == r0,
      _ => false,
    }
  }
}
impl Eq for Value {}

impl Hash for Value {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    todo!()
  }
}

impl Value {
  pub fn as_num(&self) -> Result<&Num> {
    match self {
      Number(n) => Ok(n),
      Nil => Ok(&Int(0)),
      _ => Err(Error::CantCastToNum),
    }
  }
  pub fn as_bool(&self) -> bool {
    match self {
      Bool(value) => *value,
      Nil => false,
      _ => true,
    }
  }
  pub fn description(&self) -> String {
    match self {
      Nil => "nil".to_string(),
      Bool(b) => b.to_string(),
      Char(c) => format!("'{}'", c),
      Number(n) => match n {
        Int(i) => i.to_string(),
        Float(f) => {
          let mut s = f.to_string();
          if !s.contains('.') {
            s += ".";
          }
          s
        }
      },
      List(values) => {
        format!(
          "[{}]",
          values
            .iter()
            .map(|v| v.description())
            .collect::<Vec<String>>()
            .join(", ")
        )
      }
      Hashmap(hashmap) => format!(
        "{{{}}}",
        hashmap
          .iter()
          .map(|(key, value)| key.description() + " " + &value.description())
          .collect::<Vec<String>>()
          .join(", ")
      ),
      Hashset(hashset) => format!(
        "#{{{}}}",
        hashset
          .iter()
          .map(|value| value.description())
          .collect::<Vec<String>>()
          .join(", ")
      ),
      Symbol(index) => format!("symbol {}", index),
      Str(s) => format!("\"{}\"", s),
      CompositeFn(composite_fn) => {
        format!(
          "fn({} args, {} instructions)",
          composite_fn.arg_count,
          composite_fn.instructions.len()
        )
      }
      CoreFn(_) => todo!(),
      RawVec(values) => format!(
        "(raw) [{}]",
        values
          .iter()
          .map(|v| v.description())
          .collect::<Vec<String>>()
          .join(", ")
      ),
    }
  }
}

impl Display for Value {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.description())
  }
}

impl From<Num> for Value {
  fn from(n: Num) -> Self {
    Number(n)
  }
}
impl From<i64> for Value {
  fn from(i: i64) -> Self {
    Number(i.into())
  }
}
impl From<f64> for Value {
  fn from(f: f64) -> Self {
    Number(f.into())
  }
}
impl From<OrderedFloat<f64>> for Value {
  fn from(f: OrderedFloat<f64>) -> Self {
    Number(f.into())
  }
}
impl From<bool> for Value {
  fn from(b: bool) -> Self {
    Bool(b)
  }
}
impl From<char> for Value {
  fn from(c: char) -> Self {
    Char(c)
  }
}
impl From<String> for Value {
  fn from(s: String) -> Self {
    Str(Rc::new(s))
  }
}
impl From<&str> for Value {
  fn from(s: &str) -> Self {
    Str(Rc::new(s.to_string()))
  }
}
impl From<Vec<Value>> for Value {
  fn from(values: Vec<Value>) -> Self {
    List(Rc::new(values))
  }
}
impl From<Rc<Vec<Value>>> for Value {
  fn from(values: Rc<Vec<Value>>) -> Self {
    List(values)
  }
}
