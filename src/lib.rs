mod blocks;
mod compiler;
mod instructions;
mod runtime;
mod string_utils;

use runtime::error::PidginResult;

pub fn evaluate_pidgin_sexp(sexp: String) -> PidginResult<String> {
  use compiler::{
    ast_to_ir::expression_ast_to_ir,
    parse::parse_sexp,
    transformations::{
      cleanup::erase_unused_constants, core_inlining::inline_core_fn_calls,
      lifetimes::track_register_lifetimes,
      register_allocation::allocate_registers,
    },
  };
  use runtime::vm::EvaluationState;
  let raw_ir = expression_ast_to_ir(parse_sexp(&sexp)).unwrap();
  let bytecode = allocate_registers(
    track_register_lifetimes(
      erase_unused_constants(inline_core_fn_calls(raw_ir).unwrap()).unwrap(),
    )
    .unwrap(),
  )
  .unwrap();
  let mut state = EvaluationState::new(bytecode);
  let result = state.evaluate();
  result.map(|maybe_value| {
    maybe_value
      .map(|value| value.to_string())
      .unwrap_or("nil".to_string())
  })
}
