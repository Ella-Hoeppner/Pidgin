use std::collections::HashMap;

use crate::{
  compiler::SSAValue,
  runtime::{data::GenericValue, vm::SymbolIndex},
};

use super::{error::ASTError, tree::Tree};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
  Nil,
  IntLiteral(i64),
  FloatLiteral(f64),
  StringLiteral(String),
  Symbol(String),
}

impl TryFrom<String> for Token {
  type Error = ASTError;

  fn try_from(s: String) -> Result<Self, ASTError> {
    use Token::*;
    if let Ok(i) = s.parse::<i64>() {
      Ok(IntLiteral(i))
    } else if let Ok(f) = s.parse::<f64>() {
      Ok(FloatLiteral(f))
    } else {
      if s.chars().nth(0) == Some('"') {
        if s.chars().last() == Some('"')
          && s.chars().filter(|c| *c == '"').count() == 2
        {
          Ok(StringLiteral(s.chars().skip(1).take(s.len() - 2).collect()))
        } else {
          Err(ASTError::CantParseToken(s))
        }
      } else {
        match s.as_str() {
          "nil" => Ok(Nil),
          _ => Ok(Symbol(s)),
        }
      }
    }
  }
}

impl TryFrom<&str> for Token {
  type Error = ASTError;

  fn try_from(s: &str) -> Result<Self, ASTError> {
    Token::try_from(s.to_string())
  }
}

pub type TokenTree = Tree<Token>;

impl TryFrom<Tree<String>> for TokenTree {
  type Error = ASTError;

  fn try_from(value: Tree<String>) -> Result<Self, Self::Error> {
    value.translate(Token::try_from)
  }
}

#[derive(Debug, Clone, Default)]
pub struct SymbolLedger {
  names_to_indeces: HashMap<String, SymbolIndex>,
  indeces_to_names: HashMap<SymbolIndex, String>,
}
impl SymbolLedger {
  pub(crate) fn symbol_index(&mut self, symbol: String) -> SymbolIndex {
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
  pub(crate) fn symbol_name(&self, index: SymbolIndex) -> Option<&String> {
    self.indeces_to_names.get(&index)
  }
}

pub fn token_to_value(
  symbol_ledger: &mut SymbolLedger,
  token: Token,
) -> SSAValue<()> {
  use GenericValue::*;
  match token {
    Token::Nil => Nil,
    Token::IntLiteral(i) => i.into(),
    Token::FloatLiteral(f) => f.into(),
    Token::StringLiteral(s) => s.into(),
    Token::Symbol(s) => Symbol(symbol_ledger.symbol_index(s)),
  }
}

mod tests {
  #![allow(unused_imports)]
  use super::Token;
  use Token::*;
  #[test]
  fn parse_int() {
    assert_eq!(Token::try_from("1"), Ok(IntLiteral(1)));
  }
  #[test]
  fn parse_float() {
    assert_eq!(Token::try_from("1."), Ok(FloatLiteral(1.)));
    assert_eq!(Token::try_from("-1.5".to_string()), Ok(FloatLiteral(-1.5)));
  }
  #[test]
  fn parse_nil() {
    assert_eq!(Token::try_from("nil"), Ok(Nil));
  }
  #[test]
  fn parse_symbol() {
    assert_eq!(Token::try_from("hello"), Ok(Symbol("hello".to_string())));
  }
  #[test]
  fn parse_string() {
    assert_eq!(
      Token::try_from("\"i'm a string!! :D\""),
      Ok(StringLiteral("i'm a string!! :D".to_string()))
    );
    assert!(Token::try_from("\"i'm a malformed string D:").is_err());
  }
}
