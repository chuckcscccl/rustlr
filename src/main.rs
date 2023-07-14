#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_imports)]
use std::fmt::Display;
use std::default::Default;
use std::collections::{HashMap,HashSet,BTreeSet};

mod grammar_processor;
use grammar_processor::*;
mod lr_statemachine;
use lr_statemachine::*;
pub mod lexer_interface;
pub use lexer_interface::*;
pub mod runtime_parser;
use runtime_parser::*;
mod augmenter;
use augmenter::*;
pub mod generic_absyn;
pub use generic_absyn::*;
pub mod zc_parser;
use zc_parser::*;

mod parser_writer;
use parser_writer::*;
mod sd_parserwriter;
use sd_parserwriter::*;
mod fs_parserwriter;

mod ast_writer;
use ast_writer::*;

mod fs_astwriter;

mod bumpast_writer;

mod lalr_statemachine;
use lalr_statemachine::LALRMachine;
mod selmlk;
use selmlk::{MLStatemachine};

//mod logos_lexer;
mod yacc_ast;
mod yaccparser;


fn main() 
{
  let args:Vec<String> = std::env::args().collect(); // command-line args
  //let args:Vec<&str> = std::env::args().map(|x|x.as_str()).collect();
  rustlr::rustle(&args);
}//main
