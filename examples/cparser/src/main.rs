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
mod preprocessorparser;
mod preprocessor_ast;

fn main()
{
  let args:Vec<String> = std::env::args().collect(); // command-line args
  let mut srcfile = "test1.c";
  if args.len()>1 {srcfile = &args[1];}
  let sourceopt = LexSource::new(srcfile);
  if sourceopt.is_err() {return;}
  let source = sourceopt.unwrap();

 // parse pre-processor directives first
 let mut scanner1 = preprocessorparser::preprocessorlexer::from_source(&source);
 let mut parser1 = preprocessorparser::make_parser(scanner1);
 let result1 = preprocessorparser::parse_with(&mut parser1);
 println!("PREPROCESSOR AST: {:?}",&result1);
 println!("--- Completed Preprocessor Parsing ---");

  let mut scanner2 = c11lexer::from_source(&source);
  
  //let mut parser2 = make_parser();   // for older, ZCParser
  let mut parser2 = make_parser(scanner2);
  
  let mut training = false;
  if args.len()>2 && &args[2]=="train" {training = true;}
  let result2;
  
  //if training {result2=parse_train_with(&mut parser2, &mut scanner2,"src/c11parser.rs");}
  //else {result2 = parse_with(&mut parser2, &mut scanner2);}

  if training {result2=parse_train_with(&mut parser2,"src/c11parser.rs");}
  else {result2 = parse_with(&mut parser2);}
  
  let absyntree2 = result2.unwrap_or_else(|x|{println!("Parsing Errors Encountered"); x});
  println!("abstract syntax tree after parse: {:?}\n",absyntree2);
}//main
