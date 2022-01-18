#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_imports)]
extern crate rustlr;
use rustlr::*;
use std::fmt::Display;
use std::default::Default;
mod exprtrees;
use crate::exprtrees::*;
mod calculatorparser;
use crate::calculatorparser::*; 

fn main()
{
  println!(" testing online calculator with ambiguous grammar ... ");
  let args:Vec<String> = std::env::args().collect(); // command-line args
  let mut input =
"-5-(4-2)*5; 
5-7- -9 ; 
4*3-9; 
2+1/(2-1-1);
";
  if args.len()>1 {input = &args[1];}
  let mut lexer = exprscanner::new(input);
  let mut parser1 = make_parser();
  let absyntree = parser1.parse_train(&mut lexer,"calculatorparser.rs");
  println!("expression tree after parse: {:?}",absyntree);
  if !parser1.error_occurred() {
   println!("final result after evaluation: {}", eval(&absyntree));
  } else {
   println!("parser error, best effort after recovery: {}", eval(&absyntree));
  }
  
}//main
