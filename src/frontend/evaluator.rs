use std::collections::HashMap;

use crate::{
  compiler::{
    ast::{parse::parse_sexp, to_ir::ast_to_ir, token::SymbolLedger},
    intermediate::raw_ir_to_bytecode,
  },
  runtime::evaluation::SymbolIndex,
  Block, EvaluationState, RuntimeResult, Value,
};

use super::error::{to_pidgin_result, PidginResult};

#[derive(Default)]
pub(crate) struct Evaluator {
  symbol_legder: SymbolLedger,
  global_environment: HashMap<SymbolIndex, Value>,
}

impl Evaluator {
  fn compile(&mut self, expression: &str) -> PidginResult<Block> {
    to_pidgin_result(raw_ir_to_bytecode(to_pidgin_result(ast_to_ir(
      parse_sexp(expression),
      &mut self.symbol_legder,
    ))?))
  }
  fn eval_block(&self, block: Block) -> RuntimeResult<Option<Value>> {
    EvaluationState::new(block).evaluate()
  }
  pub fn eval(&mut self, expression: &str) -> PidginResult<Option<Value>> {
    let block = self.compile(expression)?;
    to_pidgin_result(self.eval_block(block))
  }
}
