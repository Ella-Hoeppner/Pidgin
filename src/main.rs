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
  let program = program![
    Const(0, 100000000),
    Const(
      1,
      CompositeFn(Rc::new(CompositeFunction::new(
        1,
        vec![IsPos(1, 0), If(1), Dec(0, 0), Jump(0), EndIf, Return(0)]
      )))
    ),
    Call(0, 1, 1),
    StealArgument(0),
  ];
  let mut state = EvaluationState::new(program);
  state.evaluate().unwrap();
  println!("{}", time.elapsed().as_secs_f64());
}
