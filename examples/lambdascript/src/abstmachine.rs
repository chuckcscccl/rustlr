// Krivine's Abstract Machine for CBN untyped lambda calculus.

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

extern crate rustlr;
use rustlr::{Lextoken,Lexer,LBox,LRc};
extern crate basic_lexer;
pub use basic_lexer::*;

use std::cell::{RefCell,Ref,RefMut};
use std::rc::Rc;
use crate::abstmachine::Lamterm::*;

// Absyntype of grammar should be LRc<Lamterm>

#[derive(Clone)]
enum Lamterm   // change to LBox later
{
  Var(String),
  Iconst(i64),
  Abs(String,LRc<Lamterm>),
  App(LRc<Lamterm>,LRc<Lamterm>),
  Nothing,
}
impl Default for Lamterm
{
  fn default() -> Self { Nothing }
}//impl Default

struct Closure
{
  term:LRc<Lamterm>,
  env:Bindings,
}
fn newclosure(t:LRc<Lamterm>, b:Bindings) -> Closure
{
   Closure{term:t, env:b}
}

type Bindings = Option<Rc<Binding>>;

struct Binding
{
  var: String,
  cl : Rc<Closure>,
  rest: Bindings,
}
fn bind(v:String,vcl:Rc<Closure>,cur:&Bindings) -> Binding
{
   Binding {
     var:v,
     cl:vcl,
     rest: match cur {
        None => None,
        Some(rcb) => Some(Rc::clone(rcb)),
     },
  }
}
fn get<'t>(v:&str,mut env:&'t Bindings) -> Option<&'t Rc<Closure>>
{
   while let Some(rcb) = env {
      if v==&rcb.var { return Some(&rcb.cl); }
      env = &rcb.rest;
   }
   return None;
}

pub struct Machine // Krivine's Abstract Machine
{
   stack: Vec<Rc<Closure>>,
   counter: usize,  // for alpha-conversion (only used when printing)
}

impl Machine
{
  fn new(t0:LRc<Lamterm>) -> Machine
  { 
     Machine {
        stack: vec![Rc::new(Closure { term:t0, env:None, })],
        counter: 0,
     }
  }

  //apply all closures as suspended substitutions, alpha-converts, destructive
  
  fn substalpha(&mut self, term:&LRc<Lamterm>, ev:&Bindings) -> LRc<Lamterm>
  {
    match &**term {
      Var(x) => {
        match get(&x[..],ev) {
          None => term.clone(),
          Some(cl) => self.substalpha(&cl.term,&cl.env),
        }//match
      },
      App(a,b) => {
         let a2 = self.substalpha(a,ev);
         let b2 = self.substalpha(b,ev);
         term.transfer(App(a2,b2))
      },
      Abs(x,b) => {
        self.counter += 1;
        let aconv = format!("{}_{}",x,self.counter); // alpha-converted name
        let newbindings = bind(x.clone(),
                          Rc::new(newclosure(term.transfer(Var(aconv.clone())),None)),
                          ev);
        let b2 = self.substalpha(b,&Some(Rc::new(newbindings)));   
        term.transfer(Abs(aconv,b2))
      },
      _ => term.clone(),
    }//match
  }//substalpha

  pub fn beta(&mut self) -> bool // returns progress indicator
  {
    if self.stack.len()<1 {return false;}
    let topclosure = self.stack.pop().unwrap();
    match &*topclosure.term {
      Var(x) => {
         match get(&x[..],&topclosure.env) {
            None => { self.stack.push(topclosure); return false; },
            Some(xcl) => { self.stack.push(Rc::clone(xcl)); },
         }//match
      },
      App(a,b) => {
        self.stack.push(Rc::new(newclosure(LRc::clone(a),topclosure.env.clone())));
        self.stack.push(Rc::new(newclosure(LRc::clone(b),topclosure.env.clone())));
      },
      Abs(x,b) => {
        if self.stack.len()<1 {self.stack.push(topclosure); return false;}
        let cl = self.stack.pop().unwrap();
        let newenv = Some(Rc::new(bind(x.clone(),cl,&topclosure.env)));
        let newcl = newclosure(LRc::clone(b),newenv);
        self.stack.push(Rc::new(newcl));
      },
      _ => {self.stack.push(topclosure); return false;},
    }//match
    return true;
  }//beta

}// impl Machine
/*
  <(x,Env)::S> ==> <Env(x)::S>
  <((app A B),Env)::S> ==> <(A,Env)::(B,Env)::S>
  <((lambda x.B,Env)::Cl::S) ==> <(B,(x,Cl)::Env)::S>
  else S ==> S
*/
fn main(){}

/* 
  This (very) abstract machine reduces a lambda term to *weak head normal
  form*.  Essentially, we do not descend down into a lambda-abstraction
  until the lambda-abstraction is applied to a term.  The machine also
  uses *call-by-name* as opposed to the more usual call-by-value.  This
  means that arguments are not evaluated before they're passed to the 
  lambda term/function.

  Formally, a machine consists of a STACK.  A STACK is a list of CLOSURES.
  A CLOSURE is of the form (t,Env) where t is a lambda term and Env is
  a list of BINDINGS. A BINDING is of the form (x,CLOSURE) where x is a 
  variable represented by a string.  One of the nice aspects of this 
  machine is that alpha-conversion is not required during reduction.  
  It is only needed when we wish to print the result.

  The machine is initialized with <(t,null)> where t is the term to
  be computed, then transitions as follows:

  <(x,Env)::S> ==> <Env(x)::S>
  <((app A B),Env)::S> ==> <(A,Env)::(B,Env)::S>
  <((lambda x.B,Env)::Cl::S) ==> <(B,(x,Cl)::Env)::S>
  else S ==> S

  Here, :: represents list cons and Env(x) searching for the binding of
  x in Env.  Env is used in a stack-like manner so the top binding for x
  is returned.
*/

/*
struct Binding
{
  var: String,
  cls: Closure,
  rest: Option<Rc<Binding>>, // but maybe don't need Rc here
}
fn bind(v:String,cl:Closure,cur:&Rc<Binding>) -> Binding
{
   Binding {
     var:v,
     cls:cl,
     rest: Some(Rc::clone(cur)), //cur.map(|rcb|{Rc::clone(&rcb)}),
  }
}
fn get<'t>(v:&str,mut env:&'t Rc<Binding>) -> Option<&'t Closure>
{
   while v!=&env.var {
      if let None = env.rest {return None;} else {env = &env.rest.unwrap();}
   }//while
   return Some(&env.cls);
}

struct Stack
{
  car: Closure,
  cdr: Option<Rc<Stack>>,
}
fn cons(cl:Closure, cur:&Option<Rc<Stack>>) -> Stack
{
  let tail = if let Some(c) = cur {Some(Rc::clone(c))} else {None};
  Stack {car:cl, cdr:tail,}
}

*/



/*
type Binding = Vec<(String,Closure)>;

fn lookup<'t>(x:&str,env:&'t Binding) -> Option<&'t Closure>
{
   for (s,c) in env
   {
      if s==x {return Some(c);}
   }
   return None;
}
*/
