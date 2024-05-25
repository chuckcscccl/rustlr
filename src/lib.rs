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
//! A **[TUTORIAL](<https://chuckcscccl.github.io/rustlr_project/>)**
//! is separately available that will explain the
//! format of grammars and how to generate and deploy parsers for several 
//! examples.  The documentation found here should be used as a technical
//! reference.
//!
//! **INSTALLING RUSTLR**
//!
//! Rustlr consists of two main components: the parser generation routines and
//! the runtime parser routines that interpret the generated parsing tables.
//! The default installation will install both.  However, the runtime parser
//! can be installed independently.
//!
//! 
//! Rustlr should first be installed as a command-line application:
//! **`cargo install rustlr`**.  This will install both the generator and
//! runtime parser.
//!
//! Parser generation can also be invoked from within a rust
//! program with the [generate] function of the rustlr crate.
//!
//! Once a parser has been generated and included in another crate, rustlr
//! should be installed with only the runtime parsing routines with
//! **`cargo add rustlr --no-default-features`**.  Alternatively, add the
//! the following to your Cargo.toml:
//! ```
//!   [dependencies]
//!   rustlr = { version = "0.5", default-features = false }
//! ```
//!
//! **Compatibility Notice:**
//!
//! There is another optional feature, `legacy-parser`, that can be enabled
//! with or without the parser generation routines, that is required for
//! grammars and parsers for very old versions of rustlr (prior to version 0.2).
//! This feature is *not* included by default and must be installed with
//! the `cargo install/add --features legacy-parser` option.
//!
//! Many of the items exported are only required by the parsers
//! that are generated, and are not intended to be used in other programs.
//! However, rustlr uses traits and trait objects to loosely couple the 
//! various components of the runtime parser so that custom interfaces, such as
//! those for graphical IDEs, can be built around a basic [ZCParser::parse_core]
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
pub mod runtime_parser;
pub use runtime_parser::*;
mod augmenter;
use augmenter::*;
pub mod generic_absyn;
pub use generic_absyn::*;
pub mod zc_parser;
#[cfg(feature = "generator")]
mod parser_writer;
#[cfg(feature = "generator")]
mod sd_parserwriter;
#[cfg(feature = "generator")]
mod fs_parserwriter;
#[cfg(feature = "generator")]
mod ast_writer;
#[cfg(feature = "generator")]
mod fs_astwriter;
#[cfg(feature = "generator")]
mod bumpast_writer;
#[cfg(feature = "generator")]
mod lalr_statemachine;
#[cfg(feature = "generator")]
mod selmlk; // experimental

pub mod base_parser; // experimental
pub use base_parser::{BaseParser,BaseProduction};

//mod logos_lexer;

#[cfg(feature = "generator")]
mod yacc_ast;
#[cfg(feature = "generator")]
mod yaccparser;
#[cfg(feature = "generator")]
use lalr_statemachine::LALRMachine;
#[cfg(feature = "generator")]
use selmlk::{MLStatemachine};
pub use zc_parser::{ZCParser,ZCRProduction};
#[cfg(feature = "legacy-parser")]
pub use runtime_parser::{RuntimeParser,RProduction,StackedItem};

pub const RUSTLRVERSION:&'static str = "0.6.0";

/// This function can be called from within Rust to generate a parser/lexer.
/// It takes the same arguments as the rustlr command-line application.
/// Furthermore, if given the `-trace 0` option, no output will be
/// sent to stdout or stderr.  Instead, a log of events is recorded and
/// is returned.  An `Ok(_)` result indicates that some parser was created
/// and an `Err(_)` result indicates failure.
/// Example:
/// ```ignore
///   let report = rustlr::generate("simplecalc.grammar -o src/main.rs -trace 0");
/// ```
#[cfg(feature = "generator")]
pub fn generate(argv:&str) -> Result<String,String> {
  let asplit:Vec<_> = argv.split_whitespace().collect();
  rustle1(&asplit)
}


/// This function is retained for backwards compatiblity.  It is recommended
/// to call [generate] instead.
#[cfg(feature = "generator")]
pub fn rustle(args:&Vec<String>) -> Result<String,String> // called from main
{
  let mut args2 = Vec::new();
  for s in args { args2.push(&s[..]); }
  rustle1(&args2[..])
}
#[cfg(feature = "generator")]
fn rustle1(args:&[&str]) -> Result<String,String> // called from main
{
  let argc = args.len();
  if argc<2 {
    //eprintln!("Must give path of .grammar file"); return;
    return Err("Must give path of .grammar file".to_owned());
  }
  let mut filepath = "";
  let mut parserfile = String::from("");  // -o target
  let mut lalr = false;  // changed from false in version 0.2.0
  let mut newlalr = true;
  let mut tracelev:usize = 1; // trace-level
  let mut verbose = false;
  let mut zc = false;
  let mut newbase = true;
  let mut genlex = false;
  let mut genabsyn = false;
  let mut lrsd = false;
  let mut lrsdmaxk:usize = selmlk::MAXK;
  let mut regenerate = false;
  let mut mode = 0;
  let mut conv_yacc = false;
  let mut argi = 1; // next argument position
  while argi<argc
  {
     match args[argi] {
       filen if filen.ends_with(".grammar") => {filepath = args[argi];},
       filen if filen.ends_with(".y") => {
          filepath=args[argi];
	  conv_yacc=true;
	  break;
       },
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
       "-zc" | "zero_copy" => {zc=true; newbase=false;},
       "-newbase" | "-base" => {newbase = true; zc=false; genabsyn=true; genlex=true;},
       "genlex" | "-genlex" => {genlex=true; },
       "-genabsyn" | "-ast" | "-auto" => {genabsyn = true; },
       "-nozc" => {zc=false;},
       "binary" | "-binary" => { verbose=false; },       
       "-o" => {
          argi+=1;
          if argi<argc {parserfile = String::from(args[argi]);}
       },
       _ => {},    
     }//match directive
     argi+=1;
  }//while there are command-line args

  if filepath.len()==0 {
    //eprintln!("Must give path of .grammar file or .y file to convert from");
    return Err("Must give path of .grammar file or .y file to convert from".to_owned());
  }
  if conv_yacc {
    yaccparser::convert_from_yacc(filepath);
    return Ok(String::new());
    //return Ok(".y grammar converted to .grammar\n".to_owned());
  }

  if zc && verbose {
     //eprintln!("verbose mode not compatible with -zc option");
     return Err("verbose mode not compatible with -zc option".to_owned());
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
    //println!("\nFailed to process grammar");
    return Err(format!("\nFailed to process grammar at {}",filepath));
  }
  // Check grammar integrity: now done inside parse
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
     if !wres.is_ok() {
       //eprintln!("Failed to generate abstract syntax");
       return Err("Failed to generate abstract syntax".to_owned());
     }
  }

 grammar1.delay_transform(); // static delayed reduction markers


  if tracelev>2 {println!("computing Nullable set");}
  grammar1.compute_NullableRf();
  if tracelev>2 {println!("computing First sets");}
  //grammar1.compute_FirstIM();
  grammar1.compute_First();

  let mut fsm0;
  if lrsd {
    grammar1.logprint(&format!("Generating Experimental LR-Selective Delay State Machine with Max Delay = {}",lrsdmaxk));
    let mut lrsdfsm = MLStatemachine::new(grammar1);
    lrsdfsm.regenerate = regenerate;
    lrsdfsm.selml(lrsdmaxk);
    //fsm0 = lrsdfsm.to_statemachine();
    if lrsdfsm.failed {
      //println!("NO PARSER GENERATED"); return;
      return Err("LR SELECTIVE DELAY FAILURE. NO PARSER GENERATED".to_owned());
    }
    if !lrsdfsm.failed && lrsdfsm.regenerate {
      lrsdfsm.Gmr.logprint("Re-Generating LR(1) machine for transformed grammar...");
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
     grammar1.logprint("Generating LALR(1) state machine");
     let mut lalrfsm = LALRMachine::new(grammar1);
     lalrfsm.generatefsm();
     fsm0 = lalrfsm.to_statemachine();
  }
  else {
    grammar1.logprint(&format!("Generating {} state machine for grammar {}...",if lalr {"older LALR"} else {"LR1"},&gramname));
    fsm0 = Statemachine::new(grammar1);
    fsm0.lalr = lalr;
    if lalr {fsm0.Open = Vec::with_capacity(1024); } // important
    fsm0.generatefsm(); //GENERATE THE FSM
  } // old code
  if tracelev>2 && !newlalr && !lrsd { for state in &fsm0.States {printstate(state,&fsm0.Gmr);} }
  else if tracelev>1 && !newlalr && !lrsd {   printstate(&fsm0.States[0],&fsm0.Gmr); }//print states
  if parserfile.len()<1 || parserfile.ends_with('/') || parserfile.ends_with('\\') {parserfile.push_str(&format!("{}parser.{}",&gramname,pfsuffix));}
  if fsm0.States.len()>65536  {
    return Err(format!("too many states: {} execeeds limit of 65536",fsm0.States.len()));
  }
  let write_result =
    if mode==1 { fsm0.writefsparser(&parserfile) }
    else if newbase && !lrsd {
      fsm0.writebaseenumparser(&parserfile)
    }
    else if newbase && lrsd {
      fsm0.writelrsdbaseparser(&parserfile)    
    }
    else if zc {  // write zero-copy parser
      //fsm0.writezcparser(&parserfile)
      //fsm0.writelbaparser(&parserfile)
      if !lrsd {fsm0.writeenumparser(&parserfile)}
      else {fsm0.writelrsdparser(&parserfile)}
    }
    else {  // non-zc, original before version 0.2.0
      if verbose /*fsm0.States.len()<=16*/ {fsm0.write_verbose(&parserfile)}
      else {fsm0.writeparser(&parserfile)}
    }; // write_result =
  //if tracelev>0 && !lrsd {eprintln!("{} total states",fsm0.FSM.len());}
  fsm0.Gmr.logprint(&format!("{} total states",fsm0.FSM.len()));
  if let Ok(_) = write_result {
     fsm0.Gmr.logprint(&format!("Parser saved in {}",&parserfile));
  }
  else if let Err(err) = write_result {
     return Err(format!("failed to write parser, likely due to invalid -o destination\n{:?}",err));    
  }
  let mut savedlog = String::new();
  if tracelev==0 {fsm0.Gmr.swap_log(&mut savedlog);}
  Ok(savedlog)
}//rustle1
