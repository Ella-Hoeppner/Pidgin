mod blocks;
mod compiler;
mod frontend;
mod instructions;
mod runtime;
mod string_utils;

use frontend::{error::PidginResult, evaluator::Evaluator};
use rustyline::{error::ReadlineError, DefaultEditor};

pub fn evaluate_pidgin_sexp(sexp: String) -> PidginResult<String> {
  let mut evaluator = Evaluator::default();
  evaluator.eval(&sexp).map(|value| evaluator.describe(value))
}

pub fn repl() -> Result<(), ReadlineError> {
  let mut evaluator = Evaluator::default();
  println!("\nWelcome to Pidgin!! :D\n");
  let mut rl = DefaultEditor::new()?;
  if rl.load_history("history.txt").is_err() {
    println!("No previous history.");
  }
  loop {
    let readline = rl.readline("> ");
    match readline {
      Ok(line) => {
        rl.add_history_entry(line.as_str())
          .expect("failed to add line to history");
        match evaluator.eval(&line) {
          Ok(value) => println!("{value}"),
          Err(error) => println!("{error}"),
        }
      }
      Err(ReadlineError::Interrupted) => {
        println!("CTRL-C");
        break;
      }
      Err(ReadlineError::Eof) => {
        println!("CTRL-D");
        break;
      }
      Err(err) => {
        println!("Error: {:?}", err);
        break;
      }
    }
  }
  rl.save_history("history.txt")
    .expect("failed to save history");
  Ok(())
}
