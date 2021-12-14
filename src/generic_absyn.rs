//! Generic Abstract Syntax Support Module
//!
//! Rustlr allows any type that implements the Default trait to be used as
//! the abstract syntax type (grammar directive absyntype).  However, this
//! module defines a generic abstract syntax type [GenAbsyn], along with
//! custom smart pointer [LBox] (and alias [ABox]), that simplifies the
//! construction of abstract syntax trees. LBox keeps the line and column
//! numbers of each syntatic construct.

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_macros)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]


use std::ops::{Deref,DerefMut};
use std::collections::{HashMap,HashSet};
use crate::RuntimeParser;
use crate::GenAbsyn::*;

/// custom smart pointer that encapsulates line number, column and other information
/// for warnings and error messages after the parsing stage.  Implements
/// [Deref] and [DerefMut] so the encapsulated expression can be accessed as
/// in a standard Box.  For example, an abstract syntax type can be defined by
///```text
/// enum Expr {
///   Val(i64),
///   PlusExpr(LBox<Expr>,LBox<Expr>),
///   ...
/// }
///```
///```text
/// fn check(e:&Expr) {
///   match e {
///     PlusExpr(a,b) => {
///       println!("checking subexpressions on line {} and {}",a.line,b.line);
///       check(a); check(b); // Deref coercion used here
///     },
///   ...
///```
/// The [RuntimeParser::lb] function can be called from the semantic actions
/// of a grammar
/// to create LBoxed-values that include line/column information.  LBox<T>
/// implements the Default trait if T does, so an LBox type can also serve
/// as the absyntract syntax type for a grammar.
/// The src_id field of LBox can be used to point to externally kept
/// information about the source being compiled, such as the source file
/// name when mulitple files are compiled together.
pub struct LBox<T>
{
  pub exp:Box<T>,
  pub line:usize,
  pub column:usize,
  pub src_id:usize,   // must refer to info kept externally
}                   
impl<T> LBox<T>
{
  pub fn new(e:T,ln:usize,col:usize,src:usize) -> LBox<T>
  { LBox { exp:Box::new(e), line:ln, column:col, src_id:src } }
  pub fn set_src_id(&mut self, id:usize) {self.src_id=id;}
}
impl<T> Deref for LBox<T>
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.exp
    }
}
impl<T> DerefMut for LBox<T>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.exp
    }
}
impl<T:Default> Default for LBox<T>
{
  fn default() -> Self {LBox::new(T::default(),0,0,0)}
}

/// [LBox] specific to [GenAbsyn] type, implements [Debug] and [Clone],
/// unlike a generic LBox
pub type ABox = LBox<GenAbsyn>;

impl std::fmt::Debug for ABox
{
fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ABox")
         .field("exp", &self.exp)
         .field("line", &self.line)
         .field("column", &self.column)         
         .finish()
    }
}// impl Debug for ABox
impl Clone for ABox
{
  fn clone(&self) -> Self
  {
     LBox {
       exp: self.exp.clone(),
       line: self.line,
       column: self.column,
       src_id: self.src_id,
     }
  }
}// impl Clone for ABox


/// Generic Abstract Syntax type: Rustlr offers the user the option
/// of using a ready-made abstract syntax type that should be suitable for
/// ordinary situations.  Incorporates the usage of LBox so that abstract
/// syntax trees always carry line/column information for error reporting and
/// warnings.  Rustlr will implement an option to produce a parser that
/// uses GenAbsyn in conjunction with [ABox], and a custom parse_generic function
/// that should simplify the process of parsing into abstract syntax.
#[derive(Clone,Debug)]
pub enum GenAbsyn
{
  Integer(i64),
  BigInteger(String),
  Float(f64),
  Symbol(&'static str),
  Keyword(&'static str),
  Alphanum(String),
  Stringlit(String),
  Verbatim(String),
  Uniop(usize,ABox),
  Binop(usize,ABox,ABox),
  Ternop(usize,ABox,ABox,ABox),
  Sequence(usize,Vec<ABox>),
  Partial(ABox,String), // error result, line/column/msg
  NewLine,
  Whitespaces(usize),
  Nothing,
}
impl GenAbsyn
{
   pub fn is_complete(&self) -> bool
   {
      match self
      {
        Partial(_,_) => false,
        Uniop(_,s) => {
            println!("s.line {}{}",s.line,s.column);
            s.is_complete()
        },
        Binop(_,a,b) => a.is_complete() && b.is_complete(),
        Ternop(_,a,b,c) => a.is_complete() && b.is_complete() && c.is_complete(),
        Sequence(_,v) => {
          for x in v { if !x.is_complete() {return false; } }
          true
        }
        _ => true,
      }
   }//is_complete
}//impl GenAbsyn

impl Default for GenAbsyn
{
  fn default() -> Self { Nothing }
}//impl Default


//const KEYWORDS:[&'static str;4] = ["if","while","let","lambda"];
//const SYMBOLS:[&'static str;4] = ["==","(",")",":"];

// don't know what to do with this yet

/// Structure for configuring specific use of [GenAbsyn] type, which
/// provides generic representations of binary, unary, ternary and
/// vectorized operators.  GenAbsyn also distinguishes keywords from
/// other alphanumeric and non-alphanumeric symbols. Since these are
/// provided from source code, &'static str is used instead of owned
/// strings.  The index of the corresponding &'static str in 
/// the static array OPERATORS will determine
/// the usize index that identifies each Binop, Uniop, etc.
pub struct Absyn_Statics<const N:usize,const M:usize,const P:usize>
{
   pub KEYWORDS: [&'static str; N],
   pub SYMBOLS: [&'static str; M],
   /// indices determine usize identifier of Binop, Uniop, Ternop, etc.
   pub OPERATORS: [&'static str; P],
   /// maps each string to corresponding index in KEYWORDS
   pub Keyhash:HashMap<&'static str,usize>,
   pub Symhash:HashMap<&'static str,usize>,
   pub Ophash: HashMap<&'static str,usize>,
}
impl<const N:usize,const M:usize,const P:usize> Absyn_Statics<N,M,P>
{
  pub fn new(k:[&'static str; N],s:[&'static str;M],p:[&'static str;P])
      -> Absyn_Statics<N,M,P>
  {
    let mut newas = Absyn_Statics {
       KEYWORDS: k,
       SYMBOLS: s,
       OPERATORS:p,
       Keyhash: HashMap::with_capacity(N),
       Symhash: HashMap::with_capacity(M),
       Ophash: HashMap::with_capacity(P),       
    };
    for i in 0..N { newas.Keyhash.insert(k[i],i); }
    for i in 0..M { newas.Symhash.insert(s[i],i); }
    for i in 0..P { newas.Ophash.insert(p[i],i); }
    return newas;
  }//new

  pub fn is_keyword(&self,k:&str) -> bool
  { self.Keyhash.contains_key(k) }
  pub fn is_symbol(&self,k:&str) -> bool
  { self.Symhash.contains_key(k) }
  pub fn is_operator(&self,k:&str) -> bool
  { self.Ophash.contains_key(k) }    
}//impl Absyn_Statics


/* //testing  - did compile
fn check(e:&GenAbsyn) -> bool {
  match e {
    Binop(3,x,_) => {
         println!("Binop 3 not allowed on line {}, column {}",x.line,x.column);
         false
    },
    Binop(i,a,b) => { check(a) && check(b) },
    _ => true
  }
}
*/


///macro for creating [LBox] structures, for use in grammar semantic actions: each semantic action associated
/// with a grammar rule is a Rust lambda expression of the form
/// __|parser|{...}__
/// of type __fn(&mut RuntimeParser) -> AT__ where AT is the type of the abstract
/// syntax value defined for the grammar. __lbox!(e)__ expands to **parser.lb(e)**, calling the [RuntimeParser::lb] function.
/// The macro can also form an [ABox], which is an alias for LBox<GenAbsyn>

#[macro_export]
macro_rules! lbox {
  ( $x:expr ) => {
    parser.lb($x)
  };
}
