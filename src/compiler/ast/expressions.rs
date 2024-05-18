use crate::{compiler::SSAValue, runtime::vm::SymbolIndex};

use super::{
  error::ASTError,
  token::{SymbolLedger, Token, TokenTree},
  tree::Tree,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Expression {
  Literal(SSAValue<()>),
  Quoted(Box<Expression>),
  Application(Vec<Expression>),
  Function {
    arg_names: Vec<SymbolIndex>,
    body: Vec<Expression>,
  },
}
use itertools::Itertools;
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
          if let TokenTree::Leaf(Token::Symbol(first_symbol)) = &subtrees[0] {
            match first_symbol.as_str() {
              "fn" => {
                let mut subtrees_iter = subtrees.into_iter().skip(1);
                let maybe_arg_names = subtrees_iter.next();
                return if let Some(TokenTree::Inner(arg_names)) =
                  maybe_arg_names
                {
                  Ok(Function {
                    arg_names: arg_names
                      .into_iter()
                      .map(|arg_name_subtree| {
                        let arg_name_expression = Expression::from_token_tree(
                          arg_name_subtree,
                          symbol_ledger,
                        )?;
                        if let Literal(SSAValue::Symbol(
                          arg_name_symbol_index,
                        )) = arg_name_expression
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
                };
              }
              "quote" => {
                return if subtrees.len() == 2 {
                  Ok(Quoted(
                    Expression::from_token_tree(
                      subtrees.into_iter().skip(1).next().unwrap(),
                      symbol_ledger,
                    )?
                    .into(),
                  ))
                } else {
                  Err(ASTError::MultipleExpressionsInQuote)
                }
              }
              _ => (),
            }
          }
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
      Tree::Leaf(token) => Ok(Self::from_token(token, symbol_ledger)),
    }
  }

  fn unbound_internal_symbols(
    &self,
    bindings: &Vec<SymbolIndex>,
  ) -> Vec<SymbolIndex> {
    match self {
      Literal(value) => {
        if let SSAValue::Symbol(symbol) = value {
          if bindings.contains(symbol) {
            vec![]
          } else {
            vec![*symbol]
          }
        } else {
          vec![]
        }
      }
      Quoted(_subexpression) => vec![],
      Application(subexpressions) => subexpressions
        .iter()
        .flat_map(|subexpression| {
          subexpression.unbound_internal_symbols(bindings)
        })
        .collect(),
      Function { arg_names, body } => body
        .iter()
        .flat_map(|subexpression| {
          subexpression.unbound_internal_symbols(
            &bindings.iter().chain(arg_names.iter()).cloned().collect(),
          )
        })
        .collect(),
    }
  }

  fn replace_symbols(
    self,
    to_replace: &Vec<SymbolIndex>,
    symbol_ledger: &mut SymbolLedger,
    replacements: &mut Vec<(SymbolIndex, SymbolIndex)>,
  ) -> Self {
    match self {
      Literal(value) => {
        if let SSAValue::Symbol(original_symbol) = value {
          if to_replace.contains(&original_symbol) {
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
      Quoted(subexpression) => Quoted(subexpression),
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
  ) -> Result<Self, ASTError> {
    Ok(match self {
      Literal(value) => Literal(value),
      Quoted(subexpression) => Quoted(subexpression),
      Application(subexpressions) => Application(
        subexpressions
          .into_iter()
          .map(|subexpression| {
            subexpression.lift_lambdas(&parent_bindings, symbol_ledger)
          })
          .collect::<Result<_, _>>()?,
      ),
      Function { arg_names, body } => {
        let unbound_body_symbols: Vec<SymbolIndex> = body
          .iter()
          .flat_map(|body_expression| {
            body_expression.unbound_internal_symbols(&arg_names)
          })
          .unique()
          .filter(|body_symbol| !symbol_ledger.is_built_in(body_symbol))
          .collect();
        for unbound_body_symbol in unbound_body_symbols.iter() {
          if !parent_bindings.contains(unbound_body_symbol) {
            return Err(ASTError::UnboundSymbol(
              symbol_ledger
                .symbol_name(unbound_body_symbol)
                .unwrap()
                .clone(),
            ));
          }
        }
        if unbound_body_symbols.is_empty() {
          Function {
            body: body
              .into_iter()
              .map(|expression| {
                let new_bindings: Vec<SymbolIndex> = parent_bindings
                  .iter()
                  .chain(arg_names.iter())
                  .cloned()
                  .collect();
                expression.lift_lambdas(&new_bindings, symbol_ledger)
              })
              .collect::<Result<Vec<_>, _>>()?,
            arg_names,
          }
        } else {
          let mut replacements = vec![];
          let new_body = body
            .into_iter()
            .map(|expression| {
              let replaced_expression = expression.replace_symbols(
                &unbound_body_symbols,
                symbol_ledger,
                &mut replacements,
              );
              let new_bindings: Vec<SymbolIndex> = parent_bindings
                .iter()
                .map(|parent_binding| {
                  if let Some(replacement_binding) = replacements
                    .iter()
                    .filter_map(|(original_symbol, new_symbol)| {
                      (original_symbol == parent_binding).then(|| new_symbol)
                    })
                    .next()
                  {
                    replacement_binding
                  } else {
                    parent_binding
                  }
                })
                .chain(arg_names.iter())
                .cloned()
                .collect();
              replaced_expression.lift_lambdas(&new_bindings, symbol_ledger)
            })
            .collect::<Result<Vec<_>, _>>()?;
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
    })
  }

  pub(crate) fn to_string(&self, symbol_ledger: &SymbolLedger) -> String {
    match self {
      Literal(value) => value.description(Some(symbol_ledger)),
      Quoted(subexpression) => {
        format!("(quote {})", subexpression.to_string(symbol_ledger))
      }
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
  fn replace_symbols_ignores_quotes() {
    let mut symbol_ledger = SymbolLedger::default();
    let x_index = symbol_ledger.symbol_index("x".to_string());
    let replaced_expression = Expression::from_token_tree(
      parse_sexp("(x (quote x))").try_into().unwrap(),
      &mut symbol_ledger,
    )
    .unwrap()
    .replace_symbols(&vec![x_index], &mut symbol_ledger, &mut vec![]);
    assert_eq!(
      replaced_expression.to_string(&symbol_ledger),
      "(__gensym_0 (quote x))"
    )
  }

  #[test]
  fn lift_lambdas_leaves_single_function_alone() {
    let mut symbol_ledger = SymbolLedger::default();
    let lifted_expression = Expression::from_token_tree(
      parse_sexp("(fn (x) (* x x))").try_into().unwrap(),
      &mut symbol_ledger,
    )
    .unwrap()
    .lift_lambdas(&vec![], &mut symbol_ledger)
    .unwrap();
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
    .lift_lambdas(&vec![], &mut symbol_ledger)
    .unwrap();
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
    .lift_lambdas(&vec![], &mut symbol_ledger)
    .unwrap();
    assert_eq!(
      lifted_expression.to_string(&symbol_ledger),
      "(fn (x) \
         (partial (fn (__gensym_0 y) \
                    (partial (fn (__gensym_1 __gensym_2 z) \
                               (* __gensym_1 __gensym_2 z)) \
                             __gensym_0 \
                             y)) \
                  x))"
    );
  }

  #[test]
  fn lambda_lifting_ignores_quotes() {
    let mut symbol_ledger = SymbolLedger::default();
    let lifted_expression = Expression::from_token_tree(
      parse_sexp("(fn (x) (fn (y) (list (quote x) y)))")
        .try_into()
        .unwrap(),
      &mut symbol_ledger,
    )
    .unwrap()
    .lift_lambdas(&vec![], &mut symbol_ledger)
    .unwrap();
    assert_eq!(
      lifted_expression.to_string(&symbol_ledger),
      "(fn (x) (fn (y) (list (quote x) y)))"
    );
  }
}
