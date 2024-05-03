use crate::{ConstIndex, RegisterIndex, StackIndex, SymbolIndex};

type R = RegisterIndex; // for brevity
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Instruction {
  DebugPrint(u8),

  // Register manipulation
  Clear(R),
  Copy(R, R),
  Const(R, ConstIndex),

  // Output
  Print(R),

  // RawVec manipulation (used for function argument and List construction)
  EmptyRawVec(R, u8),
  CopyIntoRawVec(R, R),
  StealIntoRawVec(R, R),

  // Function control flow
  Return(R),
  CallingFunction(R),
  Apply0(R, R),
  Apply1(R, R),
  Apply2(R, R, R),
  ApplyN(R, R),
  Apply0AndReturn(R),
  Apply1AndReturn(R, R),
  Apply2AndReturn(R, R, R),
  ApplyNAndReturn(R, R),

  // Environment manipulation
  Lookup(R, SymbolIndex),

  // Control flow
  If(R),
  Else,
  ElseIf(R),
  EndIf,

  // Functions manipulation
  Partial(R, R, R),
  Compose(R, R, R),
  FindSome(R, R, R),
  ReduceWithoutInitialValue(R, R, R),
  ReduceWithInitialValue(R, R, R),
  Memoize(R, R),

  // Special function constructors
  Constantly(R, R),

  // Math
  NumericalEqual(R, R, R),
  IsZero(R, R),
  IsNan(R, R),
  IsInf(R, R),
  IsEven(R, R),
  IsOdd(R, R),
  IsPos(R, R),
  IsNeg(R, R),
  Inc(R, R),
  Dec(R, R),
  Negate(R, R),
  Abs(R, R),
  Floor(R, R),
  Ceil(R, R),
  Sqrt(R, R),
  Exp(R, R),
  Exp2(R, R),
  Ln(R, R),
  Log2(R, R),
  Add(R, R, R),
  Subtract(R, R, R),
  Multiply(R, R, R),
  Divide(R, R, R),
  Pow(R, R, R),
  Mod(R, R, R),
  Quot(R, R, R),
  Min(R, R, R),
  Max(R, R, R),
  GreaterThan(R, R, R),
  GreaterThanOrEqual(R, R, R),
  LessThan(R, R, R),
  LessThanOrEqual(R, R, R),
  Rand(R),
  UpperBoundedRand(R, R),
  LowerUpperBoundedRand(R, R, R),
  RandInt(R, R),
  LowerBoundedRandInt(R, R, R),

  // Boolean logic
  Equal(R, R, R),
  NotEqual(R, R, R),
  Not(R, R),
  And(R, R, R),
  Or(R, R, R),
  Xor(R, R, R),

  // List + Map + Set manipulation
  IsEmpty(R, R),
  First(R, R),
  Count(R, R),
  Flatten(R, R),
  Remove(R, R),
  Filter(R, R),
  Map(R, R),
  DoubleMap(R, R, R),
  MultiCollectionMap(R, R),

  // List + Map manipulation
  Set(R, R, R),
  SetIn(R, R, R),
  Get(R, R, R),
  GetIn(R, R, R),
  Update(R, R, R),
  UpdateIn(R, R, R),
  MinKey(R, R, R),
  MaxKey(R, R, R),

  // List + Set manipulation
  Push(R, R),
  Sort(R),
  SortBy(R, R),

  // List manipulation (most apply to strings as well)
  EmptyList(R),
  ListFromRawVec(R),
  Last(R, R),
  Rest(R),
  ButLast(R),
  Nth(R, R, R),
  NthFromLast(R, R, R),
  Cons(R, R),
  Concat(R, R),
  Take(R, R),
  Drop(R, R),
  Reverse(R),
  Distinct(R),
  Sub(R, R, R),
  Partition(R, R, R),
  SteppedPartition(R, R, R),
  Pad(R, R, R),

  // Map manipulation
  EmptyMap(R),
  Keys(R, R),
  Values(R, R),
  Zip(R, R, R),
  Invert(R, R),
  Merge(R, R, R),
  MergeWith(R, R, R),
  MapKeys(R, R, R),
  MapValues(R, R, R),
  SelectKeys(R, R),

  // Set manipulation
  EmptySet(R),
  Union(R, R, R),
  Intersection(R, R, R),
  Difference(R, R, R),
  SymmetricDifference(R, R, R),

  // Specialized list constructors
  InfiniteRange(R),
  UpperBoundedRange(R, R),
  LowerUpperBoundedRange(R, R, R),
  InfiniteRepeat(R, R),
  BoundedRepeat(R, R, R),
  InfiniteRepeatedly(R, R),
  BoundedRepeatedly(R, R, R),
  InfiniteIterate(R, R, R),
  BoundedIterate(R, R, R),

  // Type checkers
  IsNil(R, R),
  IsBool(R, R),
  IsChar(R, R),
  IsNum(R, R),
  IsInt(R, R),
  IsFloat(R, R),
  IsSymbol(R, R),
  IsString(R, R),
  IsList(R, R),
  IsMap(R, R),
  IsSet(R, R),
  IsCollection(R, R),
  IsFn(R, R),

  // Type conversions
  ToBool(R, R),
  ToChar(R, R),
  ToNum(R, R),
  ToInt(R, R),
  ToFloat(R, R),
  ToSymbol(R, R),
  ToString(R, R),
  ToList(R, R),
  ToMap(R, R),

  // Cells
  CreateCell(R),
  GetCellValue(R, R),
  SetCellValue(R, R),
  UpdateCell(R, R),
}
