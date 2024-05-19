fn dec_loop_benchmark() {
  /*use block_macros::block;
  use pidgin::{
    runtime::control::Block, EvaluationState, GenericInstruction::*, Value,
  };
  let time = std::time::Instant::now();
  let program = block![
    Const(0, 100000000),
    Const(
      1,
      Value::composite_fn(
        1,
        block![IsPos(1, 0), If(1), Dec(0, 0), Jump(0), EndIf, Return(0)]
      )
    ),
    Call(0, 1, 1),
    StealArgument(0),
  ];
  let mut state = EvaluationState::new(program);
  state.evaluate().unwrap();
  println!("{}", time.elapsed().as_secs_f64())*/
}

fn main() {
  dec_loop_benchmark()
}
