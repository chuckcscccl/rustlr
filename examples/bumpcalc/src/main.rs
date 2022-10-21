#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
//extern crate rustlr;
//use rustlr::*;
mod bxprtrees;
use crate::bxprtrees::*;
mod bcalcparser;
use rustlr::{LexSource};

fn main()
{
  let args:Vec<String> = std::env::args().collect(); // command-line args
  let mut srcfile = "testinput.txt";
  if args.len()>1 {srcfile = &args[1];}
  let source = LexSource::new(srcfile).expect("Cannot open source file");
   let mut scanner4 = bcalcparser::bcalclexer::from_source(&source);
   let bump = bumpalo::Bump::new();
   let mut parser4 = bcalcparser::make_parser();
   parser4.exstate.set(&bump);
   let tree4= bcalcparser::parse_with(&mut parser4, &mut scanner4);
   let result4 = tree4.unwrap_or_else(|x|{println!("Parsing errors encountered; results are partial.."); x});
   println!("ast: {:?}",&result4);
   
   let bindings4 = newenv();
   println!("result after eval: {:?}", eval(&bindings4,&result4));

   //let lexer:& dyn Tokenizer<'_,_> = &scanner4;
   //println!("\nline 10: {}",lexer.get_line(10).unwrap());
   //   println!("\nline 10: {}",scanner4.get_line(10).unwrap());
   // interesting: only need to use Tokenizer for it to recognize function,
   // don't need to typecast
}//main
