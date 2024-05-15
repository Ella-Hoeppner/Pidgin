pub mod ast_to_ir;
pub mod parse;
pub mod transformations;

use crate::{blocks::GenericBlock, GenericValue, Instruction};

pub type SSARegister = usize;
pub type SSAInstruction =
  Instruction<SSARegister, SSARegister, (SSARegister, SSARegister)>;
pub type SSABlock<M> =
  GenericBlock<SSARegister, SSARegister, (SSARegister, SSARegister), M>;
pub type SSAValue<M> =
  GenericValue<SSARegister, SSARegister, (SSARegister, SSARegister), M>;

mod tests {
  use program_macro::{block, ssa_block};
  use std::fmt::Debug;

  use crate::{
    blocks::GenericBlock,
    compiler::{
      ast_to_ir::expression_ast_to_ir,
      parse::parse_sexp,
      transformations::{allocate_registers, track_register_lifetimes},
      SSABlock,
    },
    Block, EvaluationState,
    GenericValue::{self, *},
    Instruction::*,
    Num::{self, *},
    Value,
  };

  fn debug_string<T: Debug>(x: &T) -> String {
    format!("{:?}", x)
  }

  macro_rules! test_raw_ir {
    ($sexp:expr, $expected_ir:expr) => {
      let raw_ir = expression_ast_to_ir(parse_sexp($sexp)).unwrap();
      assert_eq!(
        debug_string(&raw_ir),
        debug_string(&$expected_ir),
        "incorrect raw ir"
      );
    };
  }

  macro_rules! test_bytecode {
    ($sexp:expr, $expected_bytecode:expr) => {
      let raw_ir = expression_ast_to_ir(parse_sexp($sexp)).unwrap();
      let lifetime_ir = track_register_lifetimes(raw_ir).unwrap();
      let bytecode = allocate_registers(lifetime_ir).unwrap();
      assert_eq!(
        debug_string(&bytecode),
        debug_string(&$expected_bytecode),
        "incorrect bytecode"
      );
    };
  }

  macro_rules! test_output {
    ($sexp:expr, $expected_output:expr) => {
      let raw_ir = expression_ast_to_ir(parse_sexp($sexp)).unwrap();
      let lifetime_ir = track_register_lifetimes(raw_ir).unwrap();
      let bytecode = allocate_registers(lifetime_ir).unwrap();
      let output = EvaluationState::new(bytecode).evaluate().unwrap();
      assert_eq!(
        debug_string(&output),
        debug_string(&Some(Value::from($expected_output))),
        "incorrect output"
      );
    };
  }

  #[test]
  fn binary_addition() {
    let sexp = "(+ 1 2)";
    test_raw_ir!(
      sexp,
      (ssa_block![Const(0, 1), Const(1, 2), Add(2, 0, 1), Return(2)])
    );
    test_bytecode!(
      sexp,
      (block![Const(0, 1), Const(1, 2), Add(0, 0, 1), Return(0)])
    );
    test_output!(sexp, 3);
  }

  #[test]
  fn subtraction() {
    let sexp = "(- 1 2)";
    test_raw_ir!(
      sexp,
      (ssa_block![Const(0, 1), Const(1, 2), Subtract(2, 0, 1), Return(2)])
    );
    test_bytecode!(
      sexp,
      (block![Const(0, 1), Const(1, 2), Subtract(0, 0, 1), Return(0)])
    );
    test_output!(sexp, (-1));
  }

  #[test]
  fn binary_multiplication() {
    let sexp = "(* 1 2)";
    test_raw_ir!(
      sexp,
      (ssa_block![Const(0, 1), Const(1, 2), Multiply(2, 0, 1), Return(2)])
    );
    test_bytecode!(
      sexp,
      (block![Const(0, 1), Const(1, 2), Multiply(0, 0, 1), Return(0)])
    );
    test_output!(sexp, 2);
  }

  #[test]
  fn division() {
    let sexp = "(/ 1 2)";
    test_raw_ir!(
      sexp,
      (ssa_block![Const(0, 1), Const(1, 2), Divide(2, 0, 1), Return(2)])
    );
    test_bytecode!(
      sexp,
      (block![Const(0, 1), Const(1, 2), Divide(0, 0, 1), Return(0)])
    );
    test_output!(sexp, 0.5);
  }

  #[test]
  fn nested_binary_addition() {
    let sexp = "(+ (+ 1 2) 3)";
    test_raw_ir!(
      sexp,
      (ssa_block![
        Const(0, 1),
        Const(1, 2),
        Add(2, 0, 1),
        Const(3, 3),
        Add(4, 2, 3),
        Return(4)
      ])
    );
    test_bytecode!(
      sexp,
      (block![
        Const(0, 1),
        Const(1, 2),
        Add(0, 0, 1),
        Const(1, 3),
        Add(0, 0, 1),
        Return(0)
      ])
    );
    test_output!(sexp, 6);
  }

  #[test]
  fn trinary_addition() {
    let sexp = "(+ 1 2 3)";
    test_raw_ir!(
      sexp,
      (ssa_block![
        Const(0, 1),
        Const(1, 2),
        Const(2, 3),
        Add(3, 0, 1),
        Add(4, 3, 2),
        Return(4)
      ])
    );
    test_bytecode!(
      sexp,
      (block![
        Const(0, 1),
        Const(1, 2),
        Const(2, 3),
        Add(0, 0, 1),
        Add(0, 0, 2),
        Return(0)
      ])
    );
    test_output!(sexp, 6);
  }

  #[test]
  fn arity_5_addition() {
    let sexp = "(+ 1 2 3 4 5)";
    test_raw_ir!(
      sexp,
      (ssa_block![
        Const(0, 1),
        Const(1, 2),
        Const(2, 3),
        Const(3, 4),
        Const(4, 5),
        Add(5, 0, 1),
        Add(6, 5, 2),
        Add(7, 6, 3),
        Add(8, 7, 4),
        Return(8)
      ])
    );
    test_bytecode!(
      sexp,
      (block![
        Const(0, 1),
        Const(1, 2),
        Const(2, 3),
        Const(3, 4),
        Const(4, 5),
        Add(0, 0, 1),
        Add(0, 0, 2),
        Add(0, 0, 3),
        Add(0, 0, 4),
        Return(0)
      ])
    );
    test_output!(sexp, 15);
  }

  #[test]
  fn trinary_multiplication() {
    let sexp = "(* 2 3 4)";
    test_raw_ir!(
      sexp,
      (ssa_block![
        Const(0, 2),
        Const(1, 3),
        Const(2, 4),
        Multiply(3, 0, 1),
        Multiply(4, 3, 2),
        Return(4)
      ])
    );
    test_bytecode!(
      sexp,
      (block![
        Const(0, 2),
        Const(1, 3),
        Const(2, 4),
        Multiply(0, 0, 1),
        Multiply(0, 0, 2),
        Return(0)
      ])
    );
    test_output!(sexp, 24);
  }

  #[test]
  fn arity_5_multiplication() {
    let sexp = "(* 2 3 4 5 6)";
    test_raw_ir!(
      sexp,
      (ssa_block![
        Const(0, 2),
        Const(1, 3),
        Const(2, 4),
        Const(3, 5),
        Const(4, 6),
        Multiply(5, 0, 1),
        Multiply(6, 5, 2),
        Multiply(7, 6, 3),
        Multiply(8, 7, 4),
        Return(8)
      ])
    );
    test_bytecode!(
      sexp,
      (block![
        Const(0, 2),
        Const(1, 3),
        Const(2, 4),
        Const(3, 5),
        Const(4, 6),
        Multiply(0, 0, 1),
        Multiply(0, 0, 2),
        Multiply(0, 0, 3),
        Multiply(0, 0, 4),
        Return(0)
      ])
    );
    test_output!(sexp, 720);
  }

  #[test]
  fn empty_list() {
    let sexp = "(list)";
    test_raw_ir!(sexp, (ssa_block![EmptyList(0), Return(0)]));
    test_bytecode!(sexp, (block![EmptyList(0), Return(0)]));
    test_output!(sexp, (vec![]));
  }

  #[test]
  fn single_element_list() {
    let sexp = "(list 1)";
    test_raw_ir!(
      sexp,
      (ssa_block![Const(0, 1), EmptyList(1), Push((1, 2), 0), Return(2)])
    );
    test_bytecode!(
      sexp,
      (block![Const(0, 1), EmptyList(1), Push(1, 0), Return(1)])
    );
    test_output!(sexp, (vec![1.into()]));
  }

  #[test]
  fn multi_element_list() {
    let sexp = "(list 1 2 3)";
    test_raw_ir!(
      sexp,
      (ssa_block![
        Const(0, 1),
        Const(1, 2),
        Const(2, 3),
        EmptyList(3),
        Push((3, 4), 0),
        Push((4, 5), 1),
        Push((5, 6), 2),
        Return(6)
      ])
    );
    test_bytecode!(
      sexp,
      (block![
        Const(0, 1),
        Const(1, 2),
        Const(2, 3),
        EmptyList(3),
        Push(3, 0),
        Push(3, 1),
        Push(3, 2),
        Return(3)
      ])
    );
    test_output!(sexp, (vec![1.into(), 2.into(), 3.into()]));
  }

  #[test]
  fn nil_first() {
    let sexp = "(first nil)";
    test_raw_ir!(sexp, (ssa_block![Const(0, Nil), First(1, 0), Return(1)]));
    test_bytecode!(sexp, (block![Const(0, Nil), First(0, 0), Return(0)]));
    test_output!(sexp, Nil);
  }

  #[test]
  fn empty_list_first() {
    let sexp = "(first (list))";
    test_raw_ir!(sexp, (ssa_block![EmptyList(0), First(1, 0), Return(1)]));
    test_bytecode!(sexp, (block![EmptyList(0), First(0, 0), Return(0)]));
    test_output!(sexp, Nil);
  }

  #[test]
  fn nonempty_list_first() {
    let sexp = "(first (list 1))";
    test_raw_ir!(
      sexp,
      (ssa_block![
        Const(0, 1),
        EmptyList(1),
        Push((1, 2), 0),
        First(3, 2),
        Return(3)
      ])
    );
    test_bytecode!(
      sexp,
      (block![
        Const(0, 1),
        EmptyList(1),
        Push(1, 0),
        First(0, 1),
        Return(0)
      ])
    );
    test_output!(sexp, 1);
  }

  #[test]
  fn nil_last() {
    let sexp = "(last nil)";
    test_raw_ir!(sexp, (ssa_block![Const(0, Nil), Last(1, 0), Return(1)]));
    test_bytecode!(sexp, (block![Const(0, Nil), Last(0, 0), Return(0)]));
    test_output!(sexp, Nil);
  }

  #[test]
  fn empty_list_last() {
    let sexp = "(last (list))";
    test_raw_ir!(sexp, (ssa_block![EmptyList(0), Last(1, 0), Return(1)]));
    test_bytecode!(sexp, (block![EmptyList(0), Last(0, 0), Return(0)]));
    test_output!(sexp, Nil);
  }

  #[test]
  fn nonempty_list_last() {
    let sexp = "(last (list 1))";
    test_raw_ir!(
      sexp,
      (ssa_block![
        Const(0, 1),
        EmptyList(1),
        Push((1, 2), 0),
        Last(3, 2),
        Return(3)
      ])
    );
    test_bytecode!(
      sexp,
      (block![Const(0, 1), EmptyList(1), Push(1, 0), Last(0, 1), Return(0)])
    );
    test_output!(sexp, 1);
  }

  #[test]
  fn nil_rest() {
    let sexp = "(rest nil)";
    test_raw_ir!(sexp, (ssa_block![Const(0, Nil), Rest((0, 1)), Return(1)]));
    test_bytecode!(sexp, (block![Const(0, Nil), Rest(0), Return(0)]));
    test_output!(sexp, Nil);
  }

  #[test]
  fn empty_list_rest() {
    let sexp = "(rest (list))";
    test_raw_ir!(sexp, (ssa_block![EmptyList(0), Rest((0, 1)), Return(1)]));
    test_bytecode!(sexp, (block![EmptyList(0), Rest(0), Return(0)]));
    test_output!(sexp, (vec![]));
  }

  #[test]
  fn nonempty_list_rest() {
    let sexp = "(rest (list 1 2))";
    test_raw_ir!(
      sexp,
      (ssa_block![
        Const(0, 1),
        Const(1, 2),
        EmptyList(2),
        Push((2, 3), 0),
        Push((3, 4), 1),
        Rest((4, 5)),
        Return(5)
      ])
    );
    test_bytecode!(
      sexp,
      (block![
        Const(0, 1),
        Const(1, 2),
        EmptyList(2),
        Push(2, 0),
        Push(2, 1),
        Rest(2),
        Return(2)
      ])
    );
    test_output!(sexp, (vec![2.into()]));
  }

  #[test]
  fn nil_butlast() {
    let sexp = "(butlast nil)";
    test_raw_ir!(
      sexp,
      (ssa_block![Const(0, Nil), ButLast((0, 1)), Return(1)])
    );
    test_bytecode!(sexp, (block![Const(0, Nil), ButLast(0), Return(0)]));
    test_output!(sexp, Nil);
  }

  #[test]
  fn empty_list_butlast() {
    let sexp = "(butlast (list))";
    test_raw_ir!(sexp, (ssa_block![EmptyList(0), ButLast((0, 1)), Return(1)]));
    test_bytecode!(sexp, (block![EmptyList(0), ButLast(0), Return(0)]));
    test_output!(sexp, (vec![]));
  }

  #[test]
  fn nonempty_list_butlast() {
    let sexp = "(butlast (list 1 2))";
    test_raw_ir!(
      sexp,
      (ssa_block![
        Const(0, 1),
        Const(1, 2),
        EmptyList(2),
        Push((2, 3), 0),
        Push((3, 4), 1),
        ButLast((4, 5)),
        Return(5)
      ])
    );
    test_bytecode!(
      sexp,
      (block![
        Const(0, 1),
        Const(1, 2),
        EmptyList(2),
        Push(2, 0),
        Push(2, 1),
        ButLast(2),
        Return(2)
      ])
    );
    test_output!(sexp, (vec![1.into()]));
  }

  #[test]
  fn empty_list_push() {
    let sexp = "(push (list) 1)";
    test_raw_ir!(
      sexp,
      (ssa_block![EmptyList(0), Const(1, 1), Push((0, 2), 1), Return(2)])
    );
    test_bytecode!(
      sexp,
      (block![EmptyList(0), Const(1, 1), Push(0, 1), Return(0)])
    );
    test_output!(sexp, (vec![1.into()]));
  }

  #[test]
  fn nonempty_list_push() {
    let sexp = "(push (list 1) 2)";
    test_raw_ir!(
      sexp,
      (ssa_block![
        Const(0, 1),
        EmptyList(1),
        Push((1, 2), 0),
        Const(3, 2),
        Push((2, 4), 3),
        Return(4)
      ])
    );
    test_bytecode!(
      sexp,
      (block![
        Const(0, 1),
        EmptyList(1),
        Push(1, 0),
        Const(0, 2),
        Push(1, 0),
        Return(1)
      ])
    );
    test_output!(sexp, (vec![1.into(), 2.into()]));
  }

  #[test]
  fn empty_list_cons() {
    let sexp = "(cons (list) 1)";
    test_raw_ir!(
      sexp,
      (ssa_block![EmptyList(0), Const(1, 1), Cons((0, 2), 1), Return(2)])
    );
    test_bytecode!(
      sexp,
      (block![EmptyList(0), Const(1, 1), Cons(0, 1), Return(0)])
    );
    test_output!(sexp, (vec![1.into()]));
  }

  #[test]
  fn nonempty_list_cons() {
    let sexp = "(cons (list 2) 1)";
    test_raw_ir!(
      sexp,
      (ssa_block![
        Const(0, 2),
        EmptyList(1),
        Push((1, 2), 0),
        Const(3, 1),
        Cons((2, 4), 3),
        Return(4)
      ])
    );
    test_bytecode!(
      sexp,
      (block![
        Const(0, 2),
        EmptyList(1),
        Push(1, 0),
        Const(0, 1),
        Cons(1, 0),
        Return(1)
      ])
    );
    test_output!(sexp, (vec![1.into(), 2.into()]));
  }

  #[test]
  fn empty_list_is_empty() {
    let sexp = "(empty? (list))";
    test_raw_ir!(sexp, (ssa_block![EmptyList(0), IsEmpty(1, 0), Return(1)]));
    test_bytecode!(sexp, (block![EmptyList(0), IsEmpty(0, 0), Return(0)]));
    test_output!(sexp, true);
  }

  #[test]
  fn nonempty_list_is_nonempty() {
    let sexp = "(empty? (list 1))";
    test_raw_ir!(
      sexp,
      (ssa_block![
        Const(0, 1),
        EmptyList(1),
        Push((1, 2), 0),
        IsEmpty(3, 2),
        Return(3)
      ])
    );
    test_bytecode!(
      sexp,
      (block![
        Const(0, 1),
        EmptyList(1),
        Push(1, 0),
        IsEmpty(0, 1),
        Return(0)
      ])
    );
    test_output!(sexp, false);
  }

  #[test]
  fn fn_definition() {
    let sexp = "(fn (x) (* x x))";
    test_raw_ir!(
      sexp,
      (ssa_block![
        Const(
          0,
          GenericValue::composite_fn(
            1,
            ssa_block![Multiply(1, 0, 0), Return(1)]
          )
        ),
        Return(0)
      ])
    );
    test_bytecode!(
      sexp,
      (block![
        Const(
          0,
          Value::composite_fn(1, block![Multiply(0, 0, 0), Return(0)])
        ),
        Return(0)
      ])
    );
    test_output!(
      sexp,
      (Value::composite_fn(1, block![Multiply(0, 0, 0), Return(0)]))
    );
  }

  #[test]
  fn fn_definition_mutliarg() {
    let sexp = "(fn (x y) (* x x y y))";
    test_raw_ir!(
      sexp,
      (ssa_block![
        Const(
          0,
          GenericValue::composite_fn(
            2,
            ssa_block![
              Multiply(2, 0, 0),
              Multiply(3, 2, 1),
              Multiply(4, 3, 1),
              Return(4)
            ]
          )
        ),
        Return(0)
      ])
    );
    test_bytecode!(
      sexp,
      (block![
        Const(
          0,
          Value::composite_fn(
            2,
            block![
              Multiply(0, 0, 0),
              Multiply(0, 0, 1),
              Multiply(0, 0, 1),
              Return(0)
            ]
          )
        ),
        Return(0)
      ])
    );
    test_output!(
      sexp,
      (Value::composite_fn(
        2,
        block![
          Multiply(0, 0, 0),
          Multiply(0, 0, 1),
          Multiply(0, 0, 1),
          Return(0)
        ]
      ))
    );
  }

  #[test]
  fn fn_definition_and_application() {
    let sexp = "((fn (x y) (* x x y y)) 2 3)";
    test_raw_ir!(
      sexp,
      (ssa_block![
        Const(
          0,
          GenericValue::composite_fn(
            2,
            ssa_block![
              Multiply(2, 0, 0),
              Multiply(3, 2, 1),
              Multiply(4, 3, 1),
              Return(4)
            ]
          )
        ),
        Const(1, 2),
        Const(2, 3),
        Call(3, 0, 2),
        CopyArgument(1),
        CopyArgument(2),
        Return(3)
      ])
    );
    test_bytecode!(
      sexp,
      (block![
        Const(
          0,
          Value::composite_fn(
            2,
            block![
              Multiply(0, 0, 0),
              Multiply(0, 0, 1),
              Multiply(0, 0, 1),
              Return(0)
            ]
          )
        ),
        Const(1, 2),
        Const(2, 3),
        Call(0, 0, 2),
        CopyArgument(1),
        CopyArgument(2),
        Return(0)
      ])
    );
    test_output!(sexp, 36);
  }
}
