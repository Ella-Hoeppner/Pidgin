pub mod error;
pub mod evaluator;

#[cfg(test)]
mod tests {
  use crate::Value;

  use super::evaluator::Evaluator;

  fn assert_eval_eq<V: Into<Value>>(expr: &str, expected_value: V) {
    let mut evaluator = Evaluator::default();
    assert_eq!(evaluator.eval(expr), Ok(Some(expected_value.into())))
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
}
