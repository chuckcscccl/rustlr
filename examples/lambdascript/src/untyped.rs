#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

extern crate rustlr;
extern crate fixedstr;
use fixedstr::fstr;
use rustlr::{Tokenizer,RawToken,StrTokenizer,LBox};
use crate::untyped::Term::*;
use std::collections::{HashMap,HashSet,BTreeSet};
use std::mem::swap;
///// straightforward lambda calculus, with step by step reductions.

pub type str4 = fstr<4>; // vars are at most 4 chars long

#[derive(Debug,Clone)]
pub enum Term
{
  Var(str4),
  Const(i64),
  Abs(str4,Box<Term>),
  App(Box<Term>,Box<Term>),
}
impl Term
{
  pub fn to_string(&self) -> String
  {
    match self {
      Var(x) => format!("{}",x),
      Const(n) =>format!("{}",n),
      App(a,b) => {
        let mut bs = b.to_string();
        if let App(_,_) = &**b {bs = format!("({})",bs);}
        format!("{} {}",a.to_string(), bs)
      },
      Abs(x,a) => format!("(Lam {}.{})",x,a.to_string()),
    }//match
  }
}//impl Term
// for convenience
pub fn app(a:Term, b:Term) -> Term
{ 
   App(Box::new(a),Box::new(b))
}
pub fn abs(x:str4, a:Term) -> Term { Abs(x,Box::new(a)) }

///// determine if v appears free in t
fn isfree(v:&str4, t:&Term) -> bool
{
   match t {
     Var(y) => {v==y},
     App(a,b) => { isfree(v,a) || isfree(v,b) },
     Abs(y,a) if v!=y => { isfree(v,a) },
     _ => false, // all other cases, including all Abs(v,..)
   }//match
}//isfree


// implement call-by-name reduction
pub struct BetaReducer
{
   cx:u16,  // index for alpha-conversion
}
impl BetaReducer
{
   pub fn new() -> BetaReducer
   { BetaReducer {cx:0} }
   pub fn newvar(&mut self, x:&str4) -> str4
   {
      self.cx += 1;
      let xs = format!("{}{}",x,self.cx);
      return str4::from(xs);
   }

  // alpha-convert t apart from free vars in alpha-map.
  // always alpha-convert apart from free vars in N
  // map x->x by default, inserted into and checked for each var.
  pub fn alpha(&mut self, amap:&mut HashMap<str4,str4>, t:&mut Term, N:&Term)
  {
     match t {
       Var(x) => {
          amap.get(x).map(|y| {
            if y==x {
              let mut y2 = y.clone();
              swap(x,&mut y2);
            }
         });//lambda
       },
       App(a,b) => {self.alpha(amap,a,N); self.alpha(amap,b,N);},
       Abs(x,a) => {
         let current = amap.get(x);
         match current {
           None => {
              let mut x2 = *x;
              while isfree(&x2,N) { x2=self.newvar(x) };
              amap.insert(*x,x2);
              if x!=&x2 { swap(x,&mut x2); }
              self.alpha(amap,a,N);
           },
           Some(y) => {
             let mut y2 = y.clone();
             swap(x,&mut y2);
             self.alpha(amap,a,N);
           },
         }//match
       }, // Abs case
       _ => {}, // do nothing in other cases
     }//match
  }//alpha_apart

  // destructive substitution  M[N/x]
  fn subst(&mut self,M:&mut Term,x:&str4,N:&Term)
  {
     match M {
       Var(y) if y==x => {
         let mut N2 = N.clone();
         swap(M,&mut N2);
       },
       App(a,b) => {self.subst(a,x,N); self.subst(b,x,N);},
       Abs(y,a) => {
         let mut alphamap = HashMap::new();
         self.alpha(&mut alphamap, M,N);
         if let Abs(y2,a2) = M {self.subst(a2,x,N);}
       },
       _ => {},
     }//match
  }//subst

  // 1-step beta reduction, normal order, returns true if reduction occurred
  pub fn beta1(&mut self, t:&mut Term) -> bool
  {
    match t {
      App(A,B) =>  {
         if let Abs(x,C) = &mut **A {
            self.subst(C,x,B);
            let mut C2 = C.clone();
            swap(t,&mut C2);
            true
         }//redex
         else  { self.beta1(A) || self.beta1(B)}
      }, //app case
      Abs(x,B) => {  self.beta1(B) },
      _ => false,
    }//match
  }//beta1

  pub fn reduce_to_norm(&mut self, t:&mut Term)
  {
     let mut reducible = true;
     while reducible { reducible = self.beta1(t); }
  }
  
}//impl BetaReducer

//////////////
