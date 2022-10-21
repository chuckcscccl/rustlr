#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
//extern crate rustlr;
use rustlr::{LC};
use std::rc::Rc;
use crate::Env::*;
use bumpalo::Bump;
//#[cfg(feature = "collections")]
//use bumpalo::collections::{Vec};
use crate::Expr::*;


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
   Plus(&'t Expr<'t>,&'t Expr<'t>),
   Times(&'t Expr<'t>,&'t Expr<'t>),
   Divide(&'t Expr<'t>,&'t LC<Expr<'t>>),
   Minus(&'t Expr<'t>,&'t Expr<'t>),   
   Negative(&'t LC<Expr<'t>>),
   Letexp(&'t str,&'t Expr<'t>,&'t Expr<'t>),
   Seq(Vec<&'t LC<Expr<'t>>>),
   Expr_Nothing,                    // for integration into lexer/parser
} 
impl Default for Expr<'_>  // required for absyntypes of grammar
{
  fn default() -> Self { Expr_Nothing }
}//impl Default
impl<'t> Expr<'t>
{
  pub fn make(self, bump:&'t Bump) -> &'t Expr<'t>
  {
    bump.alloc(self)
  }//make
}//impl Expr<'t>



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
     Times(x,Val(0)) | Times(Val(0),x) => Some(0),
     Times(x,y) => eval(env,x).map(|a|{eval(env,y).map(|b|{a*b})}).flatten(),
     Minus(x,y) => eval(env,x).map(|a|{eval(env,y).map(|b|{a-b})}).flatten(),
     Negative(LC(Negative(x),_)) => eval(env,x),
     Negative(x) => eval(env,x).map(|a|{-1*a}), //no need for bind here    
     Divide(x,y) => {
       eval(env,y)
       .map(|yval|{if yval==0 {
          eprint!("Division by zero (expression starting at column {}) on line {} of {:?} at column {} ... ",y.column(),y.line(),x,y.column());
	  None
         } else {eval(env,x).map(|xval|{Some(xval/yval)})}
       })
       .flatten().flatten()
     },
     Letexp(x,e,b) => {
       eval(env,e).map(|ve|{
         let newenv = push(x,ve,env);
         eval(&newenv,b) }).flatten()
     },
     Seq(V) => {
       let mut ev = None;
       for ln@LC(x,_) in V
       //for x in V
       {
         ev = eval(env,x);
         if let Some(val) = ev {
	   println!("result for line {}: {} ;",ln.line(),&val);
         } else { eprintln!("Error evaluating line {};",ln.line()); }
       }//for
       ev
     },
     Nothing => None,
   }//match
}//eval
