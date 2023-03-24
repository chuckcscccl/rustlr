//! Rustlr is an LR-style parser generator for Rust.  Advanced features
//! include:
//!  1. Option to automatically generate the AST datatypes and semantic actions, with manual overrides possible.  Rustlr's grammar format contains a sublanguage
//!   that controls how ASTS are created, so that the generated types do
//!   not necessarily reflect the format of the grammar.
//!  2. Option to use [bumpalo](https://docs.rs/bumpalo/latest/bumpalo/index.html) to create
//!  ASTS types that enable *nested* pattern matching against recursive types.
//! 
//!  3. Recognizes regex-style operators `*`, `+` and `?`, which simplify
//!  the writing of grammars and allow better ASTs to be created.
//!  4. An experimental feature that recognizes *Selective Marcus-Leermakers*
//!  grammars.  This is a class of unambiguous grammars that's 
//!  larger than traditional LR grammars.  They are especially helpful
//!  in avoiding conflicts when new production rules are added to a grammar.
//!  5. The ability to train the parser interactively for better error reporting
//!  6. Also generates parsers for F# and other .Net languages
//!
//! A **[tutorial](<https://chuckcscccl.github.io/rustlr_project/>)**
//! is separately available that will explain the
//! format of grammars and how to generate and deploy parsers for several 
//! examples.  The documentation found here should be used as a technical
//! reference.
//!
//! Rustlr should be installed as an executable (**cargo install rustlr**),
//! although parser generation can also be invoked with the [rustle] function.
//! Many of the items exported by this crate are only required by the parsers
//! that are generated, and are not intended to be used in other programs.
//! However, rustlr uses traits and trait objects to loosely couple the 
//! various components of the runtime parser so that custom interfaces, such as
//! those for graphical IDEs, can built around a basic [ZCParser::parse_core]
//! function.
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
mod parser_writer;
mod sd_parserwriter;
mod fs_parserwriter;
mod ast_writer;
mod fs_astwriter;

mod bumpast_writer;

mod lalr_statemachine;
mod selmlk; // experimental

//mod logos_lexer;

use lalr_statemachine::LALRMachine;
use selmlk::{MLStatemachine};

pub use lr_statemachine::{Stateaction,decode_action};
pub use runtime_parser::{RuntimeParser,RProduction};
pub use zc_parser::{ZCParser,ZCRProduction};
//pub use enhancements::{ParseValue,ParseResult,Enhanced_Lexer};

pub const VERSION:&'static str = "0.4.5";

// main function, called from main with command-line args

/// This is the function called from main in the rustlr executable once
/// rustlr has been installed with **cargo install rustlr**.  This
/// function can also be called from another rust program to generate a
/// parser: add `rustlr = "0.4" to Cargo dependencies.  It accepts the same
/// command-line arguments as the executable in a vector of strings. See
/// the documentation and tutorial on how to use rustlr as an executable.
pub fn rustle(args:&Vec<String>) // called from main
{
  let argc = args.len();
  if argc<2 {eprintln!("Must give path of .grammar file"); return;}
  let filepath = &args[1];
  let mut parserfile = String::from("");  // -o target
  let mut lalr = false;  // changed from false in version 0.2.0
  let mut newlalr = true;
  let mut tracelev:usize = 1; // trace-level
  let mut verbose = false;
  let mut zc = true;
  let mut genlex = false;
  let mut genabsyn = false;
  let mut lrsd = false;
  let mut lrsdmaxk:usize = selmlk::MAXK;
  let mut regenerate = false;
  let mut mode = 0;
  let mut argi = 2; // next argument position
  while argi<argc
  {
     match &args[argi][..] {
       //filen if filen.ends_with(".grammar") => {filepath = &args[argi];},
       "lr1" | "LR1" | "-lr1" => { lalr=false; newlalr=false; },
       "lalr" | "LALR" | "-lalr" => {newlalr=true; },
       "lalr1" | "LALR1" | "-lalr1" => {newlalr=true; },
       "oldlalr" | "-oldlalr" | "-selML" => {newlalr=false; lalr=true;}
       "-lrsd" | "lrsd" => {
         newlalr=false; lalr=false; lrsd=true;
         if argi+1<argc {
           if let Ok(mk)=args[argi+1].parse::<usize>() {
             lrsdmaxk=mk; argi+=1;
           } // next arg is number
         }//if next arg exists
       },
       "-regenerate" => { regenerate=true; },
       "-fsharp" => {mode=1;},
       "-trace" => {
          argi+=1;
          if argi<argc {
            if let Ok(lv) = args[argi].parse::<usize>() {tracelev=lv; }
          if tracelev>0 {println!("trace-level set to {}",tracelev);}
          }
       },
       "verbose" | "-verbose" => { verbose=true; },
       "-zc" | "zero_copy" => {zc=true;},
       "genlex" | "-genlex" => {genlex=true; },
       "-genabsyn" | "-ast" | "-auto" => {genabsyn = true; },
       "-nozc" => {zc=false;},
       "binary" | "-binary" => { verbose=false; },       
       "-o" => {
          argi+=1;
          if argi<argc {parserfile = args[argi].clone();}
       },
       _ => {},    
     }//match directive
     argi+=1;
  }//while there are command-line args
  if zc && verbose {
     println!("verbose mode not compatible with -zc option");
     return;
  }
  if tracelev>0 && verbose {println!("verbose parsers should be used for diagnositic purposes and cannot be trained/augmented");}
  if tracelev>1 {println!("parsing grammar from {}",&filepath);}
  let mut grammar1 = Grammar::new();
  grammar1.genlex = genlex;
  grammar1.genabsyn = genabsyn;
  grammar1.tracelev = tracelev;
  grammar1.mode = mode; // 0 for rust, 1 for fsharp
  let parsedok = grammar1.parse_grammar(filepath);  //  ***
  if !parsedok {
    println!("\nFailed to process grammar");
    return;
  }
  // Check grammar integrity: now done inside parse
//  let topi = *grammar1.Symhash.get(&grammar1.topsym).expect("FATAL ERROR: Grammar start symbol 'topsym' not defined");
//  let toptype = &grammar1.Symbols[topi].rusttype;
  if grammar1.name.len()<2  { // derive grammar name from filepath
     let doti = if let Some(p)= filepath.rfind('.') {p} else {filepath.len()};
     let mut slashi = if let Some(p) = filepath.rfind('/') {p+1} else {0};
     if slashi==0 {
       slashi = if let Some(p) = filepath.rfind('\\') {p+1} else {0};
     }
     grammar1.name = filepath[slashi..doti].to_string();
  }// derive grammar name
  let gramname = grammar1.name.clone();

  let pfsuffix = if mode==1 {"fs"} else {"rs"};

  if grammar1.genabsyn {
     let mut slashpos = parserfile.rfind('/');
     if let None = slashpos {slashpos = parserfile.rfind('\\');}
     let mut astpath = format!("{}_ast.{}",&gramname,pfsuffix);
     if let Some(pos) = slashpos { astpath=format!("{}{}",&parserfile[..pos+1],&astpath); }
     let wres;
     if mode==1 {wres = grammar1.write_fsast(&astpath); }
     else if !grammar1.bumpast { wres = grammar1.writeabsyn(&astpath); }
     else {wres = grammar1.write_bumpast(&astpath); }
     if !wres.is_ok() {eprintln!("Failed to generate abstract syntax"); return;}
  }

 grammar1.delay_transform(); // static delayed reduction markers


  if tracelev>2 {println!("computing Nullable set");}
  grammar1.compute_NullableRf();
  if tracelev>2 {println!("computing First sets");}
  grammar1.compute_FirstIM();

  let mut fsm0;
  if lrsd {
    let mut lrsdfsm = MLStatemachine::new(grammar1);
    lrsdfsm.regenerate = regenerate;
    println!("Generating Experimental LR-Selective Delay State Machine with Max Delay = {}",lrsdmaxk);
    lrsdfsm.selml(lrsdmaxk);
    //fsm0 = lrsdfsm.to_statemachine();
    if lrsdfsm.failed {println!("NO PARSER GENERATED"); return;}
    if !lrsdfsm.failed && lrsdfsm.regenerate { 
      println!("Re-Generating LR(1) machine for transformed grammar...");
      lrsd = false;
      fsm0 = Statemachine::new(lrsdfsm.Gmr);
      fsm0.lalr = false;
      fsm0.generatefsm(); //GENERATE THE FSM
    } else {     fsm0 = lrsdfsm.to_statemachine(); }
    // but of course there will be more conflicts since there will be
    // more rules.  The original rules that caused conflicts for LR are
    // still there??

  } else  // not lrsd
  if newlalr { // newlalr takes precedence over other flags
     let mut lalrfsm = LALRMachine::new(grammar1);
     println!("Generating LALR(1) state machine");
     lalrfsm.generatefsm();
     fsm0 = lalrfsm.to_statemachine();
  }
  else {
    fsm0 = Statemachine::new(grammar1);
    fsm0.lalr = lalr;
    if lalr {fsm0.Open = Vec::with_capacity(1024); } // important
    if tracelev>0 {println!("Generating {} state machine for grammar {}...",if lalr {"older LALR"} else {"LR1"},&gramname);}
    fsm0.generatefsm(); //GENERATE THE FSM
  } // old code
  if tracelev>2 && !newlalr && !lrsd { for state in &fsm0.States {printstate(state,&fsm0.Gmr);} }
  else if tracelev>1 && !newlalr && !lrsd {   printstate(&fsm0.States[0],&fsm0.Gmr); }//print states
  if parserfile.len()<1 || parserfile.ends_with('/') || parserfile.ends_with('\\') {parserfile.push_str(&format!("{}parser.{}",&gramname,pfsuffix));}
  if fsm0.States.len()>65536  {
    println!("too many states: {} execeeds limit of 65536",fsm0.States.len());
    return;
  }
  let write_result =
    if mode==1 { fsm0.writefsparser(&parserfile) }
    else
    if zc {  // write zero-copy parser
      //fsm0.writezcparser(&parserfile)
      //fsm0.writelbaparser(&parserfile)
      if !lrsd {fsm0.writeenumparser(&parserfile)}
      else {fsm0.writelrsdparser(&parserfile)}
    }
    else {  // non-zc, original before version 0.2.0
      if verbose /*fsm0.States.len()<=16*/ {fsm0.write_verbose(&parserfile)}
      else {fsm0.writeparser(&parserfile)}
    }; // write_result =
  if tracelev>0 && !lrsd {eprintln!("{} total states",fsm0.FSM.len());}
  if let Ok(_) = write_result {
     if tracelev>0 {println!("Parser saved in {}",&parserfile);}
  }
  else if let Err(err) = write_result {
     println!("failed to write parser, likely due to invalid -o destination: {:?}",err);    
  }
}//rustle


/*
fn rustler(grammarname:&str, option:&str) {
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
*/
