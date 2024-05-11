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

mod tests {
  use program_macro::{block, ssa_block};

  use crate::{
    blocks::GenericBlock,
    compiler::{
      ast_to_ir::expression_ast_to_ir,
      parse::parse_sexp,
      transformations::{
        allocate_registers, ir_to_bytecode, track_register_lifetimes,
      },
      SSABlock,
    },
    Block, EvaluationState,
    GenericValue::{self, *},
    Instruction::*,
    Num::{self, *},
  };

  #[test]
  fn binary_addition() {
    let ir = expression_ast_to_ir(parse_sexp("(+ 1 2)")).unwrap();
    assert_eq!(
      ir,
      ssa_block![Const(0, 1), Const(1, 2), Add(2, 0, 1), Return(2)]
    );
    let bytecode = ir_to_bytecode(ir).unwrap();
    assert_eq!(
      bytecode,
      block![Const(0, 1), Const(1, 2), Add(0, 0, 1), Return(0)]
    );
    let output = EvaluationState::new(bytecode).evaluate().unwrap();
    assert_eq!(output, Some(3.into()));
  }
}
