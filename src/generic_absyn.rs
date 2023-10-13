//! Generic Abstract Syntax Support Module
//!
//! Rustlr allows any type that implements the Default trait to be used as
//! the abstract syntax type (grammar directive absyntype).  However, this
//! module defines custom smart pointers [LBox] and [LRc] that simplify the
//! construction of abstract syntax trees. LBox/LRc keep the line and column
//! numbers of each syntatic construct, as these are often
//! needed during later stages of code analysis post-parsing.
//!
//! For example, an abstract syntax type can be defined by
//!```text
//! enum Expr {
//!   Val(i64),
//!   PlusExpr(LBox<Expr>,LBox<Expr>),
//!   ...
//! }
//!```
//!```text
//! fn check(e:&Expr) {
//!   match e {
//!     PlusExpr(a,b) => {
//!       println!("checking expressions on lines {} and {}",a.line(),b.line());
//!       check(a); check(b); // Deref coercion used here
//!     },
//!   ...
//!```
//! The [ZCParser::lbx] function can be called from the semantic actions
//! of a grammar
//! to create LBoxed-values that include line/column information.  `LBox<T>`
//! implements the Default trait if T does, so an LBox type can also serve
//! as the absyntract syntax type for a grammar.
//! It is also possible to use `LBox<dyn Any>` as the abstract syntax type
//! along with the [LBox::upcast] and [LBox::downcast] functions and
//! convenience macros [lbup] and [lbdown].
//!
//! Sufficient functionality has also been implemented to allow the use of
//! `LBox<dyn Any>` as the abstract syntax type of Grammars.
//! This effectively allows grammar symbols to carray values of different types
//! as Any-trait objects.  The functions [LBox::upcast], [LBox::downcast], [ZCParser::lba],
//! and the convenience macros [lbup], [lbdown] and [lbget]
//! are intended to support this usage.  A simplified, sample grammar using
//! `LBox<dyn Any>` as the abstract syntax type returned by the parser is
//! found [here](https://cs.hofstra.edu/~cscccl/rustlr_project/simple.grammar),
//! which generates this LALR [parser](https://cs.hofstra.edu/~cscccl/rustlr_project/simpleparser.rs).
//!
//! Equivalent functions are available for [LRc]. 

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_macros)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::rc::Rc;
use std::ops::{Deref,DerefMut};
use std::collections::{HashMap,HashSet};
use std::any::Any;
use crate::zc_parser;
use crate::zc_parser::ZCParser;
use crate::{lbup,lbdown,lbget};
use bumpalo::Bump;

/// Custom smart pointer that encapsulates line and column numbers along with
/// a regular [Box].  Implements [Deref] and [DerefMut] so the encapsulated
/// expression can be accessed as in a standard Box.  This is intended to
/// to be used in the formation of abstract syntax trees so that the lexical
/// information is available for each construct after the parsing stage.
/// Also included in each LBox created by the runtime parser ([ZCParser])
/// is a 32 bit (u32) *unique identifier* **uid** that uniquely identifies
/// each LBox.  An expression and a subexpression can both begin at the same
/// line and column numbers and the unique id would be required to
/// distinguish them.  This device is useful for hashing information, such
/// as inferred types, based on the location of an expression in the source.
pub struct LBox<T:?Sized>
{
  pub exp:Box<T>,
  pub line:u32,
  pub column:u32,
  pub uid:u32, // unique id
  // must refer to information kept externally  
  //pub src_id:usize,   
}
impl<T> LBox<T>
{
  /// creates a new LBox with line ln, column col and uid set to zero;
  /// this function is deprecated by [LBox::make].
  pub fn new(e:T,ln:usize,col:usize /*,src:usize*/) -> LBox<T>
  { LBox { exp:Box::new(e), line:ln as u32, column:col as u32,uid:0, } }
  /// creates a new LBox enclosing e, with line ln, column col and uid u
  pub fn make(e:T,ln:usize,col:usize, u:u32) -> LBox<T>
  { LBox { exp:Box::new(e), line:ln as u32, column:col as u32,uid:u, } }  
  ///should be used to create a new LBoxed expression that inherits
  /// lexical information from existing LBox
  pub fn transfer<U>(&self,e:U) -> LBox<U>
  {
     LBox::make(e,self.line(),self.column(),self.uid)
  }
  /// Since version 0.2.4, Rustlr now stores the line and column
  /// information internally as u32 values instead of usize, and
  /// the [LBox::line] and [LBox::column] functions are provided for
  /// interface
  pub fn line(&self)-> usize {self.line as usize}
  pub fn column(&self)->usize {self.column as usize}
  pub fn uid(&self) -> u32 {self.uid}
}//impl LBox
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
  fn default() -> Self {LBox::new(T::default(),0,0/*,0*/)}
}
impl<T:Clone> Clone for LBox<T>
{
   fn clone(&self) -> Self
   {
      LBox {
        exp : self.exp.clone(),
        line: self.line,
        column: self.column,
        uid:self.uid,
        //src_id: self.src_id,
      }
   }//clone
}
impl<T:?Sized> AsRef<T> for LBox<T>
{
  fn as_ref(&self) -> &T  { &*self.exp }
}
impl<T:?Sized> AsMut<T> for LBox<T>
{
  fn as_mut(&mut self) -> &mut T  { &mut *self.exp }
}
impl<T:Default+?Sized> LBox<T>
{
   /// replaces the boxed value with T::default() and returns the value
   pub fn take(&mut self) -> T
   {   let mut res = T::default();
       std::mem::swap(&mut res, self.as_mut());
       res
   }
   /// convert information from [LC] to LBox, consumes LC
   pub fn from_lc(c:LC<T>) -> Self
   {
       LBox{exp:Box::new(c.0), line:c.1.0, column:c.1.1, uid:c.1.2}
   }
}
impl<'t> LBox<dyn Any+'t>
{
  /// emulates [Box::downcast] function, when `LBox<dyn Any>` is used as
  /// the abstract syntax type.  Note that unlike Box::downcast, an Option
  /// is returned here instead of a result.
  pub fn downcast<U:'t>(self) -> Option<LBox<U>>
  {
     let boxdown = self.exp.downcast::<U>();
     if let Err(_) = boxdown {return None;}
     Some(LBox {
       exp : boxdown.unwrap(),
       line: self.line,
       column: self.column,
       uid:self.uid,
       //src_id: self.src_id,
     })
  }
  /// do not try to create a `LBox<dyn Any>` structure with something like
  ///```
  /// let lb:LBox<dyn Any> = LBox::new(String::from("abc"),0,0);
  ///```  
  /// This does not work as LBox is simply borrowing the underlying mechanics of
  /// [Box] instead of re-creating them.  Do instead:
  ///```
  /// let lb:LBox<dyn Any> = LBox::upcast(LBox::new(String::from("abc"),0,0));
  ///```
  /// upcast always returns a `LBox<dyn Any>`.
  pub fn upcast<T:'t>(lb:LBox<T>) -> Self
  {
     let bx:Box<dyn Any+'t> = lb.exp; // this requires Any+'static ??
     LBox { exp:bx, line:lb.line, column:lb.column, uid:lb.uid, }
  }
}// downcast/upcast for LBox

///this is provided so `LBox<dyn Any>` can be used for the abstract syntax type.
/// the default is a Lbox containing a static string.
impl Default for LBox<dyn Any>
{
  fn default() -> Self {LBox::upcast(LBox::new("LBox<dyn Any> defaults to this string",0,0/*,0*/))}
}

impl<T:std::fmt::Debug> std::fmt::Debug for LBox<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&**self, f)    
//        f.debug_struct("LBox")
//         .field("exp", &self.exp)
//         .field("y", &self.y)
//         .finish()
    }
}



///Like LBox but encapsulates an Rc. Implements [Deref] and emulates the
///[Rc::clone] function.
pub struct LRc<T:?Sized>
{
  pub exp:Rc<T>,
  pub line:u32,
  pub column:u32,
  //pub src_id:usize,
}
impl<T> LRc<T>
{
  pub fn new(e:T,ln:usize,col:usize /*,src:usize*/) -> LRc<T>
  { LRc { exp:Rc::new(e), line:ln as u32, column:col as u32, /*src_id:src*/ } }
  //pub fn set_src_id(&mut self, id:usize) {self.src_id=id;}
  ///should be used to create a new LRc-expression that inherits
  /// lexical information from existing LRc
  pub fn transfer<U>(&self,e:U) -> LRc<U>
  {
     LRc::new(e,self.line(),self.column() /*,self.src_id*/)
  }
  ///uses [Rc::clone] to increase reference count of encapsulated Rc,
  ///copies line, column and source_id information.
  pub fn clone(lrc:&LRc<T>) -> LRc<T>
  {
     LRc {
        exp: Rc::clone(&lrc.exp),
        line: lrc.line,
        column: lrc.column,
        //src_id: lrc.src_id,
     }
  }//clone
  pub fn line(&self) -> usize {self.line as usize}
  //self.line.try_into().unwrap()}
  pub fn column(&self) -> usize {self.column as usize}
}

impl<T:Clone> Clone for LRc<T>
{
   fn clone(&self) -> Self
   {
     LRc::clone(self)
   }//clone
}

impl<T> Deref for LRc<T>
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.exp
    }
}
/*  DerefMut is not implemented for Rc<T>
impl<T> DerefMut for LRc<T>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.exp
    }
}
*/
impl<T:Default> Default for LRc<T>
{
  fn default() -> Self {LRc::new(T::default(),0,0/*,0*/)}
}

impl LRc<dyn Any+'static>
{
  /// emulates [LRc::downcast] function. Note that unlike Box::downcast,
  ///an Option is returned here instead of a result.
  pub fn downcast<U:'static>(self) -> Option<LRc<U>>
  {
     let rcdown = self.exp.downcast::<U>();
     if let Err(_) = rcdown {return None;}
     Some(LRc {
       exp : rcdown.unwrap(),
       line: self.line,
       column: self.column,
       //src_id: self.src_id,
     })
  }
  /// upcasts `LRc<T>` to `LRc<dyn Any>`
  pub fn upcast<T:'static>(lb:LRc<T>) -> Self
  {
     let bx:Rc<dyn Any> = lb.exp;
     LRc { exp:bx, line:lb.line, column:lb.column, /*src_id:lb.src_id,*/ }
  }
}// downcast/upcast for LRc

///this is required if `LRc<dyn Any>` is used for the abstract syntax type
impl Default for LRc<dyn Any+'static>
{
  fn default() -> Self {LRc::upcast(LRc::new("LRc<dyn Any> defaults to this string",0,0 /*,0*/))}
}

impl<T:std::fmt::Debug> std::fmt::Debug for LRc<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&**self, f)    
    }
}

/// Version of LBox that does not use a Box. This tuple struct contains
/// a value of type T in the first position and a tuple consisting of
/// (line, column, uid) numbers in the second position.  The uid or
/// *unique identifier* is a unique number placed in each LC structure
/// created by the parser.  It is useful for hashing information (such as
/// inferred types) based on their source location.  Multiple AST constructs
/// may begin with the same line/column position, but the unique id will
/// disambiguate them.
/// This feature was added to Rustlr to support bumpalo-allocated ASTs.
pub struct LC<T>(pub T,pub (u32,u32,u32));

impl<T:Default> Default for LC<T> {
  fn default() -> Self {LC(T::default(),(0,0,0))}
}

impl<T> Deref for LC<T>
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for LC<T>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<T> AsRef<T> for LC<T>
{
  fn as_ref(&self) -> &T  { &self.0 }
}
impl<T> AsMut<T> for LC<T>
{
  fn as_mut(&mut self) -> &mut T  { &mut self.0 }
}
impl<T:std::fmt::Debug> std::fmt::Debug for LC<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)    
    }
}
impl<T> LC<T>
{
  /// creates a new LC tuple struct in the form (value, (line,column,unique_id))
  pub fn make(x:T,ln:usize,cl:usize,uid:u32) -> Self {
    LC(x,(ln as u32,cl as u32,uid))
  }
  /// creates a new LC tuple struct in the form (value, (line,column))
  /// with the unique_id field set to zero (for backwards compatibility)
  pub fn new(x:T,ln:usize,cl:usize) -> Self {
    LC(x,(ln as u32,cl as u32,0))
  }
  /// returns encapsulated line
  pub fn line(&self) -> usize { (self.1.0) as usize }
  /// returns encapsulated column
  pub fn column(&self) -> usize { (self.1.1) as usize }
  /// returns (line,column) as a pair of 32 values
  pub fn lncl(&self) -> (u32,u32) { (self.1.0,self.1.1) }
  /// returns unique id
  pub fn uid(&self) -> u32 { self.1.2 }
  /// returns value reference
  pub fn value(&self) -> &T { &self.0 }
  /// transfers line/column information to another LC
  pub fn transfer<U>(&self,x:U) -> LC<U> {
   LC::make(x,self.line(),self.column(),self.1.2) 
  }
  /// consumes the LC enclosure and returns the enclosed value
  pub fn consume(self) -> T { self.0 }
}//impl LC

impl<T:Default+?Sized> LC<T>
{
  ///transfers information from an [LBox] into an LC, consumes LBox
  pub fn from_lbox(mut lb:LBox<T>) -> Self {
    LC::make(lb.take(),lb.line(),lb.column(),lb.uid())
  }
}  

impl<T:Default+?Sized> LC<T>
{
   /// replaces the enclosed value with T::default() and returns the value
   pub fn take(&mut self) -> T
   {   let mut res = T::default();
       std::mem::swap(&mut res, self.as_mut());
       res
   }
}

/// Structure intended to support [bumpalo](https://docs.rs/bumpalo/latest/bumpalo/index.html) AST generations,
///which allows recursive AST types to be defined using references instead of
///smart pointers (LBox).  The benefit of bump-allocated ASTs is the ability to
///pattern match against nested, recursive structures, e.g.,
///`Negative(Negative(x))`.  The lifetime `'t` of a Bumper should be the same
///as the lifetime of the parser's input.
/// This structure is intended to become the
///'external state' that's carried by the runtime parser (parser.exstate),
///and therefore wraps another value (exstate) of the declared `externtype`.
pub struct Bumper<'t,ET:Default>
{
  bump:Option<&'t Bump>,
  exstate: ET,
}//bumper struct
impl<T:Default> Default for Bumper<'_,T>  {
  fn default() -> Self { Bumper{exstate:T::default(), bump:None} }
}
impl<'t,T:Default> Bumper<'t,T>
{
  /// sets the encapsulated [Bump](https://docs.rs/bumpalo/latest/bumpalo/struct.Bump.html) allocator
  pub fn set(&mut self, b:&'t Bump) { self.bump = Some(b); }

  /// tests if the allocator has been set
  pub fn is_set(&self) -> bool { self.bump.is_some() }
  
  /// returns pointer to the bump allocator.  Warning: this function calls
  /// unwrap and should only be called after [Bumper::set] has been called
  pub fn get(&self)->&'t Bump { self.bump.expect("bump arena not set") }

  /// bump-allocates x and returns a reference, which will live as long
  /// as the bump allocator.  Warning: this function calls unwrap and
  /// will panic if the Bumper is not set.
  pub fn make<S>(&self,x:S) -> &'t S {
     self.get().alloc(x)
  }

  /// version of make that returns Option, but will not panic
  pub fn make_safe<S>(&self,x:S) -> Option<&'t S> {
     if self.bump.is_some() {Some(self.get().alloc(x))}
     else {None}
  }

  /// version of make_safe that returns a mutable reference to the allocated
  /// value
  pub fn alloc<S>(&self,x:S) -> Option<&'t mut S> {
     self.bump.map(|b|b.alloc(x))
  }

  /// returns a mut reference to the encapsulated "state" of type ET
  pub fn state(&mut self) -> &mut T { &mut self.exstate }
}//impl Bumper





// [LBox] specific to [GenAbsyn] type, implements [Debug] and [Clone],
// unlike a generic LBox
type ABox = LBox<GenAbsyn>;


/// Generic Abstract Syntax type: Rustlr offers the user the option
/// of using a ready-made abstract syntax type that should be suitable for
/// ordinary situations.  Incorporates the usage of LBox so that abstract
/// syntax trees always carry line/column information for error reporting and
/// warnings.  Rustlr will implement an option to produce a parser that
/// uses GenAbsyn in conjunction with [ABox], and a custom parse_generic function
/// that should simplify the process of parsing into abstract syntax.
#[derive(Clone,Debug)]
enum GenAbsyn
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
   fn is_complete(&self) -> bool
   {
      match self
      {
        GenAbsyn::Partial(_,_) => false,
        GenAbsyn::Uniop(_,s) => {
            println!("s.line {}{}",s.line,s.column);
            s.is_complete()
        },
        GenAbsyn::Binop(_,a,b) => a.is_complete() && b.is_complete(),
        GenAbsyn::Ternop(_,a,b,c) => a.is_complete() && b.is_complete() && c.is_complete(),
        GenAbsyn::Sequence(_,v) => {
          for x in v { if !x.is_complete() {return false; } }
          true
        }
        _ => true,
      }
   }//is_complete
}//impl GenAbsyn

impl Default for GenAbsyn
{
  fn default() -> Self { GenAbsyn::Nothing }
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
struct Absyn_Statics<const N:usize,const M:usize,const P:usize>
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
  fn new(k:[&'static str; N],s:[&'static str;M],p:[&'static str;P])
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

/*
///macro for creating [LBox] structures, for use in grammar semantic actions: each semantic action associated
/// with a grammar rule is a Rust lambda expression of the form
/// __|parser|{...}__
/// of type __fn(&mut RuntimeParser) -> AT__ where AT is the type of the abstract
/// syntax value defined for the grammar. __lbox!(e)__ expands to **parser.lb(e)**, calling the [RuntimeParser::lb] function.
// The macro can also form an [ABox], which is an alias for LBox<GenAbsyn>
#[macro_export]
macro_rules! lbox {
  ( $parser:expr,$x:expr ) => {
    $parser.lb($x)
  };
}
*/

///macro for creating `LBox<dyn Any>` structures that can encapsulate any type
///as abstract syntax.  **Must** be called from within the semantic actions of
///a grammar production rule.
#[macro_export]
macro_rules! lbup {
  ( $x:expr ) => {
    LBox::upcast($x)
  };
}

/// macro for downcasting `LBox<dyn Any>` to `LBox<A>` for some 
/// concrete type A. Must be called
/// from within the semantic actions of grammar productions.  **Warning:**
/// **unwrap** is called within the macro
#[macro_export]
macro_rules! lbdown {
  ( $x:expr,$t:ty ) => {
    $x.downcast::<$t>().unwrap()
  };
}

/// similar to [lbdown], but also extracts the boxed expression
#[macro_export]
macro_rules! lbget {
  ( $x:expr,$t:ty ) => {
    *$x.downcast::<$t>().unwrap().exp
  };
}
/// just extract value from LBox
#[macro_export]
macro_rules! unbox {
  ( $x:expr ) => {
    *$x.exp
  };
}

/// macro for creating an [LBox] from a [crate::StackedItem] ($si) popped from
/// the parse stack; should be called from within the semantics actions of
/// a grammar to accurately encode lexical information. 
#[macro_export]
macro_rules! makelbox {
  ($si:expr, $e:expr) => {
    LBox::new($e,$si.line,$si.column)
  };
}

/// similar to [makelbox] but creates an [LRc] from lexical information
/// inside stack item $si
#[macro_export]
macro_rules! makelrc {
  ($si:expr, $e:expr) => {
    LRc::new($e,$si.line,$si.column)
  };
}

/*
// just to see if it compiles
struct BB(usize);
fn testing() 
{
  let bb1:&dyn Any = &BB(1);
  let bb2:Box<dyn Any> = Box::new(BB(2));
  let bx:Box<dyn Any> = Box::new(String::from("abc"));
  //let b1:LBox<dyn Any> = LBox::new(BB(1),0,0,0); // doesn't work
  //let b3:LBox<dyn Any> = LBox::frombox(bb2,0,0,0); // works!
  let b5:LBox<BB> = LBox::new(BB(100),0,0,0);
  let b6:LBox<dyn Any> = LBox::upcast(b5);
  let b4:LBox<BB> = b6.downcast::<BB>().unwrap();
  let lb:LBox<dyn Any> = LBox::upcast(LBox::new(String::from("abc"),0,0,0));
  //let lbd:LBox<dyn Default> = LBox::upcast(LBox::new(String::from("ab"),0,0,0));  
}
*/
