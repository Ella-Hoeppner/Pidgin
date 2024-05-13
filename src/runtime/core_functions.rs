use std::fmt::Display;

use crate::runtime::error::PidginResult;
use crate::{GenericValue, Num, Value};
use enum_map::{enum_map, Enum, EnumMap};
use GenericValue::*;
use Num::*;

#[derive(Debug, Enum, PartialEq, Eq, Hash, Clone)]
pub(crate) enum CoreFnId {
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
  fn name(&self) -> &str {
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
  fn from_name(name: &str) -> Option<Self> {
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
  fn(Vec<Value>) -> PidginResult<Value>,
> = EnumMap::from_array([
  // Print
  |args: Vec<Value>| todo!(),
  // Apply
  |args: Vec<Value>| todo!(),
  // When
  |args: Vec<Value>| todo!(),
  // If
  |args: Vec<Value>| todo!(),
  // Partial
  |args: Vec<Value>| todo!(),
  // Compose
  |args: Vec<Value>| todo!(),
  // FindSome
  |args: Vec<Value>| todo!(),
  // Reduce
  |args: Vec<Value>| todo!(),
  // Memoize
  |args: Vec<Value>| todo!(),
  // Constantly
  |args: Vec<Value>| todo!(),
  // NumericalEqual
  |args: Vec<Value>| todo!(),
  // IsZero
  |args: Vec<Value>| todo!(),
  // IsNan
  |args: Vec<Value>| todo!(),
  // IsInf
  |args: Vec<Value>| todo!(),
  // IsEven
  |args: Vec<Value>| todo!(),
  // IsOdd
  |args: Vec<Value>| todo!(),
  // IsPos
  |args: Vec<Value>| todo!(),
  // IsNeg
  |args: Vec<Value>| todo!(),
  // Inc
  |args: Vec<Value>| todo!(),
  // Dec
  |args: Vec<Value>| todo!(),
  // Abs
  |args: Vec<Value>| todo!(),
  // Floor
  |args: Vec<Value>| todo!(),
  // Ceil
  |args: Vec<Value>| todo!(),
  // Sqrt
  |args: Vec<Value>| todo!(),
  // Exp
  |args: Vec<Value>| todo!(),
  // Exp2
  |args: Vec<Value>| todo!(),
  // Ln
  |args: Vec<Value>| todo!(),
  // Log2
  |args: Vec<Value>| todo!(),
  // Add
  |args: Vec<Value>| {
    let nums = args
      .iter()
      .map(|v| v.as_num().copied())
      .collect::<PidginResult<Vec<Num>>>()?;
    Ok(Number(nums.into_iter().fold(Int(0), |sum, n| sum + n)))
  },
  // Subtract
  |args: Vec<Value>| todo!(),
  // Multiply
  |args: Vec<Value>| todo!(),
  // Divide
  |args: Vec<Value>| todo!(),
  // Pow
  |args: Vec<Value>| todo!(),
  // Mod
  |args: Vec<Value>| todo!(),
  // Quot
  |args: Vec<Value>| todo!(),
  // Min
  |args: Vec<Value>| todo!(),
  // Max
  |args: Vec<Value>| todo!(),
  // GreaterThan
  |args: Vec<Value>| todo!(),
  // GreaterThanOrEqual
  |args: Vec<Value>| todo!(),
  // LessThan
  |args: Vec<Value>| todo!(),
  // LessThanOrEqual
  |args: Vec<Value>| todo!(),
  // Rand
  |args: Vec<Value>| todo!(),
  // RandInt
  |args: Vec<Value>| todo!(),
  // Equal
  |args: Vec<Value>| todo!(),
  // NotEqual
  |args: Vec<Value>| todo!(),
  // Not
  |args: Vec<Value>| todo!(),
  // And
  |args: Vec<Value>| todo!(),
  // Or
  |args: Vec<Value>| todo!(),
  // Xor
  |args: Vec<Value>| todo!(),
  // IsEmpty
  |args: Vec<Value>| todo!(),
  // First
  |args: Vec<Value>| todo!(),
  // Count
  |args: Vec<Value>| todo!(),
  // Flatten
  |args: Vec<Value>| todo!(),
  // Remove
  |args: Vec<Value>| todo!(),
  // Filter
  |args: Vec<Value>| todo!(),
  // Map
  |args: Vec<Value>| todo!(),
  // Set
  |args: Vec<Value>| todo!(),
  // SetIn
  |args: Vec<Value>| todo!(),
  // Get
  |args: Vec<Value>| todo!(),
  // GetIn
  |args: Vec<Value>| todo!(),
  // Update
  |args: Vec<Value>| todo!(),
  // UpdateIn
  |args: Vec<Value>| todo!(),
  // MinKey
  |args: Vec<Value>| todo!(),
  // MaxKey
  |args: Vec<Value>| todo!(),
  // Push
  |args: Vec<Value>| todo!(),
  // Sort
  |args: Vec<Value>| todo!(),
  // SortBy
  |args: Vec<Value>| todo!(),
  // CreateList
  |args: Vec<Value>| todo!(),
  // Last
  |args: Vec<Value>| todo!(),
  // Rest
  |args: Vec<Value>| todo!(),
  // ButLast
  |args: Vec<Value>| todo!(),
  // Nth
  |args: Vec<Value>| todo!(),
  // NthFromLast
  |args: Vec<Value>| todo!(),
  // Cons
  |args: Vec<Value>| todo!(),
  // Concat
  |args: Vec<Value>| todo!(),
  // Take
  |args: Vec<Value>| todo!(),
  // Drop
  |args: Vec<Value>| todo!(),
  // Reverse
  |args: Vec<Value>| todo!(),
  // Distinct
  |args: Vec<Value>| todo!(),
  // Sub
  |args: Vec<Value>| todo!(),
  // Partition
  |args: Vec<Value>| todo!(),
  // Pad
  |args: Vec<Value>| todo!(),
  // CreateMap
  |args: Vec<Value>| todo!(),
  // Keys
  |args: Vec<Value>| todo!(),
  // Values
  |args: Vec<Value>| todo!(),
  // Zip
  |args: Vec<Value>| todo!(),
  // Invert
  |args: Vec<Value>| todo!(),
  // Merge
  |args: Vec<Value>| todo!(),
  // MergeWith
  |args: Vec<Value>| todo!(),
  // MapKeys
  |args: Vec<Value>| todo!(),
  // MapValues
  |args: Vec<Value>| todo!(),
  // SelectKeys
  |args: Vec<Value>| todo!(),
  // CreateSet
  |args: Vec<Value>| todo!(),
  // Union
  |args: Vec<Value>| todo!(),
  // Intersection
  |args: Vec<Value>| todo!(),
  // Difference
  |args: Vec<Value>| todo!(),
  // SymmetricDifference
  |args: Vec<Value>| todo!(),
  // Range
  |args: Vec<Value>| todo!(),
  // Repeat
  |args: Vec<Value>| todo!(),
  // Repeatedly
  |args: Vec<Value>| todo!(),
  // Iterate
  |args: Vec<Value>| todo!(),
  // IsNil
  |args: Vec<Value>| todo!(),
  // IsBool
  |args: Vec<Value>| todo!(),
  // IsChar
  |args: Vec<Value>| todo!(),
  // IsNum
  |args: Vec<Value>| todo!(),
  // IsInt
  |args: Vec<Value>| todo!(),
  // IsFloat
  |args: Vec<Value>| todo!(),
  // IsSymbol
  |args: Vec<Value>| todo!(),
  // IsString
  |args: Vec<Value>| todo!(),
  // IsList
  |args: Vec<Value>| todo!(),
  // IsMap
  |args: Vec<Value>| todo!(),
  // IsSet
  |args: Vec<Value>| todo!(),
  // IsCollection
  |args: Vec<Value>| todo!(),
  // IsFn
  |args: Vec<Value>| todo!(),
  // ToBool
  |args: Vec<Value>| todo!(),
  // ToChar
  |args: Vec<Value>| todo!(),
  // ToNum
  |args: Vec<Value>| todo!(),
  // ToInt
  |args: Vec<Value>| todo!(),
  // ToFloat
  |args: Vec<Value>| todo!(),
  // ToSymbol
  |args: Vec<Value>| todo!(),
  // ToString
  |args: Vec<Value>| todo!(),
  // ToList
  |args: Vec<Value>| todo!(),
  // ToMap
  |args: Vec<Value>| todo!(),
  // CreateCell
  |args: Vec<Value>| todo!(),
  // GetCellValue
  |args: Vec<Value>| todo!(),
  // SetCellValue
  |args: Vec<Value>| todo!(),
  // UpdateCell
  |args: Vec<Value>| todo!(),
]);
