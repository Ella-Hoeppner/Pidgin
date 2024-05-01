use std::collections::HashMap;

use crate::Value;

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
  stack_start: StackIndex,
  result_register: RegisterIndex,
}

pub struct EvaluationState {
  stack: [Value; STACK_CAPACITY],
  stack_frames: Vec<StackFrame>,
  stack_consumption: StackIndex,
  environment: HashMap<SymbolIndex, Value>,
}

impl EvaluationState {
  pub fn new() -> Self {
    const NIL: Value = Value::Nil;
    Self {
      stack: [NIL; STACK_CAPACITY],
      stack_frames: vec![],
      stack_consumption: 0,
      environment: HashMap::new(),
    }
  }
  fn display_stack(&self) -> String {
    self
      .stack
      .iter()
      .take(self.stack_consumption as usize)
      .enumerate()
      .map(|(i, value)| format!("{}: {}", i, value.description()))
      .reduce(|a, b| a + "\n" + &b)
      .unwrap_or("".to_string())
  }
  fn display_environment(&self) -> String {
    let mut bindings: Vec<_> = self.environment.iter().collect();
    bindings.sort_by_key(|(symbol_index, _value_pointer)| **symbol_index);
    bindings
      .into_iter()
      .map(|(symbol_index, value)| {
        format!(
          "symbol_index: {}, value: {}",
          symbol_index,
          value.description()
        )
      })
      .reduce(|a, b| a + "\n" + &b)
      .unwrap_or("".to_string())
  }
  fn get_stack_value(&self, index: usize) -> &Value {
    &self.stack[index]
  }
  fn stack_index(&self, register: RegisterIndex) -> StackIndex {
    *self
      .stack_frames
      .last()
      .map(|stack_frame| &stack_frame.stack_start)
      .unwrap_or(&0)
      + register as StackIndex
  }
  fn set_register<T: Into<Value>>(
    &mut self,
    register: RegisterIndex,
    value: T,
  ) {
    let stack_index = self.stack_index(register);
    self.stack[stack_index as usize] = value.into();
    self.stack_consumption = self.stack_consumption.max(stack_index + 1);
  }
  fn get_register(&self, register: RegisterIndex) -> &Value {
    //debug
    if register as usize >= self.stack_consumption as usize {
      panic!("trying to access register that hasn't been set yet")
    }
    //
    self.get_stack_value(self.stack_index(register) as usize)
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
        println!("DEBUG {}", id);
        println!("--------------------");
        println!(
          "stack ({}):\n{}\n",
          state.stack_consumption,
          state.display_stack()
        );
        println!("environment:\n{}", state.display_environment());
        println!("--------------------\n");
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
      I::Print(value) => todo!(),
      I::Argument(SymbolIndex) => {
        panic!("Instruction::Argument called, this should never happen")
      }
      I::Return(return_value_stack_index) => {
        let return_value = state
          .get_stack_value(return_value_stack_index as usize)
          .clone();
        let stack_frame = state.stack_frames.pop().unwrap();
        for i in stack_frame.stack_start..state.stack_consumption {
          state.stack[i as usize] = Value::Nil;
        }
        state.stack_consumption = stack_frame.stack_start;
        state.set_register(stack_frame.result_register, return_value);
      }
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
      I::Apply(result, f, args) => {
        let f_value = state.get_register(f).clone();
        let arg_value = state.get_register(args).clone();
        state.stack_frames.push(StackFrame {
          stack_start: state.stack_consumption,
          result_register: result,
        });
        match f_value {
          Value::CoreFn(core_fn_index) => {
            let core_fn = CORE_FNS[core_fn_index as usize];
          }
          Value::CompositeFn(instructions) => {
            let mut x = instructions.into_iter().peekable();
            while let Some(I::Argument(symbol_index)) = x.peek() {
              state.environment.insert(*symbol_index, arg_value.clone());
              x.next();
            }
            for instruction in x.rev() {
              instruction_stack.push(instruction);
            }
          }
          _ => {
            return Err(Error::CantApply);
          }
        }
      }
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
      I::NumericalEqual(result, num_1, num_2) => todo!(),
      I::IsZero(result, num) => todo!(),
      I::IsNan(result, num) => todo!(),
      I::IsInf(result, num) => todo!(),
      I::IsEven(result, num) => todo!(),
      I::IsPos(result, num) => todo!(),
      I::IsNeg(result, num) => todo!(),
      I::Inc(result, num) => todo!(),
      I::Dec(result, num) => todo!(),
      I::Negate(result, num) => {
        state.set_register(result, -*state.get_register(num).as_num()?)
      }
      I::Abs(result, num) => todo!(),
      I::Floor(result, num) => todo!(),
      I::Ceil(result, num) => todo!(),
      I::Sqrt(result, num) => todo!(),
      I::Exp(result, num) => todo!(),
      I::Exp2(result, num) => todo!(),
      I::Ln(result, num) => todo!(),
      I::Log2(result, num) => todo!(),
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
}
