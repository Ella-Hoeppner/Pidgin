use minivec::MiniVec;
use std::{
  collections::{HashMap, HashSet},
  fmt::Debug,
  hash::Hash,
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

impl From<f64> for Num {
  fn from(f: f64) -> Self {
    Num::Float(OrderedFloat::from(f))
  }
}

impl From<OrderedFloat<f64>> for Num {
  fn from(f: OrderedFloat<f64>) -> Self {
    Num::Float(f)
  }
}

impl From<i64> for Num {
  fn from(i: i64) -> Self {
    Num::Int(i)
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
  Fn(MiniVec<Instruction>),
}

impl Hash for Value {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    todo!()
  }
}

impl Value {
  pub fn as_num(&self) -> Result<Num> {
    match self {
      Value::Num(n) => Ok(*n),
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
      Value::Char(c) => format!("'{}'", c),
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
      Value::Fn(instructions) => {
        format!("fn({})", instructions.len())
      }
    }
  }
}
