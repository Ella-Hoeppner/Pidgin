use std::{ops::Index, rc::Rc};

use crate::{
  instructions::GenericInstruction,
  runtime::{
    control::GenericCompositeFunction,
    data::GenericValue::{self, *},
  },
};

#[derive(Debug, Clone, PartialEq)]
pub struct GenericBlock<I, O, R, M> {
  pub instructions: Rc<[GenericInstruction<I, O, R>]>,
  pub constants: Rc<[GenericValue<I, O, R, M>]>,
  pub metadata: M,
}
impl<I, O, R, M> Index<usize> for GenericBlock<I, O, R, M> {
  type Output = GenericInstruction<I, O, R>;
  fn index(&self, index: usize) -> &Self::Output {
    &self.instructions[index]
  }
}
impl<I, O, R, M> GenericBlock<I, O, R, M> {
  pub fn new_with_metadata(
    instructions: Vec<GenericInstruction<I, O, R>>,
    constants: Vec<GenericValue<I, O, R, M>>,
    metadata: M,
  ) -> Self {
    Self {
      instructions: instructions.into(),
      constants: constants.into(),
      metadata,
    }
  }
}

impl<I, O, R, M> GenericBlock<I, O, R, M> {
  pub fn len(&self) -> usize {
    self.instructions.len()
  }
}

impl<I: Clone, O: Clone, R: Clone, M: Clone> GenericBlock<I, O, R, M> {
  fn translate_inner<
    NewI: Clone,
    NewO: Clone,
    NewR: Clone,
    NewM: Clone,
    E,
    F: Fn(
      u8,
      Vec<GenericInstruction<I, O, R>>,
      Vec<GenericValue<NewI, NewO, NewR, NewM>>,
      M,
    ) -> Result<GenericBlock<NewI, NewO, NewR, NewM>, E>,
  >(
    self,
    preallocated_registers: u8,
    translator: &F,
  ) -> Result<GenericBlock<NewI, NewO, NewR, NewM>, E> {
    let mut translated_constants = vec![];
    for value in self.constants.into_iter() {
      translated_constants.push(match value {
        CompositeFn(f_ref) => {
          CompositeFn(Rc::new(GenericCompositeFunction::new(
            f_ref.args.clone(),
            f_ref
              .block
              .clone()
              .translate_inner(f_ref.args.register_count(), translator)?,
          )))
        }
        Nil => Nil,
        Bool(a) => Bool(*a),
        Char(a) => Char(*a),
        Number(a) => Number(*a),
        Symbol(a) => Symbol(*a),
        Str(a) => Str(a.clone()),
        List(a) => List(a.clone()),
        Hashmap(a) => Hashmap(a.clone()),
        Hashset(a) => Hashset(a.clone()),
        CoreFn(a) => CoreFn(a.clone()),
        ExternalFn(a) => ExternalFn(a.clone()),
        ExternalObject(a) => ExternalObject(a.clone()),
        Coroutine(a) => Coroutine(a.clone()),
        Error(a) => Error(a.clone()),
      })
    }
    translator(
      preallocated_registers,
      self.instructions.to_vec(),
      translated_constants,
      self.metadata,
    )
  }
  pub fn translate<
    NewI: Clone,
    NewO: Clone,
    NewR: Clone,
    NewM: Clone,
    E,
    F: Fn(
      u8,
      Vec<GenericInstruction<I, O, R>>,
      Vec<GenericValue<NewI, NewO, NewR, NewM>>,
      M,
    ) -> Result<GenericBlock<NewI, NewO, NewR, NewM>, E>,
  >(
    self,
    translator: &F,
  ) -> Result<GenericBlock<NewI, NewO, NewR, NewM>, E> {
    self.translate_inner(0, translator)
  }
}
