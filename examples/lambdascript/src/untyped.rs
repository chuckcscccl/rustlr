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
use fixedstr::{fstr,str8};
use rustlr::{Tokenizer,RawToken,TerminalToken,StrTokenizer,LBox,LexSource,unbox};
use crate::untyped::Term::*;
use std::collections::{HashMap,HashSet};
use std::mem::swap;
///// straightforward lambda calculus, with step by step reductions.

const lowerlam:&'static str = "\u{03bb}"; // unicode 03bb is lower case lambda
const LAM:&'static str = "lambda "; // unicode 03bb is lower case lambda

#[derive(Debug,Clone)]
pub enum Term
{
  Var(str8),
  Const(i64),
  Abs(str8,LBox<Term>),
  App(LBox<Term>,LBox<Term>),
  Def(bool,str8,LBox<Term>),  // true bool means eval to weak-head form
  Weak(LBox<Term>),  // eval into weak head normal form
  CBV(LBox<Term>),   // call-by-value instead of default CBN
  Seq(Vec<LBox<Term>>), // there won't be nested seqs
  Nothing,
}
impl Default for Term { fn default()->Self {Nothing} }
impl Term
{
  pub fn to_string(&self) -> String { self.format(lowerlam) }
  pub fn format(&self,lam:&str) -> String
  {
    match self {
      Var(x) => format!("{}",x),
      Const(n) =>format!("{}",n),
      App(a,b) => {
        let mut a2 = a.to_string();
        if let Abs(_,_) = &**a {a2 = format!("({})",a2);}
        let mut bs = b.to_string();
        if let App(_,_) = &**b {bs = format!("({})",bs);}
        else if let Abs(_,_) = &**b {bs = format!("({})",bs);}
        format!("{} {}",a2, bs)
      },
      Abs(x,a) =>  {
        let a2 = a.to_string();
        let mut an = format!("{}{}",lam,x);
        if let Abs(_,_) = &**a  { if lam!=lowerlam {an.push(' ');} }
        else { an.push('.'); }
        an.push_str(&a2);
        an
      },
      x => format!("RAW({:?})",x),
    }//match
  }
}//impl Term
/*
// for convenience
pub fn app(a:Term, b:Term) -> Term
{ 
   App(Box::new(a),Box::new(b))
}
pub fn abs(x:str8, a:Term) -> Term { Abs(x,Box::new(a)) }
*/

///// determine if v appears free in t
fn isfree(v:&str8, t:&Term) -> bool
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
   trace:u8,
}
impl BetaReducer
{
   pub fn new() -> BetaReducer
   { BetaReducer {cx:0, trace:0} }
   pub fn newvar(&mut self, x:&str8) -> str8
   {
      self.cx += 1;
      let xs = format!("{}{}",x,self.cx);
      return str8::from(xs);
   }

  // alpha-convert t apart from free vars in alpha-map.
  // always alpha-convert apart from free vars in N
  // map x->x by default, inserted into and checked for each var.
  pub fn alpha(&mut self, amap:&mut HashMap<str8,str8>, t:&mut Term, N:&Term)
  {
     match t {
       Var(x) => {
          amap.get(x).map(|y| {
            if y!=x {
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
              let mut amap2 = amap.clone(); // not efficient!
              amap2.insert(*x,x2);
              if x!=&x2 {
                if self.trace>0 {
                  println!(" < alpha conversion of {} to {} >",x,&x2);
                }
                swap(x,&mut x2);
              }
              self.alpha(&mut amap2,a,N);
              //amap.remove(x); // didn't work.
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
  fn subst(&mut self,M:&mut Term,x:&str8,N:&Term)
  {
     match M {
       Var(y) if y==x => {
         let mut N2 = N.clone();
         swap(M,&mut N2);
       },
       App(a,b) => {self.subst(a,x,N); self.subst(b,x,N);},
       Abs(y,a) if x!=y => {
         let mut alphamap = HashMap::new();
         self.alpha(&mut alphamap, M,N);  // rename M away from N
         if let Abs(y2,a2) = M {self.subst(a2,x,N);}
       },
       _ => {},
     }//match
  }//subst

  // 1-step beta reduction, normal order, returns true if reduction occurred
  // expands defs only when necessary.  MOST CRUCIAL FUNCTION
  pub fn beta1(&mut self, t:&mut Term,defs:&HashMap<str8,Term>) -> bool
  {
    match t {
      App(A,B) =>  {
         while let Var(id) = &mut **A {
           if let Some(iddef) = defs.get(id) {
             //println!("= ({}) {}",iddef.to_string(),unbox!(B).to_string());
             let mut def2 = iddef.clone();
             swap(&mut **A, &mut def2);
           } else {break;}
         }// expand def  - then do again
         if let Abs(x,C) = &mut **A {
            self.subst(C,x,B);
            let mut C2 = C.clone();
            swap(t,&mut C2);
            true
         }//redex
         else { self.beta1(A,defs) || self.beta1(B,defs) }
      }, //app case
      Abs(x,B) => {  self.beta1(B,defs) },
      _ => false,
    }//match
  }//beta1

  pub fn reduce_to_norm(&mut self, t:&mut Term, defs:&HashMap<str8,Term>)
  {
     if self.trace>0 {println!("{}",t.to_string());}
     let mut reducible = true;
     while reducible {
       if self.trace>0 && expand(t,defs) {
         println!("= {}",t.to_string());
       }
       reducible = self.beta1(t,defs);
       if reducible && self.trace>0 {
           println!(" =>  {}",t.to_string());
       }
     }
  }// reduce to beta normal form (strong norm via CBN)

// weak head reduction, CBV
pub fn weak_beta(&mut self, t:&Term, defs:&HashMap<str8,Term>)
{
   if self.trace>0 {  println!("weak {}",t.to_string());  }
   let mut t2 = t.clone();
   while expand(&mut t2,&defs) {
       if self.trace>0 {println!("= {}",t2.to_string());}
   }
   while self.weak_beta1(&mut t2,defs) {
       if self.trace>0 {println!(" =>  {}",t2.to_string());}
   }
}//weak_beta
fn weak_beta1(&mut self, t:&mut Term, defs:&HashMap<str8,Term>) -> bool
{ 
  match t {
    App(a,b) => {
      if let Abs(x,body) = &**a {
        // reduce b first:
        self.weak_beta1(b,defs) || 
        self.beta1(t,defs) 
//        let wt =self.weak_beta(t,defs); // do it again
//        wb || bt || wt
      }//redex found
      else {self.weak_beta1(a,defs)}
    },
    _ => {false},
  }//match
}//weak beta

}//impl BetaReducer

//////////////

pub fn getvar(t:&Term) -> str8 {if let Var(x)=t {*x} else {str8::default()}}

//// replace all defined terms with their definitions


////// evaluation of a program
////// given hashmap of definitions

// expand definitions lazily
fn expand(t:&mut Term, defs:&HashMap<str8,Term>) -> bool
{
   match t {
     Var(x) => {
       if let Some(xdef) = defs.get(x) {
         let ref mut xdef2 = xdef.clone();
         swap(t,xdef2);
         true
       }
       else {false}
     }, // var
     App(a,b) => {
       expand(a,defs) || expand(b,defs)
     },
     Abs(x,a) => {
       if let Some(xdef) = defs.get(x) {
          panic!("BOUND VARIABLE {} CONFLICTS WITH GLOBAL DEFINITION",x);
       }
       expand(a,defs)
     },
     _ => false,
   }//match
}//expand , returns true if something was expanded


pub fn eval_prog(prog:&Vec<LBox<Term>>, defs:&mut HashMap<str8,Term>)
{
  let mut reducer = BetaReducer::new();
  //let mut defs = HashMap::<str8,Term>::new();
  for line in prog
  {
     match &**line {
       Def(weak,x,xdef) => {
         let mut xdef2 = unbox!(xdef).clone(); //*xdef.exp.clone();
         if *weak {
            reducer.trace=0; reducer.cx=0;
            reducer.reduce_to_norm(&mut xdef2,defs);            
            //reducer.weak_beta(&mut xdef2,defs);
         }
         defs.insert(*x,xdef2);
       },
       Weak(t) => {
         reducer.trace=1; reducer.cx=0;
         reducer.weak_beta(t,defs);
         println!();         
       },
       t => {
         reducer.trace=1; reducer.cx=0;
         let ref mut t2 = t.clone();
         reducer.reduce_to_norm(t2,defs);
         println!();
       },
//       _ => {
//         eprintln!("unable to evaulate ({:?})",line);
//       },
     }//match line
  }// for each line in prog
}//eval_prog



/////////////////// lexer (now autogenerated)
/*

pub struct LamLexer<'t>
{
  stk:StrTokenizer<'t>,
  keywords:HashSet<&'static str>,
}
impl<'t> LamLexer<'t>
{
  pub fn new(s:StrTokenizer<'t>) -> LamLexer<'t>
  {
    let mut kwh = HashSet::with_capacity(16);
    for kw in ["define","lambda","lam","Lam","λ","let","in","lazy","weak","CBV","strong"]
    { kwh.insert(kw);}
    LamLexer {
      stk: s,
      keywords : kwh,
    }
  }//new
}//impl LamLexer
impl<'t> Tokenizer<'t,Term> for LamLexer<'t>
{
   fn linenum(&self) -> usize {self.stk.line()}
   fn column(&self) -> usize {self.stk.column()}
   fn position(&self) -> usize {self.stk.current_position()}
   fn nextsym(&mut self) -> Option<TerminalToken<'t,Term>>
   {
      let tokopt = self.stk.next_token();
      if let None = tokopt { return None; }
      let tok = tokopt.unwrap();
      let tt =  match tok.0 {
        RawToken::Symbol(".") => TerminalToken::from_raw(tok,"DOT",Nothing),
        RawToken::Symbol(s) => TerminalToken::from_raw(tok,s,Nothing),
        RawToken::Alphanum(a) if a=="Liang" || a=="liang" || a=="LIANG" => {
          TerminalToken::from_raw(tok,"Liang",Nothing)
        },
        RawToken::Alphanum(a) if self.keywords.contains(a) => {
          TerminalToken::from_raw(tok,a,Nothing)
        },
        RawToken::Alphanum(a) => TerminalToken::from_raw(tok,"ID",Var(str8::from(a))),
        RawToken::Num(n) => TerminalToken::from_raw(tok,"INTEGER",Const(n)),
        _ => TerminalToken::from_raw(tok,"<<UNRECOGNIZED>>",Nothing),
      };//match
      Some(tt)
   }//nextsym
}
*/
