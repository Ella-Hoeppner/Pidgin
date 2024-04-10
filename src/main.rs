mod runtime;

use ordered_float::OrderedFloat;

use crate::runtime::*;

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
    DebugPrint(200),
    Const(3, 3),
    DebugPrint(210),
    Apply(5, 3, 0),
    DebugPrint(255),
  ];
  use crate::runtime::Num::*;
  use Value::*;
  let program = Program::new(
    instructions,
    vec![
      Num(Int(1)),
      Num(Float(OrderedFloat(2.))),
      Num(Int(4)),
      Value::Fn(Function::new(vec![
        Argument(2),
        Lookup(0, 2),
        Multiply(0, 0, 0),
        Multiply(0, 0, 0),
        Return(0),
      ])),
    ],
  );
  evaluate(program).unwrap();
}
