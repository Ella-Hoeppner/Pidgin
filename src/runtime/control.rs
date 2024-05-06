use std::rc::Rc;

use crate::{
  ArgumentSpecifier, ConstIndex, Instruction, InstructionBlock, RegisterIndex,
  RuntimeInstruction, StackIndex, Value,
};

const STACK_CAPACITY: usize = 1000; //u16::MAX as usize + 1;

pub type RuntimeInstructionBlock =
  InstructionBlock<RegisterIndex, ConstIndex, ()>;

impl From<Vec<Instruction<RegisterIndex, ConstIndex>>>
  for RuntimeInstructionBlock
{
  fn from(instructions: Vec<RuntimeInstruction>) -> Self {
    Self {
      instructions: instructions.into(),
      metadata: (),
    }
  }
}

#[derive(Clone, Debug)]
pub struct CompositeFunction {
  pub args: ArgumentSpecifier,
  pub instructions: RuntimeInstructionBlock,
}
impl CompositeFunction {
  pub fn new<A: Into<ArgumentSpecifier>, I: Into<RuntimeInstructionBlock>>(
    args: A,
    instructions: I,
  ) -> Self {
    Self {
      args: args.into(),
      instructions: instructions.into(),
    }
  }
}

#[derive(Debug)]
pub struct ProcessState {
  pub stack: Vec<Value>,
  pub paused_frames: Vec<StackFrame>,
  pub consumption: StackIndex,
}
impl ProcessState {
  pub fn new() -> Self {
    Self {
      stack: std::iter::repeat(Value::Nil).take(STACK_CAPACITY).collect(),
      paused_frames: vec![],
      consumption: 0,
    }
  }
  pub fn new_with_root_frame(root_frame: StackFrame) -> Self {
    Self {
      stack: std::iter::repeat(Value::Nil).take(STACK_CAPACITY).collect(),
      paused_frames: vec![root_frame],
      consumption: 0,
    }
  }
}

#[derive(Debug)]
pub struct PausedProcess {
  pub args: Option<ArgumentSpecifier>,
  pub state: ProcessState,
}
impl From<CompositeFunction> for PausedProcess {
  fn from(f: CompositeFunction) -> Self {
    Self {
      args: Some(f.args),
      state: ProcessState::new_with_root_frame(StackFrame::root(
        f.instructions,
      )),
    }
  }
}

#[derive(Debug)]
pub struct StackFrame {
  pub beginning: StackIndex,
  pub calling_function: Option<Rc<CompositeFunction>>,
  pub instructions: RuntimeInstructionBlock,
  pub instruction_index: usize,
  pub return_stack_index: StackIndex,
}
impl StackFrame {
  pub fn root(instructions: RuntimeInstructionBlock) -> Self {
    Self {
      beginning: 0,
      calling_function: None,
      instructions,
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
      instructions: f.instructions.clone(),
      instruction_index: 0,
      calling_function: Some(f),
      return_stack_index,
    }
  }
  pub fn next_instruction(&mut self) -> RuntimeInstruction {
    let instruction = self.instructions[self.instruction_index].clone();
    self.instruction_index += 1;
    instruction
  }
}
