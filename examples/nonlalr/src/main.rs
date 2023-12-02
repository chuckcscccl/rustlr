mod gram2parser;
use gram2parser::*;
mod gram2_ast;
fn main() {
  let mut input = "b b b b b c e";
  let args:Vec<String> = std::env::args().collect(); // command-line args
  if args.len()>1 {input = &args[1]; }
  let mut parser1 = make_parser();
  // parser1.set_err_report(true); // option to log errors instead of printing to stderr
  let mut tokenizer1 = gram2lexer::from_str(input);
  let result = parse_with(&mut parser1, &mut tokenizer1)
               .unwrap_or_else(|x|x);
  // println!("Error Report: {}", parser1.get_err_report()); // option
  println!("result after parsing {}: {:?}",input,result);
  // paser1.reset(); // option to reset parser before parsing from different src
}//main
