use std::{ops::Index, rc::Rc};

use crate::{
  ConstIndex, GeneralizedCompositeFunction, GeneralizedValue, Register,
  SymbolIndex, Value,
};
use GeneralizedValue::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Instruction<R, O> {
  DebugPrint(u8),

  // R manipulation
  Clear(R),
  Copy(R, R),
  Const(R, ConstIndex),

  // Output
  Print(R),

  // Function control flow
  Return(R),
  CopyArgument(R),
  StealArgument(R),
  Call(R, R, u8),
  Apply(O, R),
  CallSelf(R, u8),
  ApplySelf(O),
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
  ReduceWithoutInitialValue(O, R),
  ReduceWithInitialValue(O, R, R),
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
  Remove(O, R),
  Filter(O, R),
  Map(O, R),
  DoubleMap(O, R, R),
  MultiCollectionMap(O, R),

  // List + Map manipulation
  Set(O, R, R),
  SetIn(O, R, R),
  Get(R, R, R),
  GetIn(R, R, R),
  Update(O, R, R),
  UpdateIn(O, R, R),
  MinKey(R, R, R),
  MaxKey(R, R, R),

  // List + Set manipulation
  Push(O, R),
  Sort(O),
  SortBy(O, R),

  // List manipulation (most apply to strings as well)
  EmptyList(R),
  Last(R, R),
  Rest(O),
  ButLast(O),
  Nth(R, R, R),
  NthFromLast(R, R, R),
  Cons(O, R),
  Concat(O, R),
  Take(O, R),
  Drop(O, R),
  Reverse(O),
  Distinct(O),
  Sub(O, R, R),
  Partition(R, R, R),
  SteppedPartition(O, R, R),
  Pad(O, R, R),

  // Map manipulation
  EmptyMap(R),
  Keys(R, R),
  Values(R, R),
  Zip(R, R, R),
  Invert(O),
  Merge(R, R, R),
  MergeWith(O, R, R),
  MapKeys(O, R),
  MapValues(O, R),
  SelectKeys(O, R),

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
  BoundedIterate(O, R, R),

  // Cells
  CreateCell(R),
  GetCellValue(R, R),
  SetCellValue(R, R),
  UpdateCell(R, R),

  // Coroutines
  CreateCoroutine(O),
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
  pub fn translate_instructions<
    NewR: Clone,
    NewO: Clone,
    NewM: Clone,
    E,
    F: Fn(
      &[Instruction<R, O>],
      &M,
    ) -> Result<(Vec<Instruction<NewR, NewO>>, NewM), E>,
  >(
    &self,
    replacer: &F,
  ) -> Result<GeneralizedBlock<NewR, NewO, NewM>, E> {
    let (new_instructions, new_metadata) =
      replacer(&*self.instructions, &self.metadata)?;
    let mut translated_constants = vec![];
    for value in self.constants.into_iter() {
      translated_constants.push(match value {
        CompositeFn(f_ref) => {
          CompositeFn(Rc::new(GeneralizedCompositeFunction::new(
            (*f_ref).args.clone(),
            (*f_ref).instructions.translate_instructions(replacer)?,
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
    }
    Ok(GeneralizedBlock {
      instructions: (&*new_instructions).into(),
      constants: (&*translated_constants).into(),
      metadata: new_metadata,
    })
  }
  pub fn replace_metadata<
    NewM: Clone,
    E,
    F: Fn(
      &[Instruction<R, O>],
      &[GeneralizedValue<R, O, M>],
      &M,
    ) -> Result<NewM, E>,
  >(
    &self,
    replacer: &F,
  ) -> Result<GeneralizedBlock<R, O, NewM>, E> {
    let new_metadata =
      replacer(&*self.instructions, &*self.constants, &self.metadata)?;
    let mut translated_constants = vec![];
    for value in self.constants.into_iter() {
      translated_constants.push(match value {
        CompositeFn(f_ref) => {
          CompositeFn(Rc::new(GeneralizedCompositeFunction::new(
            (*f_ref).args.clone(),
            (*f_ref).instructions.replace_metadata(replacer)?,
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
    }
    Ok(GeneralizedBlock {
      instructions: self.instructions.clone(),
      constants: (&*translated_constants).into(),
      metadata: new_metadata,
    })
  }
}
