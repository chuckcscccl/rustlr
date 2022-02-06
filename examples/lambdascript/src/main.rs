#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_imports)]
extern crate rustlr;
use rustlr::*;
mod abstmachine;
//use crate::abstmachine::*;

mod untyped;
use untyped::*;
mod untypedparser;

fn main()
{
  let args:Vec<String> = std::env::args().collect(); // command-line args
  let mut srcfile = "";
  if args.len()>1 {srcfile = &args[1];}
  let source = LexSource::new(srcfile).unwrap();
  let mut lexer = LamLexer::new(StrTokenizer::from_source(&source));
  let mut parser = untypedparser::make_parser();
  parser.parse(&mut lexer);
  println!("Parser Error? : {}",parser.error_occurred());
  let program = parser.exstate;
  println!("program lines after parse: {}\n",program.len());
  eval_prog(&program);
}//main

