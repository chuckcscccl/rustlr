//! rustlr is a parser generator that can create LALR(1) as well as full
//! LR(1) parsers.  It is also capable of recognizing operator precedence and
//! associativity declarations that allows the use of some ambiguous grammars.
//! Parsers also have optional access to *external state* information that allows
//! them to recognize more than just context-free languages.  A 
//! classical method of error recovery is used.  The parser can generate a full
//! LR(1) parser given the grammar for Java in
//! approximately 10-20 seconds on contemporary processors.
//!
//! Most of the items
//! exported by this crate are only required by the parsers that are generated,
//! and does not form an API. The
//! user needs to provide a grammar and a lexical analyzer that implements
//! the [Lexer] trait.  Only a simple lexer that returns individual characters
//! in a string ([charlexer]) is provided.  
//!
//! Example
//!
//! Given the grammar at <https://cs.hofstra.edu/~cscccl/rustlr_project/calculator.grammar>,
//!```\ignore
//! rustlr calculator.grammar lr1
//!```
//! generates a LR(1) parser as a rust program 
//! (<https://cs.hofstra.edu/~cscccl/rustlr_project/calculatorparser.rs>).
//! This program includes a make_parser function, which can be used as in
//!
//!```ignore
//! let mut scanner = Exprscanner::new(&sourcefile);
//! let mut parser1 = make_parser();
//! let absyntree = parser1.parse(&mut scanner);
//!```
//! Here, Exprscanner is a structure that must implement the [Lexer] trait 
//! required by the generated parser.
//!
//! A relatively self-contained example, containing both a grammar and code for
//! using its generated parser, is at
//! <https://cs.hofstra.edu/~cscccl/rustlr_project/cpm.grammar>.
//!
//!
//! A detailed tutorial is being prepared at
//! <https://cs.hofstra.edu/~cscccl/rustlr_project/> that will explain the
//! format of grammars and how to generate and use parsers for several sample
//! languages.
//! The examples in the tutorial use basic_lexer
//! (<https://docs.rs/basic_lexer/0.1.2/basic_lexer/>), which was written by
//! the same author but other tokenizers can be easily adopted
//! as well, such as scanlex (<https://docs.rs/scanlex/0.1.4/scanlex/>).


#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_assignments)]
#![allow(unused_doc_comments)]
#![allow(unused_imports)]
//use std::default::Default;
mod grammar_processor;
use grammar_processor::*;
mod lr_statemachine;
use lr_statemachine::*;
mod lexer_interface;
pub use lexer_interface::*;
mod runtime_parser;
use runtime_parser::*;
mod enhancements;
pub use enhancements::*;

pub use lr_statemachine::{Stateaction,decode_action};
//pub use enhancements::{ParseValue,ParseResult,Enhanced_Lexer};
pub use runtime_parser::{RuntimeParser,RProduction};

////// main function, called from main with command-line args

/// this is the only function that can invoke the parser generator externally,
/// without running rustlr (rustlr::main) directly.
/// It expects to find a file of the form grammarname.grammar.
/// The option argument that can currently only be "lr1" or "lalr".  It generates
/// a grammar in a file named grammarnameparser.rs.
///
/// Example:
///
/// Given the grammar at <https://cs.hofstra.edu/~cscccl/rustlr_project/test1.grammar>,
///```ignore
/// rustler("test1","lalr");
///```
/// would generate the file
///<https://cs.hofstra.edu/~cscccl/rustlr_project/test1parser.rs>.
/// Since this grammar is small enought (requiring less than 16 LALR states), the
/// generated parser is readable, which is appropriate for testing.  For larger
/// grammars, the parser generator switches to a binary representation.
pub fn rustler(grammarname:&str, option:&str) {
  let mut gram1 = Grammar::new();
  let grammarfile = format!("{}.grammar",&grammarname);

  let lalr =  match option {
    "lalr" | "LALR" => true,   
    "lr1" | "LR1" => false,
    _ => {println!("Option {} not supported, defaulting to full LR1 generation",option); false},
  };
  
  if TRACE>1 {println!("parsing grammar from {}",grammarfile);}
  gram1.parse_grammar(&grammarfile);
  if TRACE>2 {println!("computing Nullable set");}
  gram1.compute_NullableRf();
  if TRACE>2 {println!("computing First sets");}
  gram1.compute_FirstIM();
  if gram1.name.len()<2 {gram1.name = grammarname.to_owned(); }
  let gramname = gram1.name.clone();
  /*
  for nt in gram1.First.keys() {
     print!("First({}): ",nt);
     let firstnt = gram1.First.get(nt).unwrap();
     for tt in firstnt { print!("{} ",tt); }
     println!();
  }//print first set
  */
  let mut fsm0 = Statemachine::new(gram1);
  fsm0.lalr = lalr;
  if lalr {fsm0.Open = Vec::with_capacity(1024); }
  println!("Generating {} state machine for grammar...",if lalr {"LALR"} else {"LR1"});
  fsm0.generatefsm();
  if TRACE>1 { for state in &fsm0.States {printstate(state,&fsm0.Gmr);} }
  else if TRACE>0 {   printstate(&fsm0.States[0],&fsm0.Gmr); }//print state
  let parserfile = format!("{}parser.rs",&gramname);
  let write_result = 
    if fsm0.States.len()<=16 {fsm0.write_verbose(&parserfile)}
    else if fsm0.States.len()<=65536 {fsm0.writeparser(&parserfile)}
    else {panic!("too many states: {}",fsm0.States.len())};
  println!("{} total states",fsm0.States.len());
  if let Ok(_) = write_result {println!("written parser to {}",&parserfile);}
}//rustler
