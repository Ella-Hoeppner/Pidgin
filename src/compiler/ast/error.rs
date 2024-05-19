use std::{error::Error, fmt::Display};

use super::expressions::{Expression, LiteralTree};

#[derive(Debug, Clone, PartialEq)]
pub enum ASTError {
  CantParseToken(String),
  InvalidFunctionDefintionArgumentNameList(Option<LiteralTree>),
  InvalidFunctionDefintionArgumentName(Expression),
  FunctionDefinitionMissingBody,
  UnboundSymbol(String),
  MultipleExpressionsInQuote,
  MultipleExpressionsInHardQuote,
  MultipleExpressionsInUnquote,
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
      MultipleExpressionsInQuote => {
        write!(f, "multiple subexpressions found in (quote ...) form")
      }
      MultipleExpressionsInHardQuote => {
        write!(f, "multiple subexpressions found in (hard-quote ...) form")
      }
      MultipleExpressionsInUnquote => {
        write!(f, "multiple subexpressions found in (unquote ...) form")
      }
    }
  }
}
impl Error for ASTError {}
pub(crate) type ASTResult<T> = Result<T, ASTError>;
