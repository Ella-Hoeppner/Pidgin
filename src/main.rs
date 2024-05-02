#![allow(warnings)]

mod runtime;
mod string_utils;

use minivec::mini_vec;
use ordered_float::OrderedFloat;

use crate::runtime::{data::*, instructions::*, vm::*};
use Instruction::*;
use Num::*;
use Value::*;

fn main() {
  let program = Program::new(
    vec![Const(0, 0), Const(1, 1), Const(2, 2), Const(3, 3)],
    vec![Number(Int(1)), Bool(false), Symbol(0), Nil],
  );
  let mut state = EvaluationState::new();
  state.evaluate(program).unwrap();
}
