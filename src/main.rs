mod runtime;

use minivec::mini_vec;
use ordered_float::OrderedFloat;

use crate::runtime::{data::*, vm::*};

fn main() {
  use Instruction::*;
  let instructions = vec![
    Const(0, 0),
    Bind(0, 0),
    Const(1, 1),
    Bind(1, 1),
    Add(0, 1, 0),
    Const(1, 2),
    Multiply(0, 1, 0),
    Clear(1),
    Lookup(1, 0),
    Lookup(2, 1),
    Const(3, 3),
    Apply(4, 3, 0),
    DebugPrint(255),
  ];
  use Value::*;
  let program = Program::new(
    instructions,
    vec![
      Int(1),
      Float(OrderedFloat(2.)),
      Int(4),
      Fn(mini_vec![
        Argument(2),
        Lookup(0, 2),
        Multiply(0, 0, 0),
        Multiply(0, 0, 0),
        Return(0),
      ]),
    ],
  );
  evaluate(program).unwrap();
  println!("{}", std::mem::size_of::<Value>())
}
