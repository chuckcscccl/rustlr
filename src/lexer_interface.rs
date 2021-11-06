// module for interfacing with any lexical analyzer
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

/// This structure is expected to be returned by the lexical analyzer ([Lexer] objects).
/// Furthermore, the .sym field of a Lextoken *must* match the name of a terminal
/// symbol specified in the grammar that defines the language.  AT is the type of the
/// *value* attached to the token, which is usually some enum that distinguishes between
/// numbers, keywords, alphanumeric symbols and other symbols.  See the tutorial and
/// examples at <https://cs.hofstra.edu/~cscccl/rustlr_project>
/// on how to define the right kind of AT.

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

/// This trait defines the interace that any lexical analyzer must be adopted to.
pub trait Lexer<AT:Default>
{
  /// retrieves the next Lextoken, or None at end-of-stream.
  fn nextsym(&mut self) -> Option<Lextoken<AT>>;
  /// returns the current line number.  The 
  fn linenum(&self) -> usize; // line number
}//trait Lexer


/// This is a sample Lexer implementation designed to return every character in a
/// string as a separate token, and is used in small grammars for testing and
/// illustration purposes.  It is assumed that the characters read are defined as
/// terminal symbols in the grammar.
pub struct charlexer<'t>
{
   chars: Chars<'t>
}
impl<'t> charlexer<'t>
{
  pub fn new<'u:'t>(input:&'u str) -> charlexer<'u>
  { charlexer {chars:input.chars()} }
}
impl<'t, AT:Default> Lexer<AT> for charlexer<'t>
{
   fn nextsym(&mut self) -> Option<Lextoken<AT>>
   {
      match self.chars.next() {
        None => {None},
        Some(c) => { Some(Lextoken::new(c.to_string(),AT::default())) },
      }
   }
   fn linenum(&self) -> usize { 0 }
}//impl Lexer for lexer
