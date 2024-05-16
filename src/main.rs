#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_imports)]
/*
use std::fmt::Display;
use std::default::Default;
use std::collections::{HashMap,HashSet,BTreeSet};

mod shared_defs;
pub use shared_defs::*;
#[cfg(feature = "generator")]
mod grammar_processor;
#[cfg(feature = "generator")]
use grammar_processor::*;
#[cfg(feature = "generator")]
mod lr_statemachine;
#[cfg(feature = "generator")]
use lr_statemachine::*;
pub mod lexer_interface;
pub use lexer_interface::*;
mod augmenter;
use augmenter::*;
pub mod generic_absyn;
pub use generic_absyn::*;
pub mod runtime_parser;
use runtime_parser::*;
pub mod zc_parser;
use zc_parser::*;
#[cfg(feature = "generator")]
mod parser_writer;
#[cfg(feature = "generator")]
use parser_writer::*;
#[cfg(feature = "generator")]
mod sd_parserwriter;
#[cfg(feature = "generator")]
use sd_parserwriter::*;
#[cfg(feature = "generator")]
mod fs_parserwriter;
#[cfg(feature = "generator")]
mod ast_writer;
#[cfg(feature = "generator")]
use ast_writer::*;
#[cfg(feature = "generator")]
mod fs_astwriter;
#[cfg(feature = "generator")]
mod bumpast_writer;
#[cfg(feature = "generator")]
mod lalr_statemachine;
#[cfg(feature = "generator")]
use lalr_statemachine::LALRMachine;
#[cfg(feature = "generator")]
mod selmlk;
#[cfg(feature = "generator")]
use selmlk::{MLStatemachine};

//mod logos_lexer;
#[cfg(feature = "generator")]
mod yacc_ast;
#[cfg(feature = "generator")]
mod yaccparser;
*/

#[cfg(feature = "generator")]
fn main()
{
  let args:Vec<String> = std::env::args().collect(); // command-line args
  let res = rustlr::rustle(&args);
  match res {
    Err(s) => { eprintln!("FAILURE: {}",s); },
    Ok(s) => { println!("{}",s);},   // for command-line app only
  }//match
}//main

#[cfg(not(feature = "generator"))]
fn main() {
  println!("the `generator` feature of rustlr is not enabled");
}// alt main
