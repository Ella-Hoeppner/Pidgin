use std::{ops::Index, rc::Rc};

use crate::{
  ConstIndex, GeneralizedCompositeFunction, GeneralizedValue, RegisterIndex,
  SymbolIndex, Value,
};
use GeneralizedValue::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Instruction<Register, OverwriteRegister> {
  DebugPrint(u8),

  // Register manipulation
  Clear(Register),
  Copy(Register, Register),
  Const(Register, ConstIndex),

  // Output
  Print(Register),

  // Function control flow
  Return(Register),
  CopyArgument(Register),
  StealArgument(Register),
  Call(Register, Register, u8),
  Apply(OverwriteRegister, Register),
  CallSelf(Register, u8),
  ApplySelf(OverwriteRegister),
  CallAndReturn(Register, u8),
  ApplyAndReturn(Register, Register),
  CallSelfAndReturn(u8),
  ApplySelfAndReturn(Register),
  CallingFunction(Register),
  Jump(u16),

  // Environment manipulation
  Lookup(Register, SymbolIndex),

  // Control flow
  If(Register),
  Else,
  ElseIf(Register),
  EndIf,

  // Functions manipulation
  Partial(Register, Register, Register),
  Compose(Register, Register, Register),
  FindSome(Register, Register, Register),
  ReduceWithoutInitialValue(OverwriteRegister, Register),
  ReduceWithInitialValue(OverwriteRegister, Register, Register),
  Memoize(Register, Register),

  // Special function constructors
  Constantly(Register, Register),

  // Math
  NumericalEqual(Register, Register, Register),
  IsZero(Register, Register),
  IsNan(Register, Register),
  IsInf(Register, Register),
  IsEven(Register, Register),
  IsOdd(Register, Register),
  IsPos(Register, Register),
  IsNeg(Register, Register),
  Inc(Register, Register),
  Dec(Register, Register),
  Negate(Register, Register),
  Abs(Register, Register),
  Floor(Register, Register),
  Ceil(Register, Register),
  Sqrt(Register, Register),
  Exp(Register, Register),
  Exp2(Register, Register),
  Ln(Register, Register),
  Log2(Register, Register),
  Add(Register, Register, Register),
  Subtract(Register, Register, Register),
  Multiply(Register, Register, Register),
  Divide(Register, Register, Register),
  Pow(Register, Register, Register),
  Mod(Register, Register, Register),
  Quot(Register, Register, Register),
  Min(Register, Register, Register),
  Max(Register, Register, Register),
  GreaterThan(Register, Register, Register),
  GreaterThanOrEqual(Register, Register, Register),
  LessThan(Register, Register, Register),
  LessThanOrEqual(Register, Register, Register),
  Rand(Register),
  UpperBoundedRand(Register, Register),
  LowerUpperBoundedRand(Register, Register, Register),
  RandInt(Register, Register),
  LowerBoundedRandInt(Register, Register, Register),

  // Boolean logic
  Equal(Register, Register, Register),
  NotEqual(Register, Register, Register),
  Not(Register, Register),
  And(Register, Register, Register),
  Or(Register, Register, Register),
  Xor(Register, Register, Register),

  // List + Map + Set manipulation
  IsEmpty(Register, Register),
  First(Register, Register),
  Count(Register, Register),
  Flatten(Register, Register),
  Remove(OverwriteRegister, Register),
  Filter(OverwriteRegister, Register),
  Map(OverwriteRegister, Register),
  DoubleMap(OverwriteRegister, Register, Register),
  MultiCollectionMap(OverwriteRegister, Register),

  // List + Map manipulation
  Set(OverwriteRegister, Register, Register),
  SetIn(OverwriteRegister, Register, Register),
  Get(Register, Register, Register),
  GetIn(Register, Register, Register),
  Update(OverwriteRegister, Register, Register),
  UpdateIn(OverwriteRegister, Register, Register),
  MinKey(Register, Register, Register),
  MaxKey(Register, Register, Register),

  // List + Set manipulation
  Push(OverwriteRegister, Register),
  Sort(OverwriteRegister),
  SortBy(OverwriteRegister, Register),

  // List manipulation (most apply to strings as well)
  EmptyList(Register),
  Last(Register, Register),
  Rest(OverwriteRegister),
  ButLast(OverwriteRegister),
  Nth(Register, Register, Register),
  NthFromLast(Register, Register, Register),
  Cons(OverwriteRegister, Register),
  Concat(OverwriteRegister, Register),
  Take(OverwriteRegister, Register),
  Drop(OverwriteRegister, Register),
  Reverse(OverwriteRegister),
  Distinct(OverwriteRegister),
  Sub(OverwriteRegister, Register, Register),
  Partition(Register, Register, Register),
  SteppedPartition(OverwriteRegister, Register, Register),
  Pad(OverwriteRegister, Register, Register),

  // Map manipulation
  EmptyMap(Register),
  Keys(Register, Register),
  Values(Register, Register),
  Zip(Register, Register, Register),
  Invert(OverwriteRegister),
  Merge(Register, Register, Register),
  MergeWith(OverwriteRegister, Register, Register),
  MapKeys(OverwriteRegister, Register),
  MapValues(OverwriteRegister, Register),
  SelectKeys(OverwriteRegister, Register),

  // Set manipulation
  EmptySet(Register),
  Union(Register, Register, Register),
  Intersection(Register, Register, Register),
  Difference(Register, Register, Register),
  SymmetricDifference(Register, Register, Register),

  // Specialized list constructors
  InfiniteRange(Register),
  UpperBoundedRange(Register, Register),
  LowerUpperBoundedRange(Register, Register, Register),
  InfiniteRepeat(Register, Register),
  BoundedRepeat(Register, Register, Register),
  InfiniteRepeatedly(Register, Register),
  BoundedRepeatedly(Register, Register, Register),
  InfiniteIterate(Register, Register, Register),
  BoundedIterate(OverwriteRegister, Register, Register),

  // Cells
  CreateCell(Register),
  GetCellValue(Register, Register),
  SetCellValue(Register, Register),
  UpdateCell(Register, Register),

  // Coroutines
  CreateCoroutine(OverwriteRegister),
  IsCoroutineAlive(Register, Register),
  Yield(Register),
  YieldAndAccept(Register, Register, u8),

  // Type checkers
  IsNil(Register, Register),
  IsBool(Register, Register),
  IsChar(Register, Register),
  IsNum(Register, Register),
  IsInt(Register, Register),
  IsFloat(Register, Register),
  IsSymbol(Register, Register),
  IsString(Register, Register),
  IsList(Register, Register),
  IsMap(Register, Register),
  IsSet(Register, Register),
  IsCollection(Register, Register),
  IsFn(Register, Register),
  IsError(Register, Register),
  IsCell(Register, Register),
  IsCoroutine(Register, Register),

  // Type conversions
  ToBool(Register, Register),
  ToChar(Register, Register),
  ToNum(Register, Register),
  ToInt(Register, Register),
  ToFloat(Register, Register),
  ToSymbol(Register, Register),
  ToString(Register, Register),
  ToList(Register, Register),
  ToMap(Register, Register),
  ToSet(Register, Register),
  ToError(Register, Register),
}
use Instruction::*;

#[derive(Debug, Clone)]
pub struct GeneralizedBlock<R, O, M> {
  pub instructions: Rc<[Instruction<R, O>]>,
  pub constants: Rc<[GeneralizedValue<R, O, M>]>,
  pub metadata: M,
}
impl<R, O, M> Index<usize> for GeneralizedBlock<R, O, M> {
  type Output = Instruction<R, O>;
  fn index(&self, index: usize) -> &Self::Output {
    &self.instructions[index]
  }
}
impl<R, O> GeneralizedBlock<R, O, ()> {
  pub fn new(
    instructions: Vec<Instruction<R, O>>,
    constants: Vec<GeneralizedValue<R, O, ()>>,
  ) -> Self {
    Self {
      instructions: instructions.into(),
      constants: constants.into(),
      metadata: (),
    }
  }
}
impl<R, O, M> GeneralizedBlock<R, O, M> {
  pub fn len(&self) -> usize {
    self.instructions.len()
  }
}
impl<R: Clone, O: Clone, M> GeneralizedBlock<R, O, M> {
  pub fn replace_metadata<
    NewM: Clone,
    F: Fn(&[Instruction<R, O>], &[GeneralizedValue<R, O, M>], &M) -> NewM,
  >(
    &self,
    replacer: &F,
  ) -> GeneralizedBlock<R, O, NewM> {
    let new_metadata =
      replacer(&*self.instructions, &*self.constants, &self.metadata);
    let translated_constants: Vec<GeneralizedValue<R, O, NewM>> = self
      .constants
      .into_iter()
      .map(|value| match value {
        CompositeFn(f_ref) => {
          CompositeFn(Rc::new(GeneralizedCompositeFunction::new(
            (*f_ref).args.clone(),
            (*f_ref).instructions.replace_metadata(replacer),
          )))
        }
        Nil => Nil,
        Bool(a) => Bool(*a),
        Char(a) => Char(*a),
        Number(a) => Number(*a),
        Symbol(a) => Symbol(*a),
        Str(a) => Str(a.clone()),
        List(a) => List(a.clone()),
        Hashmap(a) => Hashmap(a.clone()),
        Hashset(a) => Hashset(a.clone()),
        CoreFn(a) => CoreFn(a.clone()),
        ExternalFn(a) => ExternalFn(a.clone()),
        ExternalObject(a) => ExternalObject(a.clone()),
        Coroutine(a) => Coroutine(a.clone()),
        Error(a) => Error(a.clone()),
      })
      .collect();
    GeneralizedBlock {
      instructions: self.instructions.clone(),
      constants: (&*translated_constants).into(),
      metadata: new_metadata,
    }
  }
}
