mod blocks;
mod compiler;
mod frontend;
mod instructions;
mod runtime;
mod string_utils;

use frontend::{error::PidginResult, evaluator::Evaluator};

pub fn evaluate_pidgin_sexp(sexp: String) -> PidginResult<String> {
  let mut evaluator = Evaluator::default();
  evaluator
    .eval(&sexp)
    .map(|value| evaluator.describe(value.unwrap_or(runtime::data::Value::Nil)))
}
