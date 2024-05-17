mod blocks;
mod compiler;
mod instructions;
mod runtime;
mod string_utils;

pub use instructions::GenericInstruction;
pub use runtime::{
  control::Block, data::Value, error::PidginResult, vm::EvaluationState,
};

pub fn evaluate_pidgin_sexp(sexp: String) -> PidginResult<String> {
  use crate::compiler::transformations::raw_ir_to_bytecode;
  use compiler::{ast_to_ir::expression_ast_to_ir, parse::parse_sexp};
  let raw_ir = expression_ast_to_ir(parse_sexp(&sexp)).unwrap();
  let bytecode = raw_ir_to_bytecode(raw_ir).unwrap();
  let mut state = EvaluationState::new(bytecode);
  let result = state.evaluate();
  result.map(|maybe_value| {
    maybe_value
      .map(|value| value.to_string())
      .unwrap_or("nil".to_string())
  })
}
