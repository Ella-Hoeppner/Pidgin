use std::collections::HashMap;

use crate::{
  instructions::GenericInstruction::*,
  runtime::{core_functions::CoreFnId, data::GenericValue::*, vm::SymbolIndex},
};

use super::{
  super::{SSABlock, SSAInstruction, SSARegister, SSAValue},
  error::ASTError,
  expressions::Expression,
  token::SymbolLedger,
  tree::Tree,
};

pub fn push_constant(
  constant: SSAValue<()>,
  taken_virtual_registers: &mut usize,
  instructions: &mut Vec<SSAInstruction>,
  constants: &mut Vec<SSAValue<()>>,
) -> SSARegister {
  instructions.push(Const(*taken_virtual_registers, constants.len() as u16));
  *taken_virtual_registers += 1;
  constants.push(constant);
  *taken_virtual_registers - 1
}

pub fn build_expression_ir(
  expression: Expression,
  bindings: &HashMap<SymbolIndex, u8>,
  symbol_ledger: &mut SymbolLedger,
  taken_virtual_registers: &mut usize,
  instructions: &mut Vec<SSAInstruction>,
  constants: &mut Vec<SSAValue<()>>,
) -> Result<SSARegister, ASTError> {
  match expression {
    Expression::Literal(value) => {
      if let Symbol(symbol_index) = value {
        if let Some(binding_index) = bindings.get(&symbol_index) {
          Ok(*binding_index as SSARegister)
        } else {
          let symbol_name = symbol_ledger.symbol_name(&symbol_index).unwrap();
          if let Some(fn_id) = CoreFnId::from_name(&symbol_name) {
            Ok(push_constant(
              CoreFn(fn_id),
              taken_virtual_registers,
              instructions,
              constants,
            ))
          } else {
            Err(ASTError::UnboundSymbol(symbol_name.clone()))
          }
        }
      } else {
        Ok(push_constant(
          value,
          taken_virtual_registers,
          instructions,
          constants,
        ))
      }
    }
    Expression::Quoted(subexpression) => todo!(),
    Expression::Application(subexpressions) => {
      let mut subexpressions_iter = subexpressions.into_iter();
      let first_subexpression = subexpressions_iter.next().expect(
        "Encountered Expression::Application with no inner subexpressions \
        (this should never happen, as empty forms should become empty
        list literals)",
      );
      let arg_registers = subexpressions_iter
        .map(|arg| {
          build_expression_ir(
            arg,
            bindings,
            symbol_ledger,
            taken_virtual_registers,
            instructions,
            constants,
          )
        })
        .collect::<Result<Vec<SSARegister>, _>>()?;
      let f_register = build_expression_ir(
        first_subexpression,
        bindings,
        symbol_ledger,
        taken_virtual_registers,
        instructions,
        constants,
      )?;
      instructions.push(Call(
        *taken_virtual_registers,
        f_register,
        arg_registers.len() as u8,
      ));
      for arg_register in arg_registers {
        instructions.push(CopyArgument(arg_register))
      }
      *taken_virtual_registers += 1;
      Ok(*taken_virtual_registers - 1)
    }
    Expression::Function { arg_names, body } => {
      let mut new_bindings = bindings.clone();
      let arg_count = arg_names.len() as u8;
      for arg_name in arg_names {
        new_bindings.insert(arg_name, new_bindings.len() as u8);
      }
      match body.len() {
        0 => Err(ASTError::FunctionDefinitionMissingBody),
        1 => {
          let mut function_instructions = vec![];
          let mut function_constants = vec![];
          let function_return_register = build_expression_ir(
            body.into_iter().next().unwrap(),
            &new_bindings,
            symbol_ledger,
            &mut (arg_count.clone() as SSARegister),
            &mut function_instructions,
            &mut function_constants,
          )?;
          function_instructions.push(Return(function_return_register));
          let f = SSAValue::composite_fn(
            arg_count,
            SSABlock::new(function_instructions, function_constants),
          );
          Ok(push_constant(
            f,
            taken_virtual_registers,
            instructions,
            constants,
          ))
        }
        _ => {
          todo!("can't handle functions with multiple expressions in body yet")
        }
      }
    }
  }
}

pub fn ast_to_ir(ast: Tree<String>) -> Result<SSABlock<()>, ASTError> {
  let mut taken_virtual_registers = 0;
  let mut instructions = vec![];
  let mut constants = vec![];
  let mut symbol_ledger = SymbolLedger::default();
  let last_register = build_expression_ir(
    Expression::from_token_tree(ast.try_into()?, &mut symbol_ledger)?,
    &HashMap::new(),
    &mut symbol_ledger,
    &mut taken_virtual_registers,
    &mut instructions,
    &mut constants,
  )?;
  instructions.push(Return(last_register));
  Ok(SSABlock::new(instructions, constants))
}
