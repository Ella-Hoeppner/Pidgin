mod runtime;

use ordered_float::OrderedFloat;

use crate::runtime::{evaluate, Instruction, Num, Value};

fn main() {
  let program = vec![
    Instruction::Const(Value::Num(Num::Float(OrderedFloat::from(1))), 0u16),
    Instruction::Const(Value::Num(Num::Float(OrderedFloat::from(2))), 1u16),
    Instruction::Add(0u16, 1u16, 2u16),
    Instruction::Const(Value::Num(Num::Float(OrderedFloat::from(4))), 3u16),
    Instruction::Multiply(2u16, 3u16, 4u16),
  ];
  evaluate(program).unwrap();
}
