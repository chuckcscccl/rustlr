#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(dead_code)]
mod wc_ast;
use wc_ast::*;
mod wcparser;
use wcparser::*;

fn main()
{
  let args:Vec<String> = std::env::args().collect(); // command-line args
  let mut input = "a b j 44";
  if args.len()>1 {input = args[1].as_str();}
   let scanner4 = wcparser::wclexer::from_str(input);
   let mut parser4 = wcparser::make_parser(scanner4);
   let tree4= wcparser::parse_with(&mut parser4);
   let result4 = tree4.unwrap_or_else(|x|{println!("Parsing errors encountered; results are partial.."); x});
   println!("\nABSYN: {:?}\n",&result4);
}//main
