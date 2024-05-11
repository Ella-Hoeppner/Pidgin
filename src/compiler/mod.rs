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

  macro_rules! test_ir_and_bytecode_and_output {
    (
      $test_name:ident,
      $sexp:expr,
      $expected_ir:expr,
      $expected_bytecode:expr,
      $expected_output:expr
    ) => {
      #[test]
      fn $test_name() {
        let ir = expression_ast_to_ir(parse_sexp($sexp)).unwrap();
        assert_eq!(ir, $expected_ir);
        let bytecode = ir_to_bytecode(ir).unwrap();
        assert_eq!(bytecode, $expected_bytecode);
        let output = EvaluationState::new(bytecode).evaluate().unwrap();
        assert_eq!(output, Some($expected_output.into()));
      }
    };
  }

  test_ir_and_bytecode_and_output!(
    binary_addition,
    "(+ 1 2)",
    ssa_block![Const(0, 1), Const(1, 2), Add(2, 0, 1), Return(2)],
    block![Const(0, 1), Const(1, 2), Add(0, 0, 1), Return(0)],
    3
  );
}
