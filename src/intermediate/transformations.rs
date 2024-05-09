use std::{collections::HashMap, rc::Rc};

use crate::{
  ConstIndex, GeneralizedValue, Instruction, InstructionBlock, RegisterIndex,
  Value,
};

use super::ir_instructions::VirtualRegister;

pub type RegisterLifetimes = HashMap<VirtualRegister, (usize, usize)>;

use GeneralizedValue::*;
use Instruction::*;

pub fn calculate_lifetimes<C, M>(
  block: InstructionBlock<VirtualRegister, C, M>,
) -> InstructionBlock<VirtualRegister, C, RegisterLifetimes> {
  let mut lifetimes = RegisterLifetimes::new();
  for (i, instruction) in block.instructions.iter().enumerate() {
    let (inputs, outputs) = instruction.input_and_output_registers();
    for input in inputs {
      lifetimes.entry(input).or_insert((i, i));
    }
    for output in outputs {
      lifetimes
        .entry(output)
        .and_modify(|span| span.1 = i)
        .or_insert((i, i));
    }
  }
  block.replace_metadata(lifetimes)
}

#[derive(Clone, Debug)]
struct InlinedConstValue<R, M>(GeneralizedValue<R, InlinedConstValue<R, M>, M>);
type ExtractedConstValue<R, M> = GeneralizedValue<R, ConstIndex, M>;

fn extract_constants_inner<R: Clone, M: Clone>(
  existing_constants: Vec<ExtractedConstValue<R, M>>,
  instructions: Rc<[Instruction<R, InlinedConstValue<R, M>>]>,
) -> (
  Vec<ExtractedConstValue<R, M>>,
  Vec<Instruction<R, ConstIndex>>,
) {
  let instructions = (*instructions).to_vec();
  instructions.into_iter().fold(
    (existing_constants, vec![]),
    |(mut existing_constants, mut processed_instructions), instruction| {
      processed_instructions.push(match instruction {
        DebugPrint(a) => DebugPrint(a),
        Clear(a) => Clear(a),
        Copy(a, b) => Copy(a, b),
        Const(register, value) => {
          let extracted_const_value: ExtractedConstValue<R, M> = match value.0 {
            CompositeFn(f_rc) => {
              let f = (*f_rc).clone();
              todo!()
            }
            Nil => Nil,
            Bool(a) => Bool(a),
            Char(a) => Char(a),
            Number(a) => Number(a),
            Symbol(a) => Symbol(a),
            Str(a) => Str(a),
            List(a) => List(a),
            Hashmap(a) => Hashmap(a),
            Hashset(a) => Hashset(a),
            CoreFn(a) => CoreFn(a),
            ExternalFn(a) => ExternalFn(a),
            ExternalObject(a) => ExternalObject(a),
            Coroutine(a) => Coroutine(a),
            Error(a) => Error(a),
          };
          let const_index = existing_constants.len() as u16;
          existing_constants.push(extracted_const_value);
          Const(register, const_index)
        }
        Print(a) => Print(a),
        Return(a) => Return(a),
        CopyArgument(a) => CopyArgument(a),
        StealArgument(a) => StealArgument(a),
        Call(a, b, c) => Call(a, b, c),
        Apply(a, b) => Apply(a, b),
        CallSelf(a, b) => CallSelf(a, b),
        ApplySelf(a) => ApplySelf(a),
        CallAndReturn(a, b) => CallAndReturn(a, b),
        ApplyAndReturn(a, b) => ApplyAndReturn(a, b),
        CallSelfAndReturn(a) => CallSelfAndReturn(a),
        ApplySelfAndReturn(a) => ApplySelfAndReturn(a),
        CallingFunction(a) => CallingFunction(a),
        Jump(a) => Jump(a),
        Lookup(a, b) => Lookup(a, b),
        If(a) => If(a),
        Else => Else,
        ElseIf(a) => ElseIf(a),
        EndIf => EndIf,
        Partial(a, b, c) => Partial(a, b, c),
        Compose(a, b, c) => Compose(a, b, c),
        FindSome(a, b, c) => FindSome(a, b, c),
        ReduceWithoutInitialValue(a, b, c) => {
          ReduceWithoutInitialValue(a, b, c)
        }
        ReduceWithInitialValue(a, b, c) => ReduceWithInitialValue(a, b, c),
        Memoize(a, b) => Memoize(a, b),
        Constantly(a, b) => Constantly(a, b),
        NumericalEqual(a, b, c) => NumericalEqual(a, b, c),
        IsZero(a, b) => IsZero(a, b),
        IsNan(a, b) => IsNan(a, b),
        IsInf(a, b) => IsInf(a, b),
        IsEven(a, b) => IsEven(a, b),
        IsOdd(a, b) => IsOdd(a, b),
        IsPos(a, b) => IsPos(a, b),
        IsNeg(a, b) => IsNeg(a, b),
        Inc(a, b) => Inc(a, b),
        Dec(a, b) => Dec(a, b),
        Negate(a, b) => Negate(a, b),
        Abs(a, b) => Abs(a, b),
        Floor(a, b) => Floor(a, b),
        Ceil(a, b) => Ceil(a, b),
        Sqrt(a, b) => Sqrt(a, b),
        Exp(a, b) => Exp(a, b),
        Exp2(a, b) => Exp2(a, b),
        Ln(a, b) => Ln(a, b),
        Log2(a, b) => Log2(a, b),
        Add(a, b, c) => Add(a, b, c),
        Subtract(a, b, c) => Subtract(a, b, c),
        Multiply(a, b, c) => Multiply(a, b, c),
        Divide(a, b, c) => Divide(a, b, c),
        Pow(a, b, c) => Pow(a, b, c),
        Mod(a, b, c) => Mod(a, b, c),
        Quot(a, b, c) => Quot(a, b, c),
        Min(a, b, c) => Min(a, b, c),
        Max(a, b, c) => Max(a, b, c),
        GreaterThan(a, b, c) => GreaterThan(a, b, c),
        GreaterThanOrEqual(a, b, c) => GreaterThanOrEqual(a, b, c),
        LessThan(a, b, c) => LessThan(a, b, c),
        LessThanOrEqual(a, b, c) => LessThanOrEqual(a, b, c),
        Rand(a) => Rand(a),
        UpperBoundedRand(a, b) => UpperBoundedRand(a, b),
        LowerUpperBoundedRand(a, b, c) => LowerUpperBoundedRand(a, b, c),
        RandInt(a, b) => RandInt(a, b),
        LowerBoundedRandInt(a, b, c) => LowerBoundedRandInt(a, b, c),
        Equal(a, b, c) => Equal(a, b, c),
        NotEqual(a, b, c) => NotEqual(a, b, c),
        Not(a, b) => Not(a, b),
        And(a, b, c) => And(a, b, c),
        Or(a, b, c) => Or(a, b, c),
        Xor(a, b, c) => Xor(a, b, c),
        IsEmpty(a, b) => IsEmpty(a, b),
        First(a, b) => First(a, b),
        Count(a, b) => Count(a, b),
        Flatten(a, b) => Flatten(a, b),
        Remove(a, b) => Remove(a, b),
        Filter(a, b) => Filter(a, b),
        Map(a, b) => Map(a, b),
        DoubleMap(a, b, c) => DoubleMap(a, b, c),
        MultiCollectionMap(a, b) => MultiCollectionMap(a, b),
        Set(a, b, c) => Set(a, b, c),
        SetIn(a, b, c) => SetIn(a, b, c),
        Get(a, b, c) => Get(a, b, c),
        GetIn(a, b, c) => GetIn(a, b, c),
        Update(a, b, c) => Update(a, b, c),
        UpdateIn(a, b, c) => UpdateIn(a, b, c),
        MinKey(a, b, c) => MinKey(a, b, c),
        MaxKey(a, b, c) => MaxKey(a, b, c),
        Push(a, b) => Push(a, b),
        Sort(a) => Sort(a),
        SortBy(a, b) => SortBy(a, b),
        EmptyList(a) => EmptyList(a),
        Last(a, b) => Last(a, b),
        Rest(a) => Rest(a),
        ButLast(a) => ButLast(a),
        Nth(a, b, c) => Nth(a, b, c),
        NthFromLast(a, b, c) => NthFromLast(a, b, c),
        Cons(a, b) => Cons(a, b),
        Concat(a, b) => Concat(a, b),
        Take(a, b) => Take(a, b),
        Drop(a, b) => Drop(a, b),
        Reverse(a) => Reverse(a),
        Distinct(a) => Distinct(a),
        Sub(a, b, c) => Sub(a, b, c),
        Partition(a, b, c) => Partition(a, b, c),
        SteppedPartition(a, b, c) => SteppedPartition(a, b, c),
        Pad(a, b, c) => Pad(a, b, c),
        EmptyMap(a) => EmptyMap(a),
        Keys(a, b) => Keys(a, b),
        Values(a, b) => Values(a, b),
        Zip(a, b, c) => Zip(a, b, c),
        Invert(a, b) => Invert(a, b),
        Merge(a, b, c) => Merge(a, b, c),
        MergeWith(a, b, c) => MergeWith(a, b, c),
        MapKeys(a, b, c) => MapKeys(a, b, c),
        MapValues(a, b, c) => MapValues(a, b, c),
        SelectKeys(a, b) => SelectKeys(a, b),
        EmptySet(a) => EmptySet(a),
        Union(a, b, c) => Union(a, b, c),
        Intersection(a, b, c) => Intersection(a, b, c),
        Difference(a, b, c) => Difference(a, b, c),
        SymmetricDifference(a, b, c) => SymmetricDifference(a, b, c),
        InfiniteRange(a) => InfiniteRange(a),
        UpperBoundedRange(a, b) => UpperBoundedRange(a, b),
        LowerUpperBoundedRange(a, b, c) => LowerUpperBoundedRange(a, b, c),
        InfiniteRepeat(a, b) => InfiniteRepeat(a, b),
        BoundedRepeat(a, b, c) => BoundedRepeat(a, b, c),
        InfiniteRepeatedly(a, b) => InfiniteRepeatedly(a, b),
        BoundedRepeatedly(a, b, c) => BoundedRepeatedly(a, b, c),
        InfiniteIterate(a, b, c) => InfiniteIterate(a, b, c),
        BoundedIterate(a, b, c) => BoundedIterate(a, b, c),
        CreateCell(a) => CreateCell(a),
        GetCellValue(a, b) => GetCellValue(a, b),
        SetCellValue(a, b) => SetCellValue(a, b),
        UpdateCell(a, b) => UpdateCell(a, b),
        CreateCoroutine(a) => CreateCoroutine(a),
        IsCoroutineAlive(a, b) => IsCoroutineAlive(a, b),
        Yield(a) => Yield(a),
        YieldAndAccept(a, b, c) => YieldAndAccept(a, b, c),
        IsNil(a, b) => IsNil(a, b),
        IsBool(a, b) => IsBool(a, b),
        IsChar(a, b) => IsChar(a, b),
        IsNum(a, b) => IsNum(a, b),
        IsInt(a, b) => IsInt(a, b),
        IsFloat(a, b) => IsFloat(a, b),
        IsSymbol(a, b) => IsSymbol(a, b),
        IsString(a, b) => IsString(a, b),
        IsList(a, b) => IsList(a, b),
        IsMap(a, b) => IsMap(a, b),
        IsSet(a, b) => IsSet(a, b),
        IsCollection(a, b) => IsCollection(a, b),
        IsFn(a, b) => IsFn(a, b),
        IsError(a, b) => IsError(a, b),
        IsCell(a, b) => IsCell(a, b),
        IsCoroutine(a, b) => IsCoroutine(a, b),
        ToBool(a, b) => ToBool(a, b),
        ToChar(a, b) => ToChar(a, b),
        ToNum(a, b) => ToNum(a, b),
        ToInt(a, b) => ToInt(a, b),
        ToFloat(a, b) => ToFloat(a, b),
        ToSymbol(a, b) => ToSymbol(a, b),
        ToString(a, b) => ToString(a, b),
        ToList(a, b) => ToList(a, b),
        ToMap(a, b) => ToMap(a, b),
        ToSet(a, b) => ToSet(a, b),
        ToError(a, b) => ToError(a, b),
      });
      (existing_constants, processed_instructions)
    },
  )
}

pub fn extract_constants<R: Clone, M: Clone>(
  block: InstructionBlock<R, InlinedConstValue<R, M>, M>,
) -> (
  Vec<ExtractedConstValue<R, M>>,
  InstructionBlock<R, ConstIndex, M>,
) {
  let metadata = block.metadata;
  let (constants, new_instructions) =
    extract_constants_inner::<R, M>(vec![], block.instructions);
  let new_instruction_block: InstructionBlock<R, ConstIndex, ()> =
    new_instructions.into();
  (constants, new_instruction_block.with_metadata(metadata))
}
