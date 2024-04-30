#![allow(warnings)]

mod runtime;

use minivec::mini_vec;
use ordered_float::OrderedFloat;

use crate::runtime::{data::*, instructions::*, vm::*};

fn main() {
  use crate::runtime::data::Num::*;
  use Instruction::*;
  use Value::*;
  let program = Program::new(
    vec![Const(0, 0), Const(1, 1), Const(2, 2), Const(3, 3)],
    vec![Num(Int(1)), Bool(false), Symbol(0), Nil],
  );
  evaluate(program, EvaluationState::new()).unwrap();
  println!("{}", std::mem::size_of::<Value>())
}
