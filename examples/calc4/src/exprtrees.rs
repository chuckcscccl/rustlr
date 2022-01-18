#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
use crate::Expr::*;
extern crate rustlr;
use rustlr::{Lextoken,Lexer,LBox};
use rustlr::{TerminalToken,Tokenizer,RawToken,StrTokenizer}; // for zc version
use std::any::Any;

#[derive(Clone,Debug)]
pub enum Expr<'t>
{
   Var(&'t str),
   Val(i64),
   Plus(LBox<Expr<'t>>,LBox<Expr<'t>>),  // LBox replaces Box for recursive defs
   Times(LBox<Expr<'t>>,LBox<Expr<'t>>),
   Divide(LBox<Expr<'t>>,LBox<Expr<'t>>),
   Minus(LBox<Expr<'t>>,LBox<Expr<'t>>),
   Negative(LBox<Expr<'t>>),
   Seq(Vec<LBox<Expr<'t>>>),
   Nothing,                    // for integration into lexer/parser
} 


impl Default for Expr<'_>
{
  fn default() -> Self { Nothing }
}//impl Default

pub fn eval(e:&Expr) -> i64
{
   match e {
     Var(x) => {
        println!("evaluating {} gets you 1000",x); 1000
     },
     Val(x) => *x,  // x is a ref because e is
     Plus(x,y) => eval(x) + eval(y), // deref coercion works nicely here
     Times(x,y) => eval(x) * eval(y),
     Divide(x,y) => {
       let yval = eval(y);
       if yval==0 {
         eprintln!("Division by zero (expression starting at column {}) on line {} of {:?} at column {}, returning 0 as default",y.column,y.line,x,x.column);
	 0// returns default
       } else {eval(x) / yval}
     },
     Minus(x,y) => eval(x) - eval(y), 
     Negative(x) => -1 * eval(x),
     Seq(V) => {
       let mut ev = 0;
       for x in V
       {
         ev = eval(x);
	 println!("result for line {}: {} ;",x.line,&ev);
       }
       ev
     },
     Nothing => 0,
   }//match
}//eval

pub fn getint(e:&Expr) -> i64
{
   match e {
     Val(n) => *n,
     _ => 0,  // behaves like perl
   }
}



///////////////// lexer adapter
//////////////////// LBA VERSION
///////////// Zero-copy tokenizer
pub struct Zcscannerlba<'t>(StrTokenizer<'t>);
impl<'t> Zcscannerlba<'t>
{
  pub fn new(mut stk:StrTokenizer<'t>) -> Zcscannerlba<'t>
  {
     for x in ['+','-','*','/'] {stk.add_single(x)}
     Zcscannerlba(stk)
  }
}// impl Zcscannerlba

impl<'t> Tokenizer<'t,LBox<dyn Any+'t>> for Zcscannerlba<'t>
{
   fn nextsym(&mut self) -> Option<TerminalToken<'t,LBox<dyn Any+'t>>>
   {
     let tokopt = self.0.next_token();
     if let None = tokopt {return None;}
     let token = tokopt.unwrap();
     match token.0 {
       RawToken::Num(n) => Some(TerminalToken::raw_to_lba(token,"int",n)),
       RawToken::Symbol(s) => Some(TerminalToken::raw_to_lba(token,s,Nothing)),
       RawToken::Alphanum(a) => Some(TerminalToken::raw_to_lba(token,"var",a)),
       _ => Some(TerminalToken::raw_to_lba(token,"<<Lexical Error>>",Nothing)),
     }//match
   }//nextsym
   fn linenum(&self) -> usize {self.0.line()}
   fn column(&self) -> usize {self.0.column()}
   fn position(&self) -> usize {self.0.current_position()}
}//impl Tokenizer

