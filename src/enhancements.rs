#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
#![allow(unused_doc_comments)]
#![allow(unused_imports)]
use std::fmt::Display;
use std::default::Default;
use std::collections::{HashMap,HashSet,BTreeSet};
use std::io::{self,Read,Write,BufReader,BufRead};
use std::cell::{RefCell,Ref,RefMut};
use std::hash::{Hash,Hasher};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::mem;
use crate::{TRACE,Lexer,Lextoken, charlexer};
//use crate::runtime_parser::*;
use crate::ParseResult::*;

/// Abstract syntax values that are returned by enhanced_parse function
pub trait ParseValue : Default
{
   /// the abstract syntax tree should define a way to indicate an error without
   /// panic.  A typical example would be an enum with a case for Error.
   /// This function will be called by the parser when a parse error occurs.
   fn make_error(err_msg:&str) -> Self;
   /// this function should traverse the abstract syntax tree and collect
   /// all errors found in a string.  The preamble is passed in by the
   /// unified runtime parser when it detects that a partial result was
   /// returned by enhanced_parse.
   fn report_errors(&self,preamble:&str) -> String;
}

/// custom monad for parser error handling.  This was added as part of
/// a set of backwards-compatible enhancements to the parser generator
/// and runtime parser.  When an error is encountered, the parser will
/// still try to return a partial result.  The string in Partial is
/// some error message, and the usize index is an indicator of how far
/// the error was propagated by monadic flatmaps.
pub enum ParseResult<AT:ParseValue>
{
   /// a complete, error-free AT (abstract syntax tree) 
   Complete(AT),
   /// a partial abstract syntax tree with String, usize representing
   /// error message
   Partial(AT,String,usize),
}

impl<AT:ParseValue> ParseResult<AT>
{
  /// the flatmap function, version that consumes self.  The function f
  /// is applied to both complete and partial results, propagating
  /// the partiality of the result.  The index
  /// of partial results are incremented to indicate depth of error propagation.
  pub fn Fmap<T:ParseValue, F:FnOnce(AT)->T>(self, f:F) -> ParseResult<T>
  {
     match self {
        Complete(v) => Complete(f(v)),
        Partial(v,msg,i) => Partial(f(v),msg,i+1),
     }//match
  }

  /// non-consuming version of flatmap
  pub fn fmap<T:ParseValue, F:FnOnce(&AT)->T>(&self, f:F) -> ParseResult<T>
  {
     match self {
        Complete(v) => Complete(f(v)),
        Partial(v,msg,i) => Partial(f(v),msg.clone(),i+1),
     }//match
  }

/// going against formal monadic expectations, here bind does not behave the 
/// same way as fmap+join: Partial results are never ignored, but the partiality
/// of the monad is ignored.  Bind is not anticipated to be as useful as fmap.
/// Note: the join or "flatten" function is not applicable in this setting:
/// a ParseResult should not itself be used as a parse value (abstract syntax
/// tree).
  pub fn pseudobind<T:ParseValue, F:FnOnce(&AT)->ParseResult<T>>(&self, f:F) -> ParseResult<T>
  {
     match self {
        Complete(v) => f(v),
        Partial(v,msg,i) => f(v),
     }//match     
  }

  // apply function f to Complete results and g to Partial results
  pub fn fmap_else<T:ParseValue, F:FnOnce(&AT)->T>(&self, f:F, g:F) -> ParseResult<T>
  {
     match self {
        Complete(v) => Complete(f(v)),
        Partial(v,msg,i) => Partial(g(v),msg.clone(),i+1),
     }//match     
  }

  // this is probably more useful than bind:
  pub fn bind_else<T:ParseValue, F:FnOnce(&AT)->ParseResult<T>>(&self, f:F,g:F) -> ParseResult<T>
  {
     match self {
        Complete(v) => f(v),
        Partial(v,msg,i) => g(v),
     }//match     
  }

  /// applies a binary function to a pair of results.  The result returned is
  /// complete if both arguments are complete.
  pub fn fmap2<T:ParseValue, F:FnOnce(&AT,&AT)->T>(&self, second:&ParseResult<AT>,f:F) -> ParseResult<T>
  {
    match(self,second) {
      (Complete(a),Complete(b)) => Complete(f(a,b)),
      (Complete(a),Partial(b,m,i)) => Partial(f(a,b),m.clone(),i+1),
      (Partial(b,m,i),Complete(a)) => Partial(f(b,a),m.clone(),i+1),
      (Partial(a,n,k),Partial(b,m,i)) => {
        let msg = format!("{}:{}, and {}:{}",k,n,i,m);
        let index = if k>i {k+1} else {i+1};
        Partial(f(a,b),msg,index)
      },
    }//match
  }
}//impl ParseResult


//////////////////////

/*
////////////////////////////////////////////////////////////////////////
////////////////// enhanced_parser
// need: Enhanced_Parser
impl<AT:ParseValue,ET:Default> RuntimeParser<AT,ET>
{
    fn enhnext(&self, tokenizer:&mut dyn Enhanced_Lexer<AT>) -> Lextoken<AT>
    {
       if let Some(tok) = tokenizer.nextsym() {tok}
        else { Lextoken{sym:"EOF".to_owned(),  value:AT::default()} } 
    }
    
/// parse function with enhanced error handling, requires grammar and
/// abstract syntax that also recognize errors.
pub fn enhanced_parse(&mut self, tokenizer:&mut dyn Enhanced_Lexer<AT>) -> ParseResult<AT>
    {
       self.err_occured = false;
       self.stack.clear();
//       self.exstate = ET::default(); ???
       let mut result = AT::default();
       // push state 0 on stack:
       self.stack.push(Stackelement {si:0, value:AT::default()});
       let unexpected = Stateaction::Error(String::from("unexpected end of input"));
       let mut action = unexpected; //Stateaction::Error(String::from("get started"));
       self.stopparsing = false;
       let mut lookahead = Lextoken{sym:"EOF".to_owned(),value:AT::default()}; 
       if let Some(tok) = tokenizer.nextsym() {lookahead=tok;}
       else {self.stopparsing=true;}

       while !self.stopparsing
       {
         self.linenum = tokenizer.linenum(); self.column=tokenizer.column();
         let currentstate = self.stack[self.stack.len()-1].si;
         let mut actionopt = self.RSM[currentstate].get(lookahead.sym.as_str());//.unwrap();
        ///// Do error recovery
         if let None = actionopt {
            self.report(&format!("unexpected symbol {}, attempting recovery ...",&lookahead.sym));
            let mut erraction = None;
            // skip ahead until a resync symbol is found

            ///// prefer to use Errsym method
            if self.Errsym.len()>0 {
               let errsym = self.Errsym;
               //lookdown stack for "shift" action on errsym
               let mut k = self.stack.len()-1; // offset by 1 because of usize
               let mut spos = k+1;
               while k>0 && spos>k
               {
                  let ksi = self.stack[k-1].si;
                  erraction = self.RSM[ksi].get(errsym);
                  if let Some(Shift(_)) = erraction { spos=k;}
                  else {k-=1;}
               }//while k>0
               if spos==k { self.stack.truncate(k); }
               if let Some(Shift(i)) = erraction { // simulate shift errsym
                 self.stack.push(Stackelement{si:*i,value:AT::default()});
                 // now keep lookahead until action is found that transitions from
                 // current state (i).  since only terminals may follow errsym,
                 // this would have to be a shift rule
                 while let None = self.RSM[*i].get(&lookahead.sym[..]) {
                    if &lookahead.sym[..]=="EOF" {break;}
                    lookahead = self.enhnext(tokenizer);
                 }//while let
                 // either at end of input or found action on next symbol
                 erraction = self.RSM[*i].get(&lookahead.sym[..]);
               } // if shift action found down under stack
               else {erraction = None; }// don't reduce
            }//errsym exists

            // at this point, if erraction is None, then Errsym failed to recover,
            // try the resynch symbol method...
            
            if erraction==None && self.resynch.len()>0 {
               while &lookahead.sym!="EOF" &&
                      !self.resynch.contains(&lookahead.sym[..]) {
                 lookahead = self.enhnext(tokenizer);
               }
             if &lookahead.sym!="EOF" {
              // look for state on stack that has action defined on next symbol
              lookahead = self.enhnext(tokenizer); // skipp err-causing symbol
             }
              let mut k = self.stack.len()-1; // offset by 1 because of usize
              let mut position = 0;
              while k>0 && erraction==None
               {
                  let ksi = self.stack[k-1].si;
                  erraction = self.RSM[ksi].get(&lookahead.sym[..]);
                  if let None=erraction {k-=1;}
               }//while k>0 && erraction==None
              match erraction {
                 None => {}, // do nothing, whill shift next symbol
                 _ => { self.stack.truncate(k);},//pop stack
              }//match
            }// there are resync symbols

            // at this point, if erraction is None, then resynch recovery failed too.
            // only action left is to skip ahead...
            if let None = erraction { //skip input, loop back
                lookahead = self.enhnext(tokenizer);
                if &lookahead.sym=="EOF" {
                  self.abort("error recovery failed before end of input");
                }
            }
         }//error recovery
         
         else {
          action = actionopt.unwrap().clone();  // cloning stateaction is ok
          match &action {
            Stateaction::Shift(i) => { // shift to state si
                self.stack.push(Stackelement{si:*i,value:mem::replace(&mut lookahead.value,AT::default())});
                lookahead = self.enhnext(tokenizer);
             }, //shift
            Stateaction::Reduce(ri) => { //reduce by rule i
               self.reduce(ri);
             },
            Stateaction::Accept => {
              result = self.stack.pop().unwrap().value;
              self.stopparsing = true;
             },
            Stateaction::Error(msg) => {
              self.stopparsing = true;
             },
            Stateaction::Gotonext(_) => { //should not see this here
              self.stopparsing = true;
             },
          }//match & action
         }// else not in error recovery mode
       } // main parser loop
       if let Stateaction::Error(msg) = &action {
          //panic!("!!!Parsing failed on line {}, next symbol {}: {}",tokenizer.linenum(),&lookahead.sym,msg);
          self.report(&format!("failure with next symbol {}",tokenizer.linenum()));
       }
       //if self.err_occured {result = AT::default(); }
       return Complete(result);
    }//enhanced_parse

}//impl RuntimeParser
*/
