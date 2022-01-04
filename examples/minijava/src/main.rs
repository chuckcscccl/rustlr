#![allow(unused_imports)]
extern crate rustlr;
extern crate basic_lexer;
mod absyntax;
use absyntax::*;
mod mjparser;
use mjparser::*;
mod mjlexer;
use mjlexer::*;
fn main() {
  let args:Vec<String> = std::env::args().collect(); // command-line args
  let mut srcfile = "";
  if args.len()>1 {srcfile = &args[1];}
  let mut scanner = Mjscanner::new(srcfile);

  let mut parser1 = make_parser();
  let absyntree = parser1.parse(&mut scanner);
  println!("abstract syntax tree after parse: {:?}\n",absyntree);
}
