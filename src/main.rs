#![allow(warnings)]
#![feature(stmt_expr_attributes)]

mod blocks;
mod compiler;
mod instructions;
mod runtime;
mod string_utils;

use minivec::mini_vec;
use ordered_float::OrderedFloat;
use program_macro::block;

use crate::compiler::ast_to_ir::{ast_to_ir, token_to_value};
use crate::compiler::parse::parse_sexp;
use crate::instructions::*;
use crate::runtime::{control::*, data::*, vm::*};
use std::rc::Rc;
use GenericValue::*;
use Instruction::*;
use Num::*;

fn main() {
  /*let time = std::time::Instant::now();
  let program = block![
    Const(0, 100000000),
    Const(
      1,
      composite_fn(
        1,
        block![IsPos(1, 0), If(1), Dec(0, 0), Jump(0), EndIf, Return(0)],
      )
    ),
    Call(0, 1, 1),
    StealArgument(0),
    Print(0),
  ];
  let mut state = EvaluationState::new(program);
  state.evaluate().unwrap();
  println!("{}", time.elapsed().as_secs_f64());*/

  println!("{:?}", ast_to_ir(parse_sexp("(+ 1 2)")));
}
