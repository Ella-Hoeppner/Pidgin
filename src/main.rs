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

use crate::compiler::ast_to_ir::{expression_ast_to_ir, token_to_value};
use crate::compiler::parse::{parse_sexp, Token};
use crate::compiler::transformations::{
  allocate_registers, track_register_lifetimes,
};
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
  let raw_ir = expression_ast_to_ir(parse_sexp("(+ 1 2)")).unwrap();
  println!("raw ir:\n{:?}\n\n", raw_ir);
  let lifetime_ir = track_register_lifetimes(raw_ir).unwrap();
  println!("lifetime ir:\n{:?}\n\n", lifetime_ir);
  let bytecode = allocate_registers(lifetime_ir).unwrap();
  println!("bytecode:\n{:?}\n\n", bytecode);
  let mut state = EvaluationState::new(bytecode);
  let x = state.evaluate().unwrap();
  println!("{x:?}");
}
