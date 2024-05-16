use std::{ops::Index, rc::Rc};

use crate::{ConstIndex, SymbolIndex};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Instruction<I, O, R> {
  DebugPrint(u8),

  // Register manipulation
  Clear(O),
  Copy(O, I),
  Const(O, ConstIndex),

  // Output
  Print(I),

  // Function control flow
  Return(I),
  CopyArgument(I),
  StealArgument(I),
  Call(O, I, u8),
  Apply(R, I),
  CallSelf(O, u8),
  ApplySelf(R),
  CallAndReturn(I, u8),
  ApplyAndReturn(I, I),
  CallSelfAndReturn(u8),
  ApplySelfAndReturn(I),
  CallingFunction(O),
  Jump(u16),

  // Environment manipulation
  Lookup(O, SymbolIndex),

  // Control flow
  If(I),
  Else,
  ElseIf(I),
  EndIf,

  // Functions manipulation
  Partial(O, I, I),
  Compose(O, I, I),
  FindSome(O, I, I),
  ReduceWithoutInitialValue(R, I),
  ReduceWithInitialValue(R, I, I),
  Memoize(O, I),

  // Special function constructors
  Constantly(O, I),

  // Math
  NumericalEqual(O, I, I),
  IsZero(O, I),
  IsNan(O, I),
  IsInf(O, I),
  IsEven(O, I),
  IsOdd(O, I),
  IsPos(O, I),
  IsNeg(O, I),
  Inc(O, I),
  Dec(O, I),
  Negate(O, I),
  Abs(O, I),
  Floor(O, I),
  Ceil(O, I),
  Sqrt(O, I),
  Exp(O, I),
  Exp2(O, I),
  Ln(O, I),
  Log2(O, I),
  Add(O, I, I),
  Subtract(O, I, I),
  Multiply(O, I, I),
  Divide(O, I, I),
  Pow(O, I, I),
  Mod(O, I, I),
  Quot(O, I, I),
  Min(O, I, I),
  Max(O, I, I),
  GreaterThan(O, I, I),
  GreaterThanOrEqual(O, I, I),
  LessThan(O, I, I),
  LessThanOrEqual(O, I, I),
  Rand(O),
  UpperBoundedRand(O, I),
  LowerUpperBoundedRand(O, I, I),
  RandInt(O, I),
  LowerBoundedRandInt(O, I, I),

  // Boolean logic
  Equal(O, I, I),
  NotEqual(O, I, I),
  Not(O, I),
  And(O, I, I),
  Or(O, I, I),
  Xor(O, I, I),

  // List + Map + Set manipulation
  IsEmpty(O, I),
  First(O, I),
  Count(O, I),
  Flatten(O, I),
  Remove(R, I),
  Filter(R, I),
  Map(R, I),
  DoubleMap(R, I, I),
  MultiCollectionMap(R, I),

  // List + Map manipulation
  Set(R, I, I),
  SetIn(R, I, I),
  Get(O, I, I),
  GetIn(O, I, I),
  Update(R, I, I),
  UpdateIn(R, I, I),
  MinKey(O, I, I),
  MaxKey(O, I, I),

  // List + Set manipulation
  Push(R, I),
  Sort(R),
  SortBy(R, I),

  // List manipulation (most apply to strings as well)
  EmptyList(O),
  Last(O, I),
  Rest(R),
  ButLast(R),
  Nth(O, I, I),
  NthFromLast(O, I, I),
  Cons(R, I),
  Concat(R, I),
  Take(R, I),
  Drop(R, I),
  Reverse(R),
  Distinct(R),
  Sub(R, I, I),
  Partition(O, I, I),
  SteppedPartition(R, I, I),
  Pad(R, I, I),

  // Map manipulation
  EmptyMap(O),
  Keys(O, I),
  Values(O, I),
  Zip(O, I, I),
  Invert(R),
  Merge(O, I, I),
  MergeWith(R, I, I),
  MapKeys(R, I),
  MapValues(R, I),
  SelectKeys(R, I),

  // Set manipulation
  EmptySet(O),
  Union(O, I, I),
  Intersection(O, I, I),
  Difference(O, I, I),
  SymmetricDifference(O, I, I),

  // Specialized list constructors
  InfiniteRange(O),
  UpperBoundedRange(O, I),
  LowerUpperBoundedRange(O, I, I),
  InfiniteRepeat(O, I),
  BoundedRepeat(O, I, I),
  InfiniteRepeatedly(O, I),
  BoundedRepeatedly(O, I, I),
  InfiniteIterate(O, I, I),
  BoundedIterate(R, I, I),

  // Cells
  CreateCell(O),
  GetCellValue(O, I),
  SetCellValue(O, I),
  UpdateCell(O, I),

  // Coroutines
  CreateCoroutine(R),
  IsCoroutineAlive(O, I),
  Yield(O),
  YieldAndAccept(O, u8, I),

  // Type checkers
  IsNil(O, I),
  IsBool(O, I),
  IsChar(O, I),
  IsNum(O, I),
  IsInt(O, I),
  IsFloat(O, I),
  IsSymbol(O, I),
  IsString(O, I),
  IsList(O, I),
  IsMap(O, I),
  IsSet(O, I),
  IsCollection(O, I),
  IsFn(O, I),
  IsError(O, I),
  IsCell(O, I),
  IsCoroutine(O, I),

  // Type conversions
  ToBool(O, I),
  ToChar(O, I),
  ToNum(O, I),
  ToInt(O, I),
  ToFloat(O, I),
  ToSymbol(O, I),
  ToString(O, I),
  ToList(O, I),
  ToMap(O, I),
  ToSet(O, I),
  ToError(O, I),
}
use Instruction::*;

pub struct RegisterUsages<I, O, R> {
  pub inputs: Vec<I>,
  pub outputs: Vec<O>,
  pub replacements: Vec<R>,
}
impl<I: Clone, O: Clone, R: Clone> Instruction<I, O, R> {
  pub fn usages(&self) -> RegisterUsages<I, O, R> {
    let (inputs, outputs, replacements) = match self {
      DebugPrint(_) => (vec![], vec![], vec![]),
      Clear(to) => (vec![], vec![to], vec![]),
      Copy(to, from) => (vec![from], vec![to], vec![]),
      Const(to, _) => (vec![], vec![to], vec![]),
      Print(from) => (vec![from], vec![], vec![]),
      Return(from) => (vec![from], vec![], vec![]),
      CopyArgument(from) => (vec![from], vec![], vec![]),
      StealArgument(from) => (vec![from], vec![], vec![]),
      Call(to, from, _) => (vec![from], vec![to], vec![]),
      Apply(from_and_to, f) => (vec![f], vec![], vec![from_and_to]),
      CallSelf(to, _) => (vec![], vec![to], vec![]),
      ApplySelf(from_and_to) => (vec![], vec![], vec![from_and_to]),
      CallAndReturn(f, _) => (vec![f], vec![], vec![]),
      ApplyAndReturn(from, _) => (vec![from], vec![], vec![]),
      CallSelfAndReturn(_) => (vec![], vec![], vec![]),
      ApplySelfAndReturn(from) => (vec![from], vec![], vec![]),
      CallingFunction(to) => (vec![], vec![to], vec![]),
      Jump(_) => (vec![], vec![], vec![]),
      Lookup(to, _) => (vec![], vec![to], vec![]),
      If(from) => (vec![from], vec![], vec![]),
      Else => (vec![], vec![], vec![]),
      ElseIf(from) => (vec![from], vec![], vec![]),
      EndIf => (vec![], vec![], vec![]),
      Partial(to, f, arg) => (vec![f, arg], vec![to], vec![]),
      Compose(to, f_1, f_2) => (vec![f_1, f_2], vec![to], vec![]),
      FindSome(to, f, collection) => (vec![f, collection], vec![to], vec![]),
      ReduceWithoutInitialValue(from_and_to, f) => {
        (vec![f], vec![], vec![from_and_to])
      }
      ReduceWithInitialValue(from_and_to, f, initial) => {
        (vec![f, initial], vec![], vec![from_and_to])
      }
      Memoize(to, from) => (vec![from], vec![to], vec![]),
      Constantly(to, from) => (vec![from], vec![to], vec![]),
      NumericalEqual(to, a, b) => (vec![a, b], vec![to], vec![]),
      IsZero(to, from) => (vec![from], vec![to], vec![]),
      IsNan(to, from) => (vec![from], vec![to], vec![]),
      IsInf(to, from) => (vec![from], vec![to], vec![]),
      IsEven(to, from) => (vec![from], vec![to], vec![]),
      IsOdd(to, from) => (vec![from], vec![to], vec![]),
      IsPos(to, from) => (vec![from], vec![to], vec![]),
      IsNeg(to, from) => (vec![from], vec![to], vec![]),
      Inc(to, from) => (vec![from], vec![to], vec![]),
      Dec(to, from) => (vec![from], vec![to], vec![]),
      Negate(to, from) => (vec![from], vec![to], vec![]),
      Abs(to, from) => (vec![from], vec![to], vec![]),
      Floor(to, from) => (vec![from], vec![to], vec![]),
      Ceil(to, from) => (vec![from], vec![to], vec![]),
      Sqrt(to, from) => (vec![from], vec![to], vec![]),
      Exp(to, from) => (vec![from], vec![to], vec![]),
      Exp2(to, from) => (vec![from], vec![to], vec![]),
      Ln(to, from) => (vec![from], vec![to], vec![]),
      Log2(to, from) => (vec![from], vec![to], vec![]),
      Add(to, a, b) => (vec![a, b], vec![to], vec![]),
      Subtract(to, a, b) => (vec![a, b], vec![to], vec![]),
      Multiply(to, a, b) => (vec![a, b], vec![to], vec![]),
      Divide(to, a, b) => (vec![a, b], vec![to], vec![]),
      Pow(to, a, b) => (vec![a, b], vec![to], vec![]),
      Mod(to, a, b) => (vec![a, b], vec![to], vec![]),
      Quot(to, a, b) => (vec![a, b], vec![to], vec![]),
      Min(to, a, b) => (vec![a, b], vec![to], vec![]),
      Max(to, a, b) => (vec![a, b], vec![to], vec![]),
      GreaterThan(to, a, b) => (vec![a, b], vec![to], vec![]),
      GreaterThanOrEqual(to, a, b) => (vec![a, b], vec![to], vec![]),
      LessThan(to, a, b) => (vec![a, b], vec![to], vec![]),
      LessThanOrEqual(to, a, b) => (vec![a, b], vec![to], vec![]),
      Rand(to) => (vec![], vec![to], vec![]),
      UpperBoundedRand(to, from) => (vec![from], vec![to], vec![]),
      LowerUpperBoundedRand(to, a, b) => (vec![a, b], vec![to], vec![]),
      RandInt(to, from) => (vec![from], vec![to], vec![]),
      LowerBoundedRandInt(to, a, b) => (vec![a, b], vec![to], vec![]),
      Equal(to, a, b) => (vec![a, b], vec![to], vec![]),
      NotEqual(to, a, b) => (vec![a, b], vec![to], vec![]),
      Not(to, from) => (vec![from], vec![to], vec![]),
      And(to, a, b) => (vec![a, b], vec![to], vec![]),
      Or(to, a, b) => (vec![a, b], vec![to], vec![]),
      Xor(to, a, b) => (vec![a, b], vec![to], vec![]),
      IsEmpty(to, from) => (vec![from], vec![to], vec![]),
      First(to, from) => (vec![from], vec![to], vec![]),
      Count(to, from) => (vec![from], vec![to], vec![]),
      Flatten(to, from) => (vec![from], vec![to], vec![]),
      Remove(from_and_to, x) => (vec![x], vec![], vec![from_and_to]),
      Filter(from_and_to, f) => (vec![f], vec![], vec![from_and_to]),
      Map(from_and_to, f) => (vec![f], vec![], vec![from_and_to]),
      DoubleMap(from_and_to, a, b) => (vec![a, b], vec![], vec![from_and_to]),
      MultiCollectionMap(from_and_to, f) => {
        (vec![f], vec![], vec![from_and_to])
      }
      Set(from_and_to, a, b) => (vec![a, b], vec![], vec![from_and_to]),
      SetIn(from_and_to, a, b) => (vec![a, b], vec![], vec![from_and_to]),
      Get(to, a, b) => (vec![a, b], vec![to], vec![]),
      GetIn(to, a, b) => (vec![a, b], vec![to], vec![]),
      Update(from_and_to, a, b) => (vec![a, b], vec![], vec![from_and_to]),
      UpdateIn(from_and_to, a, b) => (vec![a, b], vec![], vec![from_and_to]),
      MinKey(to, a, b) => (vec![a, b], vec![to], vec![]),
      MaxKey(to, a, b) => (vec![a, b], vec![to], vec![]),
      Push(from_and_to, f) => (vec![f], vec![], vec![from_and_to]),
      Sort(from_and_to) => (vec![], vec![], vec![from_and_to]),
      SortBy(from_and_to, f) => (vec![f], vec![], vec![from_and_to]),
      EmptyList(to) => (vec![], vec![to], vec![]),
      Last(to, from) => (vec![from], vec![to], vec![]),
      Rest(from_and_to) => (vec![], vec![], vec![from_and_to]),
      ButLast(from_and_to) => (vec![], vec![], vec![from_and_to]),
      Nth(to, a, b) => (vec![a, b], vec![to], vec![]),
      NthFromLast(to, a, b) => (vec![a, b], vec![to], vec![]),
      Cons(from_and_to, x) => (vec![x], vec![], vec![from_and_to]),
      Concat(from_and_to, x) => (vec![x], vec![], vec![from_and_to]),
      Take(from_and_to, x) => (vec![x], vec![], vec![from_and_to]),
      Drop(from_and_to, x) => (vec![x], vec![], vec![from_and_to]),
      Reverse(from_and_to) => (vec![], vec![], vec![from_and_to]),
      Distinct(from_and_to) => (vec![], vec![], vec![from_and_to]),
      Sub(from_and_to, a, b) => (vec![a, b], vec![], vec![from_and_to]),
      Partition(to, a, b) => (vec![a, b], vec![to], vec![]),
      SteppedPartition(from_and_to, a, b) => {
        (vec![a, b], vec![], vec![from_and_to])
      }
      Pad(from_and_to, a, b) => (vec![a, b], vec![], vec![from_and_to]),
      EmptyMap(to) => (vec![], vec![to], vec![]),
      Keys(to, from) => (vec![from], vec![to], vec![]),
      Values(to, from) => (vec![from], vec![to], vec![]),
      Zip(to, a, b) => (vec![a, b], vec![to], vec![]),
      Invert(from_and_to) => (vec![], vec![], vec![from_and_to]),
      Merge(to, a, b) => (vec![a, b], vec![to], vec![]),
      MergeWith(from_and_to, a, b) => (vec![a, b], vec![], vec![from_and_to]),
      MapKeys(from_and_to, f) => (vec![f], vec![], vec![from_and_to]),
      MapValues(from_and_to, f) => (vec![f], vec![], vec![from_and_to]),
      SelectKeys(from_and_to, x) => (vec![x], vec![], vec![from_and_to]),
      EmptySet(to) => (vec![], vec![to], vec![]),
      Union(to, a, b) => (vec![a, b], vec![to], vec![]),
      Intersection(to, a, b) => (vec![a, b], vec![to], vec![]),
      Difference(to, a, b) => (vec![a, b], vec![to], vec![]),
      SymmetricDifference(to, a, b) => (vec![a, b], vec![to], vec![]),
      InfiniteRange(to) => (vec![], vec![to], vec![]),
      UpperBoundedRange(to, from) => (vec![from], vec![to], vec![]),
      LowerUpperBoundedRange(to, a, b) => (vec![a, b], vec![to], vec![]),
      InfiniteRepeat(to, from) => (vec![from], vec![to], vec![]),
      BoundedRepeat(to, a, b) => (vec![a, b], vec![to], vec![]),
      InfiniteRepeatedly(to, from) => (vec![from], vec![to], vec![]),
      BoundedRepeatedly(to, a, b) => (vec![a, b], vec![to], vec![]),
      InfiniteIterate(to, a, b) => (vec![a, b], vec![to], vec![]),
      BoundedIterate(from_and_to, a, b) => {
        (vec![a, b], vec![], vec![from_and_to])
      }
      CreateCell(to) => (vec![], vec![to], vec![]),
      GetCellValue(to, from) => (vec![from], vec![to], vec![]),
      SetCellValue(to, from) => (vec![from], vec![to], vec![]),
      UpdateCell(to, from) => (vec![from], vec![to], vec![]),
      CreateCoroutine(from_and_to) => (vec![], vec![], vec![from_and_to]),
      IsCoroutineAlive(to, from) => (vec![from], vec![to], vec![]),
      Yield(from) => (vec![], vec![from], vec![]),
      YieldAndAccept(from, arg_count, first_register) => todo!(),
      IsNil(to, from) => (vec![from], vec![to], vec![]),
      IsBool(to, from) => (vec![from], vec![to], vec![]),
      IsChar(to, from) => (vec![from], vec![to], vec![]),
      IsNum(to, from) => (vec![from], vec![to], vec![]),
      IsInt(to, from) => (vec![from], vec![to], vec![]),
      IsFloat(to, from) => (vec![from], vec![to], vec![]),
      IsSymbol(to, from) => (vec![from], vec![to], vec![]),
      IsString(to, from) => (vec![from], vec![to], vec![]),
      IsList(to, from) => (vec![from], vec![to], vec![]),
      IsMap(to, from) => (vec![from], vec![to], vec![]),
      IsSet(to, from) => (vec![from], vec![to], vec![]),
      IsCollection(to, from) => (vec![from], vec![to], vec![]),
      IsFn(to, from) => (vec![from], vec![to], vec![]),
      IsError(to, from) => (vec![from], vec![to], vec![]),
      IsCell(to, from) => (vec![from], vec![to], vec![]),
      IsCoroutine(to, from) => (vec![from], vec![to], vec![]),
      ToBool(to, from) => (vec![from], vec![to], vec![]),
      ToChar(to, from) => (vec![from], vec![to], vec![]),
      ToNum(to, from) => (vec![from], vec![to], vec![]),
      ToInt(to, from) => (vec![from], vec![to], vec![]),
      ToFloat(to, from) => (vec![from], vec![to], vec![]),
      ToSymbol(to, from) => (vec![from], vec![to], vec![]),
      ToString(to, from) => (vec![from], vec![to], vec![]),
      ToList(to, from) => (vec![from], vec![to], vec![]),
      ToMap(to, from) => (vec![from], vec![to], vec![]),
      ToSet(to, from) => (vec![from], vec![to], vec![]),
      ToError(to, from) => (vec![from], vec![to], vec![]),
    };
    RegisterUsages {
      inputs: inputs.into_iter().cloned().collect(),
      outputs: outputs.into_iter().cloned().collect(),
      replacements: replacements.into_iter().cloned().collect(),
    }
  }
}
impl<I, O, R> Instruction<I, O, R> {
  pub fn translate<
    NewI,
    NewO,
    NewR,
    InputTranslator: Fn(I) -> NewI,
    OututTranslator: Fn(O) -> NewO,
    ReplacementTranslator: Fn(R) -> NewR,
  >(
    self,
    input_translator: InputTranslator,
    output_translator: OututTranslator,
    replacement_translator: ReplacementTranslator,
  ) -> Instruction<NewI, NewO, NewR> {
    match self {
      DebugPrint(a) => DebugPrint(a),
      Clear(a) => Clear(output_translator(a)),
      Copy(a, b) => Copy(output_translator(a), input_translator(b)),
      Const(a, b) => Const(output_translator(a), b),
      Print(a) => Print(input_translator(a)),
      Return(a) => Return(input_translator(a)),
      CopyArgument(a) => CopyArgument(input_translator(a)),
      StealArgument(a) => StealArgument(input_translator(a)),
      Call(a, b, c) => Call(output_translator(a), input_translator(b), c),
      Apply(a, b) => Apply(replacement_translator(a), input_translator(b)),
      CallSelf(a, b) => CallSelf(output_translator(a), b),
      ApplySelf(a) => ApplySelf(replacement_translator(a)),
      CallAndReturn(a, b) => CallAndReturn(input_translator(a), b),
      ApplyAndReturn(a, b) => {
        ApplyAndReturn(input_translator(a), input_translator(b))
      }
      CallSelfAndReturn(a) => CallSelfAndReturn(a),
      ApplySelfAndReturn(a) => ApplySelfAndReturn(input_translator(a)),
      CallingFunction(a) => CallingFunction(output_translator(a)),
      Jump(a) => Jump(a),
      Lookup(a, b) => Lookup(output_translator(a), b),
      If(a) => If(input_translator(a)),
      Else => Else,
      ElseIf(a) => ElseIf(input_translator(a)),
      EndIf => EndIf,
      Partial(a, b, c) => Partial(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      Compose(a, b, c) => Compose(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      FindSome(a, b, c) => FindSome(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      ReduceWithoutInitialValue(a, b) => ReduceWithoutInitialValue(
        replacement_translator(a),
        input_translator(b),
      ),
      ReduceWithInitialValue(a, b, c) => ReduceWithInitialValue(
        replacement_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      Memoize(a, b) => Memoize(output_translator(a), input_translator(b)),
      Constantly(a, b) => Constantly(output_translator(a), input_translator(b)),
      NumericalEqual(a, b, c) => NumericalEqual(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      IsZero(a, b) => IsZero(output_translator(a), input_translator(b)),
      IsNan(a, b) => IsNan(output_translator(a), input_translator(b)),
      IsInf(a, b) => IsInf(output_translator(a), input_translator(b)),
      IsEven(a, b) => IsEven(output_translator(a), input_translator(b)),
      IsOdd(a, b) => IsOdd(output_translator(a), input_translator(b)),
      IsPos(a, b) => IsPos(output_translator(a), input_translator(b)),
      IsNeg(a, b) => IsNeg(output_translator(a), input_translator(b)),
      Inc(a, b) => Inc(output_translator(a), input_translator(b)),
      Dec(a, b) => Dec(output_translator(a), input_translator(b)),
      Negate(a, b) => Negate(output_translator(a), input_translator(b)),
      Abs(a, b) => Abs(output_translator(a), input_translator(b)),
      Floor(a, b) => Floor(output_translator(a), input_translator(b)),
      Ceil(a, b) => Ceil(output_translator(a), input_translator(b)),
      Sqrt(a, b) => Sqrt(output_translator(a), input_translator(b)),
      Exp(a, b) => Exp(output_translator(a), input_translator(b)),
      Exp2(a, b) => Exp2(output_translator(a), input_translator(b)),
      Ln(a, b) => Ln(output_translator(a), input_translator(b)),
      Log2(a, b) => Log2(output_translator(a), input_translator(b)),
      Add(a, b, c) => Add(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      Subtract(a, b, c) => Subtract(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      Multiply(a, b, c) => Multiply(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      Divide(a, b, c) => Divide(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      Pow(a, b, c) => Pow(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      Mod(a, b, c) => Mod(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      Quot(a, b, c) => Quot(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      Min(a, b, c) => Min(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      Max(a, b, c) => Max(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      GreaterThan(a, b, c) => GreaterThan(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      GreaterThanOrEqual(a, b, c) => GreaterThanOrEqual(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      LessThan(a, b, c) => LessThan(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      LessThanOrEqual(a, b, c) => LessThanOrEqual(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      Rand(a) => Rand(output_translator(a)),
      UpperBoundedRand(a, b) => {
        UpperBoundedRand(output_translator(a), input_translator(b))
      }
      LowerUpperBoundedRand(a, b, c) => LowerUpperBoundedRand(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      RandInt(a, b) => RandInt(output_translator(a), input_translator(b)),
      LowerBoundedRandInt(a, b, c) => LowerBoundedRandInt(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      Equal(a, b, c) => Equal(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      NotEqual(a, b, c) => NotEqual(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      Not(a, b) => Not(output_translator(a), input_translator(b)),
      And(a, b, c) => And(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      Or(a, b, c) => Or(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      Xor(a, b, c) => Xor(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      IsEmpty(a, b) => IsEmpty(output_translator(a), input_translator(b)),
      First(a, b) => First(output_translator(a), input_translator(b)),
      Count(a, b) => Count(output_translator(a), input_translator(b)),
      Flatten(a, b) => Flatten(output_translator(a), input_translator(b)),
      Remove(a, b) => Remove(replacement_translator(a), input_translator(b)),
      Filter(a, b) => Filter(replacement_translator(a), input_translator(b)),
      Map(a, b) => Map(replacement_translator(a), input_translator(b)),
      DoubleMap(a, b, c) => DoubleMap(
        replacement_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      MultiCollectionMap(a, b) => {
        MultiCollectionMap(replacement_translator(a), input_translator(b))
      }
      Set(a, b, c) => Set(
        replacement_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      SetIn(a, b, c) => SetIn(
        replacement_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      Get(a, b, c) => Get(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      GetIn(a, b, c) => GetIn(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      Update(a, b, c) => Update(
        replacement_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      UpdateIn(a, b, c) => UpdateIn(
        replacement_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      MinKey(a, b, c) => MinKey(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      MaxKey(a, b, c) => MaxKey(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      Push(a, b) => Push(replacement_translator(a), input_translator(b)),
      Sort(a) => Sort(replacement_translator(a)),
      SortBy(a, b) => SortBy(replacement_translator(a), input_translator(b)),
      EmptyList(a) => EmptyList(output_translator(a)),
      Last(a, b) => Last(output_translator(a), input_translator(b)),
      Rest(a) => Rest(replacement_translator(a)),
      ButLast(a) => ButLast(replacement_translator(a)),
      Nth(a, b, c) => Nth(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      NthFromLast(a, b, c) => NthFromLast(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      Cons(a, b) => Cons(replacement_translator(a), input_translator(b)),
      Concat(a, b) => Concat(replacement_translator(a), input_translator(b)),
      Take(a, b) => Take(replacement_translator(a), input_translator(b)),
      Drop(a, b) => Drop(replacement_translator(a), input_translator(b)),
      Reverse(a) => Reverse(replacement_translator(a)),
      Distinct(a) => Distinct(replacement_translator(a)),
      Sub(a, b, c) => Sub(
        replacement_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      Partition(a, b, c) => Partition(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      SteppedPartition(a, b, c) => SteppedPartition(
        replacement_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      Pad(a, b, c) => Pad(
        replacement_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      EmptyMap(a) => EmptyMap(output_translator(a)),
      Keys(a, b) => Keys(output_translator(a), input_translator(b)),
      Values(a, b) => Values(output_translator(a), input_translator(b)),
      Zip(a, b, c) => Zip(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      Invert(a) => Invert(replacement_translator(a)),
      Merge(a, b, c) => Merge(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      MergeWith(a, b, c) => MergeWith(
        replacement_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      MapKeys(a, b) => MapKeys(replacement_translator(a), input_translator(b)),
      MapValues(a, b) => {
        MapValues(replacement_translator(a), input_translator(b))
      }
      SelectKeys(a, b) => {
        SelectKeys(replacement_translator(a), input_translator(b))
      }
      EmptySet(a) => EmptySet(output_translator(a)),
      Union(a, b, c) => Union(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      Intersection(a, b, c) => Intersection(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      Difference(a, b, c) => Difference(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      SymmetricDifference(a, b, c) => SymmetricDifference(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      InfiniteRange(a) => InfiniteRange(output_translator(a)),
      UpperBoundedRange(a, b) => {
        UpperBoundedRange(output_translator(a), input_translator(b))
      }
      LowerUpperBoundedRange(a, b, c) => LowerUpperBoundedRange(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      InfiniteRepeat(a, b) => {
        InfiniteRepeat(output_translator(a), input_translator(b))
      }
      BoundedRepeat(a, b, c) => BoundedRepeat(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      InfiniteRepeatedly(a, b) => {
        InfiniteRepeatedly(output_translator(a), input_translator(b))
      }
      BoundedRepeatedly(a, b, c) => BoundedRepeatedly(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      InfiniteIterate(a, b, c) => InfiniteIterate(
        output_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      BoundedIterate(a, b, c) => BoundedIterate(
        replacement_translator(a),
        input_translator(b),
        input_translator(c),
      ),
      CreateCell(a) => CreateCell(output_translator(a)),
      GetCellValue(a, b) => {
        GetCellValue(output_translator(a), input_translator(b))
      }
      SetCellValue(a, b) => {
        SetCellValue(output_translator(a), input_translator(b))
      }
      UpdateCell(a, b) => UpdateCell(output_translator(a), input_translator(b)),
      CreateCoroutine(a) => CreateCoroutine(replacement_translator(a)),
      IsCoroutineAlive(a, b) => {
        IsCoroutineAlive(output_translator(a), input_translator(b))
      }
      Yield(a) => Yield(output_translator(a)),
      YieldAndAccept(a, b, c) => {
        YieldAndAccept(output_translator(a), b, input_translator(c))
      }
      IsNil(a, b) => IsNil(output_translator(a), input_translator(b)),
      IsBool(a, b) => IsBool(output_translator(a), input_translator(b)),
      IsChar(a, b) => IsChar(output_translator(a), input_translator(b)),
      IsNum(a, b) => IsNum(output_translator(a), input_translator(b)),
      IsInt(a, b) => IsInt(output_translator(a), input_translator(b)),
      IsFloat(a, b) => IsFloat(output_translator(a), input_translator(b)),
      IsSymbol(a, b) => IsSymbol(output_translator(a), input_translator(b)),
      IsString(a, b) => IsString(output_translator(a), input_translator(b)),
      IsList(a, b) => IsList(output_translator(a), input_translator(b)),
      IsMap(a, b) => IsMap(output_translator(a), input_translator(b)),
      IsSet(a, b) => IsSet(output_translator(a), input_translator(b)),
      IsCollection(a, b) => {
        IsCollection(output_translator(a), input_translator(b))
      }
      IsFn(a, b) => IsFn(output_translator(a), input_translator(b)),
      IsError(a, b) => IsError(output_translator(a), input_translator(b)),
      IsCell(a, b) => IsCell(output_translator(a), input_translator(b)),
      IsCoroutine(a, b) => {
        IsCoroutine(output_translator(a), input_translator(b))
      }
      ToBool(a, b) => ToBool(output_translator(a), input_translator(b)),
      ToChar(a, b) => ToChar(output_translator(a), input_translator(b)),
      ToNum(a, b) => ToNum(output_translator(a), input_translator(b)),
      ToInt(a, b) => ToInt(output_translator(a), input_translator(b)),
      ToFloat(a, b) => ToFloat(output_translator(a), input_translator(b)),
      ToSymbol(a, b) => ToSymbol(output_translator(a), input_translator(b)),
      ToString(a, b) => ToString(output_translator(a), input_translator(b)),
      ToList(a, b) => ToList(output_translator(a), input_translator(b)),
      ToMap(a, b) => ToMap(output_translator(a), input_translator(b)),
      ToSet(a, b) => ToSet(output_translator(a), input_translator(b)),
      ToError(a, b) => ToError(output_translator(a), input_translator(b)),
    }
  }
}
