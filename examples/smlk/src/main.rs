#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(dead_code)]
mod godd_ast;
mod goddparser;
use goddparser::*;

mod nuttycalcparser;
mod nuttycalc_ast;

fn main()
{
  let args:Vec<String> = std::env::args().collect(); // command-line args
  let mut input = "d a b c d y";
  if args.len()>1 {input = args[1].as_str();}
   let mut scanner4 = goddparser::goddlexer::from_str(input);
   let mut parser4 = goddparser::make_parser();
   let tree4= goddparser::parse_with(&mut parser4, &mut scanner4);
   let result4 = tree4.unwrap_or_else(|x|{println!("Parsing errors encountered; results are partial.."); x});
   println!("\nABSYN: {:?}\n",&result4);
}//main
