pub mod control;
pub mod core_functions;
pub mod data;
pub mod error;
pub mod vm;

#[cfg(test)]
mod tests {
  use std::rc::Rc;

  use crate::runtime::control::CompositeFunction;
  use crate::runtime::error::PidginError;
  use crate::EvaluationState;
  use crate::{composite_fn, Block};
  use crate::{
    runtime::core_functions::CoreFnId,
    ConstIndex, ExternalFunction,
    GenericValue::{self, *},
    Instruction::*,
    Num::{self, *},
    Register,
  };
  use minivec::mini_vec;
  use ordered_float::OrderedFloat;
  use program_macro::block;

  macro_rules! assert_register {
    ($state:expr, $register:expr, $value:expr) => {
      assert_eq!($state.get_register($register), &$value.clone().into())
    };
  }
  macro_rules! run_and_check_registers {
    ($program:expr, $(($register:expr, $value:expr)),*$(,)?) => {
      let mut state = EvaluationState::new($program);
      state.evaluate().unwrap();
      $(assert_register!(state, $register, $value);)*
    };
  }
  macro_rules! simple_register_test {
    ($test_name:ident, $program:expr, $(($register:expr, $value:expr)),*$(,)?) => {
      #[test]
      fn $test_name() {
        run_and_check_registers!($program, $(($register, $value)),*);
      }
    }
  }

  #[test]
  fn constants() {
    let constants = vec![1.into(), false.into(), "Hello!".into(), Nil];
    run_and_check_registers!(
      block![
        Const(0, constants[0].clone()),
        Const(1, constants[1].clone()),
        Const(2, constants[2].clone()),
        Const(3, constants[3].clone()),
      ],
      (0, constants[0]),
      (1, constants[1]),
      (2, constants[2]),
      (3, constants[3])
    );
  }

  simple_register_test!(
    arithmetic,
    block![
      Const(0, 1),
      Const(1, 2.),
      Add(2, 0, 1),
      Const(3, 4),
      Multiply(4, 2, 3),
      Const(5, 12),
      Subtract(6, 4, 5),
      Const(7, -6),
      Divide(8, 4, 7),
    ],
    (2, 3.),
    (4, 12.),
    (6, 0.),
    (8, -2.)
  );

  fn environment_lookup() {
    let mut state = EvaluationState::new(block![Lookup(0, 0)]);
    state.bind_symbol(0, "test!");
    state.evaluate().unwrap();
    assert_register!(state, 0, "test!");
  }

  simple_register_test!(clear, block![Const(0, 100), Clear(0)], (0, Nil));

  simple_register_test!(copy, block![Const(0, 100), Copy(1, 0)], (1, 100));

  simple_register_test!(
    call_constant_function,
    block![
      Const(0, composite_fn(0, block![Const(0, 5), Return(0)])),
      Call(1, 0, 0)
    ],
    (1, 5)
  );

  simple_register_test!(
    call_square_function,
    block![
      Const(0, 10),
      Const(1, composite_fn(1, block![Multiply(0, 0, 0), Return(0)])),
      Call(2, 1, 1),
      CopyArgument(0)
    ],
    (0, 10),
    (2, 100),
  );

  simple_register_test!(
    call_square_function_twice,
    block![
      Const(0, 10),
      Const(1, composite_fn(1, block![Multiply(0, 0, 0), Return(0)])),
      Call(0, 1, 1),
      StealArgument(0),
      Call(0, 1, 1),
      StealArgument(0),
    ],
    (0, 10000),
  );

  simple_register_test!(
    call_double_square_nested_function,
    block![
      Const(0, 10),
      Const(
        1,
        composite_fn(
          1,
          block![
            Const(1, composite_fn(1, block![Multiply(0, 0, 0), Return(0)])),
            Call(0, 1, 1),
            StealArgument(0),
            Call(0, 1, 1),
            StealArgument(0),
            Return(0)
          ]
        )
      ),
      Call(0, 1, 1),
      StealArgument(0)
    ],
    (0, 10000)
  );

  simple_register_test!(
    call_square_product_function,
    block![
      Const(0, 2),
      Const(1, 3),
      Const(
        2,
        composite_fn(
          2,
          block![Multiply(0, 1, 0), Multiply(0, 0, 0), Return(0)]
        )
      ),
      Call(0, 2, 2),
      StealArgument(0),
      StealArgument(1),
    ],
    (0, 36)
  );

  simple_register_test!(
    call_triple_product_function,
    block![
      Const(0, 2),
      Const(1, 3),
      Const(2, 4),
      Const(
        4,
        composite_fn(
          3,
          block![Multiply(0, 1, 0), Multiply(0, 2, 0), Return(0)]
        )
      ),
      Call(3, 4, 3),
      StealArgument(0),
      StealArgument(1),
      StealArgument(2),
    ],
    (3, 24)
  );

  simple_register_test!(
    apply_fn,
    block![
      Const(0, vec![2.into(), 3.into(), 4.into()]),
      Const(
        1,
        composite_fn(
          3,
          block![Multiply(0, 1, 0), Multiply(0, 2, 0), Return(0)]
        )
      ),
      Apply(0, 1)
    ],
    (0, 24)
  );

  simple_register_test!(
    apply_core_fn_add,
    block![
      Const(0, List(Rc::new(vec![1.into(), 2.into(), 3.into()]))),
      Const(1, CoreFn(CoreFnId::Add)),
      Apply(0, 1),
    ],
    (0, 6)
  );

  simple_register_test!(
    list_first_last,
    block![Const(0, vec![1.into(), 2.into()]), First(1, 0), Last(2, 0)],
    (1, 1),
    (2, 2)
  );

  simple_register_test!(
    list_push,
    block![EmptyList(0), Const(1, "test"), Push(0, 1)],
    (0, List(Rc::new(vec!["test".into()])))
  );

  simple_register_test!(
    list_rest,
    block![
      Const(0, List(Rc::new(vec![1.into(), 2.into()]))),
      Rest(0),
      Const(1, List(Rc::new(vec![1.into(), 2.into(), 3.into()]))),
      Rest(1)
    ],
    (0, List(Rc::new(vec![2.into()]))),
    (1, List(Rc::new(vec![2.into(), 3.into()])))
  );

  simple_register_test!(
    list_butlast,
    block![
      Const(0, List(Rc::new(vec![1.into(), 2.into()]))),
      ButLast(0),
      Const(1, List(Rc::new(vec![1.into(), 2.into(), 3.into()]))),
      ButLast(1)
    ],
    (0, List(Rc::new(vec![1.into()]))),
    (1, List(Rc::new(vec![1.into(), 2.into()])))
  );

  simple_register_test!(
    if_true,
    block![Const(0, true), Const(1, -5), If(0), Const(1, 5), EndIf],
    (1, 5)
  );

  simple_register_test!(
    if_false,
    block![Const(0, false), Const(1, -5), If(0), Const(1, 5), EndIf],
    (1, -5)
  );

  simple_register_test!(
    if_else_true,
    block![
      Const(0, true),
      If(0),
      Const(1, -5),
      Else,
      Const(1, 5),
      EndIf
    ],
    (1, -5)
  );

  simple_register_test!(
    if_else_false,
    block![
      Const(0, false),
      If(0),
      Const(1, -5),
      Else,
      Const(1, 5),
      EndIf
    ],
    (1, 5)
  );

  simple_register_test!(
    if_else_if_else_true_true,
    block![
      Const(0, true),
      Const(1, true),
      If(0),
      Const(2, -5),
      ElseIf(1),
      Const(2, 0),
      Else,
      Const(2, 5),
      EndIf
    ],
    (2, -5)
  );

  simple_register_test!(
    if_else_if_else_true_false,
    block![
      Const(0, true),
      Const(1, false),
      If(0),
      Const(2, -5),
      ElseIf(1),
      Const(2, 0),
      Else,
      Const(2, 5),
      EndIf
    ],
    (2, -5)
  );

  simple_register_test!(
    if_else_if_else_false_true,
    block![
      Const(0, false),
      Const(1, true),
      If(0),
      Const(2, -5),
      ElseIf(1),
      Const(2, 0),
      Else,
      Const(2, 5),
      EndIf
    ],
    (2, 0)
  );

  simple_register_test!(
    if_else_if_else_false_false,
    block![
      Const(0, false),
      Const(1, false),
      If(0),
      Const(2, -5),
      ElseIf(1),
      Const(2, 0),
      Else,
      Const(2, 5),
      EndIf
    ],
    (2, 5)
  );

  simple_register_test!(
    recursion,
    block![
      Const(0, 10),
      Const(
        1,
        composite_fn(
          1,
          block![
            IsPos(1, 0),
            If(1),
            Dec(0, 0),
            CallingFunction(2),
            Call(0, 2, 1),
            StealArgument(0),
            EndIf,
            Return(0)
          ]
        )
      ),
      Call(0, 1, 1),
      StealArgument(0),
    ],
    (0, 0)
  );

  simple_register_test!(
    call_self_recursion,
    block![
      Const(0, 10),
      Const(
        1,
        composite_fn(
          1,
          block![
            IsPos(1, 0),
            If(1),
            Dec(0, 0),
            CallSelf(0, 1),
            StealArgument(0),
            EndIf,
            Return(0)
          ]
        )
      ),
      Call(0, 1, 1),
      StealArgument(0),
    ],
    (0, 0)
  );

  simple_register_test!(
    tail_recursion,
    block![
      Const(0, (u16::MAX as i64)),
      Const(
        1,
        composite_fn(
          1,
          block![
            IsPos(1, 0),
            If(1),
            Dec(2, 0),
            CallingFunction(3),
            CallAndReturn(3, 1),
            StealArgument(2),
            EndIf,
            Return(0)
          ]
        )
      ),
      Call(0, 1, 1),
      StealArgument(0),
    ],
    (0, 0),
  );

  simple_register_test!(
    call_self_tail_recursion,
    block![
      Const(0, (u16::MAX as i64)),
      Const(
        1,
        composite_fn(
          1,
          block![
            IsPos(1, 0),
            If(1),
            Dec(2, 0),
            CallSelfAndReturn(1),
            StealArgument(2),
            EndIf,
            Return(0)
          ]
        )
      ),
      Call(0, 1, 1),
      StealArgument(0),
    ],
    (0, 0),
  );

  simple_register_test!(
    jump_loop,
    block![
      Const(0, (u16::MAX as i64)),
      Const(
        1,
        composite_fn(
          1,
          block![IsPos(1, 0), If(1), Dec(0, 0), Jump(0), EndIf, Return(0)]
        )
      ),
      Call(0, 1, 1),
      StealArgument(0),
    ],
    (0, 0),
  );

  simple_register_test!(
    call_external_function,
    block![
      Const(0, 1),
      Const(1, 2),
      Const(
        2,
        ExternalFunction::unnamed(|args| {
          Ok((args[0].as_num()? + args[1].as_num()?).into())
        })
      ),
      Call(0, 2, 2),
      StealArgument(0),
      StealArgument(1)
    ],
    (0, 3)
  );

  simple_register_test!(
    external_object,
    block![
      Const(0, GenericValue::external((1i64, 2i64))),
      Const(1, GenericValue::external((3i64, 4i64))),
      Const(
        2,
        ExternalFunction::unnamed(|mut args| {
          let a = args.pop().unwrap().casted_external::<(i64, i64)>().unwrap();
          let b = args.pop().unwrap().casted_external::<(i64, i64)>().unwrap();
          Ok(Number(Num::from(a.0 + b.0 + a.1 + b.1)))
        })
      ),
      Call(0, 2, 2),
      StealArgument(0),
      StealArgument(1)
    ],
    (0, 10)
  );

  simple_register_test!(
    create_coroutine,
    block![
      Const(0, composite_fn(0, block![EmptyList(0), Return(0)])),
      CreateCoroutine(0),
    ],
  );

  simple_register_test!(
    run_coroutine,
    block![
      Const(0, composite_fn(0, block![EmptyList(0), Return(0)])),
      CreateCoroutine(0),
      Call(1, 0, 0)
    ],
    (1, List(Rc::new(vec![])))
  );

  simple_register_test!(
    run_nested_coroutinees,
    block![
      Const(
        0,
        composite_fn(
          0,
          block![
            Const(0, composite_fn(0, block![EmptyList(0), Return(0)])),
            CreateCoroutine(0),
            Call(1, 0, 0),
            Return(1)
          ]
        )
      ),
      CreateCoroutine(0),
      Call(1, 0, 0)
    ],
    (1, List(Rc::new(vec![])))
  );

  simple_register_test!(
    coroutine_yield,
    block![
      Const(
        0,
        composite_fn(
          0,
          block![
            Const(0, "yielded value!"),
            Yield(0),
            Const(1, "returned value!"),
            Return(1)
          ]
        )
      ),
      CreateCoroutine(0),
      Call(1, 0, 0),
      Call(2, 0, 0)
    ],
    (1, "yielded value!"),
    (2, "returned value!")
  );

  simple_register_test!(
    nested_coroutine_yield,
    block![
      Const(
        0,
        composite_fn(
          0,
          block![
            Const(
              0,
              composite_fn(
                0,
                block![
                  Const(0, "first yield!"),
                  Yield(0),
                  Const(1, "first return!"),
                  Yield(1)
                ]
              )
            ),
            CreateCoroutine(0),
            Call(1, 0, 0),
            Yield(1),
            Call(2, 0, 0),
            Yield(2),
            Const(
              0,
              composite_fn(
                0,
                block![
                  Const(0, "second yield!"),
                  Yield(0),
                  Const(1, "second return!"),
                  Yield(1)
                ]
              )
            ),
            CreateCoroutine(0),
            Call(1, 0, 0),
            Yield(1),
            Call(2, 0, 0),
            Yield(2)
          ]
        )
      ),
      CreateCoroutine(0),
      Call(1, 0, 0),
      Call(2, 0, 0),
      Call(3, 0, 0),
      Call(4, 0, 0)
    ],
    (1, "first yield!"),
    (2, "first return!"),
    (3, "second yield!"),
    (4, "second return!")
  );

  simple_register_test!(
    run_coroutine_with_args,
    block![
      Const(0, composite_fn(2, block![Add(2, 0, 1), Return(2)])),
      CreateCoroutine(0),
      Const(1, 1),
      Const(2, 2),
      Call(1, 0, 2),
      StealArgument(1),
      StealArgument(2)
    ],
    (1, 3)
  );

  simple_register_test!(
    resume_coroutine_with_args,
    block![
      Const(
        0,
        composite_fn(
          2,
          block![
            Add(0, 0, 1),
            YieldAndAccept(0, 2, 1),
            Add(0, 0, 1),
            Add(0, 0, 2),
            Return(0)
          ]
        )
      ),
      CreateCoroutine(0),
      Const(1, 1),
      Const(2, 2),
      Call(1, 0, 2),
      CopyArgument(1),
      CopyArgument(2),
      Const(2, 3),
      Const(3, 4),
      Call(2, 0, 2),
      CopyArgument(2),
      CopyArgument(3),
    ],
    (1, 3),
    (2, 10)
  );

  simple_register_test!(
    coroutine_returns_error,
    block![
      Const(0, composite_fn(2, block![Add(0, 0, 1), Return(0)])),
      CreateCoroutine(0),
      Const(1, "this isn't a number!!!"),
      Const(2, "this isn't either! so adding these will throw an error"),
      Call(3, 0, 2),
      StealArgument(1),
      StealArgument(2),
      IsError(4, 3),
      DebugPrint(0),
      IsCoroutineAlive(5, 0)
    ],
    (3, PidginError::CantCastToNum),
    (4, true),
    (5, false)
  );

  simple_register_test!(
    coroutine_is_alive,
    block![
      Const(0, composite_fn(1, block![Yield(0), Return(0)])),
      CreateCoroutine(0),
      Const(1, 1),
      Call(1, 0, 1),
      CopyArgument(1),
      IsCoroutineAlive(2, 0),
      Call(1, 0, 0),
      IsCoroutineAlive(3, 0),
    ],
    (2, true),
    (3, false),
  );
}
