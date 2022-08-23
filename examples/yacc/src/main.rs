#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(dead_code)]
use rustlr::{LexSource,Tokenizer};
mod yacc_ast;
use yacc_ast::*;
mod yaccparser;
use yaccparser::*;

fn main()
{
  let args:Vec<String> = std::env::args().collect();
  let mut srcfile = "test1.y";
  if args.len()>1 {srcfile = &args[1];}
  let sourceopt = LexSource::new(srcfile);
  if sourceopt.is_err() {return;}
  let source = sourceopt.unwrap();

   let mut scanner4 = yaccparser::yacclexer::from_source(&source);
   let mut parser4 = yaccparser::make_parser();
   let tree4= yaccparser::parse_with(&mut parser4, &mut scanner4);
   let result4 = tree4.unwrap_or_else(|x|{println!("Parsing errors encountered; results are partial.."); x});
   println!("\nABSYN: {:?}\n",&result4);
}//main
