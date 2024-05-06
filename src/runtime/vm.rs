use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

use minivec::{mini_vec, MiniVec};

use crate::runtime::core_functions::CORE_FUNCTIONS;
use crate::string_utils::pad;
use crate::{string_utils::indent_lines, Instruction, Num, Value};
use crate::{ProcessState, StackFrame};
use Instruction::*;
use Num::*;
use Value::*;

use take_mut::take;

use super::control::{
  CompositeFunction, PausedProcess, RuntimeInstructionBlock,
};
use super::error::{PidginError, PidginResult};

pub type RegisterIndex = u8;
pub type StackIndex = u16;
pub type SymbolIndex = u16;
pub type ConstIndex = u16;
pub type CoreFnIndex = u8;
pub type RuntimeInstruction = Instruction<RegisterIndex, ConstIndex>;

#[derive(Debug)]
pub struct Program {
  instructions: RuntimeInstructionBlock,
  constants: Vec<Value>,
}
impl Program {
  pub fn new<T: Into<RuntimeInstructionBlock>>(
    instructions: T,
    constants: Vec<Value>,
  ) -> Self {
    Self {
      instructions: instructions.into(),
      constants,
    }
  }
}

pub struct EvaluationState {
  constants: Vec<Value>,
  current_frame: StackFrame,
  current_process: ProcessState,
  process_stack: Vec<PausedProcess>,
  environment: HashMap<SymbolIndex, Value>,
}

impl EvaluationState {
  pub fn new(program: Program) -> Self {
    const NIL: Value = Nil;
    Self {
      constants: program.constants,
      current_frame: StackFrame::root(program.instructions),
      current_process: ProcessState::new(),
      process_stack: vec![],
      environment: HashMap::new(),
    }
  }
  fn describe_stack(&self) -> String {
    self
      .current_process
      .paused_frames
      .iter()
      .chain(std::iter::once(&self.current_frame))
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
          .unwrap_or(self.current_process.consumption);
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
  fn bind_symbol<T: Into<Value>>(
    &mut self,
    symbol_index: StackIndex,
    value: T,
  ) {
    self.environment.insert(symbol_index, value.into());
  }
  fn push_frame(&mut self, mut frame: StackFrame) {
    std::mem::swap(&mut self.current_frame, &mut frame);
    self.current_process.paused_frames.push(frame);
  }
  fn complete_process(&mut self) -> StackFrame {
    let mut next_process = self
      .process_stack
      .pop()
      .expect("attempted to complete_process with no paused processes");
    let (frame, process_state) = next_process.resume();
    self.current_process = process_state;
    frame
  }
  fn complete_frame(&mut self) -> StackFrame {
    let mut next_frame = self
      .current_process
      .paused_frames
      .pop()
      .unwrap_or_else(|| self.complete_process());
    std::mem::swap(&mut self.current_frame, &mut next_frame);
    next_frame
  }
  fn set_stack_usize(&mut self, index: usize, value: Value) {
    self.current_process.stack[index] = value;
    self.current_process.consumption =
      self.current_process.consumption.max(index as u16 + 1);
  }
  fn set_stack(&mut self, index: StackIndex, value: Value) {
    self.set_stack_usize(index as usize, value);
  }
  fn swap_stack_usize(&mut self, index: usize, value: Value) -> Value {
    std::mem::replace(&mut self.current_process.stack[index], value)
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
    &self.current_process.stack[index]
  }
  fn get_stack(&self, index: StackIndex) -> &Value {
    self.get_stack_usize(index as usize)
  }
  fn get_stack_mut_usize(&mut self, index: usize) -> &mut Value {
    &mut self.current_process.stack[index]
  }
  fn get_stack_mut(&mut self, index: StackIndex) -> &mut Value {
    self.get_stack_mut_usize(index as usize)
  }
  fn register_stack_index(&self, register: RegisterIndex) -> StackIndex {
    self.current_frame.beginning + register as StackIndex
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
    #[cfg(debug_assertions)]
    if register as usize >= self.current_process.consumption as usize {
      panic!("trying to access register that hasn't been set yet")
    }
    self.get_stack(self.register_stack_index(register))
  }
  fn get_register_mut(&mut self, register: RegisterIndex) -> &mut Value {
    #[cfg(debug_assertions)]
    if register as usize >= self.current_process.consumption as usize {
      panic!("trying to access register that hasn't been set yet")
    }
    self.get_stack_mut(self.register_stack_index(register))
  }
  fn create_fn_stack_frame(
    &mut self,
    f: Rc<CompositeFunction>,
    return_stack_index: StackIndex,
  ) -> StackFrame {
    StackFrame::for_fn(f, self.current_process.consumption, return_stack_index)
  }
  fn start_fn_stack_frame(
    &mut self,
    f: Rc<CompositeFunction>,
    return_stack_index: StackIndex,
  ) {
    let frame = self.create_fn_stack_frame(f, return_stack_index);
    self.push_frame(frame);
  }
  fn next_instruction(&mut self) -> RuntimeInstruction {
    self.current_frame.next_instruction()
  }
  fn skip_to_endif(&mut self) {
    loop {
      if self.next_instruction() == EndIf {
        break;
      }
    }
  }
  fn push_process(&mut self, process: PausedProcess, return_index: StackIndex) {
    let (mut active_frame, process_state) = process.begin(return_index);
    std::mem::swap(&mut self.current_frame, &mut active_frame);
    take(&mut self.current_process, |old_process| {
      let paused_old_process = old_process.pause(active_frame);
      self.process_stack.push(paused_old_process);
      process_state
    });
  }
  fn move_args_from(
    &mut self,
    arg_count: u8,
    beginning_stack_index: StackIndex,
    frame: &mut StackFrame,
  ) {
    for i in 0..arg_count {
      match frame.next_instruction() {
        CopyArgument(arg_register) => {
          let value = self
            .get_stack(frame.beginning + arg_register as StackIndex)
            .clone();
          self.set_stack(beginning_stack_index + i as StackIndex, value)
        }
        StealArgument(arg_register) => {
          let stolen_value = self
            .steal_stack(frame.beginning + arg_register as StackIndex)
            .clone();
          self.set_stack(beginning_stack_index + i as StackIndex, stolen_value)
        }
        other => panic!(
          "Expected CopyArgument or StealArgument instruction {}/{},
           found {:?}",
          i + 1,
          arg_count,
          other
        ),
      }
    }
  }
  fn move_args(&mut self, arg_count: u8, beginning_stack_index: StackIndex) {
    for i in 0..arg_count {
      match self.next_instruction() {
        CopyArgument(arg_register) => {
          let value = self.get_register(arg_register).clone();
          self.set_stack(beginning_stack_index + i as StackIndex, value)
        }
        StealArgument(arg_register) => {
          let stolen_value = self.steal_register(arg_register).clone();
          self.set_stack(beginning_stack_index + i as StackIndex, stolen_value)
        }
        other => panic!(
          "Expected CopyArgument or StealArgument instruction {}/{},
           found {:?}",
          i + 1,
          arg_count,
          other
        ),
      }
    }
  }
  fn take_args(&mut self, arg_count: u8) -> Vec<Value> {
    (0..arg_count)
      .map(|i| match self.next_instruction() {
        CopyArgument(arg_register) => self.get_register(arg_register).clone(),
        StealArgument(arg_register) => {
          self.steal_register(arg_register).clone()
        }
        other => panic!(
          "Expected CopyArgument or StealArgument instruction {}/{},
           found {:?}",
          i + 1,
          arg_count,
          other
        ),
      })
      .collect()
  }
  pub fn evaluate(&mut self) -> PidginResult<()> {
    loop {
      if self.current_frame.instruction_index
        >= self.current_frame.instructions.len()
      {
        break;
      }
      let instruction = self.next_instruction();
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
          self
            .set_register(result, self.constants[const_index as usize].clone());
        }
        Print(value) => {
          println!("{}", self.get_register(value).description())
        }
        Return(value) => {
          let return_value = self.get_register(value).clone();
          println!(
            "Return called!\nprocess_stack count: {}",
            self.process_stack.len()
          );
          let completed_frame = self.complete_frame();
          self.current_process.consumption = completed_frame.beginning;
          println!("return setting {}", completed_frame.return_stack_index);
          self.set_stack(completed_frame.return_stack_index, return_value);
        }
        CopyArgument(f) => {
          panic!("CopyArgument instruction called, this should never happen")
        }
        StealArgument(f) => {
          panic!("CopyArgument instruction called, this should never happen")
        }
        Call(target, f, arg_count) => {
          let f_value = self.get_register(f).clone();
          match f_value {
            CompositeFn(composite_fn) => {
              let new_frame = self.create_fn_stack_frame(
                composite_fn,
                self.register_stack_index(target),
              );
              self.move_args(arg_count, new_frame.beginning);
              self.push_frame(new_frame);
            }
            CoreFn(_) => {
              panic!(
                "Call instruction called with CoreFn value, this should \
                 never happen"
              )
            }
            ExternalFn(external_fn) => {
              let f = (*external_fn).f;
              let args = self.take_args(arg_count);
              self.set_register(
                target,
                f(args).expect(
                  "external_fn returned an error, and we don't have error \
                   handling yet :(",
                ),
              );
            }
            Process(maybe_process) => {
              if let Some(process_ref) = &*maybe_process {
                if let Some(process) = process_ref.replace(None) {
                  self.push_process(process, self.register_stack_index(target));
                } else {
                  return Err(PidginError::ProcessAlreadyRunning);
                }
              } else {
                return Err(PidginError::DeadProcess);
              }
            }
            List(list) => todo!(),
            Hashmap(map) => todo!(),
            Hashset(set) => todo!(),
            _ => {
              return Err(PidginError::CantApply);
            }
          }
        }
        Apply(args_and_result, f) => {
          let f_value = self.get_register(f).clone();
          if let List(arg_list) = self.steal_register(args_and_result) {
            match f_value {
              CompositeFn(composite_fn) => {
                self.start_fn_stack_frame(
                  composite_fn.clone(),
                  self.register_stack_index(args_and_result),
                );
                let provided_arg_count = arg_list.len();
                #[cfg(debug_assertions)]
                if composite_fn.args.can_accept(provided_arg_count) {
                  panic!(
                    "Apply called on CompositeFn that expects {} arguments, \
                     {} arguments provided",
                    composite_fn.args, provided_arg_count
                  )
                }
                let x = Rc::unwrap_or_clone(arg_list);
                for (i, arg_value) in x.into_iter().enumerate() {
                  self.set_register(i as RegisterIndex, arg_value);
                }
              }
              CoreFn(core_fn_id) => {
                self.set_register(
                  args_and_result,
                  CORE_FUNCTIONS[core_fn_id](Rc::unwrap_or_clone(arg_list))?,
                );
              }
              ExternalFn(external_fn) => {
                let f = (*external_fn).f;
                self.set_register(
                  args_and_result,
                  f(Rc::unwrap_or_clone(arg_list)).expect(
                    "external_fn returned an error, and we don't have error \
                     handling yet :(",
                  ),
                );
              }
              List(list) => todo!(),
              Hashmap(map) => todo!(),
              Hashset(set) => todo!(),
              _ => {
                return Err(PidginError::CantApply);
              }
            }
          } else {
            panic!("Apply called with non-List value");
          }
        }
        CallSelf(target, arg_count) => {
          let new_frame = self.create_fn_stack_frame(
            self.current_frame.calling_function.clone().unwrap(),
            self.register_stack_index(target),
          );
          self.move_args(arg_count, new_frame.beginning);
          self.push_frame(new_frame);
        }
        ApplySelf(args_and_result) => {
          if let List(arg_list) = self.steal_register(args_and_result) {
            let composite_fn =
              self.current_frame.calling_function.clone().unwrap();
            self.start_fn_stack_frame(
              composite_fn.clone(),
              self.register_stack_index(args_and_result),
            );
            let provided_arg_count = arg_list.len();
            #[cfg(debug_assertions)]
            if composite_fn.args.can_accept(provided_arg_count) {
              panic!(
                "ApplySelf called on CompositeFn that expects {} arguments, \
                 {} arguments provided",
                composite_fn.args, provided_arg_count
              )
            }
            let x = Rc::unwrap_or_clone(arg_list);
            for (i, arg_value) in x.into_iter().enumerate() {
              self.set_register(i as RegisterIndex, arg_value);
            }
          } else {
            panic!("Apply called with non-List value");
          }
        }
        CallAndReturn(f, arg_count) => {
          let f_value = self.get_register(f).clone();
          let mut completed_frame = self.complete_frame();
          match f_value {
            CompositeFn(composite_fn) => {
              let new_frame = StackFrame::for_fn(
                composite_fn,
                completed_frame.beginning,
                completed_frame.return_stack_index,
              );
              self.move_args_from(
                arg_count,
                new_frame.beginning,
                &mut completed_frame,
              );
              self.current_process.consumption =
                completed_frame.beginning + arg_count as StackIndex;
              self.push_frame(new_frame);
            }
            ExternalFn(_) => todo!(),
            CoreFn(_) => {
              panic!(
                "CallAndReturn instruction called with CoreFn value, this \
                 should never happen"
              )
            }
            List(list) => todo!(),
            Hashmap(map) => todo!(),
            Hashset(set) => todo!(),
            other => {
              return Err(PidginError::CantApply);
            }
          }
        }
        ApplyAndReturn(args, f) => {
          todo!()
        }
        CallSelfAndReturn(arg_count) => {
          let composite_fn =
            self.current_frame.calling_function.clone().unwrap();
          let mut completed_frame = self.complete_frame();
          let new_frame = StackFrame::for_fn(
            composite_fn,
            completed_frame.beginning,
            completed_frame.return_stack_index,
          );
          self.move_args_from(
            arg_count,
            new_frame.beginning,
            &mut completed_frame,
          );
          self.current_process.consumption =
            completed_frame.beginning + arg_count as StackIndex;
          self.push_frame(new_frame);
        }
        ApplySelfAndReturn(args) => todo!(),
        Lookup(register, symbol_index) => {
          self.set_register(register, self.environment[&symbol_index].clone());
        }
        If(condition) => {
          if !self.get_register(condition).as_bool() {
            // skip to next Else, ElseIf, or EndIf instruction
            loop {
              match self.next_instruction() {
                Else => break,
                ElseIf(other_condition) => {
                  if self.get_register(other_condition).as_bool() {
                    break;
                  }
                }
                EndIf => break,
                _ => {}
              }
            }
          }
        }
        CallingFunction(result) => {
          // This instruction puts a reference to the current calling function
          // in a register, which is necessary to support recursion.
          if let Some(calling_function) =
            self.current_frame.calling_function.clone()
          {
            self.set_register(result, CompositeFn(calling_function));
          } else {
            panic!(
              "CallingFunction invoked with no calling_function in StackFrame"
            )
          }
        }
        Jump(instruction_index) => {
          self.current_frame.instruction_index = instruction_index as usize;
        }
        Else => self.skip_to_endif(),
        ElseIf(condition) => self.skip_to_endif(),
        EndIf => {}
        Partial(result, f, arg) => todo!(),
        Compose(result, f_1, f_2) => todo!(),
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
            _ => return Err(PidginError::ArgumentNotNum),
          },
        ),
        IsZero(result, num) => self.set_register(
          result,
          match self.get_register(num) {
            Number(Float(f)) => *f == 0.,
            Number(Int(i)) => *i == 0,
            _ => return Err(PidginError::ArgumentNotNum),
          },
        ),
        IsNan(result, num) => self.set_register(
          result,
          match self.get_register(num) {
            Number(Float(f)) => f.is_nan(),
            _ => return Err(PidginError::ArgumentNotNum),
          },
        ),
        IsInf(result, num) => self.set_register(
          result,
          match self.get_register(num) {
            Number(Float(f)) => f.is_infinite(),
            _ => return Err(PidginError::ArgumentNotNum),
          },
        ),
        IsEven(result, num) => self.set_register(
          result,
          match self.get_register(num) {
            Number(n) => n.as_int_lossless()? % 2 == 0,
            _ => return Err(PidginError::ArgumentNotNum),
          },
        ),
        IsOdd(result, num) => self.set_register(
          result,
          match self.get_register(num) {
            Number(n) => n.as_int_lossless()? % 2 == 1,
            _ => return Err(PidginError::ArgumentNotNum),
          },
        ),
        IsPos(result, num) => self.set_register(
          result,
          match self.get_register(num) {
            Number(Float(f)) => **f > 0.,
            Number(Int(i)) => *i > 0,
            _ => return Err(PidginError::ArgumentNotNum),
          },
        ),
        IsNeg(result, num) => self.set_register(
          result,
          match self.get_register(num) {
            Number(Float(f)) => **f < 0.,
            Number(Int(i)) => *i < 0,
            _ => return Err(PidginError::ArgumentNotNum),
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
        First(result, collection) => self.set_register(
          result,
          match self.get_register(collection) {
            List(list) => list.first().cloned().unwrap_or(Nil),
            Hashset(set) => set.iter().next().cloned().unwrap_or(Nil),
            Hashmap(hashmap) => hashmap
              .iter()
              .next()
              .map(|(key, value)| vec![key.clone(), value.clone()].into())
              .unwrap_or(Nil),
            Nil => Nil,
            _ => return Err(PidginError::ArgumentNotList),
          },
        ),
        Count(result, collection) => todo!(),
        Flatten(result, collection) => todo!(),
        Remove(collection, key) => todo!(),
        Filter(collection, f) => todo!(),
        Map(collection, f) => todo!(),
        DoubleMap(collection, other_collection, f) => {
          // Special case of multi-collection map with just 2 collections.
          // This special case comes up often enough (e.g. mapping with
          // `(range)` as a second argument for indexing) that the optimization
          // from having this instruction seems worthwhile
          todo!()
        }
        MultiCollectionMap(raw_vec_of_collections, f) => todo!(),
        Set(collection, value, key) => todo!(),
        SetIn(collection, value, path) => todo!(),
        Get(result, collection, key) => todo!(),
        GetIn(result, collection, path) => todo!(),
        Update(collection, f, key) => todo!(),
        UpdateIn(collection, f, path) => todo!(),
        MinKey(result, collection, f) => todo!(),
        MaxKey(result, collection, f) => todo!(),
        Push(collection, value) => {
          let value = self.get_register(value).clone();
          let collection_value = self.steal_register(collection);
          match collection_value {
            List(mut list_value) => {
              self.set_register(
                collection,
                if let Some(owned_list_value) = Rc::get_mut(&mut list_value) {
                  owned_list_value.push(value);
                  list_value
                } else {
                  let mut list_value_clone = (*list_value).clone();
                  list_value_clone.push(value);
                  Rc::new(list_value_clone)
                },
              );
            }
            Hashmap(hashmap) => todo!(),
            Hashset(set) => todo!(),
            Nil => self.set_register(collection, Nil),
            _ => return Err(PidginError::ArgumentNotList),
          };
        }
        Sort(collection) => todo!(),
        SortBy(collection, f) => todo!(),
        EmptyList(result) => {
          self.set_register(result, Vec::new());
        }
        Last(result, list) => self.set_register(
          result,
          match self.get_register(list) {
            List(list) => list.last().cloned().unwrap_or(Nil),
            Nil => Nil,
            _ => return Err(PidginError::ArgumentNotList),
          },
        ),
        Rest(list) => {
          let list_value = self.steal_register(list);
          match list_value {
            List(mut list_value) => {
              self.set_register(
                list,
                if let Some(owned_list_value) = Rc::get_mut(&mut list_value) {
                  owned_list_value.remove(0);
                  list_value
                } else {
                  let mut list_value_clone = (*list_value).clone();
                  list_value_clone.remove(0);
                  Rc::new(list_value_clone)
                },
              );
            }
            Nil => self.set_register(list, Nil),
            _ => return Err(PidginError::ArgumentNotList),
          };
        }
        ButLast(list) => {
          let list_value = self.steal_register(list);
          match list_value {
            List(mut list_value) => {
              self.set_register(
                list,
                if let Some(owned_list_value) = Rc::get_mut(&mut list_value) {
                  owned_list_value.pop();
                  list_value
                } else {
                  let mut list_value_clone = (*list_value).clone();
                  list_value_clone.pop();
                  Rc::new(list_value_clone)
                },
              );
            }
            Nil => self.set_register(list, Nil),
            _ => return Err(PidginError::ArgumentNotList),
          };
        }
        Nth(result, list, n) => {
          // While `Get` returns nil for a list when index is OOB, `Nth` throws
          todo!()
        }
        NthFromLast(result, list, n) => todo!(),
        Cons(list, value) => todo!(),
        Concat(list, other_list) => todo!(),
        Take(list, n) => todo!(),
        Drop(list, n) => todo!(),
        Reverse(list) => todo!(),
        Distinct(list) => todo!(),
        Sub(list, start_index, end_index) => todo!(),
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
        SelectKeys(map, keys) => todo!(),
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
        CreateCell(result) => todo!(),
        GetCellValue(result, cell) => todo!(),
        SetCellValue(result, value) => todo!(),
        UpdateCell(result, f) => todo!(),
        CreateProcess(f_and_result) => {
          let f_value = self.steal_register(f_and_result);
          match f_value {
            CompositeFn(f) => self.set_register(
              f_and_result,
              Value::fn_process(Rc::unwrap_or_clone(f)),
            ),
            ExternalFn(_) => {
              return Err(PidginError::CantCreateProcess(
                "can't create a process from an external function".to_string(),
              ))
            }
            CoreFn(_) => {
              return Err(PidginError::CantCreateProcess(
                "can't create a process from a core function".to_string(),
              ))
            }
            other => {
              return Err(PidginError::CantCreateProcess(format!(
                "can't create a process from {}",
                other
              )))
            }
          }
        }
        IsProcessAlive(result, process) => todo!(),
        Yield(result, value) => todo!(),
        IsNil(result, value) => {
          self.set_register(
            result,
            Bool(if let Nil = self.get_register(value) {
              true
            } else {
              false
            }),
          );
        }
        IsBool(result, value) => {
          self.set_register(
            result,
            Bool(if let Bool(_) = self.get_register(value) {
              true
            } else {
              false
            }),
          );
        }
        IsChar(result, value) => {
          self.set_register(
            result,
            Bool(if let Char(_) = self.get_register(value) {
              true
            } else {
              false
            }),
          );
        }
        IsNum(result, value) => {
          self.set_register(
            result,
            Bool(if let Number(_) = self.get_register(value) {
              true
            } else {
              false
            }),
          );
        }
        IsInt(result, value) => {
          self.set_register(
            result,
            Bool(if let Number(Num::Int(_)) = self.get_register(value) {
              true
            } else {
              false
            }),
          );
        }
        IsFloat(result, value) => {
          self.set_register(
            result,
            Bool(if let Number(Num::Float(_)) = self.get_register(value) {
              true
            } else {
              false
            }),
          );
        }
        IsSymbol(result, value) => {
          self.set_register(
            result,
            Bool(if let Symbol(_) = self.get_register(value) {
              true
            } else {
              false
            }),
          );
        }
        IsString(result, value) => {
          self.set_register(
            result,
            Bool(if let Str(_) = self.get_register(value) {
              true
            } else {
              false
            }),
          );
        }
        IsList(result, value) => {
          self.set_register(
            result,
            Bool(if let List(_) = self.get_register(value) {
              true
            } else {
              false
            }),
          );
        }
        IsMap(result, value) => {
          self.set_register(
            result,
            Bool(if let Hashmap(_) = self.get_register(value) {
              true
            } else {
              false
            }),
          );
        }
        IsSet(result, value) => {
          self.set_register(
            result,
            Bool(if let Hashset(_) = self.get_register(value) {
              true
            } else {
              false
            }),
          );
        }
        IsCollection(result, value) => {
          self.set_register(
            result,
            Bool(match self.get_register(value) {
              List(_) => true,
              Hashmap(_) => true,
              Hashset(_) => true,
              _ => false,
            }),
          );
        }
        IsFn(result, value) => {
          self.set_register(
            result,
            Bool(match self.get_register(value) {
              CoreFn(_) => true,
              CompositeFn(_) => true,
              ExternalFn(_) => true,
              _ => false,
            }),
          );
        }
        IsError(result, value) => {
          self.set_register(
            result,
            Bool(if let Error(_) = self.get_register(value) {
              true
            } else {
              false
            }),
          );
        }
        IsCell(result, value) => todo!(),
        IsProcess(result, value) => {
          self.set_register(
            result,
            Bool(if let Process(_) = self.get_register(value) {
              true
            } else {
              false
            }),
          );
        }
        ToBool(result, value) => todo!(),
        ToChar(result, value) => todo!(),
        ToNum(result, value) => todo!(),
        ToInt(result, value) => todo!(),
        ToFloat(result, value) => todo!(),
        ToSymbol(result, value) => todo!(),
        ToString(result, value) => todo!(),
        ToList(result, value) => todo!(),
        ToMap(result, value) => todo!(),
        ToSet(result, value) => todo!(),
        ToError(result, value) => todo!(),
      }
    }
    if self.current_process.paused_frames.len() > 0 {
      panic!(
        "Execution ended with paused stack frames remaining (maybe a \
         function didn't end with a `Return` instruction?)"
      )
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::rc::Rc;

  use super::EvaluationState;
  use crate::runtime::control::CompositeFunction;
  use crate::{
    runtime::core_functions::CoreFnId,
    ConstIndex, ExternalFunction,
    Instruction::*,
    Num::{self, *},
    Program, RegisterIndex,
    Value::{self, *},
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
      let mut state = EvaluationState::new($program);
      state.evaluate().unwrap();
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

  fn environment_lookup() {
    let mut state = EvaluationState::new(program![Lookup(0, 0)]);
    state.bind_symbol(0, "test!");
    state.evaluate().unwrap();
    assert_register!(state, 0, "test!");
  }

  simple_register_test!(clear, program![Const(0, 100), Clear(0)], (0, Nil));

  simple_register_test!(copy, program![Const(0, 100), Copy(1, 0)], (1, 100));

  #[test]
  fn call_constant_function() {
    run_and_check_registers!(
      Program::new(
        vec![Const(0, 0), Call(1, 0, 0)],
        vec![
          CompositeFn(Rc::new(CompositeFunction::new(
            0,
            vec![Const(0, 1), Return(0)]
          ))),
          5.into()
        ],
      ),
      (1, 5)
    );
  }

  simple_register_test!(
    call_square_function,
    program![
      Const(0, 10),
      Const(
        1,
        CompositeFn(Rc::new(CompositeFunction::new(
          1,
          vec![DebugPrint(0), Multiply(0, 0, 0), Return(0)]
        )))
      ),
      Call(2, 1, 1),
      CopyArgument(0)
    ],
    (0, 10),
    (2, 100),
  );

  simple_register_test!(
    call_square_function_twice,
    program![
      Const(0, 10),
      Const(
        1,
        CompositeFn(Rc::new(CompositeFunction::new(
          1,
          vec![Multiply(0, 0, 0), Return(0)]
        )))
      ),
      Call(0, 1, 1),
      StealArgument(0),
      Call(0, 1, 1),
      StealArgument(0),
    ],
    (0, 10000),
  );

  #[test]
  fn call_double_square_nested_function() {
    run_and_check_registers!(
      Program::new(
        vec![Const(0, 0), Const(1, 2), Call(0, 1, 1), StealArgument(0)],
        vec![
          10.into(),
          CompositeFn(Rc::new(CompositeFunction::new(
            1,
            vec![Multiply(0, 0, 0), Return(0)]
          ))),
          CompositeFn(Rc::new(CompositeFunction::new(
            1,
            vec![
              Const(1, 1),
              Call(0, 1, 1),
              StealArgument(0),
              Call(0, 1, 1),
              StealArgument(0),
              Return(0)
            ]
          ))),
        ],
      ),
      (0, 10000)
    );
  }

  simple_register_test!(
    call_square_product_function,
    program![
      Const(0, 2),
      Const(1, 3),
      Const(
        2,
        CompositeFn(Rc::new(CompositeFunction::new(
          2,
          vec![Multiply(0, 1, 0), Multiply(0, 0, 0), Return(0)]
        )))
      ),
      Call(0, 2, 2),
      StealArgument(0),
      StealArgument(1),
    ],
    (0, 36)
  );

  simple_register_test!(
    call_triple_product_function,
    program![
      Const(0, 2),
      Const(1, 3),
      Const(2, 4),
      Const(
        4,
        CompositeFn(Rc::new(CompositeFunction::new(
          3,
          vec![Multiply(0, 1, 0), Multiply(0, 2, 0), Return(0)]
        )))
      ),
      Call(3, 4, 3),
      StealArgument(0),
      StealArgument(1),
      StealArgument(2),
    ],
    (3, 24)
  );

  simple_register_test!(
    apply_core_fn_add,
    program![
      Const(0, List(Rc::new(vec![1.into(), 2.into(), 3.into()]))),
      Const(1, CoreFn(CoreFnId::Add)),
      Apply(0, 1),
    ],
    (0, 6)
  );

  simple_register_test!(
    list_first_last,
    program![Const(0, vec![1.into(), 2.into()]), First(1, 0), Last(2, 0)],
    (1, 1),
    (2, 2)
  );

  simple_register_test!(
    list_push,
    program![EmptyList(0), Const(1, "test"), Push(0, 1)],
    (0, List(Rc::new(vec!["test".into()])))
  );

  simple_register_test!(
    list_rest,
    program![
      Const(0, List(Rc::new(vec![1.into(), 2.into()]))),
      Rest(0),
      Const(1, List(Rc::new(vec![1.into(), 2.into(), 3.into()]))),
      Rest(1)
    ],
    (0, List(Rc::new(vec![2.into()]))),
    (1, List(Rc::new(vec![2.into(), 3.into()])))
  );

  simple_register_test!(
    list_butlast,
    program![
      Const(0, List(Rc::new(vec![1.into(), 2.into()]))),
      ButLast(0),
      Const(1, List(Rc::new(vec![1.into(), 2.into(), 3.into()]))),
      ButLast(1)
    ],
    (0, List(Rc::new(vec![1.into()]))),
    (1, List(Rc::new(vec![1.into(), 2.into()])))
  );

  simple_register_test!(
    if_true,
    program![Const(0, true), Const(1, -5), If(0), Const(1, 5), EndIf],
    (1, 5)
  );

  simple_register_test!(
    if_false,
    program![Const(0, false), Const(1, -5), If(0), Const(1, 5), EndIf],
    (1, -5)
  );

  simple_register_test!(
    if_else_true,
    program![
      Const(0, true),
      If(0),
      Const(1, -5),
      Else,
      Const(1, 5),
      EndIf
    ],
    (1, -5)
  );

  simple_register_test!(
    if_else_false,
    program![
      Const(0, false),
      If(0),
      Const(1, -5),
      Else,
      Const(1, 5),
      EndIf
    ],
    (1, 5)
  );

  simple_register_test!(
    if_else_if_else_true_true,
    program![
      Const(0, true),
      Const(1, true),
      If(0),
      Const(2, -5),
      ElseIf(1),
      Const(2, 0),
      Else,
      Const(2, 5),
      EndIf
    ],
    (2, -5)
  );

  simple_register_test!(
    if_else_if_else_true_false,
    program![
      Const(0, true),
      Const(1, false),
      If(0),
      Const(2, -5),
      ElseIf(1),
      Const(2, 0),
      Else,
      Const(2, 5),
      EndIf
    ],
    (2, -5)
  );

  simple_register_test!(
    if_else_if_else_false_true,
    program![
      Const(0, false),
      Const(1, true),
      If(0),
      Const(2, -5),
      ElseIf(1),
      Const(2, 0),
      Else,
      Const(2, 5),
      EndIf
    ],
    (2, 0)
  );

  simple_register_test!(
    if_else_if_else_false_false,
    program![
      Const(0, false),
      Const(1, false),
      If(0),
      Const(2, -5),
      ElseIf(1),
      Const(2, 0),
      Else,
      Const(2, 5),
      EndIf
    ],
    (2, 5)
  );

  simple_register_test!(
    recursion,
    program![
      Const(0, 10),
      Const(
        1,
        CompositeFn(Rc::new(CompositeFunction::new(
          1,
          vec![
            IsPos(1, 0),
            If(1),
            Dec(0, 0),
            CallingFunction(2),
            Call(0, 2, 1),
            StealArgument(0),
            EndIf,
            Return(0)
          ]
        )))
      ),
      Call(0, 1, 1),
      StealArgument(0),
    ],
    (0, 0)
  );

  simple_register_test!(
    call_self_recursion,
    program![
      Const(0, 10),
      Const(
        1,
        CompositeFn(Rc::new(CompositeFunction::new(
          1,
          vec![
            IsPos(1, 0),
            If(1),
            Dec(0, 0),
            CallSelf(0, 1),
            StealArgument(0),
            EndIf,
            Return(0)
          ]
        )))
      ),
      Call(0, 1, 1),
      StealArgument(0),
    ],
    (0, 0)
  );

  simple_register_test!(
    tail_recursion,
    program![
      Const(0, (u16::MAX as i64)),
      Const(
        1,
        CompositeFn(Rc::new(CompositeFunction::new(
          1,
          vec![
            IsPos(1, 0),
            If(1),
            Dec(2, 0),
            CallingFunction(3),
            CallAndReturn(3, 1),
            StealArgument(2),
            EndIf,
            Return(0)
          ]
        )))
      ),
      Call(0, 1, 1),
      StealArgument(0),
    ],
    (0, 0),
  );

  simple_register_test!(
    call_self_tail_recursion,
    program![
      Const(0, (u16::MAX as i64)),
      Const(
        1,
        CompositeFn(Rc::new(CompositeFunction::new(
          1,
          vec![
            IsPos(1, 0),
            If(1),
            Dec(2, 0),
            CallSelfAndReturn(1),
            StealArgument(2),
            EndIf,
            Return(0)
          ]
        )))
      ),
      Call(0, 1, 1),
      StealArgument(0),
    ],
    (0, 0),
  );

  simple_register_test!(
    jump_loop,
    program![
      Const(0, (u16::MAX as i64)),
      Const(
        1,
        CompositeFn(Rc::new(CompositeFunction::new(
          1,
          vec![IsPos(1, 0), If(1), Dec(0, 0), Jump(0), EndIf, Return(0)]
        )))
      ),
      Call(0, 1, 1),
      StealArgument(0),
    ],
    (0, 0),
  );

  simple_register_test!(
    call_external_function,
    program![
      Const(0, 1),
      Const(1, 2),
      Const(
        2,
        ExternalFunction::unnamed(|args| {
          Ok((args[0].as_num()? + args[1].as_num()?).into())
        })
      ),
      Call(0, 2, 2),
      StealArgument(0),
      StealArgument(1)
    ],
    (0, 3)
  );

  simple_register_test!(
    external_object,
    program![
      Const(0, Value::external((1i64, 2i64))),
      Const(1, Value::external((3i64, 4i64))),
      Const(
        2,
        ExternalFunction::unnamed(|mut args| {
          let a = args.pop().unwrap().casted_external::<(i64, i64)>().unwrap();
          let b = args.pop().unwrap().casted_external::<(i64, i64)>().unwrap();
          Ok(Number(Num::from(a.0 + b.0 + a.1 + b.1)))
        })
      ),
      Call(0, 2, 2),
      StealArgument(0),
      StealArgument(1)
    ],
    (0, 10)
  );

  simple_register_test!(
    create_process,
    program![
      Const(
        0,
        CompositeFn(Rc::new(CompositeFunction::new(
          0,
          vec![EmptyList(0), Return(0)]
        )))
      ),
      CreateProcess(0),
    ],
  );

  simple_register_test!(
    run_process,
    program![
      Const(
        0,
        CompositeFn(Rc::new(CompositeFunction::new(
          0,
          vec![EmptyList(0), DebugPrint(0), Return(0)]
        )))
      ),
      CreateProcess(0),
      Call(1, 0, 0)
    ],
    (1, List(Rc::new(vec![])))
  );
}
