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
//use rustlr::rustle;

fn main() 
{
  let args:Vec<String> = std::env::args().collect(); // command-line args
  rustle(&args);
  /*
  let grammar_name = if args.len()>1 {&args[1]} else {"test1"};
  let option = if args.len()==3 {&args[2]} else {"lr1"}; // lr1 or lalr
  rustler(grammar_name,option);
  */
}//main


fn rustle(args:&Vec<String>) // called from main
{
  let argc = args.len();
  if argc<2 {panic!("Must give file path of .grammar");}
  let filepath = &args[1];
  let mut parserfile = String::from("");
  let mut argi = 2; // next argument position
  let mut lalr = false;
  let mut tracelev:usize = 0; // trace-level
  while argi<argc
  {
     match &args[argi][..] {
       "lr1" | "LR1" => { lalr=false; },
       "lalr" | "LALR" => {lalr=true; },
       "-trace" => {
          argi+=1;
          if argi<argc {
            if let Ok(lv) = args[argi].parse::<usize>() {tracelev=lv; }
          if tracelev>0 {println!("trace-level set to {}",tracelev);}
          }
       },
       "-o" => {
          argi+=1;
          if argi<argc {parserfile = args[argi].clone();}
       },
       _ => {},    
     }//match directive
     argi+=1;
  }//while there are command-line args
  
  if tracelev>1 {println!("parsing grammar from {}",&filepath);}
  let mut grammar1 = Grammar::new();
  grammar1.parse_grammar(filepath);
  if tracelev>2 {println!("computing Nullable set");}
  grammar1.compute_NullableRf();
  if tracelev>2 {println!("computing First sets");}
  grammar1.compute_FirstIM();
  if grammar1.name.len()<2  { // derive grammar name from filepath
     let doti = if let Some(p)= filepath.rfind('.') {p} else {filepath.len()};
     let mut slashi = if let Some(p) = filepath.rfind('/') {p+1} else {0};
     if slashi==0 {
       slashi = if let Some(p) = filepath.rfind('\\') {p+1} else {0};
     }
     grammar1.name = filepath[slashi..doti].to_string();
  }// derive grammar name
  let gramname = grammar1.name.clone();
  let mut fsm0 = Statemachine::new(grammar1);
  fsm0.lalr = lalr;
  if lalr {fsm0.Open = Vec::with_capacity(1024); } // important
  println!("Generating {} state machine for grammar {}...",if lalr {"LALR"} else {"LR1"},&gramname);
  fsm0.generatefsm();
  if tracelev>1 { for state in &fsm0.States {printstate(state,&fsm0.Gmr);} }
  else if tracelev>0 {   printstate(&fsm0.States[0],&fsm0.Gmr); }//print states
  if parserfile.len()<1 {parserfile = format!("{}parser.rs",&gramname);}
  let write_result = 
    if fsm0.States.len()<=16 {fsm0.write_verbose(&parserfile)}
    else if fsm0.States.len()<=65536 {fsm0.writeparser(&parserfile)}
    else {panic!("too many states: {}",fsm0.States.len())};
  println!("{} total states",fsm0.States.len());
  if let Ok(_) = write_result {println!("written parser to {}",&parserfile);}
}//rustle

