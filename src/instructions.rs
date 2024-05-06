use std::{ops::Index, rc::Rc};

use crate::{
  compiler::ir::VirtualRegister, ConstIndex, RegisterIndex, SymbolIndex, Value,
};

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

  // Processes (coroutines)
  CreateProcess(R),
  IsProcessAlive(R, R),
  Yield(R, R),

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
  IsProcess(R, R),

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

pub type IntermediateInstruction = Instruction<VirtualRegister, Value>;
pub type IntermediateInstructionBlock<M> =
  InstructionBlock<VirtualRegister, Value, M>;
impl IntermediateInstruction {
  pub fn input_and_output_registers(
    &self,
  ) -> (Vec<VirtualRegister>, Vec<VirtualRegister>) {
    match self {
      DebugPrint(_) => (vec![], vec![]),
      Clear(_) => (vec![], vec![]),
      Copy(_, _) => todo!(),
      Const(_, _) => todo!(),
      Print(_) => todo!(),
      Return(_) => todo!(),
      CopyArgument(_) => todo!(),
      StealArgument(_) => todo!(),
      Call(_, _, _) => todo!(),
      Apply(_, _) => todo!(),
      CallSelf(_, _) => todo!(),
      ApplySelf(_) => todo!(),
      CallAndReturn(_, _) => todo!(),
      ApplyAndReturn(_, _) => todo!(),
      CallSelfAndReturn(_) => todo!(),
      ApplySelfAndReturn(_) => todo!(),
      CallingFunction(_) => todo!(),
      Jump(_) => todo!(),
      Lookup(_, _) => todo!(),
      If(_) => todo!(),
      Else => todo!(),
      ElseIf(_) => todo!(),
      EndIf => todo!(),
      Partial(_, _, _) => todo!(),
      Compose(_, _, _) => todo!(),
      FindSome(_, _, _) => todo!(),
      ReduceWithoutInitialValue(_, _, _) => todo!(),
      ReduceWithInitialValue(_, _, _) => todo!(),
      Memoize(_, _) => todo!(),
      Constantly(_, _) => todo!(),
      NumericalEqual(_, _, _) => todo!(),
      IsZero(_, _) => todo!(),
      IsNan(_, _) => todo!(),
      IsInf(_, _) => todo!(),
      IsEven(_, _) => todo!(),
      IsOdd(_, _) => todo!(),
      IsPos(_, _) => todo!(),
      IsNeg(_, _) => todo!(),
      Inc(_, _) => todo!(),
      Dec(_, _) => todo!(),
      Negate(_, _) => todo!(),
      Abs(_, _) => todo!(),
      Floor(_, _) => todo!(),
      Ceil(_, _) => todo!(),
      Sqrt(_, _) => todo!(),
      Exp(_, _) => todo!(),
      Exp2(_, _) => todo!(),
      Ln(_, _) => todo!(),
      Log2(_, _) => todo!(),
      Add(_, _, _) => todo!(),
      Subtract(_, _, _) => todo!(),
      Multiply(_, _, _) => todo!(),
      Divide(_, _, _) => todo!(),
      Pow(_, _, _) => todo!(),
      Mod(_, _, _) => todo!(),
      Quot(_, _, _) => todo!(),
      Min(_, _, _) => todo!(),
      Max(_, _, _) => todo!(),
      GreaterThan(_, _, _) => todo!(),
      GreaterThanOrEqual(_, _, _) => todo!(),
      LessThan(_, _, _) => todo!(),
      LessThanOrEqual(_, _, _) => todo!(),
      Rand(_) => todo!(),
      UpperBoundedRand(_, _) => todo!(),
      LowerUpperBoundedRand(_, _, _) => todo!(),
      RandInt(_, _) => todo!(),
      LowerBoundedRandInt(_, _, _) => todo!(),
      Equal(_, _, _) => todo!(),
      NotEqual(_, _, _) => todo!(),
      Not(_, _) => todo!(),
      And(_, _, _) => todo!(),
      Or(_, _, _) => todo!(),
      Xor(_, _, _) => todo!(),
      IsEmpty(_, _) => todo!(),
      First(_, _) => todo!(),
      Count(_, _) => todo!(),
      Flatten(_, _) => todo!(),
      Remove(_, _) => todo!(),
      Filter(_, _) => todo!(),
      Map(_, _) => todo!(),
      DoubleMap(_, _, _) => todo!(),
      MultiCollectionMap(_, _) => todo!(),
      Set(_, _, _) => todo!(),
      SetIn(_, _, _) => todo!(),
      Get(_, _, _) => todo!(),
      GetIn(_, _, _) => todo!(),
      Update(_, _, _) => todo!(),
      UpdateIn(_, _, _) => todo!(),
      MinKey(_, _, _) => todo!(),
      MaxKey(_, _, _) => todo!(),
      Push(_, _) => todo!(),
      Sort(_) => todo!(),
      SortBy(_, _) => todo!(),
      EmptyList(_) => todo!(),
      Last(_, _) => todo!(),
      Rest(_) => todo!(),
      ButLast(_) => todo!(),
      Nth(_, _, _) => todo!(),
      NthFromLast(_, _, _) => todo!(),
      Cons(_, _) => todo!(),
      Concat(_, _) => todo!(),
      Take(_, _) => todo!(),
      Drop(_, _) => todo!(),
      Reverse(_) => todo!(),
      Distinct(_) => todo!(),
      Sub(_, _, _) => todo!(),
      Partition(_, _, _) => todo!(),
      SteppedPartition(_, _, _) => todo!(),
      Pad(_, _, _) => todo!(),
      EmptyMap(_) => todo!(),
      Keys(_, _) => todo!(),
      Values(_, _) => todo!(),
      Zip(_, _, _) => todo!(),
      Invert(_, _) => todo!(),
      Merge(_, _, _) => todo!(),
      MergeWith(_, _, _) => todo!(),
      MapKeys(_, _, _) => todo!(),
      MapValues(_, _, _) => todo!(),
      SelectKeys(_, _) => todo!(),
      EmptySet(_) => todo!(),
      Union(_, _, _) => todo!(),
      Intersection(_, _, _) => todo!(),
      Difference(_, _, _) => todo!(),
      SymmetricDifference(_, _, _) => todo!(),
      InfiniteRange(_) => todo!(),
      UpperBoundedRange(_, _) => todo!(),
      LowerUpperBoundedRange(_, _, _) => todo!(),
      InfiniteRepeat(_, _) => todo!(),
      BoundedRepeat(_, _, _) => todo!(),
      InfiniteRepeatedly(_, _) => todo!(),
      BoundedRepeatedly(_, _, _) => todo!(),
      InfiniteIterate(_, _, _) => todo!(),
      BoundedIterate(_, _, _) => todo!(),
      CreateCell(_) => todo!(),
      GetCellValue(_, _) => todo!(),
      SetCellValue(_, _) => todo!(),
      UpdateCell(_, _) => todo!(),
      CreateProcess(_) => todo!(),
      IsProcessAlive(_, _) => todo!(),
      Yield(_, _) => todo!(),
      IsNil(_, _) => todo!(),
      IsBool(_, _) => todo!(),
      IsChar(_, _) => todo!(),
      IsNum(_, _) => todo!(),
      IsInt(_, _) => todo!(),
      IsFloat(_, _) => todo!(),
      IsSymbol(_, _) => todo!(),
      IsString(_, _) => todo!(),
      IsList(_, _) => todo!(),
      IsMap(_, _) => todo!(),
      IsSet(_, _) => todo!(),
      IsCollection(_, _) => todo!(),
      IsFn(_, _) => todo!(),
      IsError(_, _) => todo!(),
      IsCell(_, _) => todo!(),
      IsProcess(_, _) => todo!(),
      ToBool(_, _) => todo!(),
      ToChar(_, _) => todo!(),
      ToNum(_, _) => todo!(),
      ToInt(_, _) => todo!(),
      ToFloat(_, _) => todo!(),
      ToSymbol(_, _) => todo!(),
      ToString(_, _) => todo!(),
      ToList(_, _) => todo!(),
      ToMap(_, _) => todo!(),
      ToSet(_, _) => todo!(),
      ToError(_, _) => todo!(),
    }
  }
}

pub type RuntimeInstruction = Instruction<RegisterIndex, ConstIndex>;
pub type RuntimeInstructionBlock =
  InstructionBlock<RegisterIndex, ConstIndex, ()>;

impl From<Vec<Instruction<RegisterIndex, ConstIndex>>>
  for RuntimeInstructionBlock
{
  fn from(instructions: Vec<RuntimeInstruction>) -> Self {
    Self {
      instructions: instructions.into(),
      metadata: (),
    }
  }
}
