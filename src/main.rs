mod runtime;

use std::ops::Mul;

use ordered_float::OrderedFloat;

use crate::runtime::*;

fn main() {
  use Instruction::*;
  let instructions = vec![
    DebugPrint(0),
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
    DebugPrint(255),
  ];
  let program = Program::new(
    instructions,
    vec![
      Value::Num(Num::Int(1)),
      Value::Num(Num::Float(OrderedFloat(2.))),
      Value::Num(Num::Int(4)),
      /*Value::Fn(Function::new(vec![
        Argument(2),
        Argument(3),
        Lookup(0, 2),
        Lookup(1, 3),
        Multiply(0, 1, 0),
        Multiply(0, 1, 0),
        Return(0),
      ])),*/
    ],
  );
  evaluate(program).unwrap();
}
