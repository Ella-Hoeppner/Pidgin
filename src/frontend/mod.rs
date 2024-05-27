pub mod error;
pub mod evaluator;

#[cfg(test)]
mod tests {
  use crate::{
    compiler::ast::error::ASTError,
    frontend::error::PidginError,
    runtime::{data::Value, evaluation},
  };

  use super::evaluator::Evaluator;

  fn assert_eval_eq<V: Into<Value>>(expr: &str, expected_value: V) {
    let mut evaluator = Evaluator::default();
    assert_eq!(evaluator.eval(expr), Ok(expected_value.into()))
  }

  #[test]
  fn evaluate_int() {
    assert_eval_eq("1", 1);
  }

  #[test]
  fn evaluate_float() {
    assert_eval_eq("1.", 1f64);
  }

  #[test]
  fn evaluate_addition() {
    assert_eval_eq("(+ 1 2)", 3);
  }

  #[test]
  fn evaluate_def() {
    let mut evaluator = Evaluator::default();
    evaluator.eval("(def x 5)").unwrap();
    assert_eq!(evaluator.get_binding("x"), Some(&5.into()))
  }

  #[test]
  fn evaluate_def_and_usage() {
    let mut evaluator = Evaluator::default();
    evaluator.eval("(def x 5)").unwrap();
    assert_eq!(evaluator.eval("x"), Ok(5.into()))
  }

  #[test]
  fn evaluate_nested_fn() {
    let mut evaluator = Evaluator::default();
    assert_eq!(
      evaluator.eval("(((fn (x) (fn (y) (* x y))) 5) 10)"),
      Ok(50.into())
    )
  }

  #[test]
  fn evaluate_composition() {
    let mut evaluator = Evaluator::default();
    assert_eq!(evaluator.eval("((compose inc inc inc) 0)"), Ok(3.into()))
  }

  #[test]
  fn shadowing_local_causes_error() {
    let mut evaluator = Evaluator::default();
    assert_eq!(
      evaluator.eval("(((fn (x) (fn (x) (* x 2))) 5) 2)"),
      Err(PidginError::AST(ASTError::ShadowedBinding("x".to_string())))
    )
  }

  #[test]
  fn shadowing_builtin_causes_error() {
    let mut evaluator = Evaluator::default();
    assert_eq!(
      evaluator.eval("((fn (reduce) (* reduce 2)) 5)"),
      Err(PidginError::AST(ASTError::ShadowedBinding(
        "reduce".to_string()
      )))
    )
  }
}
