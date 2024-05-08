use std::{ops::Index, rc::Rc};

use crate::{ConstIndex, RegisterIndex, SymbolIndex, Value};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Instruction<R, C> {
  DebugPrint(u8),

  // Register manipulation
  Clear(R),
  Copy(R, R),
  Const(R, C),

  // Output
  Print(R),

  // Function control flow
  Return(R),
  CopyArgument(R),
  StealArgument(R),
  Call(R, R, u8),
  Apply(R, R),
  CallSelf(R, u8),
  ApplySelf(R),
  CallAndReturn(R, u8),
  ApplyAndReturn(R, R),
  CallSelfAndReturn(u8),
  ApplySelfAndReturn(R),
  CallingFunction(R),
  Jump(u16),

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

  // Cells
  CreateCell(R),
  GetCellValue(R, R),
  SetCellValue(R, R),
  UpdateCell(R, R),

  // Coroutines
  CreateCoroutine(R),
  IsCoroutineAlive(R, R),
  Yield(R),
  YieldAndAccept(R, R, u8),

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
  IsError(R, R),
  IsCell(R, R),
  IsCoroutine(R, R),

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
  ToSet(R, R),
  ToError(R, R),
}
use Instruction::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InstructionBlock<R, C, M> {
  pub instructions: Rc<[Instruction<R, C>]>,
  pub metadata: M,
}
impl<R, C, M> Index<usize> for InstructionBlock<R, C, M> {
  type Output = Instruction<R, C>;
  fn index(&self, index: usize) -> &Self::Output {
    &self.instructions[index]
  }
}
impl<R, C, M> InstructionBlock<R, C, M> {
  pub fn len(&self) -> usize {
    self.instructions.len()
  }
  pub fn replace_metadata<NewM>(
    &self,
    metadata: NewM,
  ) -> InstructionBlock<R, C, NewM> {
    InstructionBlock {
      instructions: self.instructions.clone(),
      metadata,
    }
  }
}
impl<R, C> From<Vec<Instruction<R, C>>> for InstructionBlock<R, C, ()> {
  fn from(instructions: Vec<Instruction<R, C>>) -> Self {
    Self {
      instructions: instructions.into(),
      metadata: (),
    }
  }
}
impl<R, C, OldM> InstructionBlock<R, C, OldM> {
  pub fn with_metadata<NewM>(
    self,
    metadata: NewM,
  ) -> InstructionBlock<R, C, NewM> {
    InstructionBlock {
      instructions: self.instructions,
      metadata,
    }
  }
}
