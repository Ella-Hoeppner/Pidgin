use std::rc::Rc;

use crate::{
  blocks::GenericBlock,
  compiler::intermediate::register_allocation::get_max_register,
  runtime::{
    data::{AritySpecifier, Value},
    vm::{Instruction, Register, StackIndex},
  },
};

const STACK_CAPACITY: usize = 1000; //u16::MAX as usize + 1;

pub type Block = GenericBlock<Register, Register, Register, Register>;
impl Block {
  pub fn new(instructions: Vec<Instruction>, constants: Vec<Value>) -> Self {
    let max_register = get_max_register(&instructions);
    Block {
      instructions: instructions.into(),
      constants: constants.into(),
      metadata: max_register,
    }
  }
}

#[derive(Clone, Debug)]
pub struct GenericCompositeFunction<I, O, R, M> {
  pub args: AritySpecifier,
  pub block: GenericBlock<I, O, R, M>,
}

impl<I, O, R, M> GenericCompositeFunction<I, O, R, M> {
  pub fn new<A: Into<AritySpecifier>, T: Into<GenericBlock<I, O, R, M>>>(
    args: A,
    block: T,
  ) -> Self {
    Self {
      args: args.into(),
      block: block.into(),
    }
  }
}

pub type CompositeFunction =
  GenericCompositeFunction<Register, Register, Register, Register>;

#[derive(Debug)]
pub struct CoroutineState {
  pub stack: Vec<Value>,
  pub paused_frames: Vec<StackFrame>,
}
impl CoroutineState {
  pub fn new() -> Self {
    Self {
      stack: std::iter::repeat(Value::Nil).take(STACK_CAPACITY).collect(),
      paused_frames: vec![],
    }
  }
  pub fn new_with_root_frame(root_frame: StackFrame) -> Self {
    Self {
      stack: std::iter::repeat(Value::Nil).take(STACK_CAPACITY).collect(),
      paused_frames: vec![root_frame],
    }
  }
  pub fn pause(
    mut self,
    active_stack_frame: StackFrame,
    new_arg_count_and_offset: Option<(AritySpecifier, u8)>,
  ) -> PausedCoroutine {
    self.paused_frames.push(active_stack_frame);
    let (args, arg_offset) = new_arg_count_and_offset.unwrap_or((0.into(), 0));
    PausedCoroutine {
      started: true,
      args,
      arg_offset,
      state: self,
    }
  }
}

#[derive(Debug)]
pub struct PausedCoroutine {
  pub started: bool,
  pub args: AritySpecifier,
  pub arg_offset: u8,
  pub state: CoroutineState,
}
impl PausedCoroutine {
  pub fn begin_as_child(
    mut self,
    return_index: StackIndex,
  ) -> (StackFrame, CoroutineState) {
    self.started = true;
    let mut active_frame = self
      .state
      .paused_frames
      .pop()
      .expect("attempting to resume a PausedCoroutine with no paused_frames");
    active_frame.return_stack_index = return_index;
    (active_frame, self.state)
  }
  pub fn resume_from_child(mut self) -> (StackFrame, CoroutineState) {
    let active_frame = self
      .state
      .paused_frames
      .pop()
      .expect("attempting to resume a PausedCoroutine with no paused_frames");
    (active_frame, self.state)
  }
}
impl From<CompositeFunction> for PausedCoroutine {
  fn from(f: CompositeFunction) -> Self {
    Self {
      started: false,
      args: f.args,
      arg_offset: 0,
      state: CoroutineState::new_with_root_frame(StackFrame::root(f.block)),
    }
  }
}

#[derive(Debug)]
pub struct StackFrame {
  pub beginning: StackIndex,
  pub calling_function: Option<Rc<CompositeFunction>>,
  pub block: Block,
  pub instruction_index: usize,
  pub return_stack_index: StackIndex,
}
impl StackFrame {
  pub fn root(block: Block) -> Self {
    Self {
      beginning: 0,
      calling_function: None,
      block,
      instruction_index: 0,
      return_stack_index: 0,
    }
  }
  pub fn for_fn(
    f: Rc<CompositeFunction>,
    beginning: StackIndex,
    return_stack_index: StackIndex,
  ) -> Self {
    Self {
      beginning,
      block: f.block.clone(),
      instruction_index: 0,
      calling_function: Some(f),
      return_stack_index,
    }
  }
  pub fn next_instruction(&mut self) -> Instruction {
    let instruction = self.block[self.instruction_index].clone();
    self.instruction_index += 1;
    instruction
  }
  pub fn stack_consumption(&self) -> StackIndex {
    self.block.metadata as StackIndex
  }
}
