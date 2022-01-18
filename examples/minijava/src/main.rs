#![allow(unused_imports)]
#![allow(dead_code)]
extern crate rustlr;
extern crate basic_lexer;
use basic_lexer::*;
use rustlr::{StrTokenizer,LexSource};
mod absyntax;
use absyntax::*;
mod mjparser;
use mjparser::*;
mod mjlexer;
use mjlexer::*;
mod zcmjparser;
use zcmjparser::*;
fn main() {
  let args:Vec<String> = std::env::args().collect(); // command-line args
  let mut srcfile = "";
  if args.len()>1 {srcfile = &args[1];}
/*
  let mut scanner = Mjscanner::new(srcfile);
  let mut parser1 = make_parser();
  let absyntree = parser1.parse(&mut scanner);
  println!("abstract syntax tree after parse: {:?}\n",absyntree);  
*/
  let source = LexSource::new(srcfile).unwrap();
  let mut scanner2 = Mjlexer::new(StrTokenizer::from_source(&source));
  let mut parser2 = new_parser();
  //let absyntree2 = parser2.parse_train("zcmjparser.rs");
  let absyntree2 = parser2.parse(&mut scanner2);
  println!("Parser Error? : {}",parser2.error_occurred());
  println!("abstract syntax tree after parse: {:?}\n",absyntree2);
 // main2();
}

fn main2()// just testing
{
let source = LexSource::new("Cargo.toml").unwrap();
let mut tokenizer = StrTokenizer::from_source(&source);
tokenizer.set_line_comment("#");
tokenizer.keep_comment=true;
tokenizer.keep_newline=false;
tokenizer.keep_whitespace=false; 
while let Some(token) = tokenizer.next() {
   println!("Token: {:?}",&token);
} 
}
