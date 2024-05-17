use std::{error::Error, fmt::Display};

use super::{expressions::Expression, token::TokenTree};

#[derive(Debug, Clone, PartialEq)]
pub enum ASTError {
  CantParseToken(String),
  InvalidFunctionDefintionArgumentNameList(Option<TokenTree>),
  InvalidFunctionDefintionArgumentName(Expression),
  FunctionDefinitionMissingBody,
  UnboundSymbol(String),
}
impl Display for ASTError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use ASTError::*;
    match self {
      CantParseToken(s) => {
        write!(f, "failed to parse token: \"{}\"", s)
      }
      InvalidFunctionDefintionArgumentNameList(arg_list) => {
        write!(
          f,
          "invalid argument list for function definition: {:?}",
          arg_list,
        )
      }
      InvalidFunctionDefintionArgumentName(arg) => {
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
