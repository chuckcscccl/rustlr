#![allow(unused_imports)]
#![allow(dead_code)]
extern crate rustlr;
extern crate basic_lexer;
use basic_lexer::*;
use rustlr::{StrTokenizer,LexSource};
/*
mod absyntax;
use absyntax::*;
mod mjparser;
use mjparser::*;

fn main()
{
  let args:Vec<String> = std::env::args().collect(); // command-line args
  let mut srcfile = "";
  if args.len()>1 {srcfile = &args[1];}
  let source = LexSource::new(srcfile).unwrap();
  let mut scanner2 = mjlexer::from_source(&source);
  let mut parser2 = make_parser();
  let absyntree2 = parser2.parse(&mut scanner2);
  println!("Parser Error? : {}",parser2.error_occurred());
  println!("abstract syntax tree after parse: {:?}\n",absyntree2);
}
*/

mod enumabsyn;
use enumabsyn::*;
mod mjenumparser;
use mjenumparser::*;

fn main()
{
  let args:Vec<String> = std::env::args().collect(); // command-line args
  let mut srcfile = "";
  if args.len()>1 {srcfile = &args[1];}
  let source = LexSource::new(srcfile).unwrap();
  let mut scanner3 = mjenumlexer::from_source(&source);
  let mut parser3 = make_parser();
  let absyntree3 = parse_with(&mut parser3, &mut scanner3);
  println!("Parser Error? : {}",parser3.error_occurred());
  println!("abstract syntax tree after parse: {:?}\n",absyntree3);
}//main

