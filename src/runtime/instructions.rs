use crate::{ConstIndex, RegisterIndex, SymbolIndex};

type R = RegisterIndex;
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Instruction {
  DebugPrint(u8),

  // Register manipulation
  NoOp,
  Clear(R),
  Copy(R, R),
  Const(R, ConstIndex),

  // Output
  Print(R),

  // Function control flow
  Argument(SymbolIndex),
  Return(R),

  // Environment manipulation
  Lookup(R, SymbolIndex),
  Bind(SymbolIndex, R),

  // Control flow
  // first register of If is used both for condition and return value
  If(R, R, R),

  // Higher-order functions
  Apply(R, R, R),
  Partial(R, R, R),
  Compose(R, R, R),
  Filter(R, R, R),
  Map(R, R, R),
  MultiListMap(R, R, R),
  ReduceWithoutInitialValue(R, R, R),
  // first register of ReduceWithInitialValue is used for both initial value and
  // return value
  ReduceWithInitialValue(R, R, R),
  InfiniteIterate(R, R, R),

  // Math
  NumericalEqual(R, R),
  IsZero(R, R),
  IsNan(R, R),
  IsInf(R, R),
  IsEven(R, R),
  Abs(R, R),
  Inc(R, R),
  Dec(R, R),
  Add(R, R, R),
  Subtract(R, R, R),
  Multiply(R, R, R),
  Divide(R, R, R),
  Mod(R, R, R),
  Quot(R, R, R),
  Floor(R, R),
  Ceil(R, R),
  Min(R, R, R),
  MinAll(R, R),
  Max(R, R, R),
  MaxAll(R, R),
  GreaterThan(R, R, R),
  GreaterThanOrEqual(R, R, R),
  LessThan(R, R, R),
  LessThanOrEqual(R, R, R),
  Rand(R),
  UpperBoundedRand(R, R),
  LowerUpperBoundedRand(R, R, R),

  // Boolean logic
  Equal(R, R, R),
  Not(R, R),
  And(R, R, R),
  Or(R, R, R),
  Xor(R, R, R),

  // List manipulation (most apply to strings as well)
  EmptyList(R),
  Count(R, R),
  First(R, R),
  Last(R, R),
  Cons(R, R, R),
  Push(R, R, R),
  Concat(R, R, R),
  Take(R, R, R),
  Drop(R, R, R),
  Reverse(R, R),

  // Specialized list constructors
  InfiniteRange(R),
  UpperBoundedRange(R, R),
  LowerUpperBoundedRange(R, R, R),
  InfiniteRepeat(R, R),
  BoundedRepeat(R, R, R),
  InfiniteRepeatedly(R, R),
  BoundedRepeateedly(R, R, R),

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

  // List + Map manipulation
  IsEmpty(R, R),
  Set(R, R, R),
  Get(R, R, R),
  Update(R, R, R),
}
