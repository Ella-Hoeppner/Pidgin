use std::{collections::HashMap, error::Error, fmt::Display};

use crate::{
  runtime::core_functions::CoreFnId, AritySpecifier, ConstIndex, GenericValue,
  Instruction, Num, Register, SymbolIndex,
};

use super::{
  parse::{CantParseTokenError, Token, TokenTree, Tree},
  SSABlock, SSAInstruction, SSARegister, SSAValue,
};

use GenericValue::*;
use Instruction::*;
use Tree::*;

#[derive(Debug, Clone)]
pub enum ASTError {
  Parse(CantParseTokenError),
  EmptyList,
  InvalidArity(CoreFnId, usize),
  InvalidFunctionDefintionArgumentList(Option<TokenTree>),
  InvalidFunctionDefintionArgument(TokenTree),
  FunctionDefinitionMissingBody,
  UnboundSymbol(String),
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
      InvalidArity(fn_id, given) => {
        write!(
          f,
          "invalid number of arguments {} for function {}",
          given,
          fn_id.name()
        )
      }
      InvalidFunctionDefintionArgumentList(arg_list) => {
        write!(
          f,
          "invalid argument list for function definition: {:?}",
          arg_list,
        )
      }
      InvalidFunctionDefintionArgument(arg) => {
        write!(
          f,
          "invalid argument name for function definition: {:?}",
          arg
        )
      }
      FunctionDefinitionMissingBody => {
        write!(f, "no body for function definition")
      }
      UnboundSymbol(symbol_name) => {
        write!(f, "encountered unbound symbol {symbol_name}")
      }
    }
  }
}
impl Error for ASTError {}

#[derive(Debug, Clone, Default)]
pub struct SymbolLedger {
  names_to_indeces: HashMap<String, SymbolIndex>,
  indeces_to_names: HashMap<SymbolIndex, String>,
}
impl SymbolLedger {
  fn symbol_index(&mut self, symbol: String) -> SymbolIndex {
    self
      .names_to_indeces
      .get(&symbol)
      .cloned()
      .unwrap_or_else(|| {
        let next_free_index = self.names_to_indeces.len() as u16;
        self
          .indeces_to_names
          .insert(next_free_index, symbol.clone());
        self.names_to_indeces.insert(symbol, next_free_index);
        next_free_index
      })
  }
  fn symbol_name(&self, index: SymbolIndex) -> Option<&String> {
    self.indeces_to_names.get(&index)
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

pub fn build_application_ir(
  f: TokenTree,
  args: Vec<SSARegister>,
  bindings: &HashMap<SymbolIndex, u8>,
  taken_virtual_registers: &mut usize,
  instructions: &mut Vec<SSAInstruction>,
  constants: &mut Vec<SSAValue<()>>,
) -> Result<SSARegister, ASTError> {
  match f {
    Inner(_) => todo!(),
    Leaf(leaf) => match leaf {
      Token::Symbol(symbol) => {
        if let Some(fn_id) = CoreFnId::from_name(&symbol) {
          instructions.push(Const(
            *taken_virtual_registers,
            constants.len() as ConstIndex,
          ));
          instructions.push(Call(
            *taken_virtual_registers + 1,
            *taken_virtual_registers,
            args.len() as u8,
          ));
          constants.push(CoreFn(fn_id));
          for arg_register in args {
            instructions.push(CopyArgument(arg_register))
          }
          *taken_virtual_registers += 2;
          Ok(*taken_virtual_registers - 1)
          /*use CoreFnId as F;
          if let Some(replacing_unary_instruction) = if args.len() == 1 {
            match fn_id {
              F::Rest => Some(Rest((args[0], *taken_virtual_registers))),
              F::ButLast => Some(ButLast((args[0], *taken_virtual_registers))),
              other => None,
            }
          } else {
            None
          } {
            instructions.push(replacing_unary_instruction);
            *taken_virtual_registers += 1;
            Ok(*taken_virtual_registers - 1)
          } else if let Some(nonreplacing_unary_instruction) =
            if args.len() == 1 {
              match fn_id {
                F::First => Some(First(*taken_virtual_registers, args[0])),
                F::Last => Some(Last(*taken_virtual_registers, args[0])),
                F::IsEmpty => Some(IsEmpty(*taken_virtual_registers, args[0])),
                other => None,
              }
            } else {
              None
            }
          {
            instructions.push(nonreplacing_unary_instruction);
            *taken_virtual_registers += 1;
            Ok(*taken_virtual_registers - 1)
          } else if let Some(replacing_binary_instruction) = if args.len() == 2
          {
            match fn_id {
              F::Push => {
                Some(Push((args[0], *taken_virtual_registers), args[1]))
              }
              F::Cons => {
                Some(Cons((args[0], *taken_virtual_registers), args[1]))
              }
              other => None,
            }
          } else {
            None
          } {
            instructions.push(replacing_binary_instruction);
            *taken_virtual_registers += 1;
            Ok(*taken_virtual_registers - 1)
          } else if let Some(nonreplacing_binary_instruction) =
            if args.len() == 2 {
              match fn_id {
                F::Add => Some(Add(*taken_virtual_registers, args[0], args[1])),
                F::Subtract => {
                  Some(Subtract(*taken_virtual_registers, args[0], args[1]))
                }
                F::Multiply => {
                  Some(Multiply(*taken_virtual_registers, args[0], args[1]))
                }
                F::Divide => {
                  Some(Divide(*taken_virtual_registers, args[0], args[1]))
                }
                other => None,
              }
            } else {
              None
            }
          {
            instructions.push(nonreplacing_binary_instruction);
            *taken_virtual_registers += 1;
            Ok(*taken_virtual_registers - 1)
          } else {
            let maybe_instruction_builder: Option<
              fn(SSARegister, SSARegister, SSARegister) -> SSAInstruction,
            > = match fn_id {
              F::Add => Some(|o, a, b| Add(o, a, b)),
              F::Multiply => Some(|o, a, b| Multiply(o, a, b)),
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
              match fn_id {
                F::CreateList => {
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
                _ => Err(ASTError::InvalidArity(fn_id, args.len())),
              }
            }
          }*/
        } else {
          todo!()
        }
      }
      _ => todo!(),
    },
  }
}

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
      if let Some(first_token) = list_iter.next() {
        if first_token == Leaf(Token::Symbol("fn".to_string())) {
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
          build_application_ir(
            first_token,
            arg_registers,
            bindings,
            taken_virtual_registers,
            instructions,
            constants,
          )
        }
      } else {
        Err(ASTError::EmptyList)
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
    ast.try_into().map_err(|e| ASTError::Parse(e))?,
    &HashMap::new(),
    &mut ledger,
    &mut taken_virtual_registers,
    &mut instructions,
    &mut constants,
  )?;
  instructions.push(Return(last_register));
  Ok(SSABlock::new(instructions, constants))
}
