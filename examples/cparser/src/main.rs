#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(dead_code)]
use rustlr::{LexSource,Tokenizer};
/*
mod cauto_ast;
mod cautoparser;
use cautoparser::*;
*/
mod c11_ast;
mod c11parser;
use c11parser::*;

fn main()
{
  let args:Vec<String> = std::env::args().collect(); // command-line args
  let mut srcfile = "test1.c";
  if args.len()>1 {srcfile = &args[1];}
  let source = LexSource::new(srcfile).unwrap();
  let mut scanner2 = c11lexer::from_source(&source);
  let mut parser2 = make_parser();
  let result2 = parse_with(&mut parser2, &mut scanner2);
  let absyntree2 = result2.unwrap_or_else(|x|{println!("Parsing Errors Encountered"); x});
  println!("abstract syntax tree after parse: {:?}\n",absyntree2);
}//main
