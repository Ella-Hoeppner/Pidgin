use crate::{compiler::SSAValue, runtime::vm::SymbolIndex};

use super::{
  error::ASTError,
  token::{SymbolLedger, Token, TokenTree},
  tree::Tree,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Expression {
  Literal(SSAValue<()>),
  Application(Vec<Expression>),
  Function {
    arg_names: Vec<SymbolIndex>,
    body: Vec<Expression>,
  },
}

impl Expression {
  pub(crate) fn from_token_tree(
    token_tree: TokenTree,
    symbol_ledger: &mut SymbolLedger,
  ) -> Result<Self, ASTError> {
    use Expression::*;
    match token_tree {
      Tree::Inner(subtrees) => {
        if subtrees.len() == 0 {
          Ok(Literal(SSAValue::List(vec![].into())))
        } else {
          if subtrees[0] == TokenTree::Leaf(Token::Symbol("fn".to_string())) {
            let mut subtrees_iter = subtrees.into_iter().skip(1);
            let maybe_arg_names = subtrees_iter.next();
            if let Some(TokenTree::Inner(arg_names)) = maybe_arg_names {
              Ok(Function {
                arg_names: arg_names
                  .into_iter()
                  .map(|arg_name_subtree| {
                    let arg_name_expression = Expression::from_token_tree(
                      arg_name_subtree,
                      symbol_ledger,
                    )?;
                    if let Literal(SSAValue::Symbol(arg_name_symbol_index)) =
                      arg_name_expression
                    {
                      Ok(arg_name_symbol_index)
                    } else {
                      Err(ASTError::InvalidFunctionDefintionArgumentName(
                        arg_name_expression,
                      ))
                    }
                  })
                  .collect::<Result<_, _>>()?,
                body: subtrees_iter
                  .map(|body_subtree| {
                    Expression::from_token_tree(body_subtree, symbol_ledger)
                  })
                  .collect::<Result<_, _>>()?,
              })
            } else {
              Err(ASTError::InvalidFunctionDefintionArgumentNameList(
                maybe_arg_names,
              ))
            }
          } else {
            Ok(Application(
              subtrees
                .into_iter()
                .map(|subtree| {
                  Expression::from_token_tree(subtree, symbol_ledger)
                })
                .collect::<Result<_, _>>()?,
            ))
          }
        }
      }
      Tree::Leaf(token) => Ok(match token {
        Token::Nil => Literal(SSAValue::Nil),
        Token::IntLiteral(i) => Literal(i.into()),
        Token::FloatLiteral(f) => Literal(f.into()),
        Token::StringLiteral(s) => Literal(s.into()),
        Token::Symbol(s) => {
          Literal(SSAValue::Symbol(symbol_ledger.symbol_index(s)))
        }
      }),
    }
  }
}
