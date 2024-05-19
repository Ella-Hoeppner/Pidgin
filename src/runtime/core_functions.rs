use std::fmt::Display;

use crate::runtime::{
  data::{
    GenericValue::*,
    Num::{self, *},
    Value,
  },
  error::RuntimeResult,
};
use enum_map::{Enum, EnumMap};

#[derive(Debug, Enum, PartialEq, Eq, Hash, Clone, Copy)]
pub enum CoreFnId {
  Print,
  Apply,
  When,
  If,
  Partial,
  Compose,
  FindSome,
  Reduce,
  Memoize,
  Constantly,
  NumericalEqual,
  IsZero,
  IsNan,
  IsInf,
  IsEven,
  IsOdd,
  IsPos,
  IsNeg,
  Inc,
  Dec,
  Abs,
  Floor,
  Ceil,
  Sqrt,
  Exp,
  Exp2,
  Ln,
  Log2,
  Add,
  Subtract,
  Multiply,
  Divide,
  Pow,
  Mod,
  Quot,
  Min,
  Max,
  GreaterThan,
  GreaterThanOrEqual,
  LessThan,
  LessThanOrEqual,
  Rand,
  RandInt,
  Equal,
  NotEqual,
  Not,
  And,
  Or,
  Xor,
  IsEmpty,
  First,
  Count,
  Flatten,
  Remove,
  Filter,
  Map,
  Set,
  SetIn,
  Get,
  GetIn,
  Update,
  UpdateIn,
  MinKey,
  MaxKey,
  Push,
  Sort,
  SortBy,
  CreateList,
  Last,
  Rest,
  ButLast,
  Nth,
  NthFromLast,
  Cons,
  Concat,
  Take,
  Drop,
  Reverse,
  Distinct,
  Sub,
  Partition,
  Pad,
  CreateMap,
  Keys,
  Values,
  Zip,
  Invert,
  Merge,
  MergeWith,
  MapKeys,
  MapValues,
  SelectKeys,
  CreateSet,
  Union,
  Intersection,
  Difference,
  SymmetricDifference,
  Range,
  Repeat,
  Repeatedly,
  Iterate,
  IsNil,
  IsBool,
  IsChar,
  IsNum,
  IsInt,
  IsFloat,
  IsSymbol,
  IsString,
  IsList,
  IsMap,
  IsSet,
  IsCollection,
  IsFn,
  ToBool,
  ToChar,
  ToNum,
  ToInt,
  ToFloat,
  ToSymbol,
  ToString,
  ToList,
  ToMap,
  CreateCell,
  GetCellValue,
  SetCellValue,
  UpdateCell,
}
use CoreFnId as F;

impl CoreFnId {
  pub fn name(&self) -> &str {
    match self {
      F::Print => "print",
      F::Apply => "apply",
      F::When => "when",
      F::If => "if",
      F::Partial => "partial",
      F::Compose => "compose",
      F::FindSome => "some",
      F::Reduce => "reduce",
      F::Memoize => "memoize",
      F::Constantly => "constantly",
      F::NumericalEqual => "==",
      F::IsZero => "zero?",
      F::IsNan => "nan?",
      F::IsInf => "inf?",
      F::IsEven => "even?",
      F::IsOdd => "odd?",
      F::IsPos => "pos?",
      F::IsNeg => "neg?",
      F::Inc => "inc",
      F::Dec => "dec",
      F::Abs => "abs",
      F::Floor => "floor",
      F::Ceil => "ceil",
      F::Sqrt => "sqrt",
      F::Exp => "exp",
      F::Exp2 => "exp2",
      F::Ln => "ln",
      F::Log2 => "log2",
      F::Add => "+",
      F::Subtract => "-",
      F::Multiply => "*",
      F::Divide => "/",
      F::Pow => "pow",
      F::Mod => "mod",
      F::Quot => "quot",
      F::Min => "min",
      F::Max => "max",
      F::GreaterThan => ">",
      F::GreaterThanOrEqual => ">=",
      F::LessThan => "<",
      F::LessThanOrEqual => "<=",
      F::Rand => "rand",
      F::RandInt => "rand-int",
      F::Equal => "=",
      F::NotEqual => "not=",
      F::Not => "not",
      F::And => "and",
      F::Or => "or",
      F::Xor => "xor",
      F::IsEmpty => "empty?",
      F::First => "first",
      F::Count => "count",
      F::Flatten => "flatten",
      F::Remove => "remove",
      F::Filter => "filter",
      F::Map => "map",
      F::Set => "set",
      F::SetIn => "set-in",
      F::Get => "get",
      F::GetIn => "get-in",
      F::Update => "update",
      F::UpdateIn => "update-in",
      F::MinKey => "min-key",
      F::MaxKey => "max-key",
      F::Push => "push",
      F::Sort => "sort",
      F::SortBy => "sort-by",
      F::CreateList => "list",
      F::Last => "last",
      F::Rest => "rest",
      F::ButLast => "butlast",
      F::Nth => "nth",
      F::NthFromLast => "nth-from-last",
      F::Cons => "cons",
      F::Concat => "concat",
      F::Take => "take",
      F::Drop => "drop",
      F::Reverse => "reverse",
      F::Distinct => "distinct",
      F::Sub => "sub",
      F::Partition => "partition",
      F::Pad => "pad",
      F::CreateMap => "hashmap",
      F::Keys => "keys",
      F::Values => "vals",
      F::Zip => "zip",
      F::Invert => "invert",
      F::Merge => "merge",
      F::MergeWith => "merge-with",
      F::MapKeys => "map-keys",
      F::MapValues => "map-vals",
      F::SelectKeys => "select-keys",
      F::CreateSet => "hashset",
      F::Union => "union",
      F::Intersection => "intersection",
      F::Difference => "difference",
      F::SymmetricDifference => "sym-difference",
      F::Range => "range",
      F::Repeat => "repeat",
      F::Repeatedly => "repeatedly",
      F::Iterate => "iterate",
      F::IsNil => "nil?",
      F::IsBool => "bool?",
      F::IsChar => "char?",
      F::IsNum => "num?",
      F::IsInt => "int?",
      F::IsFloat => "float?",
      F::IsSymbol => "symbol?",
      F::IsString => "str?",
      F::IsList => "list?",
      F::IsMap => "hashmap?",
      F::IsSet => "hashset?",
      F::IsCollection => "collection?",
      F::IsFn => "fn?",
      F::ToBool => "bool",
      F::ToChar => "char",
      F::ToNum => "num",
      F::ToInt => "int",
      F::ToFloat => "float",
      F::ToSymbol => "symbol",
      F::ToString => "str",
      F::ToList => "to-list",
      F::ToMap => "to-hashmap",
      F::CreateCell => todo!(),
      F::GetCellValue => todo!(),
      F::SetCellValue => todo!(),
      F::UpdateCell => todo!(),
    }
  }
  pub fn from_name(name: &str) -> Option<Self> {
    match name {
      "print" => Some(F::Print),
      "apply" => Some(F::Apply),
      "when" => Some(F::When),
      "if" => Some(F::If),
      "partial" => Some(F::Partial),
      "compose" => Some(F::Compose),
      "some" => Some(F::FindSome),
      "reduce" => Some(F::Reduce),
      "memoize" => Some(F::Memoize),
      "constantly" => Some(F::Constantly),
      "==" => Some(F::NumericalEqual),
      "zero?" => Some(F::IsZero),
      "nan?" => Some(F::IsNan),
      "inf?" => Some(F::IsInf),
      "even?" => Some(F::IsEven),
      "odd?" => Some(F::IsOdd),
      "pos?" => Some(F::IsPos),
      "neg?" => Some(F::IsNeg),
      "inc" => Some(F::Inc),
      "dec" => Some(F::Dec),
      "abs" => Some(F::Abs),
      "floor" => Some(F::Floor),
      "ceil" => Some(F::Ceil),
      "sqrt" => Some(F::Sqrt),
      "exp" => Some(F::Exp),
      "exp2" => Some(F::Exp2),
      "ln" => Some(F::Ln),
      "log2" => Some(F::Log2),
      "+" => Some(F::Add),
      "-" => Some(F::Subtract),
      "*" => Some(F::Multiply),
      "/" => Some(F::Divide),
      "pow" => Some(F::Pow),
      "mod" => Some(F::Mod),
      "quot" => Some(F::Quot),
      "min" => Some(F::Min),
      "max" => Some(F::Max),
      ">" => Some(F::GreaterThan),
      ">=" => Some(F::GreaterThanOrEqual),
      "<" => Some(F::LessThan),
      "<=" => Some(F::LessThanOrEqual),
      "rand" => Some(F::Rand),
      "rand-int" => Some(F::RandInt),
      "=" => Some(F::Equal),
      "not=" => Some(F::NotEqual),
      "not" => Some(F::Not),
      "and" => Some(F::And),
      "or" => Some(F::Or),
      "xor" => Some(F::Xor),
      "empty?" => Some(F::IsEmpty),
      "first" => Some(F::First),
      "count" => Some(F::Count),
      "flatten" => Some(F::Flatten),
      "remove" => Some(F::Remove),
      "filter" => Some(F::Filter),
      "map" => Some(F::Map),
      "set" => Some(F::Set),
      "set-in" => Some(F::SetIn),
      "get" => Some(F::Get),
      "get-in" => Some(F::GetIn),
      "update" => Some(F::Update),
      "update-in" => Some(F::UpdateIn),
      "min-key" => Some(F::MinKey),
      "max-key" => Some(F::MaxKey),
      "push" => Some(F::Push),
      "sort" => Some(F::Sort),
      "sort-by" => Some(F::SortBy),
      "list" => Some(F::CreateList),
      "last" => Some(F::Last),
      "rest" => Some(F::Rest),
      "butlast" => Some(F::ButLast),
      "nth" => Some(F::Nth),
      "nth-from-last" => Some(F::NthFromLast),
      "cons" => Some(F::Cons),
      "concat" => Some(F::Concat),
      "take" => Some(F::Take),
      "drop" => Some(F::Drop),
      "reverse" => Some(F::Reverse),
      "distinct" => Some(F::Distinct),
      "sub" => Some(F::Sub),
      "partition" => Some(F::Partition),
      "pad" => Some(F::Pad),
      "hashmap" => Some(F::CreateMap),
      "keys" => Some(F::Keys),
      "vals" => Some(F::Values),
      "zip" => Some(F::Zip),
      "invert" => Some(F::Invert),
      "merge" => Some(F::Merge),
      "merge-with" => Some(F::MergeWith),
      "map-keys" => Some(F::MapKeys),
      "map-vals" => Some(F::MapValues),
      "select-keys" => Some(F::SelectKeys),
      "hashset" => Some(F::CreateSet),
      "union" => Some(F::Union),
      "intersection" => Some(F::Intersection),
      "difference" => Some(F::Difference),
      "sym-difference" => Some(F::SymmetricDifference),
      "range" => Some(F::Range),
      "repeat" => Some(F::Repeat),
      "repeatedly" => Some(F::Repeatedly),
      "iterate" => Some(F::Iterate),
      "nil?" => Some(F::IsNil),
      "bool?" => Some(F::IsBool),
      "char?" => Some(F::IsChar),
      "num?" => Some(F::IsNum),
      "int?" => Some(F::IsInt),
      "float?" => Some(F::IsFloat),
      "symbol?" => Some(F::IsSymbol),
      "str?" => Some(F::IsString),
      "list?" => Some(F::IsList),
      "hashmap?" => Some(F::IsMap),
      "hashset?" => Some(F::IsSet),
      "collection?" => Some(F::IsCollection),
      "fn?" => Some(F::IsFn),
      "bool" => Some(F::ToBool),
      "char" => Some(F::ToChar),
      "num" => Some(F::ToNum),
      "int" => Some(F::ToInt),
      "float" => Some(F::ToFloat),
      "symbol" => Some(F::ToSymbol),
      "str" => Some(F::ToString),
      "to-list" => Some(F::ToList),
      "to-hashmap" => Some(F::ToMap),
      _ => None,
    }
  }
}

impl Display for CoreFnId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.name())
  }
}

pub(crate) const CORE_FUNCTIONS: EnumMap<
  CoreFnId,
  fn(Vec<Value>) -> RuntimeResult<Value>,
> = EnumMap::from_array([
  // Print
  |_args: Vec<Value>| todo!(),
  // Apply
  |_args: Vec<Value>| todo!(),
  // When
  |_args: Vec<Value>| todo!(),
  // If
  |_args: Vec<Value>| todo!(),
  // Partial
  |_args: Vec<Value>| todo!(),
  // Compose
  |_args: Vec<Value>| todo!(),
  // FindSome
  |_args: Vec<Value>| todo!(),
  // Reduce
  |_args: Vec<Value>| todo!(),
  // Memoize
  |_args: Vec<Value>| todo!(),
  // Constantly
  |_args: Vec<Value>| todo!(),
  // NumericalEqual
  |_args: Vec<Value>| todo!(),
  // IsZero
  |_args: Vec<Value>| todo!(),
  // IsNan
  |_args: Vec<Value>| todo!(),
  // IsInf
  |_args: Vec<Value>| todo!(),
  // IsEven
  |_args: Vec<Value>| todo!(),
  // IsOdd
  |_args: Vec<Value>| todo!(),
  // IsPos
  |_args: Vec<Value>| todo!(),
  // IsNeg
  |_args: Vec<Value>| todo!(),
  // Inc
  |_args: Vec<Value>| todo!(),
  // Dec
  |_args: Vec<Value>| todo!(),
  // Abs
  |_args: Vec<Value>| todo!(),
  // Floor
  |_args: Vec<Value>| todo!(),
  // Ceil
  |_args: Vec<Value>| todo!(),
  // Sqrt
  |_args: Vec<Value>| todo!(),
  // Exp
  |_args: Vec<Value>| todo!(),
  // Exp2
  |_args: Vec<Value>| todo!(),
  // Ln
  |_args: Vec<Value>| todo!(),
  // Log2
  |_args: Vec<Value>| todo!(),
  // Add
  |args: Vec<Value>| {
    let nums = args
      .iter()
      .map(|v| v.as_num().copied())
      .collect::<RuntimeResult<Vec<Num>>>()?;
    Ok(Number(nums.into_iter().fold(Int(0), |sum, n| sum + n)))
  },
  // Subtract
  |_args: Vec<Value>| todo!(),
  // Multiply
  |_args: Vec<Value>| todo!(),
  // Divide
  |_args: Vec<Value>| todo!(),
  // Pow
  |_args: Vec<Value>| todo!(),
  // Mod
  |_args: Vec<Value>| todo!(),
  // Quot
  |_args: Vec<Value>| todo!(),
  // Min
  |_args: Vec<Value>| todo!(),
  // Max
  |_args: Vec<Value>| todo!(),
  // GreaterThan
  |_args: Vec<Value>| todo!(),
  // GreaterThanOrEqual
  |_args: Vec<Value>| todo!(),
  // LessThan
  |_args: Vec<Value>| todo!(),
  // LessThanOrEqual
  |_args: Vec<Value>| todo!(),
  // Rand
  |_args: Vec<Value>| todo!(),
  // RandInt
  |_args: Vec<Value>| todo!(),
  // Equal
  |_args: Vec<Value>| todo!(),
  // NotEqual
  |_args: Vec<Value>| todo!(),
  // Not
  |_args: Vec<Value>| todo!(),
  // And
  |_args: Vec<Value>| todo!(),
  // Or
  |_args: Vec<Value>| todo!(),
  // Xor
  |_args: Vec<Value>| todo!(),
  // IsEmpty
  |_args: Vec<Value>| todo!(),
  // First
  |_args: Vec<Value>| todo!(),
  // Count
  |_args: Vec<Value>| todo!(),
  // Flatten
  |_args: Vec<Value>| todo!(),
  // Remove
  |_args: Vec<Value>| todo!(),
  // Filter
  |_args: Vec<Value>| todo!(),
  // Map
  |_args: Vec<Value>| todo!(),
  // Set
  |_args: Vec<Value>| todo!(),
  // SetIn
  |_args: Vec<Value>| todo!(),
  // Get
  |_args: Vec<Value>| todo!(),
  // GetIn
  |_args: Vec<Value>| todo!(),
  // Update
  |_args: Vec<Value>| todo!(),
  // UpdateIn
  |_args: Vec<Value>| todo!(),
  // MinKey
  |_args: Vec<Value>| todo!(),
  // MaxKey
  |_args: Vec<Value>| todo!(),
  // Push
  |_args: Vec<Value>| todo!(),
  // Sort
  |_args: Vec<Value>| todo!(),
  // SortBy
  |_args: Vec<Value>| todo!(),
  // CreateList
  |_args: Vec<Value>| todo!(),
  // Last
  |_args: Vec<Value>| todo!(),
  // Rest
  |_args: Vec<Value>| todo!(),
  // ButLast
  |_args: Vec<Value>| todo!(),
  // Nth
  |_args: Vec<Value>| todo!(),
  // NthFromLast
  |_args: Vec<Value>| todo!(),
  // Cons
  |_args: Vec<Value>| todo!(),
  // Concat
  |_args: Vec<Value>| todo!(),
  // Take
  |_args: Vec<Value>| todo!(),
  // Drop
  |_args: Vec<Value>| todo!(),
  // Reverse
  |_args: Vec<Value>| todo!(),
  // Distinct
  |_args: Vec<Value>| todo!(),
  // Sub
  |_args: Vec<Value>| todo!(),
  // Partition
  |_args: Vec<Value>| todo!(),
  // Pad
  |_args: Vec<Value>| todo!(),
  // CreateMap
  |_args: Vec<Value>| todo!(),
  // Keys
  |_args: Vec<Value>| todo!(),
  // Values
  |_args: Vec<Value>| todo!(),
  // Zip
  |_args: Vec<Value>| todo!(),
  // Invert
  |_args: Vec<Value>| todo!(),
  // Merge
  |_args: Vec<Value>| todo!(),
  // MergeWith
  |_args: Vec<Value>| todo!(),
  // MapKeys
  |_args: Vec<Value>| todo!(),
  // MapValues
  |_args: Vec<Value>| todo!(),
  // SelectKeys
  |_args: Vec<Value>| todo!(),
  // CreateSet
  |_args: Vec<Value>| todo!(),
  // Union
  |_args: Vec<Value>| todo!(),
  // Intersection
  |_args: Vec<Value>| todo!(),
  // Difference
  |_args: Vec<Value>| todo!(),
  // SymmetricDifference
  |_args: Vec<Value>| todo!(),
  // Range
  |_args: Vec<Value>| todo!(),
  // Repeat
  |_args: Vec<Value>| todo!(),
  // Repeatedly
  |_args: Vec<Value>| todo!(),
  // Iterate
  |_args: Vec<Value>| todo!(),
  // IsNil
  |_args: Vec<Value>| todo!(),
  // IsBool
  |_args: Vec<Value>| todo!(),
  // IsChar
  |_args: Vec<Value>| todo!(),
  // IsNum
  |_args: Vec<Value>| todo!(),
  // IsInt
  |_args: Vec<Value>| todo!(),
  // IsFloat
  |_args: Vec<Value>| todo!(),
  // IsSymbol
  |_args: Vec<Value>| todo!(),
  // IsString
  |_args: Vec<Value>| todo!(),
  // IsList
  |_args: Vec<Value>| todo!(),
  // IsMap
  |_args: Vec<Value>| todo!(),
  // IsSet
  |_args: Vec<Value>| todo!(),
  // IsCollection
  |_args: Vec<Value>| todo!(),
  // IsFn
  |_args: Vec<Value>| todo!(),
  // ToBool
  |_args: Vec<Value>| todo!(),
  // ToChar
  |_args: Vec<Value>| todo!(),
  // ToNum
  |_args: Vec<Value>| todo!(),
  // ToInt
  |_args: Vec<Value>| todo!(),
  // ToFloat
  |_args: Vec<Value>| todo!(),
  // ToSymbol
  |_args: Vec<Value>| todo!(),
  // ToString
  |_args: Vec<Value>| todo!(),
  // ToList
  |_args: Vec<Value>| todo!(),
  // ToMap
  |_args: Vec<Value>| todo!(),
  // CreateCell
  |_args: Vec<Value>| todo!(),
  // GetCellValue
  |_args: Vec<Value>| todo!(),
  // SetCellValue
  |_args: Vec<Value>| todo!(),
  // UpdateCell
  |_args: Vec<Value>| todo!(),
]);
