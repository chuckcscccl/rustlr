#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
use crate::Expr::*;
extern crate rustlr;
use rustlr::{LBox,TerminalToken,Tokenizer,RawToken,StrTokenizer};
use std::rc::Rc;
use crate::Env::*;

//// simple linked list with non-destructive cons to represent scoped
//// environment. (used during evaluation, not parsing)
pub enum Env<'t> {
  Nil,
  Cons(&'t str, i64, Rc<Env<'t>>)
}
pub fn newenv<'t>() -> Rc<Env<'t>>
{ Rc::new(Nil) }
fn push<'t>(var:&'t str, val:i64, env:&Rc<Env<'t>>) -> Rc<Env<'t>>
{ Rc::new(Cons(var,val,Rc::clone(env))) }
fn pop<'t>(env:Rc<Env<'t>>) ->  Rc<Env<'t>> //not used here, just being complete
{
   match &*env {
      Nil => env,
      Cons(x,v,e) => Rc::clone(e),
   }
}//push
fn lookup<'t>(x:&'t str, env:&Rc<Env<'t>>) -> Option<i64>
{
    let mut current = env;
    while let Cons(y,v,e) = &**current {
      if &x==y {return Some(*v);}
      else {current = e;}
    }
    return None;
}//lookup

// main abstract syntax type
#[derive(Debug)]
pub enum Expr<'t>
{
   Var(&'t str),
   Val(i64),
   Plus(LBox<Expr<'t>>,LBox<Expr<'t>>),  // LBox replaces Box for recursive defs
   Times(LBox<Expr<'t>>,LBox<Expr<'t>>),
   Divide(LBox<Expr<'t>>,LBox<Expr<'t>>),
   Minus(LBox<Expr<'t>>,LBox<Expr<'t>>),
   Negative(LBox<Expr<'t>>),
   Letexp(&'t str,LBox<Expr<'t>>,LBox<Expr<'t>>),
   Seq(Vec<LBox<Expr<'t>>>),
   Nothing,                    // for integration into lexer/parser
} 

impl Default for Expr<'_>  // required for absyntypes of grammar
{
  fn default() -> Self { Nothing }
}//impl Default

// evaluation/interpretation
pub fn eval<'t>(env:&Rc<Env<'t>>, exp:&Expr<'t>) -> Option<i64>
{
   match exp {
     Var(x) => {
       if let Some(v) = lookup(x,env) {Some(v)}
       else { eprint!("UNBOUND VARIABLE {} ... ",x);  None}
     },
     Val(x) => Some(*x),
     Plus(x,y) => eval(env,x).map(|a|{eval(env,y).map(|b|{a+b})}).flatten(),
     Times(x,y) => eval(env,x).map(|a|{eval(env,y).map(|b|{a*b})}).flatten(),
     Minus(x,y) => eval(env,x).map(|a|{eval(env,y).map(|b|{a-b})}).flatten(),
     Negative(x) => eval(env,x).map(|a|{-1*a}), //no need for bind here    
     Divide(x,y) => {
       eval(env,y)
       .map(|yval|{if yval==0 {
          eprint!("Division by zero (expression starting at column {}) on line {} of {:?} at column {} ... ",y.column(),y.line(),x,x.column());
	  None
         } else {eval(env,x).map(|xval|{Some(xval/yval)})}
       })
       .flatten().flatten()
     },
     Letexp(x,e,b) => {
       eval(env,e).map(|ve|{
         let newenv = push(x,ve,env);
         eval(&newenv,b) }).flatten()
     }
     Seq(V) => {
       let mut ev = None;
       for x in V
       {
         ev = eval(env,x);
         if let Some(val) = ev {
	   println!("result for line {}: {} ;",x.line(),&val);
         } else { eprintln!("Error evaluating line {};",x.line()); }
       }//for
       ev
     },
     Nothing => None,
   }//match
}//eval


///////////// Zero-copy tokenizer  (now auto generated - this was hand-coded)
pub struct Calcscanner<'t>(StrTokenizer<'t>);
impl<'t> Calcscanner<'t>
{
  pub fn new(mut stk:StrTokenizer<'t>) -> Calcscanner<'t>
  {
     for x in ['+','-','*','/','=',';'] {stk.add_single(x)}
     stk.set_line_comment("#");
     Calcscanner(stk)
  }
}// impl Calcscanner

impl<'t> Tokenizer<'t,Expr<'t>> for Calcscanner<'t>
{
   fn nextsym(&mut self) -> Option<TerminalToken<'t,Expr<'t>>>
   {
     let tokopt = self.0.next_token();
     if let None = tokopt {return None;}
     let token = tokopt.unwrap();
     match token.0 {
       RawToken::Num(n) => Some(TerminalToken::from_raw(token,"int",Val(n))),
       RawToken::Symbol(s) => Some(TerminalToken::from_raw(token,s,Nothing)),
       RawToken::Alphanum(s) if s=="let" => Some(TerminalToken::from_raw(token,"let",Nothing)),
       RawToken::Alphanum(s) if s=="in" => Some(TerminalToken::from_raw(token,"in",Nothing)),       
       RawToken::Alphanum(a) => Some(TerminalToken::from_raw(token,"var",Var(a))),
       _ => Some(TerminalToken::from_raw(token,"<<Lexical Error>>",Nothing)),
     }//match
   }//nextsym
   fn linenum(&self) -> usize {self.0.line()}
   fn column(&self) -> usize {self.0.column()}
   fn position(&self) -> usize {self.0.current_position()}
}//impl Tokenizer
