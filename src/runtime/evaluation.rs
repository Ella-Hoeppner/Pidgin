use std::collections::HashMap;
use std::rc::Rc;
use std::result;

use crate::runtime::core_functions::CORE_FUNCTIONS;
use crate::string_utils::pad;
use crate::{
  instructions::GenericInstruction::{self, *},
  runtime::{
    control::{CoroutineState, StackFrame},
    data::{
      AritySpecifier,
      GenericValue::*,
      Num::{self, *},
      Value,
    },
  },
  string_utils::indent_lines,
};

use take_mut::take;

use super::control::{Block, CompositeFunction, PausedCoroutine};
use super::error::{RuntimeError, RuntimeResult};

pub type Register = u8;
pub type StackIndex = u16;
pub type SymbolIndex = u16;
pub type ConstIndex = u16;
pub type Instruction = GenericInstruction<Register, Register, Register>;

pub struct EvaluationState {
  current_frame: StackFrame,
  current_coroutine: CoroutineState,
  parent_coroutine_stack: Vec<(StackIndex, PausedCoroutine)>,
}

impl EvaluationState {
  pub fn new(block: Block) -> Self {
    Self {
      current_frame: StackFrame::root(block),
      current_coroutine: CoroutineState::new(),
      parent_coroutine_stack: vec![],
    }
  }
  fn describe_stack(&self) -> String {
    self
      .current_coroutine
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
        let end = start
          + maybe_next_frame
            .unwrap_or(&self.current_frame)
            .stack_consumption();
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
            (start..=end)
              .map(|i| format!(
                "{}: {}",
                i,
                self.get_stack(i).description(None)
              ))
              .collect::<Vec<String>>()
              .join("\n")
          ),
        )
      })
      .collect::<Vec<String>>()
      .join("\n")
  }
  fn push_frame(&mut self, mut frame: StackFrame) {
    std::mem::swap(&mut self.current_frame, &mut frame);
    self.current_coroutine.paused_frames.push(frame);
  }
  fn complete_child_coroutine(&mut self) -> Option<StackFrame> {
    if let Some((child_coroutine_stack_index, parent_coroutine)) =
      self.parent_coroutine_stack.pop()
    {
      let (frame, coroutine_state) = parent_coroutine.resume_from_child();
      self.current_coroutine = coroutine_state;
      self.set_stack(
        child_coroutine_stack_index,
        Value::Coroutine(Rc::new(None)),
      );
      Some(frame)
    } else {
      None
    }
  }
  fn complete_frame(&mut self) -> Option<StackFrame> {
    if let Some(mut next_frame) = self
      .current_coroutine
      .paused_frames
      .pop()
      .map(|x| Some(x))
      .unwrap_or_else(|| self.complete_child_coroutine())
    {
      std::mem::swap(&mut self.current_frame, &mut next_frame);
      Some(next_frame)
    } else {
      None
    }
  }
  fn return_value(&mut self, value: Value) -> Option<Value> {
    if let Some(completed_frame) = self.complete_frame() {
      self.set_stack(completed_frame.return_stack_index, value);
      None
    } else {
      Some(value)
    }
  }
  fn yield_value(
    &mut self,
    yielded_value: Value,
    new_arg_count_and_offset: Option<(AritySpecifier, u8)>,
    kill: bool,
  ) {
    let return_stack_index = self.current_frame.return_stack_index;
    let (child_coroutine_stack_index, parent_coroutine) = self
      .parent_coroutine_stack
      .pop()
      .expect("attempted to yield_coroutine with no paused parent coroutinees");
    let (mut resumed_frame, mut coroutine_state) =
      parent_coroutine.resume_from_child();
    std::mem::swap(&mut coroutine_state, &mut self.current_coroutine);
    let child_coroutine_value =
      self.get_stack(child_coroutine_stack_index).clone();
    if let Coroutine(coroutine) = child_coroutine_value {
      if let Some(active_coroutine_ref) = &*coroutine {
        #[cfg(debug_assertions)]
        assert!(
          (*active_coroutine_ref).borrow().is_none(),
          "coroutine pointed to by child_coroutine_stack_index when attempting \
          to yield_value isn't inactive"
        );
        std::mem::swap(&mut resumed_frame, &mut self.current_frame);
        active_coroutine_ref.replace(Some(
          coroutine_state.pause(resumed_frame, new_arg_count_and_offset),
        ));
        self.set_stack(return_stack_index, yielded_value);
      } else {
        panic!(
          "coroutine pointed to by child_coroutine_stack_index when attempting \
          to yield_value is dead"
        )
      }
    } else {
      panic!(
        "value pointed to by child_coroutine_stack_index when attempting to \
        yield_value is not a Coroutine"
      )
    }
    if kill {
      self.set_stack(
        child_coroutine_stack_index,
        Value::Coroutine(Rc::new(None)),
      );
    }
  }
  fn set_stack_usize(&mut self, index: usize, value: Value) {
    self.current_coroutine.stack[index] = value;
  }
  fn set_stack(&mut self, index: StackIndex, value: Value) {
    self.set_stack_usize(index as usize, value);
  }
  fn swap_stack_usize(&mut self, index: usize, value: Value) -> Value {
    std::mem::replace(&mut self.current_coroutine.stack[index], value)
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
    &self.current_coroutine.stack[index]
  }
  fn get_stack(&self, index: StackIndex) -> &Value {
    self.get_stack_usize(index as usize)
  }
  fn get_stack_mut_usize(&mut self, index: usize) -> &mut Value {
    &mut self.current_coroutine.stack[index]
  }
  fn get_stack_mut(&mut self, index: StackIndex) -> &mut Value {
    self.get_stack_mut_usize(index as usize)
  }
  fn register_stack_index(&self, register: Register) -> StackIndex {
    self.current_frame.beginning + register as StackIndex
  }
  fn set_register<T: Into<Value>>(&mut self, register: Register, value: T) {
    self.set_stack(self.register_stack_index(register), value.into());
  }
  fn swap_register<T: Into<Value>>(
    &mut self,
    register: Register,
    value: T,
  ) -> Value {
    self.swap_stack(self.register_stack_index(register), value.into())
  }
  fn steal_register(&mut self, register: Register) -> Value {
    self.steal_stack(self.register_stack_index(register))
  }
  pub(crate) fn get_register(&self, register: Register) -> &Value {
    self.get_stack(self.register_stack_index(register))
  }
  pub(crate) fn get_register_mut(&mut self, register: Register) -> &mut Value {
    self.get_stack_mut(self.register_stack_index(register))
  }
  fn create_fn_stack_frame(
    &mut self,
    f: Rc<CompositeFunction>,
    return_stack_index: StackIndex,
  ) -> StackFrame {
    StackFrame::for_fn(
      f,
      self.current_frame.beginning + self.current_frame.stack_consumption() + 1,
      return_stack_index,
    )
  }
  fn start_fn_stack_frame(
    &mut self,
    f: Rc<CompositeFunction>,
    return_stack_index: StackIndex,
  ) {
    let frame = self.create_fn_stack_frame(f, return_stack_index);
    self.push_frame(frame);
  }
  fn next_instruction(&mut self) -> Instruction {
    self.current_frame.next_instruction()
  }
  fn skip_to_endif(&mut self) {
    loop {
      if self.next_instruction() == EndIf {
        break;
      }
    }
  }
  fn push_child_coroutine(
    &mut self,
    coroutine: PausedCoroutine,
    parent_stack_reference_index: StackIndex,
    return_index: StackIndex,
  ) {
    let (mut active_frame, coroutine_state) =
      coroutine.begin_as_child(return_index);
    std::mem::swap(&mut self.current_frame, &mut active_frame);
    take(&mut self.current_coroutine, |parent_coroutine| {
      let paused_parent_coroutine = parent_coroutine.pause(active_frame, None);
      self
        .parent_coroutine_stack
        .push((parent_stack_reference_index, paused_parent_coroutine));
      coroutine_state
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
  fn set_args(&mut self, args: Vec<Value>, arg_offset: u8) {
    for (i, arg_value) in args.into_iter().enumerate() {
      self.set_register(i as Register + arg_offset, arg_value);
    }
  }
  fn apply(
    &mut self,
    result_register: Register,
    f: &Value,
    args: Vec<Value>,
  ) -> RuntimeResult<()> {
    match f {
      CompositeFn(composite_fn) => {
        self.start_fn_stack_frame(
          composite_fn.clone(),
          self.register_stack_index(result_register),
        );
        let provided_arg_count = args.len();
        #[cfg(debug_assertions)]
        if !composite_fn.args.can_accept(provided_arg_count) {
          panic!(
            "Apply called on CompositeFn that expects {} arguments, \
            {} arguments provided",
            composite_fn.args, provided_arg_count
          )
        }
        self.set_args(args, 0);
      }
      CoreFn(core_fn_id) => match CORE_FUNCTIONS[*core_fn_id](args) {
        Ok(value) => self.set_register(result_register, value),
        Err(error) => return Err(error),
      },
      ExternalFn(external_fn) => {
        let f = (*external_fn).f;
        self.set_register(
          result_register,
          f(args).expect(
            "external_fn returned an error, and we don't have error \
            handling yet :(",
          ),
        );
      }
      PartialApplication(f_and_args) => todo!(),
      List(list) => todo!(),
      Hashmap(map) => todo!(),
      Hashset(set) => todo!(),
      Coroutine(maybe_coroutine) => todo!(),
      value => {
        return Err(RuntimeError::CantApply(value.clone()));
      }
    }
    Ok(())
  }
  pub fn evaluate(
    &mut self,
    global_bindings: &HashMap<SymbolIndex, Value>,
  ) -> RuntimeResult<Option<Value>> {
    loop {
      if self.current_frame.instruction_index >= self.current_frame.block.len()
      {
        break;
      }
      let instruction_result: RuntimeResult<Option<Value>> = 'instruction: {
        match self.next_instruction() {
          DebugPrint(id) => {
            println!(
              "{}\n\
              paused coroutinees: {}\n\
              stack:\n{}\n\n\n\
              ----------------------------------------\n\n\n",
              pad(40, '-', format!("DEBUG {} ", id)),
              self.parent_coroutine_stack.len(),
              self.describe_stack(),
            );
          }
          Clear(register) => self.set_register(register, Nil),
          Copy(result, value) => {
            self.set_register(result, self.get_register(value).clone())
          }
          Const(result, const_index) => {
            self.set_register(
              result,
              self.current_frame.block.constants[const_index as usize].clone(),
            );
          }
          Print(value) => {
            println!("{}", self.get_register(value).description(None))
          }
          Return(value) => {
            let return_value = self.steal_register(value);
            if let Some(final_value) = self.return_value(return_value) {
              break 'instruction Ok(Some(final_value));
            }
          }
          CopyArgument(_) => {
            panic!("CopyArgument instruction called, this should never happen")
          }
          StealArgument(_) => {
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
              CoreFn(f) => {
                let args = self.take_args(arg_count);
                match CORE_FUNCTIONS[f](args) {
                  Ok(output) => self.set_register(target, output),
                  Err(e) => break 'instruction Err(e),
                }
              }
              ExternalFn(external_fn) => {
                let args = self.take_args(arg_count);
                let f = (*external_fn).f;
                self.set_register(
                  target,
                  f(args).expect(
                    "external_fn returned an error, and we don't have error \
                    handling yet :(",
                  ),
                );
              }
              PartialApplication(f_and_args) => {
                let (partial_f, partial_args) = &*f_and_args;
                let args = self.take_args(arg_count);
                println!("partial application!\n{partial_f:?}\n{partial_args:?}\n{args:?}\n");
                if let Err(err) = self.apply(
                  target,
                  partial_f,
                  partial_args
                    .iter()
                    .cloned()
                    .chain(args.into_iter())
                    .collect(),
                ) {
                  break 'instruction Err(err);
                }
              }
              Coroutine(maybe_coroutine) => {
                if let Some(coroutine_ref) = &*maybe_coroutine {
                  if let Some(coroutine) = coroutine_ref.replace(None) {
                    if !coroutine.args.can_accept(arg_count as usize) {
                      break 'instruction Err(RuntimeError::InvalidArity);
                    }
                    let args = self.take_args(arg_count);
                    let arg_offset = coroutine.arg_offset;
                    self.push_child_coroutine(
                      coroutine,
                      self.register_stack_index(f),
                      self.register_stack_index(target),
                    );
                    self.set_args(args, arg_offset);
                  } else {
                    break 'instruction Err(
                      RuntimeError::CoroutineAlreadyRunning,
                    );
                  }
                } else {
                  break 'instruction Err(RuntimeError::DeadCoroutine);
                }
              }
              List(list) => todo!(),
              Hashmap(map) => todo!(),
              Hashset(set) => todo!(),
              value => {
                break 'instruction Err(RuntimeError::CantApply(value));
              }
            }
          }
          Apply(args_and_result, f) => {
            let f_value = self.get_register(f).clone();
            if let List(arg_list) = self.steal_register(args_and_result) {
              if let Err(err) = self.apply(
                args_and_result,
                &f_value,
                Rc::unwrap_or_clone(arg_list),
              ) {
                break 'instruction Err(err);
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
                self.set_register(i as Register, arg_value);
              }
            } else {
              panic!("Apply called with non-List value");
            }
          }
          CallAndReturn(f, arg_count) => {
            let f_value = self.get_register(f).clone();
            if let Some(mut completed_frame) = self.complete_frame() {
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
                  self.push_frame(new_frame);
                }
                ExternalFn(_) => todo!(),
                CoreFn(_) => {
                  panic!(
                    "CallAndReturn instruction called with CoreFn value, this \
                  should never happen"
                  )
                }
                PartialApplication(f_and_args) => todo!(),
                Coroutine(maybe_coroutine) => todo!(),
                List(list) => todo!(),
                Hashmap(map) => todo!(),
                Hashset(set) => todo!(),
                value => {
                  break 'instruction Err(RuntimeError::CantApply(value));
                }
              }
            } else {
              panic!("CallAndReturn failed to complete the current frame")
            }
          }
          ApplyAndReturn(args, f) => {
            todo!()
          }
          CallSelfAndReturn(arg_count) => {
            let composite_fn =
              self.current_frame.calling_function.clone().unwrap();
            if let Some(mut completed_frame) = self.complete_frame() {
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
              self.push_frame(new_frame);
            } else {
              panic!("CallSelfAndReturn failed to complete the current frame")
            }
          }
          ApplySelfAndReturn(args) => todo!(),
          Lookup(register, symbol_index) => {
            self.set_register(register, global_bindings[&symbol_index].clone());
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
          Else => self.skip_to_endif(),
          ElseIf(condition) => self.skip_to_endif(),
          EndIf => {}
          Partial(result, f, arg) => todo!(),
          Compose(result, f_1, f_2) => todo!(),
          FindSome(result, f, collection) => todo!(),
          ReduceWithoutInitialValue(collection_and_result, f) => todo!(),
          ReduceWithInitialValue(collection_and_result, f, initial_value) => {
            todo!()
          }
          Memoize(result, f) => todo!(),
          Constantly(result, value) => todo!(),
          NumericalEqual(result, num_1, num_2) => self.set_register(
            result,
            match (self.get_register(num_1), self.get_register(num_2)) {
              (Number(a), Number(b)) => a.numerical_equal(b),
              _ => break 'instruction Err(RuntimeError::ArgumentNotNum),
            },
          ),
          IsZero(result, num) => self.set_register(
            result,
            match self.get_register(num) {
              Number(Float(f)) => *f == 0.,
              Number(Int(i)) => *i == 0,
              _ => break 'instruction Err(RuntimeError::ArgumentNotNum),
            },
          ),
          IsNan(result, num) => self.set_register(
            result,
            match self.get_register(num) {
              Number(Float(f)) => f.is_nan(),
              Number(Int(_)) => false,
              _ => break 'instruction Err(RuntimeError::ArgumentNotNum),
            },
          ),
          IsInf(result, num) => self.set_register(
            result,
            match self.get_register(num) {
              Number(Float(f)) => f.is_infinite(),
              Number(Int(_)) => false,
              _ => break 'instruction Err(RuntimeError::ArgumentNotNum),
            },
          ),
          IsEven(result, num) => self.set_register(
            result,
            match self.get_register(num) {
              Number(n) => match n.as_int_lossless() {
                Ok(i) => i % 2 == 0,
                Err(error) => break 'instruction Err(error),
              },
              _ => break 'instruction Err(RuntimeError::ArgumentNotNum),
            },
          ),
          IsOdd(result, num) => self.set_register(
            result,
            match self.get_register(num) {
              Number(n) => match n.as_int_lossless() {
                Ok(i) => i % 2 == 1,
                Err(error) => break 'instruction Err(error),
              },
              _ => break 'instruction Err(RuntimeError::ArgumentNotNum),
            },
          ),
          IsPos(result, num) => self.set_register(
            result,
            match self.get_register(num) {
              Number(Float(f)) => **f > 0.,
              Number(Int(i)) => *i > 0,
              _ => break 'instruction Err(RuntimeError::ArgumentNotNum),
            },
          ),
          IsNeg(result, num) => self.set_register(
            result,
            match self.get_register(num) {
              Number(Float(f)) => **f < 0.,
              Number(Int(i)) => *i < 0,
              _ => break 'instruction Err(RuntimeError::ArgumentNotNum),
            },
          ),
          Inc(result, num) => self.set_register(
            result,
            match self.get_register(num).as_num() {
              Ok(n) => n.inc(),
              Err(error) => break 'instruction Err(error),
            },
          ),
          Dec(result, num) => self.set_register(
            result,
            match self.get_register(num).as_num() {
              Ok(n) => n.dec(),
              Err(error) => break 'instruction Err(error),
            },
          ),
          Negate(result, num) => self.set_register(
            result,
            match self.get_register(num).as_num() {
              Ok(n) => -*n,
              Err(error) => break 'instruction Err(error),
            },
          ),
          Abs(result, num) => self.set_register(
            result,
            match self.get_register(num).as_num() {
              Ok(n) => n.abs(),
              Err(error) => break 'instruction Err(error),
            },
          ),
          Floor(result, num) => self.set_register(
            result,
            match self.get_register(num).as_num() {
              Ok(n) => n.floor(),
              Err(error) => break 'instruction Err(error),
            },
          ),
          Ceil(result, num) => self.set_register(
            result,
            match self.get_register(num).as_num() {
              Ok(n) => n.ceil(),
              Err(error) => break 'instruction Err(error),
            },
          ),
          Sqrt(result, num) => self.set_register(
            result,
            match self.get_register(num).as_num() {
              Ok(n) => n.as_float().sqrt(),
              Err(error) => break 'instruction Err(error),
            },
          ),
          Exp(result, num) => self.set_register(
            result,
            match self.get_register(num).as_num() {
              Ok(n) => n.as_float().exp(),
              Err(error) => break 'instruction Err(error),
            },
          ),
          Exp2(result, num) => self.set_register(
            result,
            match self.get_register(num).as_num() {
              Ok(n) => n.as_float().exp2(),
              Err(error) => break 'instruction Err(error),
            },
          ),
          Ln(result, num) => self.set_register(
            result,
            match self.get_register(num).as_num() {
              Ok(n) => n.as_float().ln(),
              Err(error) => break 'instruction Err(error),
            },
          ),
          Log2(result, num) => self.set_register(
            result,
            match self.get_register(num).as_num() {
              Ok(n) => n.as_float().log2(),
              Err(error) => break 'instruction Err(error),
            },
          ),
          Add(result, num_1, num_2) => self.set_register(
            result,
            match self.get_register(num_1).as_num() {
              Ok(n) => *n,
              Err(error) => break 'instruction Err(error),
            } + match self.get_register(num_2).as_num() {
              Ok(n) => *n,
              Err(error) => break 'instruction Err(error),
            },
          ),
          Subtract(result, num_1, num_2) => self.set_register(
            result,
            match self.get_register(num_1).as_num() {
              Ok(n) => *n,
              Err(error) => break 'instruction Err(error),
            } - match self.get_register(num_2).as_num() {
              Ok(n) => *n,
              Err(error) => break 'instruction Err(error),
            },
          ),
          Multiply(result, num_1, num_2) => {
            self.set_register(
              result,
              match self.get_register(num_1).as_num() {
                Ok(n) => *n,
                Err(error) => break 'instruction Err(error),
              } * match self.get_register(num_2).as_num() {
                Ok(n) => *n,
                Err(error) => break 'instruction Err(error),
              },
            );
          }
          Divide(result, num_1, num_2) => self.set_register(
            result,
            match self.get_register(num_1).as_num() {
              Ok(n) => *n,
              Err(error) => break 'instruction Err(error),
            } / match self.get_register(num_2).as_num() {
              Ok(n) => *n,
              Err(error) => break 'instruction Err(error),
            },
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
          IsEmpty(result, collection) => self.set_register(
            result,
            match self.get_register(collection) {
              List(list) => list.is_empty(),
              Hashset(set) => set.is_empty(),
              Hashmap(hashmap) => hashmap.is_empty(),
              Nil => true,
              _ => break 'instruction Err(RuntimeError::ArgumentNotList),
            },
          ),
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
              _ => break 'instruction Err(RuntimeError::ArgumentNotList),
            },
          ),
          Count(result, collection) => todo!(),
          Flatten(result, collection) => todo!(),
          Remove(collection_and_result, key) => todo!(),
          Filter(collection_and_result, f) => todo!(),
          Map(collection_and_result, f) => todo!(),
          DoubleMap(collection_and_result, other_collection, f) => {
            // Special case of multi-collection map with just 2 collections.
            // This special case comes up often enough (e.g. mapping with
            // `(range)` as a second argument for indexing) that the
            // optimization from having this instruction seems worthwhile
            todo!()
          }
          MultiCollectionMap(list_of_collections_and_result, f) => todo!(),
          Set(collection_and_result, value, key) => todo!(),
          SetIn(collection_and_result, value, path) => todo!(),
          Get(result, collection, key) => todo!(),
          GetIn(result, collection, path) => todo!(),
          Update(collection_and_result, f, key) => todo!(),
          UpdateIn(collection_and_result, f, path) => todo!(),
          MinKey(result, collection, f) => todo!(),
          MaxKey(result, collection, f) => todo!(),
          Push(collection_and_result, value) => {
            let value = self.get_register(value).clone();
            let collection_value = self.steal_register(collection_and_result);
            match collection_value {
              List(mut list_value) => {
                self.set_register(
                  collection_and_result,
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
              Nil => self.set_register(collection_and_result, Nil),
              _ => break 'instruction Err(RuntimeError::ArgumentNotList),
            };
          }
          Sort(collection_and_result) => todo!(),
          SortBy(collection_and_result, f) => todo!(),
          EmptyList(result) => {
            self.set_register(result, Vec::new());
          }
          Last(result, list) => self.set_register(
            result,
            match self.get_register(list) {
              List(list) => list.last().cloned().unwrap_or(Nil),
              Nil => Nil,
              _ => break 'instruction Err(RuntimeError::ArgumentNotList),
            },
          ),
          Rest(list_and_result) => {
            let list_value = self.steal_register(list_and_result);
            match list_value {
              List(mut list_value) => {
                self.set_register(
                  list_and_result,
                  if let Some(owned_list_value) = Rc::get_mut(&mut list_value) {
                    if !owned_list_value.is_empty() {
                      owned_list_value.remove(0);
                    }
                    list_value
                  } else {
                    let mut list_value_clone = (*list_value).clone();
                    list_value_clone.remove(0);
                    Rc::new(list_value_clone)
                  },
                );
              }
              Nil => self.set_register(list_and_result, Nil),
              _ => break 'instruction Err(RuntimeError::ArgumentNotList),
            };
          }
          ButLast(list_and_result) => {
            let list_value = self.steal_register(list_and_result);
            match list_value {
              List(mut list_value) => {
                self.set_register(
                  list_and_result,
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
              Nil => self.set_register(list_and_result, Nil),
              _ => break 'instruction Err(RuntimeError::ArgumentNotList),
            };
          }
          Nth(result, list, n) => {
            // While `Get` returns nil for a list when index is OOB, `Nth`
            // throws
            todo!()
          }
          NthFromLast(result, list, n) => todo!(),
          Cons(list_and_result, value) => {
            let value = self.get_register(value).clone();
            let collection_value = self.steal_register(list_and_result);
            match collection_value {
              List(mut list_value) => {
                self.set_register(
                  list_and_result,
                  if list_value.is_empty() {
                    if let Some(owned_list_value) = Rc::get_mut(&mut list_value)
                    {
                      owned_list_value.push(value);
                      list_value
                    } else {
                      let mut list_value_clone = (*list_value).clone();
                      list_value_clone.push(value);
                      Rc::new(list_value_clone)
                    }
                  } else {
                    if let Some(owned_list_value) = Rc::get_mut(&mut list_value)
                    {
                      owned_list_value.insert(0, value);
                      list_value
                    } else {
                      Rc::new(
                        std::iter::once(value)
                          .chain((*list_value).iter().cloned())
                          .collect(),
                      )
                    }
                  },
                );
              }
              Hashmap(hashmap) => todo!(),
              Hashset(set) => todo!(),
              Nil => self.set_register(list_and_result, Nil),
              _ => break 'instruction Err(RuntimeError::ArgumentNotList),
            };
          }
          Concat(list_and_result, other_list) => todo!(),
          Take(list_and_result, n) => todo!(),
          Drop(list_and_result, n) => todo!(),
          Reverse(list_and_result) => todo!(),
          Distinct(list_and_result) => todo!(),
          Sub(list_and_result, start_index, end_index) => todo!(),
          Partition(result, list, size) => todo!(),
          SteppedPartition(step_and_return, list, size) => todo!(),
          Pad(list_and_result, value, size) => todo!(),
          EmptyMap(result) => todo!(),
          Keys(result, map) => todo!(),
          Values(result, map) => todo!(),
          Zip(result, key_list, value_list) => todo!(),
          Invert(map_and_result) => todo!(),
          Merge(result, map_1, map_2) => todo!(),
          MergeWith(map_1_and_result, f, map_2) => todo!(),
          MapKeys(map_and_result, f) => todo!(),
          MapValues(map_and_result, f) => todo!(),
          SelectKeys(map_and_result, keys) => todo!(),
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
          CreateCoroutine(f_and_result) => {
            let f_value = self.steal_register(f_and_result);
            match f_value {
              CompositeFn(f) => self.set_register(
                f_and_result,
                Value::fn_coroutine(Rc::unwrap_or_clone(f)),
              ),
              ExternalFn(_) => {
                break 'instruction Err(RuntimeError::CantCreateCoroutine(
                  "can't create a coroutine from an external function"
                    .to_string(),
                ))
              }
              CoreFn(_) => {
                break 'instruction Err(RuntimeError::CantCreateCoroutine(
                  "can't create a coroutine from a core function".to_string(),
                ))
              }
              other => {
                break 'instruction Err(RuntimeError::CantCreateCoroutine(
                  format!("can't create a coroutine from {}", other),
                ))
              }
            }
          }
          IsCoroutineAlive(result, coroutine) => {
            if let Coroutine(maybe_coroutine) = self.get_register(coroutine) {
              self.set_register(result, (**maybe_coroutine).is_some());
            } else {
              break 'instruction Err(RuntimeError::IsntCoroutine);
            }
          }
          Yield(value) => {
            let yielded_value = self.get_register(value).clone();
            self.yield_value(yielded_value, None, false);
          }
          YieldAndAccept(value, arg_count, new_args_first_register) => {
            let yielded_value = self.get_register(value).clone();
            self.yield_value(
              yielded_value,
              Some((arg_count.into(), new_args_first_register)),
              false,
            );
          }
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
          IsCoroutine(result, value) => {
            self.set_register(
              result,
              Bool(if let Coroutine(_) = self.get_register(value) {
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
        Ok(None)
      };
      match instruction_result {
        Ok(None) => {}
        Ok(Some(value)) => return Ok(Some(value)),
        Err(error) => {
          if self.parent_coroutine_stack.is_empty() {
            return Err(error);
          } else {
            self.yield_value(error.into(), None, true)
          }
        }
      }
    }
    if self.current_coroutine.paused_frames.len() > 0 {
      panic!(
        "Execution ended with paused stack frames remaining (maybe a \
        function didn't end with a `Return` instruction?)"
      )
    }
    Ok(None)
  }
}
