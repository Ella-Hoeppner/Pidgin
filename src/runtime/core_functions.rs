use crate::runtime::Result;
use crate::{Num, Value};
use enum_map::{enum_map, Enum, EnumMap};
use minivec::MiniVec;
use Num::*;
use Value::*;

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
pub(crate) const CORE_FUNCTIONS: EnumMap<
  CoreFnId,
  fn(MiniVec<Value>) -> Result<Value>,
> = EnumMap::from_array([
  // Print
  |args: MiniVec<Value>| todo!(),
  // Apply
  |args: MiniVec<Value>| todo!(),
  // When
  |args: MiniVec<Value>| todo!(),
  // If
  |args: MiniVec<Value>| todo!(),
  // Partial
  |args: MiniVec<Value>| todo!(),
  // Compose
  |args: MiniVec<Value>| todo!(),
  // FindSome
  |args: MiniVec<Value>| todo!(),
  // Reduce
  |args: MiniVec<Value>| todo!(),
  // Memoize
  |args: MiniVec<Value>| todo!(),
  // Constantly
  |args: MiniVec<Value>| todo!(),
  // NumericalEqual
  |args: MiniVec<Value>| todo!(),
  // IsZero
  |args: MiniVec<Value>| todo!(),
  // IsNan
  |args: MiniVec<Value>| todo!(),
  // IsInf
  |args: MiniVec<Value>| todo!(),
  // IsEven
  |args: MiniVec<Value>| todo!(),
  // IsOdd
  |args: MiniVec<Value>| todo!(),
  // IsPos
  |args: MiniVec<Value>| todo!(),
  // IsNeg
  |args: MiniVec<Value>| todo!(),
  // Inc
  |args: MiniVec<Value>| todo!(),
  // Dec
  |args: MiniVec<Value>| todo!(),
  // Abs
  |args: MiniVec<Value>| todo!(),
  // Floor
  |args: MiniVec<Value>| todo!(),
  // Ceil
  |args: MiniVec<Value>| todo!(),
  // Sqrt
  |args: MiniVec<Value>| todo!(),
  // Exp
  |args: MiniVec<Value>| todo!(),
  // Exp2
  |args: MiniVec<Value>| todo!(),
  // Ln
  |args: MiniVec<Value>| todo!(),
  // Log2
  |args: MiniVec<Value>| todo!(),
  // Add
  |args: MiniVec<Value>| {
    let nums = args
      .iter()
      .map(|v| v.as_num().copied())
      .collect::<Result<Vec<Num>>>()?;
    Ok(Number(nums.into_iter().fold(Int(0), |sum, n| sum + n)))
  },
  // Subtract
  |args: MiniVec<Value>| todo!(),
  // Multiply
  |args: MiniVec<Value>| todo!(),
  // Divide
  |args: MiniVec<Value>| todo!(),
  // Pow
  |args: MiniVec<Value>| todo!(),
  // Mod
  |args: MiniVec<Value>| todo!(),
  // Quot
  |args: MiniVec<Value>| todo!(),
  // Min
  |args: MiniVec<Value>| todo!(),
  // Max
  |args: MiniVec<Value>| todo!(),
  // GreaterThan
  |args: MiniVec<Value>| todo!(),
  // GreaterThanOrEqual
  |args: MiniVec<Value>| todo!(),
  // LessThan
  |args: MiniVec<Value>| todo!(),
  // LessThanOrEqual
  |args: MiniVec<Value>| todo!(),
  // Rand
  |args: MiniVec<Value>| todo!(),
  // RandInt
  |args: MiniVec<Value>| todo!(),
  // Equal
  |args: MiniVec<Value>| todo!(),
  // NotEqual
  |args: MiniVec<Value>| todo!(),
  // Not
  |args: MiniVec<Value>| todo!(),
  // And
  |args: MiniVec<Value>| todo!(),
  // Or
  |args: MiniVec<Value>| todo!(),
  // Xor
  |args: MiniVec<Value>| todo!(),
  // IsEmpty
  |args: MiniVec<Value>| todo!(),
  // First
  |args: MiniVec<Value>| todo!(),
  // Count
  |args: MiniVec<Value>| todo!(),
  // Flatten
  |args: MiniVec<Value>| todo!(),
  // Remove
  |args: MiniVec<Value>| todo!(),
  // Filter
  |args: MiniVec<Value>| todo!(),
  // Map
  |args: MiniVec<Value>| todo!(),
  // Set
  |args: MiniVec<Value>| todo!(),
  // SetIn
  |args: MiniVec<Value>| todo!(),
  // Get
  |args: MiniVec<Value>| todo!(),
  // GetIn
  |args: MiniVec<Value>| todo!(),
  // Update
  |args: MiniVec<Value>| todo!(),
  // UpdateIn
  |args: MiniVec<Value>| todo!(),
  // MinKey
  |args: MiniVec<Value>| todo!(),
  // MaxKey
  |args: MiniVec<Value>| todo!(),
  // Push
  |args: MiniVec<Value>| todo!(),
  // Sort
  |args: MiniVec<Value>| todo!(),
  // SortBy
  |args: MiniVec<Value>| todo!(),
  // CreateList
  |args: MiniVec<Value>| todo!(),
  // Last
  |args: MiniVec<Value>| todo!(),
  // Rest
  |args: MiniVec<Value>| todo!(),
  // ButLast
  |args: MiniVec<Value>| todo!(),
  // Nth
  |args: MiniVec<Value>| todo!(),
  // NthFromLast
  |args: MiniVec<Value>| todo!(),
  // Cons
  |args: MiniVec<Value>| todo!(),
  // Concat
  |args: MiniVec<Value>| todo!(),
  // Take
  |args: MiniVec<Value>| todo!(),
  // Drop
  |args: MiniVec<Value>| todo!(),
  // Reverse
  |args: MiniVec<Value>| todo!(),
  // Distinct
  |args: MiniVec<Value>| todo!(),
  // Sub
  |args: MiniVec<Value>| todo!(),
  // Partition
  |args: MiniVec<Value>| todo!(),
  // Pad
  |args: MiniVec<Value>| todo!(),
  // CreateMap
  |args: MiniVec<Value>| todo!(),
  // Keys
  |args: MiniVec<Value>| todo!(),
  // Values
  |args: MiniVec<Value>| todo!(),
  // Zip
  |args: MiniVec<Value>| todo!(),
  // Invert
  |args: MiniVec<Value>| todo!(),
  // Merge
  |args: MiniVec<Value>| todo!(),
  // MergeWith
  |args: MiniVec<Value>| todo!(),
  // MapKeys
  |args: MiniVec<Value>| todo!(),
  // MapValues
  |args: MiniVec<Value>| todo!(),
  // SelectKeys
  |args: MiniVec<Value>| todo!(),
  // CreateSet
  |args: MiniVec<Value>| todo!(),
  // Union
  |args: MiniVec<Value>| todo!(),
  // Intersection
  |args: MiniVec<Value>| todo!(),
  // Difference
  |args: MiniVec<Value>| todo!(),
  // SymmetricDifference
  |args: MiniVec<Value>| todo!(),
  // Range
  |args: MiniVec<Value>| todo!(),
  // Repeat
  |args: MiniVec<Value>| todo!(),
  // Repeatedly
  |args: MiniVec<Value>| todo!(),
  // Iterate
  |args: MiniVec<Value>| todo!(),
  // IsNil
  |args: MiniVec<Value>| todo!(),
  // IsBool
  |args: MiniVec<Value>| todo!(),
  // IsChar
  |args: MiniVec<Value>| todo!(),
  // IsNum
  |args: MiniVec<Value>| todo!(),
  // IsInt
  |args: MiniVec<Value>| todo!(),
  // IsFloat
  |args: MiniVec<Value>| todo!(),
  // IsSymbol
  |args: MiniVec<Value>| todo!(),
  // IsString
  |args: MiniVec<Value>| todo!(),
  // IsList
  |args: MiniVec<Value>| todo!(),
  // IsMap
  |args: MiniVec<Value>| todo!(),
  // IsSet
  |args: MiniVec<Value>| todo!(),
  // IsCollection
  |args: MiniVec<Value>| todo!(),
  // IsFn
  |args: MiniVec<Value>| todo!(),
  // ToBool
  |args: MiniVec<Value>| todo!(),
  // ToChar
  |args: MiniVec<Value>| todo!(),
  // ToNum
  |args: MiniVec<Value>| todo!(),
  // ToInt
  |args: MiniVec<Value>| todo!(),
  // ToFloat
  |args: MiniVec<Value>| todo!(),
  // ToSymbol
  |args: MiniVec<Value>| todo!(),
  // ToString
  |args: MiniVec<Value>| todo!(),
  // ToList
  |args: MiniVec<Value>| todo!(),
  // ToMap
  |args: MiniVec<Value>| todo!(),
  // CreateCell
  |args: MiniVec<Value>| todo!(),
  // GetCellValue
  |args: MiniVec<Value>| todo!(),
  // SetCellValue
  |args: MiniVec<Value>| todo!(),
  // UpdateCell
  |args: MiniVec<Value>| todo!(),
]);
