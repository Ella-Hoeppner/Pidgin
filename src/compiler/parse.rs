use std::fmt::Display;

use crate::runtime::core_functions::CoreFnId;

#[derive(Debug, Clone, PartialEq)]
pub enum Tree<T> {
  Inner(Vec<Tree<T>>),
  Leaf(T),
}
use Tree::*;

impl<T> Tree<T> {
  pub fn translate<NewT, E>(
    self,
    translator: fn(T) -> Result<NewT, E>,
  ) -> Result<Tree<NewT>, E> {
    match self {
      Inner(subtrees) => Ok(Inner(
        subtrees
          .into_iter()
          .map(|subtree| subtree.translate(translator))
          .collect::<Result<Vec<Tree<NewT>>, E>>()?,
      )),
      Leaf(leaf) => translator(leaf).map(|x| Leaf(x)),
    }
  }
}

// the following code is purely temporary, as Pidgin will eventually use
// [GSE](https://github.com/Ella-Hoeppner/GSE) for parsing
fn tokenize(input: &str) -> Vec<String> {
  input
    .replace("(", " ( ")
    .replace(")", " ) ")
    .split_whitespace()
    .map(String::from)
    .collect()
}

pub fn parse_sexp(input: &str) -> Tree<String> {
  let mut ast_stack: Vec<Vec<Tree<String>>> = vec![vec![]];
  for token in tokenize(input) {
    match token.as_str() {
      "(" => ast_stack.push(vec![]),
      ")" => {
        let finished_list = ast_stack.pop().unwrap();
        let tree = Tree::Inner(finished_list);
        let l = ast_stack.len();
        ast_stack[l - 1].push(tree);
      }
      other => {
        let l = ast_stack.len();
        ast_stack[l - 1].push(Tree::Leaf(other.to_string()))
      }
    }
  }
  ast_stack.pop().unwrap().first().unwrap().clone()
}
// end temporary code

#[derive(Debug, Clone, PartialEq)]
pub struct CantParseTokenError(String);
impl Display for CantParseTokenError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "failed to parse token: \"{}\"", self.0)
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
  Nil,
  IntLiteral(i64),
  FloatLiteral(f64),
  StringLiteral(String),
  Symbol(String),
}

pub type TokenTree = Tree<Token>;

impl TryFrom<String> for Token {
  type Error = CantParseTokenError;

  fn try_from(s: String) -> Result<Self, CantParseTokenError> {
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
          Err(CantParseTokenError(s))
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
  type Error = CantParseTokenError;

  fn try_from(s: &str) -> Result<Self, CantParseTokenError> {
    Token::try_from(s.to_string())
  }
}

impl TryFrom<Tree<String>> for TokenTree {
  type Error = CantParseTokenError;

  fn try_from(value: Tree<String>) -> Result<Self, Self::Error> {
    value.translate(Token::try_from)
  }
}

mod test {
  use Token::*;

  use super::Token;
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
