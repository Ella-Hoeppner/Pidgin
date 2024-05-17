#[derive(Debug, Clone, PartialEq)]
pub enum Tree<T> {
  Inner(Vec<Tree<T>>),
  Leaf(T),
}
use Tree::*;

impl<T> Tree<T> {
  pub fn translate<NewT, E>(
    self,
    translator: fn(T) -> Result<NewT, E>,
  ) -> Result<Tree<NewT>, E> {
    match self {
      Inner(subtrees) => Ok(Inner(
        subtrees
          .into_iter()
          .map(|subtree| subtree.translate(translator))
          .collect::<Result<Vec<Tree<NewT>>, E>>()?,
      )),
      Leaf(leaf) => translator(leaf).map(|x| Leaf(x)),
    }
  }
}
