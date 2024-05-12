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

  use crate::{
    blocks::GenericBlock,
    compiler::{
      ast_to_ir::expression_ast_to_ir,
      parse::parse_sexp,
      transformations::{
        allocate_registers, ir_to_bytecode, track_register_lifetimes,
      },
      SSABlock,
    },
    Block, EvaluationState,
    GenericValue::{self, *},
    Instruction::*,
    Num::{self, *},
  };

  macro_rules! test_ir_and_bytecode_and_output {
    (
      $test_name:ident,
      $sexp:expr,
      $expected_ir:expr,
      $expected_bytecode:expr,
      $expected_output:expr
    ) => {
      #[test]
      fn $test_name() {
        let ir = expression_ast_to_ir(parse_sexp($sexp)).unwrap();
        assert_eq!(ir, $expected_ir, "incorrect intermediate representation");
        let bytecode = ir_to_bytecode(ir).unwrap();
        assert_eq!(bytecode, $expected_bytecode, "incorrect bytecode");
        let output = EvaluationState::new(bytecode).evaluate().unwrap();
        assert_eq!(output, Some($expected_output.into()), "incorrect output");
      }
    };
  }

  test_ir_and_bytecode_and_output!(
    binary_addition,
    "(+ 1 2)",
    ssa_block![Const(0, 1), Const(1, 2), Add(2, 0, 1), Return(2)],
    block![Const(0, 1), Const(1, 2), Add(0, 0, 1), Return(0)],
    3
  );

  test_ir_and_bytecode_and_output!(
    subtraction,
    "(- 1 2)",
    ssa_block![Const(0, 1), Const(1, 2), Subtract(2, 0, 1), Return(2)],
    block![Const(0, 1), Const(1, 2), Subtract(0, 0, 1), Return(0)],
    -1
  );

  test_ir_and_bytecode_and_output!(
    binary_multiplication,
    "(* 1 2)",
    ssa_block![Const(0, 1), Const(1, 2), Multiply(2, 0, 1), Return(2)],
    block![Const(0, 1), Const(1, 2), Multiply(0, 0, 1), Return(0)],
    2
  );

  test_ir_and_bytecode_and_output!(
    division,
    "(/ 1 2)",
    ssa_block![Const(0, 1), Const(1, 2), Divide(2, 0, 1), Return(2)],
    block![Const(0, 1), Const(1, 2), Divide(0, 0, 1), Return(0)],
    0.5
  );

  test_ir_and_bytecode_and_output!(
    nested_binary_addition,
    "(+ (+ 1 2) 3)",
    ssa_block![
      Const(0, 1),
      Const(1, 2),
      Add(2, 0, 1),
      Const(3, 3),
      Add(4, 2, 3),
      Return(4)
    ],
    block![
      Const(0, 1),
      Const(1, 2),
      Add(0, 0, 1),
      Const(1, 3),
      Add(0, 0, 1),
      Return(0)
    ],
    6
  );

  test_ir_and_bytecode_and_output!(
    trinary_addition,
    "(+ 1 2 3)",
    ssa_block![
      Const(0, 1),
      Const(1, 2),
      Const(2, 3),
      Add(3, 0, 1),
      Add(4, 3, 2),
      Return(4)
    ],
    block![
      Const(0, 1),
      Const(1, 2),
      Const(2, 3),
      Add(0, 0, 1),
      Add(0, 0, 2),
      Return(0)
    ],
    6
  );

  test_ir_and_bytecode_and_output!(
    arity_5_addition,
    "(+ 1 2 3 4 5)",
    ssa_block![
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
    ],
    block![
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
    ],
    15
  );

  test_ir_and_bytecode_and_output!(
    trinary_multiplication,
    "(* 2 3 4)",
    ssa_block![
      Const(0, 2),
      Const(1, 3),
      Const(2, 4),
      Multiply(3, 0, 1),
      Multiply(4, 3, 2),
      Return(4)
    ],
    block![
      Const(0, 2),
      Const(1, 3),
      Const(2, 4),
      Multiply(0, 0, 1),
      Multiply(0, 0, 2),
      Return(0)
    ],
    24
  );

  test_ir_and_bytecode_and_output!(
    arity_5_multiplication,
    "(* 2 3 4 5 6)",
    ssa_block![
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
    ],
    block![
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
    ],
    720
  );

  test_ir_and_bytecode_and_output!(
    empty_list,
    "(list)",
    ssa_block![EmptyList(0), Return(0)],
    block![EmptyList(0), Return(0)],
    vec![]
  );

  test_ir_and_bytecode_and_output!(
    single_element_list,
    "(list 1)",
    ssa_block![Const(0, 1), EmptyList(1), Push((1, 2), 0), Return(2)],
    block![Const(0, 1), EmptyList(1), Push(1, 0), Return(1)],
    vec![1.into()]
  );

  test_ir_and_bytecode_and_output!(
    multi_element_list,
    "(list 1 2 3)",
    ssa_block![
      Const(0, 1),
      Const(1, 2),
      Const(2, 3),
      EmptyList(3),
      Push((3, 4), 0),
      Push((4, 5), 1),
      Push((5, 6), 2),
      Return(6)
    ],
    block![
      Const(0, 1),
      Const(1, 2),
      Const(2, 3),
      EmptyList(3),
      Push(3, 0),
      Push(3, 1),
      Push(3, 2),
      Return(3)
    ],
    vec![1.into(), 2.into(), 3.into()]
  );

  test_ir_and_bytecode_and_output!(
    nil_first,
    "(first nil)",
    ssa_block![Const(0, Nil), First(1, 0), Return(1)],
    block![Const(0, Nil), First(0, 0), Return(0)],
    Nil
  );

  test_ir_and_bytecode_and_output!(
    empty_list_first,
    "(first (list))",
    ssa_block![EmptyList(0), First(1, 0), Return(1)],
    block![EmptyList(0), First(0, 0), Return(0)],
    Nil
  );

  test_ir_and_bytecode_and_output!(
    nonempty_list_first,
    "(first (list 1))",
    ssa_block![
      Const(0, 1),
      EmptyList(1),
      Push((1, 2), 0),
      First(3, 2),
      Return(3)
    ],
    block![
      Const(0, 1),
      EmptyList(1),
      Push(1, 0),
      First(0, 1),
      Return(0)
    ],
    1
  );

  test_ir_and_bytecode_and_output!(
    nil_last,
    "(last nil)",
    ssa_block![Const(0, Nil), Last(1, 0), Return(1)],
    block![Const(0, Nil), Last(0, 0), Return(0)],
    Nil
  );

  test_ir_and_bytecode_and_output!(
    empty_list_last,
    "(last (list))",
    ssa_block![EmptyList(0), Last(1, 0), Return(1)],
    block![EmptyList(0), Last(0, 0), Return(0)],
    Nil
  );

  test_ir_and_bytecode_and_output!(
    nonempty_list_last,
    "(last (list 1))",
    ssa_block![
      Const(0, 1),
      EmptyList(1),
      Push((1, 2), 0),
      Last(3, 2),
      Return(3)
    ],
    block![Const(0, 1), EmptyList(1), Push(1, 0), Last(0, 1), Return(0)],
    1
  );

  test_ir_and_bytecode_and_output!(
    nil_rest,
    "(rest nil)",
    ssa_block![Const(0, Nil), Rest((0, 1)), Return(1)],
    block![Const(0, Nil), Rest(0), Return(0)],
    Nil
  );

  test_ir_and_bytecode_and_output!(
    empty_list_rest,
    "(rest (list))",
    ssa_block![EmptyList(0), Rest((0, 1)), Return(1)],
    block![EmptyList(0), Rest(0), Return(0)],
    vec![]
  );

  test_ir_and_bytecode_and_output!(
    nonempty_list_rest,
    "(rest (list 1 2))",
    ssa_block![
      Const(0, 1),
      Const(1, 2),
      EmptyList(2),
      Push((2, 3), 0),
      Push((3, 4), 1),
      Rest((4, 5)),
      Return(5)
    ],
    block![
      Const(0, 1),
      Const(1, 2),
      EmptyList(2),
      Push(2, 0),
      Push(2, 1),
      Rest(2),
      Return(2)
    ],
    vec![2.into()]
  );

  test_ir_and_bytecode_and_output!(
    nil_butlast,
    "(butlast nil)",
    ssa_block![Const(0, Nil), ButLast((0, 1)), Return(1)],
    block![Const(0, Nil), ButLast(0), Return(0)],
    Nil
  );

  test_ir_and_bytecode_and_output!(
    empty_list_butlast,
    "(butlast (list))",
    ssa_block![EmptyList(0), ButLast((0, 1)), Return(1)],
    block![EmptyList(0), ButLast(0), Return(0)],
    vec![]
  );

  test_ir_and_bytecode_and_output!(
    nonempty_list_butlast,
    "(butlast (list 1 2))",
    ssa_block![
      Const(0, 1),
      Const(1, 2),
      EmptyList(2),
      Push((2, 3), 0),
      Push((3, 4), 1),
      ButLast((4, 5)),
      Return(5)
    ],
    block![
      Const(0, 1),
      Const(1, 2),
      EmptyList(2),
      Push(2, 0),
      Push(2, 1),
      ButLast(2),
      Return(2)
    ],
    vec![1.into()]
  );

  test_ir_and_bytecode_and_output!(
    empty_list_push,
    "(push (list) 1)",
    ssa_block![EmptyList(0), Const(1, 1), Push((0, 2), 1), Return(2)],
    block![EmptyList(0), Const(1, 1), Push(0, 1), Return(0)],
    vec![1.into()]
  );

  test_ir_and_bytecode_and_output!(
    nonempty_list_push,
    "(push (list 1) 2)",
    ssa_block![
      Const(0, 1),
      EmptyList(1),
      Push((1, 2), 0),
      Const(3, 2),
      Push((2, 4), 3),
      Return(4)
    ],
    block![
      Const(0, 1),
      EmptyList(1),
      Push(1, 0),
      Const(0, 2),
      Push(1, 0),
      Return(1)
    ],
    vec![1.into(), 2.into()]
  );

  test_ir_and_bytecode_and_output!(
    empty_list_cons,
    "(cons (list) 1)",
    ssa_block![EmptyList(0), Const(1, 1), Cons((0, 2), 1), Return(2)],
    block![EmptyList(0), Const(1, 1), Cons(0, 1), Return(0)],
    vec![1.into()]
  );

  test_ir_and_bytecode_and_output!(
    nonempty_list_cons,
    "(cons (list 2) 1)",
    ssa_block![
      Const(0, 2),
      EmptyList(1),
      Push((1, 2), 0),
      Const(3, 1),
      Cons((2, 4), 3),
      Return(4)
    ],
    block![
      Const(0, 2),
      EmptyList(1),
      Push(1, 0),
      Const(0, 1),
      Cons(1, 0),
      Return(1)
    ],
    vec![1.into(), 2.into()]
  );

  test_ir_and_bytecode_and_output!(
    empty_list_is_empty,
    "(empty? (list))",
    ssa_block![EmptyList(0), IsEmpty(1, 0), Return(1)],
    block![EmptyList(0), IsEmpty(0, 0), Return(0)],
    true
  );

  test_ir_and_bytecode_and_output!(
    nonempty_list_is_nonempty,
    "(empty? (list 1))",
    ssa_block![
      Const(0, 1),
      EmptyList(1),
      Push((1, 2), 0),
      IsEmpty(3, 2),
      Return(3)
    ],
    block![
      Const(0, 1),
      EmptyList(1),
      Push(1, 0),
      IsEmpty(0, 1),
      Return(0)
    ],
    false
  );
}
