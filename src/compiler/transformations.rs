use std::{
  collections::{HashMap, HashSet},
  error::Error,
  fmt::Display,
  rc::Rc,
};

use crate::{
  Block, ConstIndex, GeneralizedBlock, GeneralizedValue, Instruction, Register,
  Value,
};

use super::{SSABlock, SSAInstruction, SSARegister};

type InstructionTimestamp = u16;
use GeneralizedValue::*;
use Instruction::*;

struct RegisterUsages {
  inputs: Vec<SSARegister>,
  outputs: Vec<SSARegister>,
  replacement: Option<(SSARegister, SSARegister)>,
}
impl SSAInstruction {
  pub fn register_lifetime_constraints(&self) -> RegisterUsages {
    let (inputs, outputs, replacement) = match self {
      DebugPrint(_) => (vec![], vec![], None),
      Clear(to) => (vec![], vec![to], None),
      Copy(to, from) => (vec![from], vec![to], None),
      Const(to, _) => (vec![], vec![to], None),
      Print(from) => (vec![from], vec![], None),
      Return(from) => (vec![from], vec![], None),
      CopyArgument(from) => (vec![from], vec![], None),
      StealArgument(from) => (vec![from], vec![], None),
      Call(to, from, _) => (vec![from], vec![to], None),
      Apply((from, to), f) => (vec![f], vec![], Some((from, to))),
      CallSelf(to, _) => (vec![], vec![to], None),
      ApplySelf((from, to)) => (vec![], vec![], Some((from, to))),
      CallAndReturn(f, _) => (vec![f], vec![], None),
      ApplyAndReturn(from, _) => (vec![from], vec![], None),
      CallSelfAndReturn(_) => (vec![], vec![], None),
      ApplySelfAndReturn(from) => (vec![from], vec![], None),
      CallingFunction(to) => (vec![], vec![to], None),
      Jump(_) => (vec![], vec![], None),
      Lookup(to, _) => (vec![], vec![to], None),
      If(from) => (vec![from], vec![], None),
      Else => (vec![], vec![], None),
      ElseIf(from) => (vec![from], vec![], None),
      EndIf => (vec![], vec![], None),
      Partial(to, f, arg) => (vec![f, arg], vec![to], None),
      Compose(to, f_1, f_2) => (vec![f_1, f_2], vec![to], None),
      FindSome(to, f, collection) => (vec![f, collection], vec![to], None),
      ReduceWithoutInitialValue((from, to), f) => {
        (vec![f], vec![], Some((from, to)))
      }
      ReduceWithInitialValue((from, to), f, initial) => {
        (vec![f, initial], vec![], Some((from, to)))
      }
      Memoize(to, from) => (vec![from], vec![to], None),
      Constantly(to, from) => (vec![from], vec![to], None),
      NumericalEqual(to, a, b) => (vec![a, b], vec![to], None),
      IsZero(to, from) => (vec![from], vec![to], None),
      IsNan(to, from) => (vec![from], vec![to], None),
      IsInf(to, from) => (vec![from], vec![to], None),
      IsEven(to, from) => (vec![from], vec![to], None),
      IsOdd(to, from) => (vec![from], vec![to], None),
      IsPos(to, from) => (vec![from], vec![to], None),
      IsNeg(to, from) => (vec![from], vec![to], None),
      Inc(to, from) => (vec![from], vec![to], None),
      Dec(to, from) => (vec![from], vec![to], None),
      Negate(to, from) => (vec![from], vec![to], None),
      Abs(to, from) => (vec![from], vec![to], None),
      Floor(to, from) => (vec![from], vec![to], None),
      Ceil(to, from) => (vec![from], vec![to], None),
      Sqrt(to, from) => (vec![from], vec![to], None),
      Exp(to, from) => (vec![from], vec![to], None),
      Exp2(to, from) => (vec![from], vec![to], None),
      Ln(to, from) => (vec![from], vec![to], None),
      Log2(to, from) => (vec![from], vec![to], None),
      Add(to, a, b) => (vec![a, b], vec![to], None),
      Subtract(to, a, b) => (vec![a, b], vec![to], None),
      Multiply(to, a, b) => (vec![a, b], vec![to], None),
      Divide(to, a, b) => (vec![a, b], vec![to], None),
      Pow(to, a, b) => (vec![a, b], vec![to], None),
      Mod(to, a, b) => (vec![a, b], vec![to], None),
      Quot(to, a, b) => (vec![a, b], vec![to], None),
      Min(to, a, b) => (vec![a, b], vec![to], None),
      Max(to, a, b) => (vec![a, b], vec![to], None),
      GreaterThan(to, a, b) => (vec![a, b], vec![to], None),
      GreaterThanOrEqual(to, a, b) => (vec![a, b], vec![to], None),
      LessThan(to, a, b) => (vec![a, b], vec![to], None),
      LessThanOrEqual(to, a, b) => (vec![a, b], vec![to], None),
      Rand(to) => (vec![], vec![to], None),
      UpperBoundedRand(to, from) => (vec![from], vec![to], None),
      LowerUpperBoundedRand(to, a, b) => (vec![a, b], vec![to], None),
      RandInt(to, from) => (vec![from], vec![to], None),
      LowerBoundedRandInt(to, a, b) => (vec![a, b], vec![to], None),
      Equal(to, a, b) => (vec![a, b], vec![to], None),
      NotEqual(to, a, b) => (vec![a, b], vec![to], None),
      Not(to, from) => (vec![from], vec![to], None),
      And(to, a, b) => (vec![a, b], vec![to], None),
      Or(to, a, b) => (vec![a, b], vec![to], None),
      Xor(to, a, b) => (vec![a, b], vec![to], None),
      IsEmpty(to, from) => (vec![from], vec![to], None),
      First(to, from) => (vec![from], vec![to], None),
      Count(to, from) => (vec![from], vec![to], None),
      Flatten(to, from) => (vec![from], vec![to], None),
      Remove((from, to), x) => (vec![x], vec![], Some((from, to))),
      Filter((from, to), f) => (vec![f], vec![], Some((from, to))),
      Map((from, to), f) => (vec![f], vec![], Some((from, to))),
      DoubleMap((from, to), a, b) => (vec![a, b], vec![], Some((from, to))),
      MultiCollectionMap((from, to), f) => (vec![f], vec![], Some((from, to))),
      Set((from, to), a, b) => (vec![a, b], vec![], Some((from, to))),
      SetIn((from, to), a, b) => (vec![a, b], vec![], Some((from, to))),
      Get(to, a, b) => (vec![a, b], vec![to], None),
      GetIn(to, a, b) => (vec![a, b], vec![to], None),
      Update((from, to), a, b) => (vec![a, b], vec![], Some((from, to))),
      UpdateIn((from, to), a, b) => (vec![a, b], vec![], Some((from, to))),
      MinKey(to, a, b) => (vec![a, b], vec![to], None),
      MaxKey(to, a, b) => (vec![a, b], vec![to], None),
      Push((from, to), f) => (vec![f], vec![], Some((from, to))),
      Sort((from, to)) => (vec![], vec![], Some((from, to))),
      SortBy((from, to), f) => (vec![f], vec![], Some((from, to))),
      EmptyList(to) => (vec![], vec![to], None),
      Last(to, from) => (vec![from], vec![to], None),
      Rest((from, to)) => (vec![], vec![], Some((from, to))),
      ButLast((from, to)) => (vec![], vec![], Some((from, to))),
      Nth(to, a, b) => (vec![a, b], vec![to], None),
      NthFromLast(to, a, b) => (vec![a, b], vec![to], None),
      Cons((from, to), x) => (vec![x], vec![], Some((from, to))),
      Concat((from, to), x) => (vec![x], vec![], Some((from, to))),
      Take((from, to), x) => (vec![x], vec![], Some((from, to))),
      Drop((from, to), x) => (vec![x], vec![], Some((from, to))),
      Reverse((from, to)) => (vec![from], vec![to], Some((from, to))),
      Distinct((from, to)) => (vec![from], vec![to], Some((from, to))),
      Sub((from, to), a, b) => (vec![a, b], vec![], Some((from, to))),
      Partition(to, a, b) => (vec![a, b], vec![to], None),
      SteppedPartition((from, to), a, b) => {
        (vec![a, b], vec![], Some((from, to)))
      }
      Pad((from, to), a, b) => (vec![a, b], vec![], Some((from, to))),
      EmptyMap(to) => (vec![], vec![to], None),
      Keys(to, from) => (vec![from], vec![to], None),
      Values(to, from) => (vec![from], vec![to], None),
      Zip(to, a, b) => (vec![a, b], vec![to], None),
      Invert((from, to)) => (vec![], vec![], Some((from, to))),
      Merge(to, a, b) => (vec![a, b], vec![to], None),
      MergeWith((from, to), a, b) => (vec![a, b], vec![], Some((from, to))),
      MapKeys((from, to), f) => (vec![f], vec![], Some((from, to))),
      MapValues((from, to), f) => (vec![f], vec![], Some((from, to))),
      SelectKeys((from, to), x) => (vec![x], vec![], Some((from, to))),
      EmptySet(to) => (vec![], vec![to], None),
      Union(to, a, b) => (vec![a, b], vec![to], None),
      Intersection(to, a, b) => (vec![a, b], vec![to], None),
      Difference(to, a, b) => (vec![a, b], vec![to], None),
      SymmetricDifference(to, a, b) => (vec![a, b], vec![to], None),
      InfiniteRange(to) => (vec![], vec![to], None),
      UpperBoundedRange(to, from) => (vec![from], vec![to], None),
      LowerUpperBoundedRange(to, a, b) => (vec![a, b], vec![to], None),
      InfiniteRepeat(to, from) => (vec![from], vec![to], None),
      BoundedRepeat(to, a, b) => (vec![a, b], vec![to], None),
      InfiniteRepeatedly(to, from) => (vec![from], vec![to], None),
      BoundedRepeatedly(to, a, b) => (vec![a, b], vec![to], None),
      InfiniteIterate(to, a, b) => (vec![a, b], vec![to], None),
      BoundedIterate((from, to), a, b) => {
        (vec![a, b], vec![], Some((from, to)))
      }
      CreateCell(to) => (vec![], vec![to], None),
      GetCellValue(to, from) => (vec![from], vec![to], None),
      SetCellValue(to, from) => (vec![from], vec![to], None),
      UpdateCell(to, from) => (vec![from], vec![to], None),
      CreateCoroutine((from, to)) => (vec![], vec![], Some((from, to))),
      IsCoroutineAlive(to, from) => (vec![from], vec![to], None),
      Yield(from) => (vec![from], vec![], None),
      YieldAndAccept(from, _, _) => (vec![from], vec![], None),
      IsNil(to, from) => (vec![from], vec![to], None),
      IsBool(to, from) => (vec![from], vec![to], None),
      IsChar(to, from) => (vec![from], vec![to], None),
      IsNum(to, from) => (vec![from], vec![to], None),
      IsInt(to, from) => (vec![from], vec![to], None),
      IsFloat(to, from) => (vec![from], vec![to], None),
      IsSymbol(to, from) => (vec![from], vec![to], None),
      IsString(to, from) => (vec![from], vec![to], None),
      IsList(to, from) => (vec![from], vec![to], None),
      IsMap(to, from) => (vec![from], vec![to], None),
      IsSet(to, from) => (vec![from], vec![to], None),
      IsCollection(to, from) => (vec![from], vec![to], None),
      IsFn(to, from) => (vec![from], vec![to], None),
      IsError(to, from) => (vec![from], vec![to], None),
      IsCell(to, from) => (vec![from], vec![to], None),
      IsCoroutine(to, from) => (vec![from], vec![to], None),
      ToBool(to, from) => (vec![from], vec![to], None),
      ToChar(to, from) => (vec![from], vec![to], None),
      ToNum(to, from) => (vec![from], vec![to], None),
      ToInt(to, from) => (vec![from], vec![to], None),
      ToFloat(to, from) => (vec![from], vec![to], None),
      ToSymbol(to, from) => (vec![from], vec![to], None),
      ToString(to, from) => (vec![from], vec![to], None),
      ToList(to, from) => (vec![from], vec![to], None),
      ToMap(to, from) => (vec![from], vec![to], None),
      ToSet(to, from) => (vec![from], vec![to], None),
      ToError(to, from) => (vec![from], vec![to], None),
    };
    RegisterUsages {
      inputs: inputs.into_iter().cloned().collect(),
      outputs: outputs.into_iter().cloned().collect(),
      replacement: replacement.map(|(from, to)| (*from, *to)),
    }
  }
}

#[derive(Clone, Debug)]
enum LifetimeError {
  UsedBeforeCreation(SSARegister, InstructionTimestamp),
  OutputToExisting(SSARegister, InstructionTimestamp, InstructionTimestamp),
  ReplacingNonexistent(SSARegister, InstructionTimestamp),
  UsedAfterReplacement(
    SSARegister,
    InstructionTimestamp,
    SSARegister,
    InstructionTimestamp,
  ),
  ReplacingAfterReplacement(
    SSARegister,
    InstructionTimestamp,
    SSARegister,
    InstructionTimestamp,
  ),
  Unused(SSARegister, InstructionTimestamp),
}
impl Display for LifetimeError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use LifetimeError::*;
    match *self {
      UsedBeforeCreation(register, timestamp) => write!(
        f,
        "attempted to use register {register} before creation at \
         timestamp {timestamp}"
      ),
      OutputToExisting(register, created_timestamp, new_timestamp) => write!(
        f,
        "attempted to output to register {register} at timestamp \
         {new_timestamp}, when register was already created at timestamp \
         {created_timestamp}"
      ),
      ReplacingNonexistent(register, timestamp) => write!(
        f,
        "attempting to replace register {register} at timestamp {timestamp}, \
         but the register does not exist"
      ),
      UsedAfterReplacement(
        register,
        timestamp,
        already_replaced_register,
        already_replaced_timestamp,
      ) => write!(
        f,
        "attempting to use register {register} at timestamp {timestamp}, \
         but the register as already replaced by {already_replaced_register} at
         timestamp {already_replaced_timestamp}"
      ),
      ReplacingAfterReplacement(
        register,
        timestamp,
        already_replaced_register,
        already_replaced_timestamp,
      ) => write!(
        f,
        "attempting to replace register {register} at timestamp {timestamp}, \
         but the register as already replaced by {already_replaced_register} at
         timestamp {already_replaced_timestamp}"
      ),
      Unused(register, timestamp) => {
        write!(
          f,
          "register {register}, created at timestamp {timestamp}, is never used"
        )
      }
    }
  }
}
impl Error for LifetimeError {}

#[derive(Clone, Debug)]
pub struct RegisterLifetime {
  creation: InstructionTimestamp,
  last_usage: Option<InstructionTimestamp>,
  replacing: Option<SSARegister>,
  replaced_by: Option<SSARegister>,
}
impl RegisterLifetime {
  fn new(creation_timestamp: InstructionTimestamp) -> Self {
    Self {
      creation: creation_timestamp,
      last_usage: None,
      replacing: None,
      replaced_by: None,
    }
  }
  fn new_replacing(
    creation_timestamp: InstructionTimestamp,
    replacing: SSARegister,
  ) -> Self {
    Self {
      creation: creation_timestamp,
      last_usage: None,
      replacing: Some(replacing),
      replaced_by: None,
    }
  }
}

pub fn track_register_lifetimes<M>(
  block: SSABlock<M>,
) -> Result<SSABlock<HashMap<SSARegister, RegisterLifetime>>, LifetimeError> {
  block.replace_metadata(&|instructions, _, _| {
    let mut lifetimes: HashMap<SSARegister, RegisterLifetime> = HashMap::new();
    for (timestamp, instruction) in block.instructions.iter().enumerate() {
      let timestamp = timestamp as InstructionTimestamp;
      let usages = instruction.register_lifetime_constraints();
      for input_register in usages.inputs {
        if let Some(lifetime) = lifetimes.get_mut(&input_register) {
          if let Some(replaced_by) = lifetime.replaced_by {
            return Err(LifetimeError::UsedAfterReplacement(
              input_register,
              timestamp,
              replaced_by,
              lifetime.last_usage.unwrap(),
            ));
          }
          lifetime.last_usage = Some(timestamp);
        } else {
          return Err(LifetimeError::UsedBeforeCreation(
            input_register,
            timestamp,
          ));
        }
      }
      for output_register in usages.outputs {
        if let Some(existing_lifetime) = lifetimes.get(&output_register) {
          return Err(LifetimeError::OutputToExisting(
            output_register,
            existing_lifetime.creation,
            timestamp,
          ));
        } else {
          lifetimes.insert(output_register, RegisterLifetime::new(timestamp));
        }
      }
      if let Some((from_register, to_register)) = usages.replacement {
        if let Some(from_lifetime) = lifetimes.get_mut(&from_register) {
          if let Some(replaced_by_register) = from_lifetime.replaced_by {
            return Err(LifetimeError::UsedAfterReplacement(
              from_register,
              timestamp,
              replaced_by_register,
              from_lifetime.last_usage.unwrap(),
            ));
          } else {
            from_lifetime.last_usage = Some(timestamp);
            from_lifetime.replaced_by = Some(to_register);
          }
        } else {
          return Err(LifetimeError::ReplacingNonexistent(
            from_register,
            timestamp,
          ));
        }
        if let Some(to_lifetime) = lifetimes.get(&to_register) {
          return Err(LifetimeError::OutputToExisting(
            to_register,
            to_lifetime.creation,
            timestamp,
          ));
        } else {
          lifetimes.insert(
            to_register,
            RegisterLifetime::new_replacing(timestamp, from_register),
          );
        }
      }
    }
    for (register, lifetime) in lifetimes.iter() {
      if lifetime.last_usage.is_none() {
        return Err(LifetimeError::Unused(*register, lifetime.creation));
      }
    }
    Ok(lifetimes)
  })
}

#[derive(Clone, Debug)]
enum RegisterAllocationError {}
impl Display for RegisterAllocationError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match *self {}
  }
}
impl Error for RegisterAllocationError {}

pub fn allocate_registers(
  block: SSABlock<HashMap<SSARegister, RegisterLifetime>>,
) -> Result<Block, RegisterAllocationError> {
  block.translate_instructions(&|instructions, lifetimes| {
    Ok((
      {
        let mut ssa_to_real_registers: HashMap<SSARegister, Register> =
          HashMap::new();
        let mut taken_registers: HashSet<Register> = HashSet::new();
        let mut translated_instructions = vec![];
        for (timestamp, instruction) in instructions.iter().enumerate() {
          let timestamp = timestamp as u16;
          for (virtual_register, register_lifetime) in lifetimes.iter() {
            if register_lifetime.creation == timestamp {
              if let Some(replaced_virtual_register) =
                register_lifetime.replacing
              {
                let register = ssa_to_real_registers
                  .remove(&replaced_virtual_register)
                  .expect("Didn't find register when trying to replace");
                ssa_to_real_registers.insert(*virtual_register, register);
              } else {
                let min_unused_register = (0..Register::MAX)
                  .filter(|i| !taken_registers.contains(i))
                  .next()
                  .expect("Failed to find unused register");
                let replaced_register = ssa_to_real_registers
                  .insert(*virtual_register, min_unused_register);
                #[cfg(debug_assertions)]
                assert!(replaced_register.is_none());
                let register_already_taken =
                  taken_registers.insert(min_unused_register);
                #[cfg(debug_assertions)]
                assert!(!register_already_taken);
              }
            }
          }
          todo!() // push a value into translated_instructions
        }
        translated_instructions
      },
      (),
    ))
  })
}
