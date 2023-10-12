#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
#![allow(unused_doc_comments)]
#![allow(unused_imports)]


/// this enum is only exported because it's used by the generated parsers.
/// There is no reason to use it in other programs.
#[derive(Copy,Clone,PartialEq,Eq,Debug)]
pub enum Stateaction {
  Shift(usize),     // shift then go to state index
  Reduce(usize),    // reduce by rule index
  Gotonext(usize),  // folded into same table, only for non-terminals
  Accept,
  /// note: this has been changed after version 0.1.1 from String to
  /// &'static str for increased efficiency. Error action entries are
  /// not generated by rustlr: they can only be added with the parser's
  /// training capability.  Parsers already trained can be hand-modified
  /// by removing all instances of ".to_string()" from the load_extras function.
  Error(&'static str),
}

/// Determines if action is not valid
pub fn iserror(actionopt:&Option<&Stateaction>) -> bool
    { use crate::Stateaction::*;
       match actionopt {
           None => true,
           Some(Error(_)) => true,
           _ => false,
         }
    }//iserror

// encode a state transition: FSM[i].get(key)=action as u64 numbers
/// this function is only exported because it's used by the generated parsers.
pub fn decode_action(code:u64) -> Stateaction
{   use crate::Stateaction::*;
    let actiontype =   code & 0x000000000000ffff;
    let actionvalue = (code & 0x00000000ffff0000) >> 16;
    //let symboli =     (code & 0x0000ffff00000000) >> 32;
    //let statei =      (code & 0xffff000000000000) >> 48;    
    match (actiontype,actionvalue) {
      (0,si) => Shift(si as usize),
      (1,si) => Gotonext(si as usize),
      (2,ri) => Reduce(ri as usize),
      (3,_)  => Accept,
      (4,x)  => Error("shouldn't be here"),
      _      => Error("unrecognized action in TABLE"),
    }
}//decode - must be independent function seen by parsers
