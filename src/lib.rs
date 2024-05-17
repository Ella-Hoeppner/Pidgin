mod blocks;
mod compiler;
mod instructions;
mod runtime;
mod string_utils;

use runtime::error::PidginResult;

use crate::compiler::transformations::raw_ir_to_bytecode;

pub fn evaluate_pidgin_sexp(sexp: String) -> PidginResult<String> {
  use compiler::{ast_to_ir::expression_ast_to_ir, parse::parse_sexp};
  use runtime::vm::EvaluationState;
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
