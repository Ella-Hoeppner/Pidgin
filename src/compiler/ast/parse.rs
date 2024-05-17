// the code in this file is purely temporary, as Pidgin will eventually use
// [GSE](https://github.com/Ella-Hoeppner/GSE) for parsing

use super::tree::Tree;

fn tokenize(input: &str) -> Vec<String> {
  input
    .replace("(", " ( ")
    .replace(")", " ) ")
    .split_whitespace()
    .map(String::from)
    .collect()
}

pub fn parse_sexp(input: &str) -> Tree<String> {
  let mut ast_stack: Vec<Vec<Tree<String>>> = vec![vec![]];
  for token in tokenize(input) {
    match token.as_str() {
      "(" => ast_stack.push(vec![]),
      ")" => {
        let finished_list = ast_stack.pop().unwrap();
        let tree = Tree::Inner(finished_list);
        let l = ast_stack.len();
        ast_stack[l - 1].push(tree);
      }
      other => {
        let l = ast_stack.len();
        ast_stack[l - 1].push(Tree::Leaf(other.to_string()))
      }
    }
  }
  ast_stack.pop().unwrap().first().unwrap().clone()
}
