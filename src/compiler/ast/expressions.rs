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
use Expression::*;

impl Expression {
  fn from_token(token: Token, symbol_ledger: &mut SymbolLedger) -> Self {
    Literal(match token {
      Token::Nil => SSAValue::Nil,
      Token::IntLiteral(i) => i.into(),
      Token::FloatLiteral(f) => f.into(),
      Token::StringLiteral(s) => s.into(),
      Token::Symbol(s) => SSAValue::Symbol(symbol_ledger.symbol_index(s)),
    })
  }
  pub(crate) fn from_token_tree(
    token_tree: TokenTree,
    symbol_ledger: &mut SymbolLedger,
  ) -> Result<Self, ASTError> {
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
      Tree::Leaf(token) => Ok(Self::from_token(token, symbol_ledger)),
    }
  }

  fn replace_symbols(
    self,
    to_replace: &Vec<SymbolIndex>,
    symbol_ledger: &mut SymbolLedger,
    replacements: &mut Vec<(SymbolIndex, SymbolIndex)>,
  ) -> Self {
    println!("trying to replace symbols... {to_replace:?}");
    match self {
      Literal(value) => {
        if let SSAValue::Symbol(original_symbol) = value {
          println!(
            "symbol {}: {:?}",
            original_symbol,
            symbol_ledger.symbol_name(&original_symbol)
          );
          if to_replace.contains(&original_symbol) {
            println!("replacing symbol!!!!!!!!!!!!!!!!!!!");
            Literal(SSAValue::Symbol(
              replacements
                .iter()
                .filter_map(|(lifted_symbol, replacement_symbol)| {
                  (*lifted_symbol == original_symbol)
                    .then(|| *replacement_symbol)
                })
                .next()
                .unwrap_or_else(|| {
                  let replacement_symbol =
                    symbol_ledger.generate_unique_symbol();
                  replacements.push((original_symbol, replacement_symbol));
                  replacement_symbol
                }),
            ))
          } else {
            Literal(SSAValue::Symbol(original_symbol))
          }
        } else {
          Literal(value)
        }
      }
      Application(subexpressions) => Application(
        subexpressions
          .into_iter()
          .map(|subexpression| {
            subexpression.replace_symbols(
              to_replace,
              symbol_ledger,
              replacements,
            )
          })
          .collect(),
      ),
      Function { arg_names, body } => Function {
        arg_names,
        body: body
          .into_iter()
          .map(|body_expression| {
            body_expression.replace_symbols(
              to_replace,
              symbol_ledger,
              replacements,
            )
          })
          .collect(),
      },
    }
  }

  pub(crate) fn lift_lambdas(
    self,
    parent_bindings: &Vec<SymbolIndex>,
    symbol_ledger: &mut SymbolLedger,
  ) -> Self {
    match self {
      Literal(value) => Literal(value),
      Application(subexpressions) => Application(
        subexpressions
          .into_iter()
          .map(|subexpression| {
            subexpression.lift_lambdas(&parent_bindings, symbol_ledger)
          })
          .collect(),
      ),
      Function { arg_names, body } => {
        println!("lifting function!!");
        let mut replacements = vec![];
        let new_body = body
          .into_iter()
          .map(|expression| {
            println!("lifting and replacing symbols in body");
            let new_bindings: Vec<SymbolIndex> = parent_bindings
              .iter()
              .chain(arg_names.iter())
              .cloned()
              .collect();
            expression
              .replace_symbols(
                &parent_bindings,
                symbol_ledger,
                &mut replacements,
              )
              .lift_lambdas(&new_bindings, symbol_ledger)
          })
          .collect::<Vec<_>>();
        if replacements.is_empty() {
          Function {
            arg_names,
            body: new_body,
          }
        } else {
          Expression::Application(
            std::iter::once(Expression::from_token(
              Token::Symbol("partial".to_string()),
              symbol_ledger,
            ))
            .chain(std::iter::once(Expression::Function {
              arg_names: replacements
                .iter()
                .map(|(_, replacement_symbol)| *replacement_symbol)
                .chain(arg_names.into_iter())
                .collect(),
              body: new_body,
            }))
            .chain(replacements.iter().map(|(lifted_symbol, _)| {
              Expression::Literal(SSAValue::Symbol(*lifted_symbol))
            }))
            .collect(),
          )
        }
      }
    }
  }

  pub(crate) fn to_string(&self, symbol_ledger: &SymbolLedger) -> String {
    match self {
      Literal(value) => value.description(Some(symbol_ledger)),
      Application(subexpressions) => format!(
        "({})",
        subexpressions
          .iter()
          .map(|subexpression| subexpression.to_string(symbol_ledger))
          .collect::<Vec<String>>()
          .join(" ")
      ),
      Function { arg_names, body } => format!(
        "(fn ({}) {})",
        arg_names
          .iter()
          .map(|arg_name| symbol_ledger
            .symbol_name(arg_name)
            .expect("symbol ledger didn't contain a name for symbol")
            .clone())
          .collect::<Vec<String>>()
          .join(" "),
        body
          .iter()
          .map(|body_expression| body_expression.to_string(symbol_ledger))
          .collect::<Vec<String>>()
          .join(" ")
      ),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::Expression;
  use crate::compiler::ast::{parse::parse_sexp, token::SymbolLedger};

  #[test]
  fn replace_symbol() {
    let mut symbol_ledger = SymbolLedger::default();
    let x_index = symbol_ledger.symbol_index("x".to_string());
    let replaced_expression = Expression::from_token_tree(
      parse_sexp("x").try_into().unwrap(),
      &mut symbol_ledger,
    )
    .unwrap()
    .replace_symbols(&vec![x_index], &mut symbol_ledger, &mut vec![]);
    assert_eq!(replaced_expression.to_string(&symbol_ledger), "__gensym_0")
  }

  #[test]
  fn lift_lambdas_leaves_single_function_alone() {
    let mut symbol_ledger = SymbolLedger::default();
    let lifted_expression = Expression::from_token_tree(
      parse_sexp("(fn (x) (* x x))").try_into().unwrap(),
      &mut symbol_ledger,
    )
    .unwrap()
    .lift_lambdas(&vec![], &mut symbol_ledger);
    assert_eq!(
      lifted_expression.to_string(&symbol_ledger),
      "(fn (x) (* x x))"
    );
  }

  #[test]
  fn lift_single_lambda() {
    let mut symbol_ledger = SymbolLedger::default();
    let lifted_expression = Expression::from_token_tree(
      parse_sexp("(fn (x) (fn (y) (* x y)))").try_into().unwrap(),
      &mut symbol_ledger,
    )
    .unwrap()
    .lift_lambdas(&vec![], &mut symbol_ledger);
    assert_eq!(
      lifted_expression.to_string(&symbol_ledger),
      "(fn (x) (partial (fn (__gensym_0 y) (* __gensym_0 y)) x))"
    );
  }

  #[test]
  fn lift_nested_lambda() {
    let mut symbol_ledger = SymbolLedger::default();
    let lifted_expression = Expression::from_token_tree(
      parse_sexp("(fn (x) (fn (y) (fn (z) (* x y z))))")
        .try_into()
        .unwrap(),
      &mut symbol_ledger,
    )
    .unwrap()
    .lift_lambdas(&vec![], &mut symbol_ledger);
    assert_eq!(
      lifted_expression.to_string(&symbol_ledger),
      "(fn (x) \
         (partial (fn (__gensym_0 y) \
                    (partial (fn (__gensym_2 __gensym_1 z) \
                               (* __gensym_2 __gensym_1 z)) \
                             __gensym_0 \
                             y)) \
                  x))"
    );
  }
}
