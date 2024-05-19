use std::collections::HashMap;

use crate::{
  compiler::{
    ast::{
      expressions::Expression,
      parse::parse_sexp,
      to_ir::build_expression_ir,
      token::{SymbolLedger, TokenTree},
    },
    intermediate::raw_ir_to_bytecode,
    SSABlock, SSAValue,
  },
  instructions::GenericInstruction,
  runtime::{
    control::Block,
    data::Value,
    error::RuntimeResult,
    evaluation::{EvaluationState, SymbolIndex},
  },
};

use super::error::PidginResult;

#[derive(Default)]
pub(crate) struct Evaluator {
  symbol_ledger: SymbolLedger,
  global_environment: HashMap<SymbolIndex, Value>,
}

impl Evaluator {
  pub fn describe(&self, value: Value) -> String {
    value.description(Some(&self.symbol_ledger))
  }
  fn parse(&mut self, expression_string: &str) -> PidginResult<Expression> {
    Ok(Expression::from_token_tree(
      TokenTree::try_from(parse_sexp(expression_string))?,
      &mut self.symbol_ledger,
    )?)
  }
  fn compile_ast_to_ir(
    &mut self,
    expression: Expression,
  ) -> PidginResult<SSABlock<()>> {
    let mut instructions = vec![];
    let mut constants = vec![];
    let last_register = build_expression_ir(
      expression,
      &|symbol| self.global_environment.contains_key(&symbol),
      &HashMap::new(),
      &mut self.symbol_ledger,
      &mut 0,
      &mut instructions,
      &mut constants,
    )?;
    instructions.push(GenericInstruction::Return(last_register));
    Ok(SSABlock::new(instructions, constants))
  }
  fn compile_ir_to_bytecode(
    &mut self,
    ir: SSABlock<()>,
  ) -> PidginResult<Block> {
    Ok(raw_ir_to_bytecode(ir)?)
  }
  fn eval_bytecode(&self, block: Block) -> RuntimeResult<Option<Value>> {
    EvaluationState::new(block).evaluate(&self.global_environment)
  }
  pub fn get_binding(&mut self, name: &str) -> Option<&Value> {
    let symbol_index = self.symbol_ledger.symbol_index(name.to_string());
    self.global_environment.get(&symbol_index)
  }
  pub fn eval(
    &mut self,
    expression_string: &str,
  ) -> PidginResult<Option<Value>> {
    let expression = self.parse(expression_string)?;
    if let Some((name, value_expression)) =
      expression.as_definition(&self.symbol_ledger)?
    {
      let ir = self.compile_ast_to_ir(value_expression)?;
      let bytecode = self.compile_ir_to_bytecode(ir)?;
      let value = self.eval_bytecode(bytecode)?;
      self
        .global_environment
        .insert(name, value.clone().unwrap_or(Value::Nil));
      Ok(value)
    } else {
      let ir = self.compile_ast_to_ir(expression)?;
      let bytecode = self.compile_ir_to_bytecode(ir)?;
      Ok(self.eval_bytecode(bytecode)?)
    }
  }
}
