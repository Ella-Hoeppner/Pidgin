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

impl Display for CoreFnId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      CoreFnId::Print => todo!(),
      CoreFnId::Apply => todo!(),
      CoreFnId::When => todo!(),
      CoreFnId::If => todo!(),
      CoreFnId::Partial => todo!(),
      CoreFnId::Compose => todo!(),
      CoreFnId::FindSome => todo!(),
      CoreFnId::Reduce => todo!(),
      CoreFnId::Memoize => todo!(),
      CoreFnId::Constantly => todo!(),
      CoreFnId::NumericalEqual => todo!(),
      CoreFnId::IsZero => todo!(),
      CoreFnId::IsNan => todo!(),
      CoreFnId::IsInf => todo!(),
      CoreFnId::IsEven => todo!(),
      CoreFnId::IsOdd => todo!(),
      CoreFnId::IsPos => todo!(),
      CoreFnId::IsNeg => todo!(),
      CoreFnId::Inc => todo!(),
      CoreFnId::Dec => todo!(),
      CoreFnId::Abs => todo!(),
      CoreFnId::Floor => todo!(),
      CoreFnId::Ceil => todo!(),
      CoreFnId::Sqrt => todo!(),
      CoreFnId::Exp => todo!(),
      CoreFnId::Exp2 => todo!(),
      CoreFnId::Ln => todo!(),
      CoreFnId::Log2 => todo!(),
      CoreFnId::Add => todo!(),
      CoreFnId::Subtract => todo!(),
      CoreFnId::Multiply => todo!(),
      CoreFnId::Divide => todo!(),
      CoreFnId::Pow => todo!(),
      CoreFnId::Mod => todo!(),
      CoreFnId::Quot => todo!(),
      CoreFnId::Min => todo!(),
      CoreFnId::Max => todo!(),
      CoreFnId::GreaterThan => todo!(),
      CoreFnId::GreaterThanOrEqual => todo!(),
      CoreFnId::LessThan => todo!(),
      CoreFnId::LessThanOrEqual => todo!(),
      CoreFnId::Rand => todo!(),
      CoreFnId::RandInt => todo!(),
      CoreFnId::Equal => todo!(),
      CoreFnId::NotEqual => todo!(),
      CoreFnId::Not => todo!(),
      CoreFnId::And => todo!(),
      CoreFnId::Or => todo!(),
      CoreFnId::Xor => todo!(),
      CoreFnId::IsEmpty => todo!(),
      CoreFnId::First => todo!(),
      CoreFnId::Count => todo!(),
      CoreFnId::Flatten => todo!(),
      CoreFnId::Remove => todo!(),
      CoreFnId::Filter => todo!(),
      CoreFnId::Map => todo!(),
      CoreFnId::Set => todo!(),
      CoreFnId::SetIn => todo!(),
      CoreFnId::Get => todo!(),
      CoreFnId::GetIn => todo!(),
      CoreFnId::Update => todo!(),
      CoreFnId::UpdateIn => todo!(),
      CoreFnId::MinKey => todo!(),
      CoreFnId::MaxKey => todo!(),
      CoreFnId::Push => todo!(),
      CoreFnId::Sort => todo!(),
      CoreFnId::SortBy => todo!(),
      CoreFnId::CreateList => todo!(),
      CoreFnId::Last => todo!(),
      CoreFnId::Rest => todo!(),
      CoreFnId::ButLast => todo!(),
      CoreFnId::Nth => todo!(),
      CoreFnId::NthFromLast => todo!(),
      CoreFnId::Cons => todo!(),
      CoreFnId::Concat => todo!(),
      CoreFnId::Take => todo!(),
      CoreFnId::Drop => todo!(),
      CoreFnId::Reverse => todo!(),
      CoreFnId::Distinct => todo!(),
      CoreFnId::Sub => todo!(),
      CoreFnId::Partition => todo!(),
      CoreFnId::Pad => todo!(),
      CoreFnId::CreateMap => todo!(),
      CoreFnId::Keys => todo!(),
      CoreFnId::Values => todo!(),
      CoreFnId::Zip => todo!(),
      CoreFnId::Invert => todo!(),
      CoreFnId::Merge => todo!(),
      CoreFnId::MergeWith => todo!(),
      CoreFnId::MapKeys => todo!(),
      CoreFnId::MapValues => todo!(),
      CoreFnId::SelectKeys => todo!(),
      CoreFnId::CreateSet => todo!(),
      CoreFnId::Union => todo!(),
      CoreFnId::Intersection => todo!(),
      CoreFnId::Difference => todo!(),
      CoreFnId::SymmetricDifference => todo!(),
      CoreFnId::Range => todo!(),
      CoreFnId::Repeat => todo!(),
      CoreFnId::Repeatedly => todo!(),
      CoreFnId::Iterate => todo!(),
      CoreFnId::IsNil => todo!(),
      CoreFnId::IsBool => todo!(),
      CoreFnId::IsChar => todo!(),
      CoreFnId::IsNum => todo!(),
      CoreFnId::IsInt => todo!(),
      CoreFnId::IsFloat => todo!(),
      CoreFnId::IsSymbol => todo!(),
      CoreFnId::IsString => todo!(),
      CoreFnId::IsList => todo!(),
      CoreFnId::IsMap => todo!(),
      CoreFnId::IsSet => todo!(),
      CoreFnId::IsCollection => todo!(),
      CoreFnId::IsFn => todo!(),
      CoreFnId::ToBool => todo!(),
      CoreFnId::ToChar => todo!(),
      CoreFnId::ToNum => todo!(),
      CoreFnId::ToInt => todo!(),
      CoreFnId::ToFloat => todo!(),
      CoreFnId::ToSymbol => todo!(),
      CoreFnId::ToString => todo!(),
      CoreFnId::ToList => todo!(),
      CoreFnId::ToMap => todo!(),
      CoreFnId::CreateCell => todo!(),
      CoreFnId::GetCellValue => todo!(),
      CoreFnId::SetCellValue => todo!(),
      CoreFnId::UpdateCell => todo!(),
    }
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
