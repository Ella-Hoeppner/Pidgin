pub mod error;
pub mod evaluator;

#[cfg(test)]
mod tests {
  use crate::runtime::data::Value;

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
}
