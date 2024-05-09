pub mod ast_to_ir;
pub mod parse;
pub mod transformations;

use crate::{GeneralizedBlock, GeneralizedValue, Instruction};

pub type SSARegister = usize;
pub type SSAInstruction = Instruction<SSARegister, (SSARegister, SSARegister)>;
pub type SSABlock<M> =
  GeneralizedBlock<SSARegister, (SSARegister, SSARegister), M>;
pub type SSAValue<M> =
  GeneralizedValue<SSARegister, (SSARegister, SSARegister), M>;
