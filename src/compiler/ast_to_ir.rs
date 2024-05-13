use std::{collections::HashMap, error::Error, fmt::Display};

use crate::{AritySpecifier, GenericValue, Instruction, Num, Register};

use super::{
  parse::{CantParseTokenError, Token, Tree},
  SSABlock, SSAInstruction, SSARegister, SSAValue,
};

use GenericValue::*;
use Instruction::*;
use Tree::*;

#[derive(Debug, Clone)]
pub enum ASTError {
  Parse(CantParseTokenError),
  EmptyList,
  UnrecognizedFunction(String, usize),
  InvalidArity(String, usize, AritySpecifier),
}
impl Display for ASTError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use ASTError::*;
    match self {
      Parse(e) => {
        write!(f, "{}", e)
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

#[derive(Debug, Clone, Default)]
pub struct SymbolLedger {
  assignments: HashMap<String, u16>,
}
impl SymbolLedger {
  fn symbol_index(&mut self, symbol: String) -> u16 {
    self.assignments.get(&symbol).cloned().unwrap_or_else(|| {
      let next_free_index = (self.assignments.len() - 1) as u16;
      self.assignments.insert(symbol, next_free_index);
      next_free_index
    })
  }
}

pub fn token_to_value(
  symbol_ledger: &mut SymbolLedger,
  token: Token,
) -> SSAValue<()> {
  match token {
    Token::Nil => Nil,
    Token::IntLiteral(i) => i.into(),
    Token::FloatLiteral(f) => f.into(),
    Token::StringLiteral(s) => s.into(),
    Token::Symbol(s) => Symbol(symbol_ledger.symbol_index(s)),
  }
}

impl From<Token> for SSAValue<()> {
  fn from(token: Token) -> Self {
    match token {
      Token::Nil => Nil,
      Token::IntLiteral(i) => i.into(),
      Token::FloatLiteral(f) => f.into(),
      Token::StringLiteral(s) => s.into(),
      Token::Symbol(s) => todo!(),
    }
  }
}

pub fn build_ir_from_fn_application(
  f: Tree<Token>,
  args: Vec<SSARegister>,
  taken_virtual_registers: &mut usize,
  instructions: &mut Vec<SSAInstruction>,
  constants: &mut Vec<SSAValue<()>>,
) -> Result<SSARegister, ASTError> {
  match f {
    Inner(_) => todo!(),
    Leaf(f) => match f {
      Token::Symbol(fn_name) => {
        if let Some(replacing_unary_instruction) = if args.len() == 1 {
          match fn_name.as_str() {
            "rest" => Some(Rest((args[0], *taken_virtual_registers))),
            "butlast" => Some(ButLast((args[0], *taken_virtual_registers))),
            other => None,
          }
        } else {
          None
        } {
          instructions.push(replacing_unary_instruction);
          *taken_virtual_registers += 1;
          Ok(*taken_virtual_registers - 1)
        } else if let Some(nonreplacing_unary_instruction) = if args.len() == 1
        {
          match fn_name.as_str() {
            "first" => Some(First(*taken_virtual_registers, args[0])),
            "last" => Some(Last(*taken_virtual_registers, args[0])),
            "empty?" => Some(IsEmpty(*taken_virtual_registers, args[0])),
            other => None,
          }
        } else {
          None
        } {
          instructions.push(nonreplacing_unary_instruction);
          *taken_virtual_registers += 1;
          Ok(*taken_virtual_registers - 1)
        } else if let Some(replacing_binary_instruction) = if args.len() == 2 {
          match fn_name.as_str() {
            "push" => Some(Push((args[0], *taken_virtual_registers), args[1])),
            "cons" => Some(Cons((args[0], *taken_virtual_registers), args[1])),
            other => None,
          }
        } else {
          None
        } {
          instructions.push(replacing_binary_instruction);
          *taken_virtual_registers += 1;
          Ok(*taken_virtual_registers - 1)
        } else if let Some(nonreplacing_binary_instruction) = if args.len() == 2
        {
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
          instructions.push(nonreplacing_binary_instruction);
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
      _ => todo!(),
    },
  }
}

pub fn build_ir_from_ast(
  ast: Tree<Token>,
  symbol_ledger: &mut SymbolLedger,
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
              symbol_ledger,
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
      constants.push(token_to_value(symbol_ledger, s));
      Ok(*taken_virtual_registers - 1)
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
  let last_register = build_ir_from_ast(
    ast.try_into().map_err(|e| ASTError::Parse(e))?,
    &mut ledger,
    &mut taken_virtual_registers,
    &mut instructions,
    &mut constants,
  )?;
  instructions.push(Return(last_register));
  Ok(SSABlock::new(instructions, constants))
}
