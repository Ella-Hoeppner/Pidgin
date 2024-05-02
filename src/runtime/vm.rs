use std::collections::HashMap;

use minivec::{mini_vec, MiniVec};

use crate::string_utils::pad;
use crate::{string_utils::indent_lines, Instruction, Num, Value};
use Instruction::*;
use Num::*;
use Value::*;

use super::{Error, Result};

const STACK_CAPACITY: usize = 30000; //u16::MAX as usize + 1;

pub type RegisterIndex = u8;
pub type StackIndex = u16;
pub type SymbolIndex = u16;
pub type ConstIndex = u16;
pub type CoreFnIndex = u8;

#[derive(Debug)]
pub struct Program {
  instructions: Vec<Instruction>,
  constants: Vec<Value>,
}
impl Program {
  pub fn new(instructions: Vec<Instruction>, constants: Vec<Value>) -> Self {
    Self {
      instructions,
      constants,
    }
  }
}

struct StackFrame {
  beginning: StackIndex,
  return_stack_index: StackIndex,
}
impl StackFrame {
  fn root() -> Self {
    Self {
      beginning: 0,
      return_stack_index: 0,
    }
  }
}

pub struct EvaluationState {
  stack: [Value; STACK_CAPACITY],
  frames: Vec<StackFrame>,
  consumption: StackIndex,
  environment: HashMap<SymbolIndex, Value>,
}

impl EvaluationState {
  pub fn new() -> Self {
    const NIL: Value = Nil;
    Self {
      stack: [NIL; STACK_CAPACITY],
      frames: vec![StackFrame::root()],
      consumption: 0,
      environment: HashMap::new(),
    }
  }
  fn current_stack_frame_beginning(&self) -> StackIndex {
    self
      .frames
      .last()
      .map(|stack_frame| stack_frame.beginning)
      .unwrap_or(0)
  }
  fn describe_stack(&self) -> String {
    self
      .frames
      .iter()
      .map(|frame| Some(frame))
      .chain(std::iter::once(None))
      .collect::<Vec<_>>()
      .windows(2)
      .enumerate()
      .map(|(frame_index, window)| {
        let frame = window[0].unwrap();
        let maybe_next_frame = window[1];
        let start = frame.beginning;
        let end = maybe_next_frame
          .map(|next_frame| next_frame.beginning)
          .unwrap_or(self.consumption);
        format!(
          "{}\n{}",
          pad(
            30,
            '-',
            format!(
              "------ {}: ({} - {}) -> {} ",
              frame_index, start, end, frame.return_stack_index
            )
          ),
          indent_lines(
            7,
            (start..end)
              .map(|i| format!("{}: {}", i, self.get_stack(i).description()))
              .collect::<Vec<String>>()
              .join("\n")
          ),
        )
      })
      .collect::<Vec<String>>()
      .join("\n")
  }
  fn describe_environment(&self) -> String {
    let mut bindings: Vec<_> = self.environment.iter().collect();
    bindings.sort_by_key(|(symbol_index, _value_pointer)| **symbol_index);
    bindings
      .into_iter()
      .map(|(symbol_index, value)| {
        format!("{}: {}", symbol_index, value.description())
      })
      .collect::<Vec<String>>()
      .join("\n")
  }
  fn set_stack_usize(&mut self, index: usize, value: Value) {
    self.stack[index] = value;
    self.consumption = self.consumption.max(index as u16 + 1);
  }
  fn set_stack(&mut self, index: StackIndex, value: Value) {
    self.set_stack_usize(index as usize, value);
  }
  fn swap_stack_usize(&mut self, index: usize, value: Value) -> Value {
    std::mem::replace(&mut self.stack[index], value)
  }
  fn swap_stack(&mut self, index: StackIndex, value: Value) -> Value {
    self.swap_stack_usize(index as usize, value)
  }
  fn steal_stack_usize(&mut self, index: usize) -> Value {
    self.swap_stack_usize(index, Nil)
  }
  fn steal_stack(&mut self, index: StackIndex) -> Value {
    self.steal_stack_usize(index as usize)
  }
  fn get_stack_usize(&self, index: usize) -> &Value {
    &self.stack[index]
  }
  fn get_stack(&self, index: StackIndex) -> &Value {
    self.get_stack_usize(index as usize)
  }
  fn get_stack_mut_usize(&mut self, index: usize) -> &mut Value {
    &mut self.stack[index]
  }
  fn get_stack_mut(&mut self, index: StackIndex) -> &mut Value {
    self.get_stack_mut_usize(index as usize)
  }
  fn register_stack_index(&self, register: RegisterIndex) -> StackIndex {
    self.current_stack_frame_beginning() + register as StackIndex
  }
  fn set_register<T: Into<Value>>(
    &mut self,
    register: RegisterIndex,
    value: T,
  ) {
    self.set_stack(self.register_stack_index(register), value.into());
  }
  fn swap_register<T: Into<Value>>(
    &mut self,
    register: RegisterIndex,
    value: T,
  ) -> Value {
    self.swap_stack(self.register_stack_index(register), value.into())
  }
  fn steal_register(&mut self, register: RegisterIndex) -> Value {
    self.steal_stack(self.register_stack_index(register))
  }
  fn get_register(&self, register: RegisterIndex) -> &Value {
    //debug
    if register as usize >= self.consumption as usize {
      panic!("trying to access register that hasn't been set yet")
    }
    //
    self.get_stack(self.register_stack_index(register))
  }
  fn get_register_mut(&mut self, register: RegisterIndex) -> &mut Value {
    //debug
    if register as usize >= self.consumption as usize {
      panic!("trying to access register that hasn't been set yet")
    }
    //
    self.get_stack_mut(self.register_stack_index(register))
  }
  pub fn evaluate(&mut self, program: Program) -> Result<()> {
    let mut instruction_stack = program.instructions.clone();
    instruction_stack.reverse();
    while let Some(instruction) = instruction_stack.pop() {
      match instruction {
        DebugPrint(id) => {
          println!(
            "{}\n\
             stack:\n{}\n\n\n\
             environment:\n{}\n\
             ----------------------------------------\n\n\n",
            pad(40, '-', format!("DEBUG {} ", id)),
            self.describe_stack(),
            indent_lines(2, self.describe_environment())
          );
        }
        Clear(register) => self.set_register(register, Nil),
        Copy(result, value) => {
          self.set_register(result, self.get_register(value).clone())
        }
        Const(result, const_index) => {
          self.set_register(
            result,
            program.constants[const_index as usize].clone(),
          );
        }
        Print(value) => {
          println!("{}", self.get_register(value).description())
        }
        EmptyRawVec(result, arg_count) => self.set_register(
          result,
          RawVec(MiniVec::with_capacity(arg_count as usize)),
        ),
        CopyIntoRawVec(args, new_arg) => {
          let new_arg_value = self.get_register(new_arg).clone();
          if let RawVec(args_vec) = self.get_register_mut(args) {
            args_vec.push(new_arg_value);
          } else {
            panic!("CopyArg called with non-RawVec value")
          }
        }
        StealIntoRawVec(args, new_arg) => {
          let new_arg_value = self.steal_register(new_arg);
          if let RawVec(args_vec) = self.get_register_mut(args) {
            args_vec.push(new_arg_value);
          } else {
            panic!("CopyArg called with non-RawVec value")
          }
        }
        Argument(SymbolIndex) => {
          panic!("Instruction::Argument called, this should never happen")
        }
        Return(value) => {
          let return_value = self.get_register(value).clone();
          let finished_stack_frame = self.frames.pop().unwrap();
          for i in finished_stack_frame.beginning..self.consumption {
            self.set_stack(i, Nil);
          }
          self.consumption = finished_stack_frame.beginning;
          self.set_stack(finished_stack_frame.return_stack_index, return_value);
        }
        Apply0(result, f) => {
          // Applies a function of 0 arguments (a thunk)
          let f_value = self.get_register(f).clone();
          self.frames.push(StackFrame {
            beginning: self.consumption,
            return_stack_index: self.register_stack_index(result),
          });
          match f_value {
            CoreFn(core_fn_index) => {
              let core_fn = CORE_FNS[core_fn_index as usize];
              todo!();
            }
            CompositeFn(instructions) => {
              for instruction in instructions.iter().cloned().rev() {
                instruction_stack.push(instruction);
              }
            }
            List(_) | Hashmap(_) | Hashset(_) => {
              return Err(Error::InvalidArity)
            }
            _ => {
              return Err(Error::CantApply);
            }
          }
        }
        Apply1(arg_and_result, f) => {
          // Applies a function of a single argument.
          let f_value = self.get_register(f).clone();
          let arg_value = self.steal_register(arg_and_result);
          self.frames.push(StackFrame {
            beginning: self.consumption,
            return_stack_index: self.register_stack_index(arg_and_result),
          });
          match f_value {
            CoreFn(core_fn_index) => {
              let core_fn = CORE_FNS[core_fn_index as usize];
              todo!();
            }
            CompositeFn(instructions) => {
              let mut remaining_instructions = instructions.iter().cloned();
              if let Some(Argument(symbol_index)) =
                remaining_instructions.next()
              {
                self.environment.insert(symbol_index, arg_value.clone());
              } else {
                panic!(
                  "CompositeFn missing Argument instruction (called from \
                  Apply1)"
                )
              }
              for instruction in remaining_instructions.rev() {
                instruction_stack.push(instruction);
              }
            }
            List(list) => todo!(),
            Hashmap(map) => todo!(),
            Hashset(set) => todo!(),
            _ => {
              return Err(Error::CantApply);
            }
          }
        }
        Apply2(arg_1_and_result, f, arg_2) => {
          // Applies a function of 2 arguments.
          let f_value = self.get_register(f).clone();
          let arg_1_value = self.steal_register(arg_1_and_result);
          let arg_2_value = self.get_register(arg_2).clone();
          self.frames.push(StackFrame {
            beginning: self.consumption,
            return_stack_index: self.register_stack_index(arg_1_and_result),
          });
          match f_value {
            CoreFn(core_fn_index) => {
              let core_fn = CORE_FNS[core_fn_index as usize];
              todo!();
            }
            CompositeFn(instructions) => {
              let mut remaining_instructions = instructions.iter().cloned();
              if let Some(Argument(symbol_1_index)) =
                remaining_instructions.next()
              {
                self.environment.insert(symbol_1_index, arg_1_value.clone());
              } else {
                panic!(
                  "CompositeFn missing first Argument instruction (called from\
                   Apply2)"
                )
              }
              if let Some(Argument(symbol_2_index)) =
                remaining_instructions.next()
              {
                self.environment.insert(symbol_2_index, arg_2_value.clone());
              } else {
                panic!(
                  "CompositeFn missing second Argument instruction (called from\
                   Apply2)"
                )
              }
              for instruction in remaining_instructions.rev() {
                instruction_stack.push(instruction);
              }
            }
            List(_) | Hashmap(_) | Hashset(_) => {
              return Err(Error::InvalidArity)
            }
            _ => {
              return Err(Error::CantApply);
            }
          }
        }
        ApplyN(args_and_result, f) => {
          // Applies a function of a single argument.
          let f_value = self.get_register(f).clone();
          if let RawVec(args_vec) = self.steal_register(args_and_result) {
            self.frames.push(StackFrame {
              beginning: self.consumption,
              return_stack_index: self.register_stack_index(args_and_result),
            });
            match f_value {
              CoreFn(core_fn_index) => {
                let core_fn = CORE_FNS[core_fn_index as usize];
                todo!();
              }
              CompositeFn(instructions) => {
                let mut remaining_instructions =
                  instructions.iter().cloned().into_iter();
                for (i, arg_value) in args_vec.into_iter().enumerate() {
                  if let Some(Argument(symbol_index)) =
                    remaining_instructions.next()
                  {
                    self.environment.insert(symbol_index, arg_value);
                  } else {
                    panic!(
                      "CompositeFn called by ApplyN with wrong number of\
                      arguments"
                    )
                  }
                }
                for instruction in remaining_instructions.rev() {
                  instruction_stack.push(instruction);
                }
              }
              List(list) => todo!(),
              Hashmap(map) => todo!(),
              Hashset(set) => todo!(),
              _ => {
                return Err(Error::CantApply);
              }
            }
          } else {
            panic!("ApplyN called with non-RawVec value");
          }
        }
        Apply0AndReturn(f) => todo!(),
        Apply1AndReturn(f, args) => {
          // This instruction is for supporting tail-call elimination. It takes
          // a function and its arguments just like `Apply`, but before invoking
          // the function it cleans up the current stack frame, so tail-call
          // recursive functions don't consume more space than necessary on the
          // stack. Any time a `Apply` instruction would be immediately followed
          // by a `Return` instruction, it should be replaced with this (maybe
          // that can actually just be done in an optimization pass?)
          todo!()
        }
        Apply2AndReturn(arg_1_and_result, f, arg_2) => todo!(),
        ApplyNAndReturn(args_and_result, f) => todo!(),
        Lookup(register, symbol_index) => {
          self.set_register(register, self.environment[&symbol_index].clone());
        }
        Bind(symbol_index, register) => {
          self
            .environment
            .insert(symbol_index, self.get_register(register).clone());
        }
        When(result, condition, thunk) => todo!(),
        If(condition_and_result, thunk_1, thunk_2) => todo!(),
        Partial(result, f, arg) => todo!(),
        Compose(result, f_1, f_2) => todo!(),
        Filter(result, f, collection) => todo!(),
        Map(result, f, collection) => todo!(),
        MultiListMap(result, f, collections) => todo!(),
        FindSome(result, f, collection) => todo!(),
        ReduceWithoutInitialValue(result, f, collection) => todo!(),
        ReduceWithInitialValue(initial_value_and_result, f, collection) => {
          todo!()
        }
        Memoize(result, f) => todo!(),
        Constantly(result, value) => todo!(),
        NumericalEqual(result, num_1, num_2) => self.set_register(
          result,
          match (self.get_register(num_1), self.get_register(num_2)) {
            (Number(a), Number(b)) => a.numerical_equal(b),
            _ => return Err(Error::ArgumentNotNum),
          },
        ),
        IsZero(result, num) => self.set_register(
          result,
          match self.get_register(num) {
            Number(Float(f)) => *f == 0.,
            Number(Int(i)) => *i == 0,
            _ => return Err(Error::ArgumentNotNum),
          },
        ),
        IsNan(result, num) => self.set_register(
          result,
          match self.get_register(num) {
            Number(Float(f)) => f.is_nan(),
            _ => return Err(Error::ArgumentNotNum),
          },
        ),
        IsInf(result, num) => self.set_register(
          result,
          match self.get_register(num) {
            Number(Float(f)) => f.is_infinite(),
            _ => return Err(Error::ArgumentNotNum),
          },
        ),
        IsEven(result, num) => self.set_register(
          result,
          match self.get_register(num) {
            Number(n) => n.as_int_lossless()? % 2 == 0,
            _ => return Err(Error::ArgumentNotNum),
          },
        ),
        IsOdd(result, num) => self.set_register(
          result,
          match self.get_register(num) {
            Number(n) => n.as_int_lossless()? % 2 == 1,
            _ => return Err(Error::ArgumentNotNum),
          },
        ),
        IsPos(result, num) => self.set_register(
          result,
          match self.get_register(num) {
            Number(Float(f)) => **f > 0.,
            Number(Int(i)) => *i > 0,
            _ => return Err(Error::ArgumentNotNum),
          },
        ),
        IsNeg(result, num) => self.set_register(
          result,
          match self.get_register(num) {
            Number(Float(f)) => **f < 0.,
            Number(Int(i)) => *i < 0,
            _ => return Err(Error::ArgumentNotNum),
          },
        ),
        Inc(result, num) => {
          self.set_register(result, self.get_register(num).as_num()?.inc())
        }
        Dec(result, num) => {
          self.set_register(result, self.get_register(num).as_num()?.dec())
        }
        Negate(result, num) => {
          self.set_register(result, -*self.get_register(num).as_num()?)
        }
        Abs(result, num) => {
          self.set_register(result, self.get_register(num).as_num()?.abs())
        }
        Floor(result, num) => {
          self.set_register(result, self.get_register(num).as_num()?.floor())
        }
        Ceil(result, num) => {
          self.set_register(result, self.get_register(num).as_num()?.ceil())
        }
        Sqrt(result, num) => self.set_register(
          result,
          self.get_register(num).as_num()?.as_float().sqrt(),
        ),
        Exp(result, num) => self.set_register(
          result,
          self.get_register(num).as_num()?.as_float().exp(),
        ),
        Exp2(result, num) => self.set_register(
          result,
          self.get_register(num).as_num()?.as_float().exp2(),
        ),
        Ln(result, num) => self.set_register(
          result,
          self.get_register(num).as_num()?.as_float().ln(),
        ),
        Log2(result, num) => self.set_register(
          result,
          self.get_register(num).as_num()?.as_float().log2(),
        ),
        Add(result, num_1, num_2) => self.set_register(
          result,
          *self.get_register(num_1).as_num()?
            + *self.get_register(num_2).as_num()?,
        ),
        Subtract(result, num_1, num_2) => self.set_register(
          result,
          *self.get_register(num_1).as_num()?
            - *self.get_register(num_2).as_num()?,
        ),
        Multiply(result, num_1, num_2) => {
          self.set_register(
            result,
            *self.get_register(num_1).as_num()?
              * *self.get_register(num_2).as_num()?,
          );
        }
        Divide(result, num_1, num_2) => self.set_register(
          result,
          *self.get_register(num_1).as_num()?
            / *self.get_register(num_2).as_num()?,
        ),
        Pow(result, num_1, num_2) => todo!(),
        Mod(result, num_1, num_2) => todo!(),
        Quot(result, num_1, num_2) => todo!(),
        Min(result, num_1, num_2) => todo!(),
        Max(result, num_1, num_2) => todo!(),
        GreaterThan(result, num_1, num_2) => todo!(),
        GreaterThanOrEqual(result, num_1, num_2) => todo!(),
        LessThan(result, num_1, num_2) => todo!(),
        LessThanOrEqual(result, num_1, num_2) => todo!(),
        Rand(result) => todo!(),
        UpperBoundedRand(result, upper_bound) => todo!(),
        LowerUpperBoundedRand(result, lower_bound, upper_bound) => todo!(),
        RandInt(result, upper_bound) => todo!(),
        LowerBoundedRandInt(result, lower_bound, upper_bound) => todo!(),
        Equal(result, value_1, value_2) => todo!(),
        NotEqual(result, value_1, value_2) => todo!(),
        Not(result, value) => todo!(),
        And(result, bool_1, bool_2) => todo!(),
        Or(result, bool_1, bool_2) => todo!(),
        Xor(result, bool_1, bool_2) => todo!(),
        IsEmpty(result, collection) => todo!(),
        Count(result, collection) => todo!(),
        Flatten(result, collection) => todo!(),
        Remove(result, collection, key) => todo!(),
        Set(value_and_result, collection, key) => todo!(),
        SetIn(value_and_result, collection, path) => todo!(),
        Get(result, collection, key) => todo!(),
        GetIn(result, collection, path) => todo!(),
        Update(f_and_result, collection, key) => todo!(),
        UpdateIn(f_and_result, collection, path) => todo!(),
        MinKey(result, collection, f) => todo!(),
        MaxKey(result, collection, f) => todo!(),
        First(result, collection) => todo!(),
        Sort(result, collection) => todo!(),
        SortBy(result, collection, f) => todo!(),
        EmptyList(result) => todo!(),
        ListFromRawVec(result) => todo!(),
        Last(result, list) => todo!(),
        Nth(result, list, n) => {
          // While `Get` returns nil for a list when index is OOB, `Nth` throws
          todo!()
        }
        NthFromLast(result, list, n) => todo!(),
        Cons(result, list, value) => todo!(),
        Push(result, list, value) => todo!(),
        Concat(result, list_1, list_2) => todo!(),
        Take(result, list, n) => todo!(),
        Drop(result, list, n) => todo!(),
        Reverse(result, list) => todo!(),
        Distinct(result, list) => todo!(),
        Sub(start_index_and_result, list, end_index) => todo!(),
        Partition(result, list, size) => todo!(),
        SteppedPartition(step_and_return, list, size) => todo!(),
        Pad(value_and_result, list, size) => todo!(),
        EmptyMap(result) => todo!(),
        Keys(result, map) => todo!(),
        Values(result, map) => todo!(),
        Zip(result, key_list, value_list) => todo!(),
        Invert(result, map) => todo!(),
        Merge(result, map_1, map_2) => todo!(),
        MergeWith(merge_f_and_result, map_1, map_2) => todo!(),
        MapKeys(result, f, map) => todo!(),
        MapValues(result, f, map) => todo!(),
        EmptySet(result) => todo!(),
        Union(result, set_1, set_2) => todo!(),
        Intersection(result, set_1, set_2) => todo!(),
        Difference(result, set_1, set_2) => todo!(),
        SymmetricDifference(result, set_1, set_2) => todo!(),
        InfiniteRange(result) => todo!(),
        UpperBoundedRange(result, size) => todo!(),
        LowerUpperBoundedRange(result, lower_bound, upper_bound) => todo!(),
        InfiniteRepeat(result, value) => todo!(),
        BoundedRepeat(result, value, count) => todo!(),
        InfiniteRepeatedly(result, f) => todo!(),
        BoundedRepeatedly(result, f, count) => todo!(),
        InfiniteIterate(result, f, initial_value) => todo!(),
        BoundedIterate(bound_and_result, f, initial_value) => todo!(),
        IsNil(result, value) => todo!(),
        IsBool(result, value) => todo!(),
        IsChar(result, value) => todo!(),
        IsNum(result, value) => todo!(),
        IsInt(result, value) => todo!(),
        IsFloat(result, value) => todo!(),
        IsSymbol(result, value) => todo!(),
        IsString(result, value) => todo!(),
        IsList(result, value) => todo!(),
        IsMap(result, value) => todo!(),
        IsSet(result, value) => todo!(),
        IsCollection(result, value) => todo!(),
        IsFn(result, value) => todo!(),
        ToBool(result, value) => todo!(),
        ToChar(result, value) => todo!(),
        ToNum(result, value) => todo!(),
        ToInt(result, value) => todo!(),
        ToFloat(result, value) => todo!(),
        ToSymbol(result, value) => todo!(),
        ToString(result, value) => todo!(),
        ToList(result, value) => todo!(),
        ToMap(result, value) => todo!(),
      }
    }
    Ok(())
  }
}

fn test_fn(args: &[Value]) -> Result<Value> {
  Ok(Nil)
}

const CORE_FNS: [fn(&[Value]) -> Result<Value>; 1] = [test_fn];

#[cfg(test)]
mod tests {
  use std::rc::Rc;

  use super::EvaluationState;
  use crate::{
    ConstIndex, Instruction::*, Num::*, Program, RegisterIndex, Value::*,
  };
  use minivec::mini_vec;
  use ordered_float::OrderedFloat;
  use program_macro::program;

  macro_rules! assert_register {
    ($state:expr, $register:expr, $value:expr) => {
      assert_eq!($state.get_register($register), &$value.clone().into())
    };
  }
  macro_rules! run_and_check_registers {
    ($program:expr, $(($register:expr, $value:expr)),*$(,)?) => {
      let mut state = EvaluationState::new();
      state.evaluate($program).unwrap();
      $(assert_register!(state, $register, $value);)*
    };
  }
  macro_rules! simple_register_test {
    ($name:ident, $program:expr, $(($register:expr, $value:expr)),*$(,)?) => {
      #[test]
      fn $name() {
        run_and_check_registers!($program, $(($register, $value)),*);
      }
    }
  }

  #[test]
  fn constants() {
    let constants = vec![1.into(), false.into(), "Hello!".into(), Nil];
    run_and_check_registers!(
      Program::new(
        vec![Const(0, 0), Const(1, 1), Const(2, 2), Const(3, 3)],
        constants.clone()
      ),
      (0, constants[0]),
      (1, constants[1]),
      (2, constants[2]),
      (3, constants[3])
    );
  }

  simple_register_test!(
    arithmetic,
    program![
      Const(0, 1),
      Const(1, 2.),
      Add(2, 0, 1),
      Const(3, 4),
      Multiply(4, 2, 3),
      Const(5, 12),
      Subtract(6, 4, 5),
      Const(7, -6),
      Divide(8, 4, 7),
    ],
    (2, 3.),
    (4, 12.),
    (6, 0.),
    (8, -2.)
  );

  simple_register_test!(
    environment,
    program![Const(0, 100), Bind(0, 0), Lookup(1, 0)],
    (1, 100)
  );

  simple_register_test!(clear, program![Const(0, 100), Clear(0)], (0, Nil));

  simple_register_test!(copy, program![Const(0, 100), Copy(1, 0)], (1, 100));

  #[test]
  fn apply_0_constant_function() {
    run_and_check_registers!(
      Program::new(
        vec![Const(0, 0), Apply0(1, 0)],
        vec![
          CompositeFn(Rc::new(mini_vec![Const(0, 1), Return(0)])),
          5.into()
        ],
      ),
      (1, 5)
    );
  }

  simple_register_test!(
    apply_1_square_function,
    program![
      Const(0, 10),
      Const(
        1,
        CompositeFn(Rc::new(mini_vec![
          Argument(0),
          Lookup(0, 0),
          Multiply(0, 0, 0),
          Return(0)
        ]))
      ),
      Apply1(0, 1),
    ],
    (0, 100)
  );

  #[test]
  fn apply_1_double_square_nested_function() {
    run_and_check_registers!(
      Program::new(
        vec![Const(0, 0), Const(1, 2), Apply1(0, 1)],
        vec![
          10.into(),
          CompositeFn(Rc::new(mini_vec![
            Argument(0),
            Lookup(0, 0),
            Multiply(0, 0, 0),
            Return(0)
          ])),
          CompositeFn(Rc::new(mini_vec![
            Argument(0),
            Lookup(0, 0),
            Const(1, 1),
            Apply1(0, 1),
            Apply1(0, 1),
            Return(0)
          ])),
        ],
      ),
      (0, 10000)
    );
  }

  simple_register_test!(
    apply_2_square_product_function,
    program![
      Const(0, 2),
      Const(1, 3),
      Const(
        2,
        CompositeFn(Rc::new(mini_vec![
          Argument(0),
          Argument(1),
          Lookup(0, 0),
          Lookup(1, 1),
          Multiply(0, 1, 0),
          Multiply(0, 0, 0),
          Return(0)
        ]))
      ),
      Apply2(0, 2, 1),
    ],
    (0, 36)
  );

  simple_register_test!(
    clone_into_raw_vec,
    program![EmptyRawVec(0, 1), Const(1, "Hello!"), CopyIntoRawVec(0, 1),],
    (0, RawVec(mini_vec!["Hello!".into()])),
    (1, "Hello!")
  );

  simple_register_test!(
    steal_into_raw_vec,
    program![EmptyRawVec(0, 1), Const(1, "Hello!"), StealIntoRawVec(0, 1),],
    (0, RawVec(mini_vec!["Hello!".into()])),
    (1, Nil)
  );

  simple_register_test!(
    apply_n_triple_product_function,
    program![
      Const(0, 2),
      Const(1, 3),
      Const(2, 4),
      Const(
        4,
        CompositeFn(Rc::new(mini_vec![
          Argument(0),
          Argument(1),
          Argument(2),
          Lookup(0, 0),
          Lookup(1, 1),
          Lookup(2, 2),
          Multiply(0, 1, 0),
          Multiply(0, 2, 0),
          Return(0)
        ]))
      ),
      EmptyRawVec(3, 3),
      StealIntoRawVec(3, 0),
      StealIntoRawVec(3, 1),
      StealIntoRawVec(3, 2),
      ApplyN(3, 4),
    ],
    (3, 24)
  );
}
