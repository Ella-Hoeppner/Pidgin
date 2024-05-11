extern crate proc_macro;
use proc_macro::{TokenStream, TokenTree};

fn generic_block(block_type_name: &str, input: TokenStream) -> TokenStream {
  use TokenTree::*;
  let mut instructions: Vec<String> = vec![];
  let mut constants: Vec<String> = vec![];
  let mut tokens = input.into_iter();
  while let Some(hopefully_instruction_identifier) = tokens.next() {
    instructions.push(match hopefully_instruction_identifier {
      Ident(instruction_identifier) => {
        let input_instruction_string = instruction_identifier.to_string();
        if let Some(next_token) = tokens.next() {
          match next_token {
            Group(group) => {
              let result_instruction_string = if input_instruction_string
                == "Const"
              {
                match group.stream().into_iter().collect::<Vec<_>>().as_slice()
                {
                  [Literal(lit), Punct(separator), const_expression @ ..] => {
                    let separator_string = separator.to_string();
                    if separator_string == "," {
                      constants.push(
                        const_expression
                          .into_iter()
                          .map(|e| e.to_string())
                          .collect::<Vec<_>>()
                          .join(""),
                      );
                      format!("Const({},{})", lit, constants.len() - 1)
                    } else {
                      panic!(
                        "invalid separator in Const in program!, expected ',', 
                        got '{}'",
                        separator_string
                      )
                    }
                  }
                  _ => panic!(
                    "expected literal and expression in Const in program!"
                  ),
                }
              } else {
                input_instruction_string.to_owned() + &group.to_string()
              };
              if let Some(comma_token) = tokens.next() {
                match comma_token {
                  Punct(punct) => {
                    let punct_string = punct.to_string();
                    if punct_string == "," {
                      result_instruction_string
                    } else {
                      panic!(
                        "expected ',' after instruction in program!, got {}",
                        punct_string
                      )
                    }
                  }
                  other => panic!(
                    "expected ',' after instruction in program!, got {}",
                    other.to_string()
                  ),
                }
              } else {
                result_instruction_string
              }
            }
            Punct(punct) => {
              let punct_string = punct.to_string();
              if punct_string == "," {
                input_instruction_string
              } else {
                panic!(
                  "unexpected punctuation '{}' in program!, expecting group \
                  or ','",
                  punct_string
                )
              }
            }
            Literal(_) => {
              panic!("unexpected literal in program!, expecting group or ','")
            }
            Ident(_) => {
              panic!(
                "unexpected identifier in program!, expecting group or ','"
              )
            }
          }
        } else {
          input_instruction_string
        }
      }
      Group(_) => {
        panic!("unexpected group in program!, expecting instruction identifier")
      }
      Punct(_) => {
        panic!(
          "unexpected punctuation in program!, expecting instruction \
          identifier"
        )
      }
      Literal(_) => {
        panic!(
          "unexpected raw literal in program!, expecting instruction \
          identifier"
        )
      }
    })
  }
  format!(
    "{}::new(vec![{}], vec![{}])",
    block_type_name,
    instructions.join(", "),
    constants
      .into_iter()
      .map(|x| format!("({}).into()", x))
      .collect::<Vec<_>>()
      .join(", ")
  )
  .parse()
  .unwrap()
}

#[proc_macro]
pub fn block(input: TokenStream) -> TokenStream {
  generic_block("Block", input)
}

#[proc_macro]
pub fn ssa_block(input: TokenStream) -> TokenStream {
  generic_block("SSABlock", input)
}
