#![allow(unused_imports)]
#![allow(dead_code)]
extern crate rustlr;
extern crate basic_lexer;
use basic_lexer::*;
use rustlr::{StrTokenizer,LexSource};
mod absyntax;
use absyntax::*;
mod mjparser;
use mjparser::*;
//nmod mjlexer;
//use mjlexer::*;
fn main()
{
  let args:Vec<String> = std::env::args().collect(); // command-line args
  let mut srcfile = "";
  if args.len()>1 {srcfile = &args[1];}
  let source = LexSource::new(srcfile).unwrap();
//  let mut scanner2 = Mjlexer::new(StrTokenizer::from_source(&source));
  let mut scanner2 = mjlexer::from_source(&source);
  let mut parser2 = make_parser();
  let absyntree2 = parser2.parse(&mut scanner2);
  println!("Parser Error? : {}",parser2.error_occurred());
  println!("abstract syntax tree after parse: {:?}\n",absyntree2);
}
