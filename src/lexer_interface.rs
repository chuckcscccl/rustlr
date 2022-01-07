//! Rustlr allows the use of any lexical analyzer (tokenizer) that satisfies
//! the [Lexer] trait.  Only a simple [charlexer] tokenizer that separates
//! non-whitespaces characters is provided as an example.

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
#![allow(unused_doc_comments)]
#![allow(unused_imports)]
use std::str::Chars;
//use crate::{ParseResult,ParseValue};


/// This structure is expected to be returned by the lexical analyzer ([Lexer] objects).
/// Furthermore, the .sym field of a Lextoken *must* match the name of a terminal
/// symbol specified in the grammar that defines the language.  AT is the type of the
/// *value* attached to the token, which is usually some enum that distinguishes between
/// numbers, keywords, alphanumeric symbols and other symbols.  See the [tutorial and examples](<https://cs.hofstra.edu/~cscccl/rustlr_project>)
/// on how to define the right kind of AT type.

pub struct Lextoken<AT:Default> // now separated from Gsym
{
   pub sym: String, // must correspond to terminal symbol
   pub value: AT,         // value of terminal symbol, if any
}
impl<AT:Default> Lextoken<AT>
{
  /// creates a new Lextoken
  pub fn new(name:String, val:AT) -> Lextoken<AT>   
  {
     Lextoken {
       sym : name,
       value : val,
     }
  }//new Lextoken
}//impl Lextoken

/// This trait defines the interace that any lexical analyzer must be adopted
/// to.  The default implementations for linenum, column and
/// current_line *should be replaced.* They're provided only for compatibility.
pub trait Lexer<AT:Default>
{
  /// retrieves the next Lextoken, or None at end-of-stream. 
  fn nextsym(&mut self) -> Option<Lextoken<AT>>;
  /// returns the current line number.  The default implementation
  /// returns 0.
  fn linenum(&self) -> usize { 0 } // line number
  /// returns the current column (character position) on the current line.
  /// The default implementation returns 0;
  fn column(&self) -> usize { 0 }
  /// returns the current line being tokenized.  The
  /// default implementation returns the empty string.
  fn current_line(&self) -> &str  { "" }
/*  
  /// function that modifies a Lextoken
  /// For example, some symbols such as {, } and |
  /// are reserved and cannot be used for terminal symbols.  Lextokens
  /// containing them have to be modified.  The default implementation
  /// returns the given token unmodified.  Note that this function is
  /// **not** called automatically by [RuntimeParser::parse], and it is
  /// up to the implementor of [Lexer::nextsym] to call it.
  fn modify(t:Lextoken<AT>)->Lextoken<AT> { t }

  /// this function takes a functional argument intended to change the
  /// [Lexer::modify] function.  The default implementation does nothing.
  fn set_modify(&mut self,fn(Lextoken<AT>)->Lextoken<AT>) {}
*/  
}//trait Lexer


/// This is a sample Lexer implementation designed to return every character in a
/// string as a separate token, and is used in small grammars for testing and
/// illustration purposes.  It is assumed that the characters read are defined as
/// terminal symbols in the grammar.
pub struct charlexer<'t>
{
   chars: Chars<'t>,
   index: usize,
   len: usize,
   line:usize,
   keep_ws: bool,  // keep whitespace chars
   /// function to modify char returned by nextsym, can be changed.
   /// Both [charlexer::make] and [charlexer::new] sets this function
   /// initially to `|x|{x.to_string()}`.  For example, some characters such
   /// as '{' and '}' cannot be used as terminal symbols of a grammar and must
   /// be translated into something like "LBRACE" and "RBRACE"
   pub modify: fn(char)->String, 
}
impl<'t> charlexer<'t>
{
  /// creates a charlexer that emits only non-whitespace chars
  pub fn new<'u:'t>(input:&'u str) -> charlexer<'u>
  { charlexer {chars:input.chars(), index:0, len:input.len(), line:1, keep_ws:false, modify: |x|{x.to_string()}} }
  /// creates a charlexer with the option of keeping whitespace chars if kws=true
  pub fn make<'u:'t>(input:&'u str, kws:bool) -> charlexer<'u>
  { charlexer {chars:input.chars(), index:0, len:input.len(), line:1, keep_ws:kws, modify:|x|{x.to_string()}} } 
}
impl<'t, AT:Default> Lexer<AT> for charlexer<'t>
{
   fn nextsym(&mut self) -> Option<Lextoken<AT>>
   {
      let mut res = None;
      let mut stop = false;
      while !stop && self.index<self.len
      {
       let nc = self.chars.next();
       res=match nc { //self.chars.next() {
        None => {stop=true; None},
        Some(c) => {
          self.index+=1;
          if c=='\n' {self.line+=1;}
          if c.is_whitespace() && !self.keep_ws {None}
          else {
            stop=true;
            let mc = (self.modify)(c);
            Some(Lextoken::new(mc,AT::default()))}
        },
       }//match
      }//while
      if (self.index<=self.len) {res} else {None}
   }//nextsym
   /// returns current line number starting from 1
   fn linenum(&self) -> usize { self.line }
   /// returns the index of the current char, starting from 1
   fn column(&self) -> usize { self.index }
   /// returns slice of underlying data using [std::str::Chars::as_str]
   fn current_line(&self) -> &str
   { 
     self.chars.as_str()
   }   
}//impl Lexer for lexer

//////////////////////// new stuff

/// This struct is intended to replace Lextoken, and will not use owned string
pub struct LexToken<'t,AT:Default>
{
  pub sym: &'t str,
  pub value: AT,
}
impl<'t,AT:Default> LexToken<'t,AT>
{
  pub fn new(s:&'t str, v:AT) -> LexToken<'t,AT> { LexToken{sym:s, value:v} }
}

/// This trait is intended to replace Lexer, and won't use owned strings
pub trait Tokenizer<AT:Default>
{
  /// retrieves the next Lextoken, or None at end-of-stream. 
  fn nextsym(&mut self) -> Option<LexToken<AT>>;
  /// returns the current line number.  The default implementation
  /// returns 0.
  fn linenum(&self) -> usize { 0 } // line number
  /// returns the current column (character position) on the current line.
  /// The default implementation returns 0;
  fn column(&self) -> usize { 0 }
  /// returns the current line being tokenized.  The
  /// default implementation returns the empty string.
  fn current_line(&self) -> &str  { "" }
}
//rewrite parse as parse_zc, which uses Tokenizer/LexToken - just rewrite it.
