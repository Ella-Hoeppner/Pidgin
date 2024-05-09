#![allow(warnings)]
#![feature(stmt_expr_attributes)]

mod instructions;
mod intermediate;
mod runtime;
mod string_utils;

use minivec::mini_vec;
use ordered_float::OrderedFloat;
use program_macro::program;

use crate::instructions::*;
use crate::runtime::{control::*, data::*, vm::*};
use std::rc::Rc;
use GeneralizedValue::*;
use Instruction::*;
use Num::*;

fn main() {
  let time = std::time::Instant::now();
  let program = Block::new(
    vec![
      Const(0, 0),
      Const(1, 1),
      Call(0, 1, 1),
      StealArgument(0),
      Print(0),
    ],
    vec![
      1000.into(),
      Value::composite_fn(
        1,
        Block::new(
          vec![IsPos(1, 0), If(1), Dec(0, 0), Jump(0), EndIf, Return(0)],
          vec![],
        ),
      ),
    ],
  );
  let mut state = EvaluationState::new(program);
  state.evaluate().unwrap();
  println!("{}", time.elapsed().as_secs_f64());
}
