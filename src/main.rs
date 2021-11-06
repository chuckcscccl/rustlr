#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_imports)]
use std::fmt::Display;
use std::default::Default;
use std::collections::{HashMap,HashSet,BTreeSet};
//mod lib;
//use lib::*;
//use crate::grammar_processor::{TRACE}
//mod bunch;
//use crate::bunch::*;
//mod runtime_parser;
//use crate::runtime_parser::*;

mod grammar_processor;
use grammar_processor::*;
mod lr_statemachine;
use lr_statemachine::*;
mod lexer_interface;
pub use lexer_interface::*;
mod runtime_parser;
use runtime_parser::*;
use rustlr::rustler;

fn main() 
{
  let args:Vec<String> = std::env::args().collect(); // command-line args
  let grammar_name = if args.len()>1 {&args[1]} else {"test1"};
  let option = if args.len()==3 {&args[2]} else {"lr1"}; // lr1 or lalr
  rustler(grammar_name,option);
}//main

