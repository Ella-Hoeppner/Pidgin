use minivec::MiniVec;
use std::{
  any::Any,
  cell::RefCell,
  collections::{HashMap, HashSet},
  error::Error,
  fmt::{Debug, Display},
  hash::Hash,
  ops::{Add, Div, Mul, Neg, Sub},
  rc::Rc,
};

use ordered_float::OrderedFloat;

use crate::{
  ConstIndex, CoreFnIndex, CoroutineState, GeneralizedCompositeFunction,
  Instruction, InstructionBlock, RegisterIndex, StackFrame,
};

use super::{
  control::{CompositeFunction, PausedCoroutine, RuntimeInstructionBlock},
  core_functions::CoreFnId,
  error::{PidginError, PidginResult},
  vm::SymbolIndex,
};

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
  pub fn as_int_lossless(&self) -> PidginResult<i64> {
    match self {
      Int(i) => Ok(*i),
      Float(f) => {
        let i = f.into_inner() as i64;
        if i as f64 == **f {
          Ok(i)
        } else {
          Err(PidginError::ArgumentNotInt)
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
impl Add for &Num {
  type Output = Num;
  fn add(self, other: &Num) -> Num {
    match (self, other) {
      (Int(a), Int(b)) => (a + b).into(),
      (Float(a), Float(b)) => (*a + *b).into(),
      (Int(a), Float(b)) => ((*a as f64) + **b).into(),
      (Float(a), Int(b)) => (a + (*b as f64)).into(),
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
impl Sub for &Num {
  type Output = Num;
  fn sub(self, other: &Num) -> Num {
    match (self, other) {
      (Int(a), Int(b)) => (a - b).into(),
      (Float(a), Float(b)) => (*a - b).into(),
      (Int(a), Float(b)) => ((*a as f64) - **b).into(),
      (Float(a), Int(b)) => (*a - (*b as f64)).into(),
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
impl Neg for &Num {
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
impl Mul for &Num {
  type Output = Num;
  fn mul(self, other: &Num) -> Num {
    match (self, other) {
      (Int(a), Int(b)) => (a * b).into(),
      (Float(a), Float(b)) => (*a * b).into(),
      (Int(a), Float(b)) => ((*a as f64) * **b).into(),
      (Float(a), Int(b)) => (a * (*b as f64)).into(),
    }
  }
}

impl Div for Num {
  type Output = Num;
  fn div(self, other: Num) -> Num {
    (self.as_float() / other.as_float()).into()
  }
}
impl Div for &Num {
  type Output = Num;
  fn div(self, other: &Num) -> Num {
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

#[derive(Clone, Debug)]
pub struct ExternalFunction {
  pub name: Option<String>,
  pub f: fn(Vec<Value>) -> Result<Value, Rc<dyn std::error::Error>>,
}
impl ExternalFunction {
  pub fn unnamed(
    f: fn(Vec<Value>) -> Result<Value, Rc<dyn std::error::Error>>,
  ) -> Self {
    Self { name: None, f }
  }
}

#[derive(Clone, Debug)]
pub struct ArgumentSpecifier {
  pub count: u8,
}
impl ArgumentSpecifier {
  pub fn can_accept(&self, count: usize) -> bool {
    self.count as usize == count
  }
}
impl Display for ArgumentSpecifier {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.count)
  }
}
impl From<u8> for ArgumentSpecifier {
  fn from(count: u8) -> Self {
    Self { count }
  }
}

#[derive(Clone, Debug)]
pub enum GeneralizedValue<R, C, M> {
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
  CompositeFn(Rc<GeneralizedCompositeFunction<R, C, M>>),
  ExternalFn(Rc<ExternalFunction>),
  ExternalObject(Rc<dyn Any>),
  Coroutine(Rc<Option<RefCell<Option<PausedCoroutine>>>>),
  Error(PidginError),
}

pub type Value = GeneralizedValue<RegisterIndex, ConstIndex, ()>;
use GeneralizedValue::*;

impl PartialEq for Value {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Nil, Self::Nil) => true,
      (Self::Bool(a), Self::Bool(b)) => a == b,
      (Self::Char(a), Self::Char(b)) => a == b,
      (Self::Number(a), Self::Number(b)) => a == b,
      (Self::Symbol(a), Self::Symbol(b)) => a == b,
      (Self::Str(a), Self::Str(b)) => a == b,
      (Self::List(a), Self::List(b)) => a == b,
      (Self::Hashmap(a), Self::Hashmap(b)) => a == b,
      (Self::Hashset(a), Self::Hashset(b)) => a == b,
      (Self::CoreFn(a), Self::CoreFn(b)) => a == b,
      (Self::CompositeFn(a), Self::CompositeFn(b)) => Rc::ptr_eq(a, b),
      (Self::ExternalFn(a), Self::ExternalFn(b)) => Rc::ptr_eq(a, b),
      (Self::ExternalObject(a), Self::ExternalObject(b)) => Rc::ptr_eq(a, b),
      (Self::Coroutine(a), Self::Coroutine(b)) => Rc::ptr_eq(a, b),
      (Self::Error(a), Self::Error(b)) => a == b,
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
  pub fn as_num(&self) -> PidginResult<&Num> {
    match self {
      Number(n) => Ok(n),
      Nil => Ok(&Int(0)),
      _ => Err(PidginError::CantCastToNum),
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
          "fn( {} args, {} instructions )",
          composite_fn.args.count,
          composite_fn.instructions.len()
        )
      }
      CoreFn(core_fn_id) => {
        format!("core_fn( {} )", core_fn_id)
      }
      ExternalFn(external_fn) => {
        format!(
          "external_fn( {} )",
          if let Some(name) = &(*external_fn).name {
            name
          } else {
            "<unnamed>"
          }
        )
      }
      Coroutine(x) => format!(
        "coroutine ({})",
        if let Some(maybe_paused_coroutine) = &**x {
          if let Some(paused_coroutine) = (&*(*maybe_paused_coroutine).borrow())
          {
            format!(
              "{}, awaiting {} arguments",
              if paused_coroutine.started {
                "paused"
              } else {
                "unstarted"
              },
              paused_coroutine.args
            )
          } else {
            "active".to_string()
          }
        } else {
          "dead".to_string()
        }
      ),
      ExternalObject(_) => "external_object".to_string(),
      Error(e) => format!("error: {}", e),
    }
  }
  pub fn external<T: Any>(external_object: T) -> Self {
    ExternalObject(Rc::new(external_object))
  }
  pub fn casted_external<T: Any>(self) -> Option<Rc<T>> {
    if let ExternalObject(external_object) = self {
      external_object.downcast::<T>().ok()
    } else {
      None
    }
  }
  pub fn composite_fn<
    A: Into<ArgumentSpecifier>,
    I: Into<RuntimeInstructionBlock>,
  >(
    args: A,
    instructions: I,
  ) -> Value {
    CompositeFn(Rc::new(CompositeFunction::new(args, instructions)))
  }
  pub fn fn_coroutine(f: CompositeFunction) -> Value {
    Coroutine(Rc::new(Some(RefCell::new(Some(f.into())))))
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
impl From<ExternalFunction> for Value {
  fn from(f: ExternalFunction) -> Self {
    ExternalFn(Rc::new(f))
  }
}
impl From<PidginError> for Value {
  fn from(e: PidginError) -> Self {
    Error(e)
  }
}
