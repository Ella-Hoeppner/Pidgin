use std::{ops::Index, rc::Rc};

use crate::{GenericCompositeFunction, GenericValue, Instruction};

use GenericValue::*;
use Instruction::*;

#[derive(Debug, Clone, PartialEq)]
pub struct GenericBlock<I, O, R, M> {
  pub instructions: Rc<[Instruction<I, O, R>]>,
  pub constants: Rc<[GenericValue<I, O, R, M>]>,
  pub metadata: M,
}
impl<I, O, R, M> Index<usize> for GenericBlock<I, O, R, M> {
  type Output = Instruction<I, O, R>;
  fn index(&self, index: usize) -> &Self::Output {
    &self.instructions[index]
  }
}
impl<I, O, R> GenericBlock<I, O, R, ()> {
  pub fn new(
    instructions: Vec<Instruction<I, O, R>>,
    constants: Vec<GenericValue<I, O, R, ()>>,
  ) -> Self {
    Self {
      instructions: instructions.into(),
      constants: constants.into(),
      metadata: (),
    }
  }
}
impl<I, O, R, M> GenericBlock<I, O, R, M> {
  pub fn len(&self) -> usize {
    self.instructions.len()
  }
}

impl<I: Clone, O: Clone, R: Clone, M> GenericBlock<I, O, R, M> {
  pub fn translate_instructions<
    NewI: Clone,
    NewO: Clone,
    NewR: Clone,
    NewM: Clone,
    E,
    F: Fn(
      u8,
      &[Instruction<I, O, R>],
      &M,
    ) -> Result<(Vec<Instruction<NewI, NewO, NewR>>, NewM), E>,
  >(
    &self,
    preallocated_registers: u8,
    replacer: &F,
  ) -> Result<GenericBlock<NewI, NewO, NewR, NewM>, E> {
    let (new_instructions, new_metadata) =
      replacer(preallocated_registers, &*self.instructions, &self.metadata)?;
    let mut translated_constants = vec![];
    for value in self.constants.into_iter() {
      translated_constants.push(match value {
        CompositeFn(f_ref) => {
          CompositeFn(Rc::new(GenericCompositeFunction::new(
            f_ref.args.clone(),
            f_ref
              .instructions
              .translate_instructions(f_ref.args.register_count(), replacer)?,
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
    Ok(GenericBlock {
      instructions: (&*new_instructions).into(),
      constants: (&*translated_constants).into(),
      metadata: new_metadata,
    })
  }
  pub fn replace_metadata<
    NewM: Clone,
    E,
    F: Fn(
      u8,
      &[Instruction<I, O, R>],
      &[GenericValue<I, O, R, M>],
      &M,
    ) -> Result<NewM, E>,
  >(
    &self,
    preallocated_registers: u8,
    replacer: &F,
  ) -> Result<GenericBlock<I, O, R, NewM>, E> {
    let new_metadata = replacer(
      preallocated_registers,
      &*self.instructions,
      &*self.constants,
      &self.metadata,
    )?;
    let mut translated_constants = vec![];
    for value in self.constants.into_iter() {
      translated_constants.push(match value {
        CompositeFn(f_ref) => {
          CompositeFn(Rc::new(GenericCompositeFunction::new(
            f_ref.args.clone(),
            f_ref
              .instructions
              .replace_metadata(f_ref.args.register_count(), replacer)?,
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
    Ok(GenericBlock {
      instructions: self.instructions.clone(),
      constants: (&*translated_constants).into(),
      metadata: new_metadata,
    })
  }
}
