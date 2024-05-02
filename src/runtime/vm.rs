use std::collections::HashMap;

use crate::string_utils::pad;
use crate::{string_utils::indent_lines, Value};

use crate::runtime::instructions::Instruction;

use super::{data::Num, Error, Result};

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
    const NIL: Value = Value::Nil;
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
    /*format!(
      "frame count:              {}\n\
       current frame beginning:  {}\n\
       consumption:              {}\n\
       values:\n\
       {}",
      self.frames.len() - 1,
      self.current_stack_frame_beginning(),
      self.consumption,
      self
        .frames
        .iter()
        .map(|frame| frame.beginning)
        .chain(std::iter::once(self.consumption))
        .collect::<Vec<StackIndex>>()
        .windows(2)
        .enumerate()
        .map(|(frame_index, window)| {
          let start = window[0];
          let end = window[1];
          format!(
            "{}\n{}",
            pad(
              34,
              '-',
              format!("---------- {}: ({} - {}) ", frame_index, start, end,)
            ),
            indent_lines(
              11,
              (start..end)
                .map(|i| format!("{}: {}", i, self.get_stack(i).description()))
                .collect::<Vec<String>>()
                .join("\n")
            ),
          )
        })
        .collect::<Vec<String>>()
        .join("\n")
    )*/
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
    self.swap_stack_usize(index, Value::Nil)
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
}

fn test_fn(args: &[Value]) -> Result<Value> {
  Ok(Value::Nil)
}

const CORE_FNS: [fn(&[Value]) -> Result<Value>; 1] = [test_fn];

pub fn evaluate(
  program: Program,
  mut state: EvaluationState,
) -> Result<EvaluationState> {
  let mut instruction_stack = program.instructions.clone();
  instruction_stack.reverse();
  while let Some(instruction) = instruction_stack.pop() {
    type I = Instruction;
    match instruction {
      I::DebugPrint(id) => {
        println!(
          "{}\n\
           stack:\n{}\n\n\n\
           environment:\n{}\n\
           ----------------------------------------\n\n\n",
          pad(40, '-', format!("DEBUG {} ", id)),
          state.describe_stack(),
          indent_lines(2, state.describe_environment())
        );
      }
      I::NoOp => {
        println!(
          "Instruction::NoOp called! this probably shouldn't be happening :)"
        )
      }
      I::Clear(register) => state.set_register(register, Value::Nil),
      I::Copy(result, value) => {
        state.set_register(result, state.get_register(value).clone())
      }
      I::Const(result, const_index) => {
        state.set_register(
          result,
          program.constants[const_index as usize].clone(),
        );
      }
      I::Print(value) => {
        println!("{}", state.get_register(value).description())
      }
      I::Argument(SymbolIndex) => {
        panic!("Instruction::Argument called, this should never happen")
      }
      I::Return(value) => {
        let return_value = state.get_register(value).clone();
        let finished_stack_frame = state.frames.pop().unwrap();
        for i in finished_stack_frame.beginning..state.consumption {
          state.set_stack(i, Value::Nil);
        }
        state.consumption = finished_stack_frame.beginning;
        state.set_stack(finished_stack_frame.return_stack_index, return_value);
      }
      I::Apply0(result, f) => {
        // Applies a function of 0 arguments (a thunk)
        let f_value = state.get_register(f).clone();
        state.frames.push(StackFrame {
          beginning: state.consumption,
          return_stack_index: state.register_stack_index(result),
        });
        match f_value {
          Value::CoreFn(core_fn_index) => {
            let core_fn = CORE_FNS[core_fn_index as usize];
            todo!();
          }
          Value::CompositeFn(instructions) => {
            for instruction in instructions.into_iter().rev() {
              instruction_stack.push(instruction);
            }
          }
          Value::List(list) => todo!(),
          Value::Map(list) => todo!(),
          Value::Set(list) => todo!(),
          _ => {
            return Err(Error::CantApply);
          }
        }
      }
      I::Apply1(arg_and_result, f) => {
        // Applies a function of a single argument.
        let f_value = state.get_register(f).clone();
        let arg_value = state.steal_register(arg_and_result);
        state.frames.push(StackFrame {
          beginning: state.consumption,
          return_stack_index: state.register_stack_index(arg_and_result),
        });
        match f_value {
          Value::CoreFn(core_fn_index) => {
            let core_fn = CORE_FNS[core_fn_index as usize];
            todo!();
          }
          Value::CompositeFn(instructions) => {
            let mut remaining_instructions = instructions.into_iter();
            if let Some(I::Argument(symbol_index)) =
              remaining_instructions.next()
            {
              state.environment.insert(symbol_index, arg_value.clone());
            } else {
              panic!(
                "CompositeFn missing Argument instruction (called from Apply1)"
              )
            }
            for instruction in remaining_instructions.rev() {
              instruction_stack.push(instruction);
            }
          }
          Value::List(list) => todo!(),
          Value::Map(list) => todo!(),
          Value::Set(list) => todo!(),
          _ => {
            return Err(Error::CantApply);
          }
        }
      }
      I::Apply2(arg_1_and_result, f, arg_2) => {
        // Applies a function of 2 arguments.
        let f_value = state.get_register(f).clone();
        let arg_1_value = state.steal_register(arg_1_and_result);
        let arg_2_value = state.get_register(arg_2).clone();
        state.frames.push(StackFrame {
          beginning: state.consumption,
          return_stack_index: state.register_stack_index(arg_1_and_result),
        });
        match f_value {
          Value::CoreFn(core_fn_index) => {
            let core_fn = CORE_FNS[core_fn_index as usize];
            todo!();
          }
          Value::CompositeFn(instructions) => {
            let mut remaining_instructions = instructions.into_iter();
            if let Some(I::Argument(symbol_1_index)) =
              remaining_instructions.next()
            {
              state
                .environment
                .insert(symbol_1_index, arg_1_value.clone());
            } else {
              panic!(
                "CompositeFn missing first Argument instruction (called from\
                 Apply2)"
              )
            }
            if let Some(I::Argument(symbol_2_index)) =
              remaining_instructions.next()
            {
              state
                .environment
                .insert(symbol_2_index, arg_2_value.clone());
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
          Value::List(list) => todo!(),
          Value::Map(list) => todo!(),
          Value::Set(list) => todo!(),
          _ => {
            return Err(Error::CantApply);
          }
        }
      }
      I::ApplyN(args_and_result, f) => todo!(),
      I::Apply0AndReturn(f) => todo!(),
      I::Apply1AndReturn(f, args) => {
        // This instruction is for supporting tail-call elimination. It takes a
        // function and its arguments just like `Apply`, but before invoking
        // the function it cleans up the current stack frame, so tail-call
        // recursive functions don't consume more space than necessary on the
        // stack. Any time a `Apply` instruction would be immediately followed
        // by a `Return` instruction, it should be replaced with this (maybe
        // that can actually just be done in an optimization pass?)
        todo!()
      }
      I::Apply2AndReturn(arg_1_and_result, f, arg_2) => todo!(),
      I::ApplyNAndReturn(args_and_result, f) => todo!(),
      I::Lookup(register, symbol_index) => {
        state.set_register(register, state.environment[&symbol_index].clone());
      }
      I::Bind(symbol_index, register) => {
        state
          .environment
          .insert(symbol_index, state.get_register(register).clone());
      }
      I::When(result, condition, thunk) => todo!(),
      I::If(condition_and_result, thunk_1, thunk_2) => todo!(),
      I::Partial(result, f, arg) => todo!(),
      I::Compose(result, f_1, f_2) => todo!(),
      I::Filter(result, f, collection) => todo!(),
      I::Map(result, f, collection) => todo!(),
      I::MultiListMap(result, f, collections) => todo!(),
      I::Some(result, f, collection) => todo!(),
      I::ReduceWithoutInitialValue(result, f, collection) => todo!(),
      I::ReduceWithInitialValue(initial_value_and_result, f, collection) => {
        todo!()
      }
      I::Memoize(result, f) => todo!(),
      I::Constantly(result, value) => todo!(),
      I::NumericalEqual(result, num_1, num_2) => state.set_register(
        result,
        match (state.get_register(num_1), state.get_register(num_2)) {
          (Value::Num(a), Value::Num(b)) => a.numerical_equal(b),
          _ => return Err(Error::ArgumentNotNum),
        },
      ),
      I::IsZero(result, num) => state.set_register(
        result,
        match state.get_register(num) {
          Value::Num(Num::Float(f)) => *f == 0.,
          Value::Num(Num::Int(i)) => *i == 0,
          _ => return Err(Error::ArgumentNotNum),
        },
      ),
      I::IsNan(result, num) => state.set_register(
        result,
        match state.get_register(num) {
          Value::Num(Num::Float(f)) => f.is_nan(),
          _ => return Err(Error::ArgumentNotNum),
        },
      ),
      I::IsInf(result, num) => state.set_register(
        result,
        match state.get_register(num) {
          Value::Num(Num::Float(f)) => f.is_infinite(),
          _ => return Err(Error::ArgumentNotNum),
        },
      ),
      I::IsEven(result, num) => state.set_register(
        result,
        match state.get_register(num) {
          Value::Num(n) => n.as_int_lossless()? % 2 == 0,
          _ => return Err(Error::ArgumentNotNum),
        },
      ),
      I::IsOdd(result, num) => state.set_register(
        result,
        match state.get_register(num) {
          Value::Num(n) => n.as_int_lossless()? % 2 == 1,
          _ => return Err(Error::ArgumentNotNum),
        },
      ),
      I::IsPos(result, num) => state.set_register(
        result,
        match state.get_register(num) {
          Value::Num(Num::Float(f)) => **f > 0.,
          Value::Num(Num::Int(i)) => *i > 0,
          _ => return Err(Error::ArgumentNotNum),
        },
      ),
      I::IsNeg(result, num) => state.set_register(
        result,
        match state.get_register(num) {
          Value::Num(Num::Float(f)) => **f < 0.,
          Value::Num(Num::Int(i)) => *i < 0,
          _ => return Err(Error::ArgumentNotNum),
        },
      ),
      I::Inc(result, num) => {
        state.set_register(result, state.get_register(num).as_num()?.inc())
      }
      I::Dec(result, num) => {
        state.set_register(result, state.get_register(num).as_num()?.dec())
      }
      I::Negate(result, num) => {
        state.set_register(result, -*state.get_register(num).as_num()?)
      }
      I::Abs(result, num) => {
        state.set_register(result, state.get_register(num).as_num()?.abs())
      }
      I::Floor(result, num) => {
        state.set_register(result, state.get_register(num).as_num()?.floor())
      }
      I::Ceil(result, num) => {
        state.set_register(result, state.get_register(num).as_num()?.ceil())
      }
      I::Sqrt(result, num) => state.set_register(
        result,
        state.get_register(num).as_num()?.as_float().sqrt(),
      ),
      I::Exp(result, num) => state.set_register(
        result,
        state.get_register(num).as_num()?.as_float().exp(),
      ),
      I::Exp2(result, num) => state.set_register(
        result,
        state.get_register(num).as_num()?.as_float().exp2(),
      ),
      I::Ln(result, num) => state.set_register(
        result,
        state.get_register(num).as_num()?.as_float().ln(),
      ),
      I::Log2(result, num) => state.set_register(
        result,
        state.get_register(num).as_num()?.as_float().log2(),
      ),
      I::Add(result, num_1, num_2) => state.set_register(
        result,
        *state.get_register(num_1).as_num()?
          + *state.get_register(num_2).as_num()?,
      ),
      I::Subtract(result, num_1, num_2) => state.set_register(
        result,
        *state.get_register(num_1).as_num()?
          - *state.get_register(num_2).as_num()?,
      ),
      I::Multiply(result, num_1, num_2) => {
        state.set_register(
          result,
          *state.get_register(num_1).as_num()?
            * *state.get_register(num_2).as_num()?,
        );
      }
      I::Divide(result, num_1, num_2) => state.set_register(
        result,
        *state.get_register(num_1).as_num()?
          / *state.get_register(num_2).as_num()?,
      ),
      I::Pow(result, num_1, num_2) => todo!(),
      I::Mod(result, num_1, num_2) => todo!(),
      I::Quot(result, num_1, num_2) => todo!(),
      I::Min(result, num_1, num_2) => todo!(),
      I::Max(result, num_1, num_2) => todo!(),
      I::GreaterThan(result, num_1, num_2) => todo!(),
      I::GreaterThanOrEqual(result, num_1, num_2) => todo!(),
      I::LessThan(result, num_1, num_2) => todo!(),
      I::LessThanOrEqual(result, num_1, num_2) => todo!(),
      I::Rand(result) => todo!(),
      I::UpperBoundedRand(result, upper_bound) => todo!(),
      I::LowerUpperBoundedRand(result, lower_bound, upper_bound) => todo!(),
      I::RandInt(result, upper_bound) => todo!(),
      I::LowerBoundedRandInt(result, lower_bound, upper_bound) => todo!(),
      I::Equal(result, value_1, value_2) => todo!(),
      I::NotEqual(result, value_1, value_2) => todo!(),
      I::Not(result, value) => todo!(),
      I::And(result, bool_1, bool_2) => todo!(),
      I::Or(result, bool_1, bool_2) => todo!(),
      I::Xor(result, bool_1, bool_2) => todo!(),
      I::IsEmpty(result, collection) => todo!(),
      I::Count(result, collection) => todo!(),
      I::Flatten(result, collection) => todo!(),
      I::Remove(result, collection, key) => todo!(),
      I::Set(value_and_result, collection, key) => todo!(),
      I::SetIn(value_and_result, collection, path) => todo!(),
      I::Get(result, collection, key) => todo!(),
      I::GetIn(result, collection, path) => todo!(),
      I::Update(f_and_result, collection, key) => todo!(),
      I::UpdateIn(f_and_result, collection, path) => todo!(),
      I::MinKey(result, collection, f) => todo!(),
      I::MaxKey(result, collection, f) => todo!(),
      I::First(result, collection) => todo!(),
      I::Sort(result, collection) => todo!(),
      I::SortBy(result, collection, f) => todo!(),
      I::EmptyList(result) => todo!(),
      I::Last(result, list) => todo!(),
      I::Nth(result, list, n) => {
        // While `Get` returns nil for a list when index is OOB, `Nth` throws
        todo!()
      }
      I::NthFromLast(result, list, n) => todo!(),
      I::Cons(result, list, value) => todo!(),
      I::Push(result, list, value) => todo!(),
      I::Concat(result, list_1, list_2) => todo!(),
      I::Take(result, list, n) => todo!(),
      I::Drop(result, list, n) => todo!(),
      I::Reverse(result, list) => todo!(),
      I::Distinct(result, list) => todo!(),
      I::Sub(start_index_and_result, list, end_index) => todo!(),
      I::Partition(result, list, size) => todo!(),
      I::SteppedPartition(step_and_return, list, size) => todo!(),
      I::Pad(value_and_result, list, size) => todo!(),
      I::EmptyMap(result) => todo!(),
      I::Keys(result, map) => todo!(),
      I::Values(result, map) => todo!(),
      I::Zip(result, key_list, value_list) => todo!(),
      I::Invert(result, map) => todo!(),
      I::Merge(result, map_1, map_2) => todo!(),
      I::MergeWith(merge_f_and_result, map_1, map_2) => todo!(),
      I::MapKeys(result, f, map) => todo!(),
      I::MapValues(result, f, map) => todo!(),
      I::EmptySet(result) => todo!(),
      I::Union(result, set_1, set_2) => todo!(),
      I::Intersection(result, set_1, set_2) => todo!(),
      I::Difference(result, set_1, set_2) => todo!(),
      I::SymmetricDifference(result, set_1, set_2) => todo!(),
      I::InfiniteRange(result) => todo!(),
      I::UpperBoundedRange(result, size) => todo!(),
      I::LowerUpperBoundedRange(result, lower_bound, upper_bound) => todo!(),
      I::InfiniteRepeat(result, value) => todo!(),
      I::BoundedRepeat(result, value, count) => todo!(),
      I::InfiniteRepeatedly(result, f) => todo!(),
      I::BoundedRepeatedly(result, f, count) => todo!(),
      I::InfiniteIterate(result, f, initial_value) => todo!(),
      I::BoundedIterate(bound_and_result, f, initial_value) => todo!(),
      I::IsNil(result, value) => todo!(),
      I::IsBool(result, value) => todo!(),
      I::IsChar(result, value) => todo!(),
      I::IsNum(result, value) => todo!(),
      I::IsInt(result, value) => todo!(),
      I::IsFloat(result, value) => todo!(),
      I::IsSymbol(result, value) => todo!(),
      I::IsString(result, value) => todo!(),
      I::IsList(result, value) => todo!(),
      I::IsMap(result, value) => todo!(),
      I::IsSet(result, value) => todo!(),
      I::IsCollection(result, value) => todo!(),
      I::IsFn(result, value) => todo!(),
      I::ToBool(result, value) => todo!(),
      I::ToChar(result, value) => todo!(),
      I::ToNum(result, value) => todo!(),
      I::ToInt(result, value) => todo!(),
      I::ToFloat(result, value) => todo!(),
      I::ToSymbol(result, value) => todo!(),
      I::ToString(result, value) => todo!(),
      I::ToList(result, value) => todo!(),
      I::ToMap(result, value) => todo!(),
    }
  }
  Ok(state)
}

#[cfg(test)]
mod tests {
  use std::rc::Rc;

  use super::EvaluationState;
  use crate::{
    evaluate, ConstIndex, Instruction::*, Num::*, Program, RegisterIndex,
    Value::*,
  };
  use minivec::mini_vec;
  use ordered_float::OrderedFloat;
  use program_macro::program;

  macro_rules! run {
    ($program:expr) => {
      evaluate($program, EvaluationState::new()).unwrap()
    };
  }
  macro_rules! assert_register {
    ($state:expr, $register:expr, $value:expr) => {
      assert_eq!($state.get_register($register), &$value.clone().into())
    };
  }
  macro_rules! run_and_check_registers {
    ($program:expr, $(($register:expr, $value:expr)),*$(,)?) => {
      let final_state = run!($program);
      $(assert_register!(final_state, $register, $value);)*
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
  fn apply0_constant_function() {
    run_and_check_registers!(
      Program::new(
        vec![Const(0, 0), Apply0(1, 0)],
        vec![CompositeFn(mini_vec![Const(0, 1), Return(0)]), 5.into()],
      ),
      (1, 5)
    );
  }

  simple_register_test!(
    apply1_square_function,
    program![
      Const(0, 10),
      Const(
        1,
        CompositeFn(mini_vec![
          Argument(0),
          Lookup(0, 0),
          Multiply(0, 0, 0),
          Return(0)
        ])
      ),
      Apply1(0, 1),
    ],
    (0, 100)
  );

  #[test]
  fn apply1_double_square_nested_function() {
    run_and_check_registers!(
      Program::new(
        vec![Const(0, 0), Const(1, 2), Apply1(0, 1)],
        vec![
          10.into(),
          CompositeFn(mini_vec![
            Argument(0),
            Lookup(0, 0),
            Multiply(0, 0, 0),
            Return(0)
          ]),
          CompositeFn(mini_vec![
            Argument(0),
            Lookup(0, 0),
            Const(1, 1),
            Apply1(0, 1),
            Apply1(0, 1),
            Return(0)
          ]),
        ],
      ),
      (0, 10000)
    );
  }

  simple_register_test!(
    apply2_square_product_function,
    program![
      Const(0, 2),
      Const(1, 3),
      Const(
        2,
        CompositeFn(mini_vec![
          Argument(0),
          Argument(1),
          Lookup(0, 0),
          Lookup(1, 1),
          Multiply(0, 1, 0),
          Multiply(0, 0, 0),
          Return(0)
        ])
      ),
      Apply2(0, 2, 1),
    ],
    (0, 36)
  );
}
