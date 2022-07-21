#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(dead_code)]
mod json_ast;
use json_ast::*;
mod jsonparser;
use jsonparser::*;
use rustlr::{StrTokenizer,LexSource};

fn main()
{
  let args:Vec<String> = std::env::args().collect(); // command-line args
  let mut srcfile = "person.json";
  if args.len()>1 {srcfile = &args[1];}
  let source = LexSource::new(srcfile).unwrap();
  let mut scanner4 = jsonlexer::from_source(&source);
  
  //let mut input = "[1,2,5.6,true]";
  // let mut scanner4 = jsonparser::jsonlexer::from_str(input);
   let mut parser4 = jsonparser::make_parser();
   let tree4= jsonparser::parse_with(&mut parser4, &mut scanner4);
   let result4 = tree4.unwrap_or_else(|x|{println!("Parsing errors encountered; results not guaranteed.."); x});
   println!("\nABSYN: {:?}\n",&result4);
}//main

