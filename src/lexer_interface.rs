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
  /// returns the current line being tokenized as an owned string.  The
  /// default implementation returns the empty string.
  fn current_line(&self) -> String  { // with default implementation
     String::from("")
  }
}//trait Lexer


/// This is a sample Lexer implementation designed to return every non-whitespace character in a
/// string as a separate token, and is used in small grammars for testing and
/// illustration purposes.  It is assumed that the characters read are defined as
/// terminal symbols in the grammar.
pub struct charlexer<'t>
{
   pub chars: Chars<'t>,
   index: usize,
}
impl<'t> charlexer<'t>
{
  pub fn new<'u:'t>(input:&'u str) -> charlexer<'u>
  { charlexer {chars:input.chars(), index:0} }
}
impl<'t, AT:Default> Lexer<AT> for charlexer<'t>
{
   fn nextsym(&mut self) -> Option<Lextoken<AT>>
   {
      let mut res = None;
      let mut stop = false;
      while !stop
      {
       res=match self.chars.next() {
        None => {stop=true; None},
        Some(c) => {
          self.index+=1;
          if c.is_whitespace() {None}
          else {stop=true; Some(Lextoken::new(c.to_string(),AT::default()))}
        },
       }//match
      }//while
      res
   }//nextsym
   fn linenum(&self) -> usize { 1 }
   fn column(&self) -> usize { self.index }
   fn current_line(&self) -> String
   { 
     self.chars.clone().collect()
   }   
}//impl Lexer for lexer
