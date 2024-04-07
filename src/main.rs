mod runtime;

use ordered_float::OrderedFloat;

use crate::runtime::{evaluate, Instruction, Num, Value};

fn main() {
  use Instruction::*;
  let program = vec![
    DebugPrint(Some("beginning".to_string())),
    Const(0, Value::Num(Num::Int(1))),
    DebugPrint(Some("after const 1".to_string())),
    Bind(0, 0),
    DebugPrint(Some("after binding 1".to_string())),
    Const(1, Value::Num(Num::Int(2))),
    Bind(1, 1),
    DebugPrint(Some("after binding 2".to_string())),
    Add(0, 1, 0),
    DebugPrint(Some("after add".to_string())),
    Const(1, Value::Num(Num::Int(4))),
    Add(0, 1, 0),
    DebugPrint(Some("after second add".to_string())),
    Clear(1),
    DebugPrint(Some("after clear".to_string())),
    Lookup(1, 0),
    Lookup(2, 1),
    DebugPrint(Some("final".to_string())),
  ];
  evaluate(program).unwrap();
}
