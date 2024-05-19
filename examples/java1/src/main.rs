#![allow(unused_imports)]


mod java14auto_ast;
use java14auto_ast::*;
mod java14autoparser;

//mod java15_ast;
//mod java15parser;

use rustlr::{LexSource,Tokenizer};
/*
fn main()
{
  let args:Vec<String> = std::env::args().collect(); // command-line args
  let mut srcfile = "";
  if args.len()>1 {srcfile = &args[1];}
  else {eprintln!("no source file given");}
  let source = LexSource::new(srcfile).expect("Cannot open source file");
  let mut scanner = java15parser::java15lexer::from_source(&source);
  //let mut scanner = java15parser::java15lexer::from_str("...");  
  let mut parser = java15parser::make_parser();
  let result= java15parser::parse_with(&mut parser, &mut scanner);
  let ast = result.unwrap_or_else(|tree|{println!("Parsing errors encountered; results not guaranteed.."); tree});
  println!("\nAbstract Syntax: {:?}\n",&ast);
  println!("\nParsing errors: {}",parser.error_occurred());
   
}//main
*/
/*
fn main()
{
  let args:Vec<String> = std::env::args().collect(); // command-line args
  let mut srcfile = "";
  if args.len()>1 {srcfile = &args[1];}
  else {eprintln!("no source file given");}
  let source = LexSource::new(srcfile).expect("Cannot open source file");
  let mut scanner = java14autoparser::java14autolexer::from_source(&source);
  //let mut scanner = java14autoparser::java14autolexer::from_str("...");  
  let mut parser = java14autoparser::make_parser();
  let result= java14autoparser::parse_with(&mut parser, &mut scanner);
  let ast = result.unwrap_or_else(|tree|{println!("Parsing errors encountered; results not guaranteed.."); tree});
   println!("\nAbstract Syntax: {:?}\n",&ast);
}//main
*/

// for base_parser
fn main()
{
  let args:Vec<String> = std::env::args().collect(); // command-line args
  let mut srcfile = "";
  if args.len()>1 {srcfile = &args[1];}
  else {eprintln!("no source file given");}
  let source = LexSource::new(srcfile).expect("Cannot open source file");
  let mut scanner = java14autoparser::java14autolexer::from_source(&source);
  //let mut scanner = java14autoparser::java14autolexer::from_str("...");  
  let mut parser = java14autoparser::make_parser(scanner);
  let result= java14autoparser::parse_with(&mut parser);
  let ast = result.unwrap_or_else(|tree|{println!("Parsing errors encountered; results not guaranteed.."); tree});
   println!("\nAbstract Syntax: {:?}\n",&ast);
}//main
