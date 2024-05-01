pub(crate) fn indent_lines(spaces: usize, s: String) -> String {
  s.split("\n")
    .map(|line| " ".repeat(spaces) + &line)
    .collect::<Vec<String>>()
    .join("\n")
}

pub(crate) fn pad(length: usize, c: char, mut s: String) -> String {
  while s.len() < length {
    s.push(c)
  }
  s
}
