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
//mod enhancements;
//pub use enhancements::*;
pub mod zc_parser;
use zc_parser::*;

mod parser_writer;
use parser_writer::*;
mod sd_parserwriter;
use sd_parserwriter::*;
mod fs_parserwriter;

mod ast_writer;
use ast_writer::*;

mod bumpast_writer;

mod lalr_statemachine;
use lalr_statemachine::LALRMachine;
mod selmlk;
use selmlk::{MLStatemachine};

fn main() 
{
  let args:Vec<String> = std::env::args().collect(); // command-line args
  rustle(&args);
}//main


fn rustle(args:&Vec<String>) // called from main
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
     eprintln!("verbose mode not compatible with -zc option");
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
    eprintln!("\nFailed to process grammar");
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
     if !grammar1.bumpast { wres = grammar1.writeabsyn(&astpath); }
     else {wres = grammar1.write_bumpast(&astpath); }
     if !wres.is_ok() {eprintln!("Failed to generate abstract syntax"); return;}
  }

 grammar1.delay_transform(); // hope this works!


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
    if lrsdfsm.failed {eprintln!("NO PARSER GENERATED"); return;}
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
    eprintln!("too many states: {} execeeds limit of 65536",fsm0.States.len());
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
     eprintln!("failed to write parser, likely due to invalid -o destination: {:?}",err);
  }
}//rustle
