pub mod ast_to_ir;
pub mod parse;
pub mod transformations;

use crate::{blocks::GenericBlock, GenericValue, Instruction};

pub type SSARegister = usize;
pub type SSAInstruction =
  Instruction<SSARegister, SSARegister, (SSARegister, SSARegister)>;
pub type SSABlock<M> =
  GenericBlock<SSARegister, SSARegister, (SSARegister, SSARegister), M>;
pub type SSAValue<M> =
  GenericValue<SSARegister, SSARegister, (SSARegister, SSARegister), M>;
