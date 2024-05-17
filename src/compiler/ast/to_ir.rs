use std::collections::HashMap;

use crate::{
  instructions::GenericInstruction::*,
  runtime::{core_functions::CoreFnId, data::GenericValue::*, vm::SymbolIndex},
};

use super::{
  super::{SSABlock, SSAInstruction, SSARegister, SSAValue},
  error::ASTError,
  token::{token_to_value, SymbolLedger, Token, TokenTree},
  tree::Tree::{self, *},
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

pub fn build_ast_ir(
  ast: TokenTree,
  bindings: &HashMap<SymbolIndex, u8>,
  symbol_ledger: &mut SymbolLedger,
  taken_virtual_registers: &mut usize,
  instructions: &mut Vec<SSAInstruction>,
  constants: &mut Vec<SSAValue<()>>,
) -> Result<SSARegister, ASTError> {
  match ast {
    Inner(list) => {
      let mut list_iter = list.into_iter();
      if let Some(first_subtree) = list_iter.next() {
        if first_subtree == Leaf(Token::Symbol("fn".to_string())) {
          let args = list_iter.next();
          let mut new_bindings = bindings.clone();
          if let Some(Inner(arg_tokens)) = args {
            let arg_count = arg_tokens.len() as u8;
            for arg_token in arg_tokens {
              if let Leaf(Token::Symbol(arg_name)) = arg_token {
                new_bindings.insert(
                  symbol_ledger.symbol_index(arg_name),
                  new_bindings.len() as u8,
                );
              } else {
                return Err(ASTError::InvalidFunctionDefintionArgument(
                  arg_token,
                ));
              }
            }
            if let Some(body) = list_iter.next() {
              let mut function_instructions = vec![];
              let mut function_constants = vec![];
              let function_return_register = build_ast_ir(
                body,
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
            } else {
              return Err(ASTError::FunctionDefinitionMissingBody);
            }
          } else {
            return Err(ASTError::InvalidFunctionDefintionArgumentList(args));
          }
        } else {
          let arg_registers = list_iter
            .map(|arg| {
              build_ast_ir(
                arg,
                bindings,
                symbol_ledger,
                taken_virtual_registers,
                instructions,
                constants,
              )
            })
            .collect::<Result<Vec<SSARegister>, _>>()?;
          let f_register = build_ast_ir(
            first_subtree,
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
      } else {
        instructions.push(EmptyList(*taken_virtual_registers));
        *taken_virtual_registers += 1;
        Ok(*taken_virtual_registers - 1)
      }
    }
    Leaf(s) => {
      let value = token_to_value(symbol_ledger, s.clone());
      if let Symbol(symbol_index) = value {
        if let Some(binding_index) = bindings.get(&symbol_index) {
          Ok(*binding_index as SSARegister)
        } else {
          let symbol_name = symbol_ledger.symbol_name(symbol_index).unwrap();
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
  }
}

pub fn expression_ast_to_ir(
  ast: Tree<String>,
) -> Result<SSABlock<()>, ASTError> {
  let mut taken_virtual_registers = 0;
  let mut instructions = vec![];
  let mut constants = vec![];
  let mut ledger = SymbolLedger::default();
  let last_register = build_ast_ir(
    ast.try_into()?,
    &HashMap::new(),
    &mut ledger,
    &mut taken_virtual_registers,
    &mut instructions,
    &mut constants,
  )?;
  instructions.push(Return(last_register));
  Ok(SSABlock::new(instructions, constants))
}
