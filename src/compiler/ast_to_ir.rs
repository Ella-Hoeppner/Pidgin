use std::{error::Error, fmt::Display};

use crate::{AritySpecifier, GenericValue, Instruction, Num, Register};

use super::{SSABlock, SSAInstruction, SSARegister, SSAValue};

use GenericValue::*;
use Instruction::*;

#[derive(Debug, Clone)]
pub enum ASTError {
  CantParseToken(String),
  EmptyList,
  UnrecognizedFunction(String, usize),
  InvalidArity(String, usize, AritySpecifier),
}
impl Display for ASTError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use ASTError::*;
    match self {
      CantParseToken(token) => {
        write!(f, "can't parse token \"{}\"", token)
      }
      EmptyList => {
        write!(f, "found empty when parsing AST")
      }
      UnrecognizedFunction(fn_name, arity) => {
        write!(
          f,
          "unrecognized function name \"{}\" for arity {}",
          fn_name, arity
        )
      }
      InvalidArity(fn_name, given, expected) => {
        write!(
          f,
          "invalid number of arguments {} for function {}, expected {}",
          given, fn_name, expected
        )
      }
    }
  }
}
impl Error for ASTError {}

#[derive(Debug, Clone)]
pub enum AST {
  Inner(Vec<AST>),
  Leaf(String),
}
use AST::*;

pub fn token_to_value(token: String) -> Result<SSAValue<()>, ASTError> {
  if let Ok(i) = token.parse::<i64>() {
    Ok(Number(Num::Int(i)))
  } else if let Ok(f) = token.parse::<f64>() {
    Ok(Number(Num::Float(f.into())))
  } else {
    Err(ASTError::CantParseToken(token))
  }
}

pub fn build_ir_from_fn_application(
  f: AST,
  args: Vec<SSARegister>,
  taken_virtual_registers: &mut usize,
  instructions: &mut Vec<SSAInstruction>,
  constants: &mut Vec<SSAValue<()>>,
) -> Result<SSARegister, ASTError> {
  match f {
    Inner(_) => todo!(),
    Leaf(fn_name) => {
      if let Some(instruction) = if args.len() == 2 {
        match fn_name.as_str() {
          "+" => Some(Add(*taken_virtual_registers, args[0], args[1])),
          "-" => Some(Subtract(*taken_virtual_registers, args[0], args[1])),
          "*" => Some(Multiply(*taken_virtual_registers, args[0], args[1])),
          "/" => Some(Divide(*taken_virtual_registers, args[0], args[1])),
          other => None,
        }
      } else {
        None
      } {
        instructions.push(instruction);
        *taken_virtual_registers += 1;
        Ok(*taken_virtual_registers - 1)
      } else {
        let maybe_instruction_builder: Option<
          fn(SSARegister, SSARegister, SSARegister) -> SSAInstruction,
        > = match fn_name.as_str() {
          "+" => Some(|o, a, b| Add(o, a, b)),
          "*" => Some(|o, a, b| Multiply(o, a, b)),
          _ => None,
        };
        if let Some(instruction_builder) = maybe_instruction_builder {
          instructions.push(instruction_builder(
            *taken_virtual_registers,
            args[0],
            args[1],
          ));
          *taken_virtual_registers += 1;
          for i in 2..args.len() {
            instructions.push(instruction_builder(
              *taken_virtual_registers,
              *taken_virtual_registers - 1,
              args[i],
            ));
            *taken_virtual_registers += 1;
          }
          Ok(*taken_virtual_registers - 1)
        } else {
          match fn_name.as_str() {
            "list" => {
              instructions.push(EmptyList(*taken_virtual_registers));
              *taken_virtual_registers += 1;
              for i in 0..args.len() {
                instructions.push(Push(
                  (*taken_virtual_registers - 1, *taken_virtual_registers),
                  args[i],
                ));
                *taken_virtual_registers += 1;
              }
              Ok(*taken_virtual_registers - 1)
            }
            _ => Err(ASTError::UnrecognizedFunction(fn_name, args.len())),
          }
        }
      }
    }
  }
}

pub fn build_ir_from_ast(
  ast: AST,
  taken_virtual_registers: &mut usize,
  instructions: &mut Vec<SSAInstruction>,
  constants: &mut Vec<SSAValue<()>>,
) -> Result<SSARegister, ASTError> {
  match ast {
    Inner(list) => {
      let mut iter = list.into_iter();
      if let Some(f) = iter.next() {
        let arg_registers = iter
          .map(|arg| {
            build_ir_from_ast(
              arg,
              taken_virtual_registers,
              instructions,
              constants,
            )
          })
          .collect::<Result<Vec<SSARegister>, _>>()?;
        build_ir_from_fn_application(
          f,
          arg_registers,
          taken_virtual_registers,
          instructions,
          constants,
        )
      } else {
        Err(ASTError::EmptyList)
      }
    }
    Leaf(s) => {
      instructions
        .push(Const(*taken_virtual_registers, constants.len() as u16));
      *taken_virtual_registers += 1;
      constants.push(token_to_value(s)?);
      Ok(*taken_virtual_registers - 1)
    }
  }
}

pub fn expression_ast_to_ir(ast: AST) -> Result<SSABlock<()>, ASTError> {
  let mut taken_virtual_registers = 0;
  let mut instructions = vec![];
  let mut constants = vec![];
  let last_register = build_ir_from_ast(
    ast,
    &mut taken_virtual_registers,
    &mut instructions,
    &mut constants,
  )?;
  instructions.push(Return(last_register));
  Ok(SSABlock::new(instructions, constants))
}
