#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_imports)]
extern crate rustlr;
use rustlr::*;
//mod abstmachine;
//use crate::abstmachine::*;
use chrono;
use chrono::Datelike;
use std::io::Write;

mod untyped;
use untyped::*;
mod untypedparser;

fn main()
{
  println!("Beta-Reducer for Untyped Lambda Calculus, by Chuck Liang.");
  println!("This program may be redistributed under the MIT license but the program will stop working after a period of time for educational purposes.\n");
  let time = chrono::offset::Local::now();
  if time.year()>2022 || time.month()>8 {
    println!("\nThe lifetime of this program has expired. A new version will be released at the appropriate time.");
    return;
  }

  let mut parser = untypedparser::make_parser();
  let args:Vec<String> = std::env::args().collect(); // command-line args
  if args.len()>1 {
    let srcfile = &args[1];
    let source = LexSource::new(srcfile).unwrap();
    let mut lexer = LamLexer::new(StrTokenizer::from_source(&source));
    parser.parse(&mut lexer);
    eval_prog(&parser.exstate);
    if parser.error_occurred() {
      println!("\nPARSER ERRORS OCCURRED, RESULTS NOT GUARANTEED");
    }
    return;
  } // source file indicated
  println!("Entering interactive mode, enter empty line to quit...");
  loop // will break from within
  {
    print!("<<< ");     let res =std::io::stdout().flush();
    let mut buf = String::new();
    let res2 = std::io::stdin().read_line(&mut buf);
    if buf.len()<3 {break;}
    let mut lexer = LamLexer::new(StrTokenizer::from_str(buf.trim()));
    parser.exstate = Vec::new(); /* needed before rustlr 0.2.1 */
    parser.parse(&mut lexer);
    //parser.parse_train(&mut lexer,"src/untypedparser.rs");    
    eval_prog(&parser.exstate);
  } // repl 
}//main
