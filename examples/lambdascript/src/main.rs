#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_imports)]
extern crate rustlr;
use rustlr::*;
extern crate fixedstr;
use fixedstr::str8;
//mod abstmachine;
//use crate::abstmachine::*;
use chrono;
use chrono::{Datelike,Timelike};
use std::io::Write;
use std::collections::HashMap;

mod untyped;
use untyped::*;
mod untypedparser;
use untypedparser::*;

fn main()
{
  println!("Beta-Reducer for Untyped Lambda Calculus, by Chuck Liang.");
  println!("For educational reasons this program may be temporarily disabled during certain time periods");
  let time = chrono::offset::Local::now();

  if time.year()>2022 || time.month()>8 {
    println!("\nThe lifetime of this program has expired. A new version will be released at the appropriate time.");
    return;
  }

  if time.year()==2022 && time.month()==2 && time.day()>=15 && time.hour()>=4 && time.minute()>=20 && time.day()<=17 {
    println!("\nThis tool is temporarily disabled because of online exams in CSC252DL");
    return;
  }

  let mut parser = make_parser();
  let ref mut defs = HashMap::<str8,Term>::new();
  let args:Vec<String> = std::env::args().collect(); // command-line args
  if args.len()>1 {
    let srcfile = &args[1];
    let source = LexSource::new(srcfile).unwrap();
//    let mut lexer = LamLexer::new(StrTokenizer::from_source(&source));
    let mut lexer = untypedlexer::from_source(&source);
    parser.parse(&mut lexer);
    //parser.parse_train(&mut lexer,"src/untypedparser.rs");        
    eval_prog(&parser.exstate,defs);
    if parser.error_occurred() {
      println!("\nPARSER ERRORS OCCURRED, RESULTS NOT GUARANTEED");
    }
    //return;
  } // source file indicated
  println!("Entering interactive mode, enter 'exit' to quit...");
  loop // will break from within
  {
    print!("<<< ");     let res =std::io::stdout().flush();
    let mut buf = String::new();
    let res2 = std::io::stdin().read_line(&mut buf);
    if buf.len()<3 {continue;}
    else if buf.trim()=="exit" || buf.trim()=="quit" {break;}
    //let mut lexer = LamLexer::new(StrTokenizer::from_str(buf.trim()));
    let mut lexer = untypedlexer::from_str(buf.trim());

    parser.parse(&mut lexer);
    //parser.parse_train(&mut lexer,"src/untypedparser.rs");    

    eval_prog(&parser.exstate,defs);
  } // repl 
}//main

