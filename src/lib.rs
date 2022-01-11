//! rustlr is a parser generator that can create LALR(1) as well as full
//! LR(1) parsers.  It is also capable of recognizing operator precedence and
//! associativity declarations that allows the use of some ambiguous grammars.
//! Parsers also have optional access to *external state* information that allows
//! them to recognize more than just context-free languages.  Rustlr implements
//! methods of error recovery similar to those found in other LR generators.
//! For error reporting, however, rustlr parsers can run in
//! *training mode*, in which, when a parse error is encountered, the human 
//! trainer can augment the parser's state transition table with an
//! appropriate error message. The augmented parser is automatically saved
//! along with a training script that can be used to retrain a new parser after
//! a grammar has been modified.
//! See the [RuntimeParser::parse_stdio_train] and [RuntimeParser::train_from_script] functions.
//!
//! The parser can generate a full
//! LR(1) parser given the ANSI C grammar in
//! approximately 2-4 seconds on contemporary processors.
//!
//! A [**detailed tutorial**](<https://cs.hofstra.edu/~cscccl/rustlr_project/>)
//! is being prepared that will explain the
//! format of grammars and how to generate and use parsers for several sample
//! languages.
//!
//! Rustlr should be installed as an executable (cargo install rustlr).
//! Many of the items exported by this crate are only required by the parsers
//! that are generated, and are not intended to be used in other programs.
//! The user needs to provide a grammar and a lexical analyzer that implements
//! the [Lexer] trait.  Only a simple lexer that returns individual characters
//! in a string ([charlexer]) is provided.
//! The examples in the tutorial use
//! [basic_lexer](<https://docs.rs/basic_lexer/0.1.2/basic_lexer/>), which was written by
//! the same author but other tokenizers can be easily adopted
//! as well, such as [scanlex](<https://docs.rs/scanlex/0.1.4/scanlex/>).
//!
//! As a simplified, **self-contained example** of how to use rustlr,
//! given **[this grammar](<https://cs.hofstra.edu/~cscccl/rustlr_project/brackets.grammar>)** with file name "brackets.grammar",
//!```\ignore
//! rustlr brackets.grammar lalr
//!```
//! generates a LALR parser as 
//! [a rust program](<https://cs.hofstra.edu/~cscccl/rustlr_project/bracketsparser.rs>).
//! This program includes a 'make_parser' function, which can be used as in
//! the included main.  The program also contains a 'load_extras' function,
//! which can be modified by interactive training to give more helpful error
//! messages other than the generic *"unexpected symbol.."*.
//!
//! Another self-contained example is found
//! [here](<https://cs.hofstra.edu/~cscccl/rustlr_project/cpm.grammar>).
//!
//!


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


pub use lr_statemachine::{Stateaction,decode_action};
pub use runtime_parser::{RuntimeParser,RProduction};
pub use zc_parser::{ZCParser,ZCRProduction};
//pub use lexer_interface::{Lexer,Lextoken,charlexer};
//pub use lexer_interface::{TerminalToken,Tokenizer,RawToken,StrTokenizer,LexSource};
//pub use enhancements::{ParseValue,ParseResult,Enhanced_Lexer};

////// main function, called from main with command-line args

/// this is the only function that can invoke the parser generator externally,
/// without running rustlr (rustlr::main) directly.
/// It expects to find a file of the form grammarname.grammar.
/// The option argument that can currently only be "lr1" or "lalr".  It generates
/// a grammar in a file named grammarnameparser.rs.
///
/// Example:
///
/// Given the grammar called [test1.grammar](<https://cs.hofstra.edu/~cscccl/rustlr_project/test1.grammar>),
///```ignore
/// rustler("test1","lalr");
///```
/// would generate 
///[this parser](<https://cs.hofstra.edu/~cscccl/rustlr_project/test1parser.rs>).
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
