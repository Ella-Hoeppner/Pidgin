use std::{
  any::Any,
  cell::RefCell,
  collections::{HashMap, HashSet},
  fmt::{Debug, Display},
  hash::Hash,
  ops::{Add, Div, Mul, Neg, Sub},
  rc::Rc,
};

use ordered_float::OrderedFloat;

use crate::{
  blocks::GenericBlock,
  compiler::ast::token::SymbolLedger,
  instructions::GenericInstruction,
  runtime::{control::GenericCompositeFunction, evaluation::Register},
};

use super::{
  control::{CompositeFunction, PausedCoroutine},
  core_functions::CoreFnId,
  error::{RuntimeError, RuntimeResult},
  evaluation::SymbolIndex,
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
  pub fn as_int_lossless(&self) -> RuntimeResult<i64> {
    match self {
      Int(i) => Ok(*i),
      Float(f) => {
        let i = f.into_inner() as i64;
        if i as f64 == **f {
          Ok(i)
        } else {
          Err(RuntimeError::ArgumentNotInt)
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
pub struct AritySpecifier {
  pub count: u8,
}
impl AritySpecifier {
  pub fn can_accept(&self, count: usize) -> bool {
    self.count as usize == count
  }
  pub fn register_count(&self) -> u8 {
    self.count
  }
}
impl Display for AritySpecifier {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.count)
  }
}
impl From<u8> for AritySpecifier {
  fn from(count: u8) -> Self {
    Self { count }
  }
}

#[derive(Clone, Debug)]
pub enum GenericValue<I, O, R, M> {
  Nil,
  Bool(bool),
  Char(char),
  Number(Num),
  Symbol(SymbolIndex),
  Str(Rc<String>),
  List(Rc<Vec<GenericValue<I, O, R, M>>>),
  Hashmap(Rc<HashMap<GenericValue<I, O, R, M>, GenericValue<I, O, R, M>>>),
  Hashset(Rc<HashSet<GenericValue<I, O, R, M>>>),
  CoreFn(CoreFnId),
  CompositeFn(Rc<GenericCompositeFunction<I, O, R, M>>),
  ExternalFn(Rc<ExternalFunction>),
  PartialApplication(
    Rc<(GenericValue<I, O, R, M>, Vec<GenericValue<I, O, R, M>>)>,
  ),
  Composition(Rc<Vec<GenericValue<I, O, R, M>>>),
  ExternalObject(Rc<Rc<dyn Any>>),
  Coroutine(Rc<Option<RefCell<Option<PausedCoroutine>>>>),
  Error(Rc<RuntimeError>),
}

pub type Value = GenericValue<Register, Register, Register, Register>;
use GenericValue::*;

impl<I, O, R, M> PartialEq for GenericValue<I, O, R, M>
where
  GenericValue<I, O, R, M>: Hash,
{
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
impl<I, O, R, M> Eq for GenericValue<I, O, R, M> where
  GenericValue<I, O, R, M>: Hash
{
}

impl<I: Clone, O: Clone, R: Clone, M: Clone> GenericValue<I, O, R, M> {
  pub fn translate<
    NewI: Clone,
    NewO: Clone,
    NewR: Clone,
    NewM: Clone,
    E,
    F: Fn(
      u8,
      Vec<GenericInstruction<I, O, R>>,
      Vec<GenericValue<NewI, NewO, NewR, NewM>>,
      M,
    ) -> Result<GenericBlock<NewI, NewO, NewR, NewM>, E>,
  >(
    self,
    translator: &F,
  ) -> Result<GenericValue<NewI, NewO, NewR, NewM>, E> {
    Ok(match self {
      CompositeFn(f_ref) => {
        CompositeFn(Rc::new(GenericCompositeFunction::new(
          f_ref.args.clone(),
          f_ref
            .block
            .clone()
            .translate_inner(f_ref.args.register_count(), translator)?,
        )))
      }
      Nil => Nil,
      Bool(b) => Bool(b),
      Char(c) => Char(c),
      Number(n) => Number(n),
      Symbol(s) => Symbol(s),
      Str(s) => Str(s),
      List(vec) => List(Rc::new(
        Rc::unwrap_or_clone(vec)
          .into_iter()
          .map(|value| value.translate(translator))
          .collect::<Result<Vec<_>, E>>()?,
      )),
      Hashmap(hashmap) => Hashmap(Rc::new(
        Rc::unwrap_or_clone(hashmap)
          .into_iter()
          .map(
            |(key, value)| -> Result<
              (
                GenericValue<NewI, NewO, NewR, NewM>,
                GenericValue<NewI, NewO, NewR, NewM>,
              ),
              E,
            > {
              Ok((key.translate(translator)?, value.translate(translator)?))
            },
          )
          .collect::<Result<HashMap<_, _>, E>>()?,
      )),
      Hashset(set) => Hashset(Rc::new(
        Rc::unwrap_or_clone(set)
          .into_iter()
          .map(|value| value.translate(translator))
          .collect::<Result<HashSet<_>, E>>()?,
      )),
      CoreFn(f) => CoreFn(f),
      ExternalFn(f) => ExternalFn(f),
      PartialApplication(f_and_values) => {
        let (f, args) = Rc::unwrap_or_clone(f_and_values);
        PartialApplication(Rc::new((
          f.translate(translator)?,
          args
            .into_iter()
            .map(|arg| arg.translate(translator))
            .collect::<Result<_, _>>()?,
        )))
      }
      Composition(fs) => Composition(Rc::new(
        Rc::unwrap_or_clone(fs)
          .into_iter()
          .map(|f| f.translate(translator))
          .collect::<Result<_, _>>()?,
      )),
      ExternalObject(o) => ExternalObject(o),
      Coroutine(c) => Coroutine(c),
      Error(e) => Error(e),
    })
  }
}

impl<I, O, R, M> GenericValue<I, O, R, M> {
  pub(crate) fn composite_fn<
    A: Into<AritySpecifier>,
    B: Into<GenericBlock<I, O, R, M>>,
  >(
    args: A,
    instructions: B,
  ) -> Self {
    CompositeFn(Rc::new(GenericCompositeFunction::new(args, instructions)))
  }
  pub(crate) fn description(
    &self,
    symbol_ledger: Option<&SymbolLedger>,
  ) -> String {
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
            .map(|v| v.description(symbol_ledger))
            .collect::<Vec<String>>()
            .join(", ")
        )
      }
      Hashmap(hashmap) => format!(
        "{{{}}}",
        hashmap
          .iter()
          .map(|(key, value)| key.description(symbol_ledger)
            + " "
            + &value.description(symbol_ledger))
          .collect::<Vec<String>>()
          .join(", ")
      ),
      Hashset(hashset) => format!(
        "#{{{}}}",
        hashset
          .iter()
          .map(|value| value.description(symbol_ledger))
          .collect::<Vec<String>>()
          .join(", ")
      ),
      Symbol(index) => {
        if let Some(symbol_ledger) = symbol_ledger {
          symbol_ledger
            .symbol_name(index)
            .cloned()
            .unwrap_or(format!("<unrecognized symbol, index {index}>"))
        } else {
          format!("<symbol {}>", index)
        }
      }
      Str(s) => format!("\"{}\"", s),
      CompositeFn(composite_fn) => {
        format!(
          "fn( {} args, {} instructions )\n",
          composite_fn.args.count,
          composite_fn.block.len()
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
      PartialApplication(f_and_values) => {
        let (f, args) = &**f_and_values;
        format!(
          "partial application: f = {}, args = [{}]",
          f.description(symbol_ledger),
          args
            .iter()
            .map(|arg| arg.description(symbol_ledger))
            .collect::<Vec<_>>()
            .join(", ")
        )
      }
      Composition(fs) => {
        format!(
          "composition: [{}]",
          fs.iter()
            .map(|f| f.description(symbol_ledger))
            .collect::<Vec<_>>()
            .join(", ")
        )
      }
      Coroutine(x) => format!(
        "coroutine ({})",
        if let Some(maybe_paused_coroutine) = &**x {
          if let Some(paused_coroutine) = &*(*maybe_paused_coroutine).borrow() {
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
}

impl<I, O, R, M> Hash for GenericValue<I, O, R, M> {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    todo!()
  }
}

impl Value {
  pub fn as_num(&self) -> RuntimeResult<&Num> {
    match self {
      Number(n) => Ok(n),
      Nil => Ok(&Int(0)),
      _ => Err(RuntimeError::CantCastToNum(self.clone())),
    }
  }
  pub fn as_bool(&self) -> bool {
    match self {
      Bool(value) => *value,
      Nil => false,
      _ => true,
    }
  }
  pub fn external<T: Any>(external_object: T) -> Self {
    ExternalObject(Rc::new(Rc::new(external_object)))
  }
  pub fn casted_external<T: Any>(self) -> Option<Rc<T>> {
    if let ExternalObject(external_object) = self {
      Rc::downcast::<T>((*external_object).clone()).ok()
    } else {
      None
    }
  }
  pub fn fn_coroutine(f: CompositeFunction) -> Value {
    Coroutine(Rc::new(Some(RefCell::new(Some(f.into())))))
  }
}

impl Display for Value {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.description(None))
  }
}

impl<I, O, R, M> From<Num> for GenericValue<I, O, R, M> {
  fn from(n: Num) -> Self {
    Number(n)
  }
}
impl<I, O, R, M> From<i64> for GenericValue<I, O, R, M> {
  fn from(i: i64) -> Self {
    Number(i.into())
  }
}
impl<I, O, R, M> From<f64> for GenericValue<I, O, R, M> {
  fn from(f: f64) -> Self {
    Number(f.into())
  }
}
impl<I, O, R, M> From<OrderedFloat<f64>> for GenericValue<I, O, R, M> {
  fn from(f: OrderedFloat<f64>) -> Self {
    Number(f.into())
  }
}
impl<I, O, R, M> From<bool> for GenericValue<I, O, R, M> {
  fn from(b: bool) -> Self {
    Bool(b)
  }
}
impl<I, O, R, M> From<char> for GenericValue<I, O, R, M> {
  fn from(c: char) -> Self {
    Char(c)
  }
}
impl<I, O, R, M> From<String> for GenericValue<I, O, R, M> {
  fn from(s: String) -> Self {
    Str(Rc::new(s))
  }
}
impl<I, O, R, M> From<&str> for GenericValue<I, O, R, M> {
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
impl From<RuntimeError> for Value {
  fn from(e: RuntimeError) -> Self {
    Error(Rc::new(e))
  }
}
