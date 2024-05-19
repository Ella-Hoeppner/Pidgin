use std::hash::Hash;
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
  pub(crate) fn translate_inner<
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
    let translated_constants = (*self.constants)
      .to_vec()
      .into_iter()
      .map(|value| value.translate(translator))
      .collect::<Result<Vec<_>, E>>()?;
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
