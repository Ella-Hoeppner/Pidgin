fn main() {
  use pidgin::evaluate_pidgin_sexp;
  println!("{}", evaluate_pidgin_sexp("(+ 1 2)".to_string()).unwrap());
}
