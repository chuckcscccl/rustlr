use regex::Regex;

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

/*
 Expected sym fields:
 "num", value i64
 "hex", value u64
 "float", value f64
 "strlit", value &'t str with enclosing ""
 "alphanum", value &'t str
 "symbol" , value &'t str with symbol itself- non-alphanumeric
*/

pub enum RawToken<'t>
{
  Num(i64),
  Hex(u64),
  Float(f64),
  Strlit(&'t str),
  Alphanum(&'t str),
  Symbol(&'t str),
  Newline(&'t str),
  Verbatim(&'t str),
  Whitespace(usize), // counts number of non-newline whitespaces
}//RawToken


fn main()
{
  let re = Regex::new(r"\d{4}-\d{2}-\d{2}\sxyz$").unwrap();
  //println!("{}",re.is_match("abc2014-01-01 xyz"));

  // Regex for common token types

  // decimal unsigned int
  let decuint = Regex::new(r"^\d+$").unwrap();
  // hexadecimal number
  let hexnum = Regex::new(r"^0x[\dABCDEFabcdef]+$").unwrap();
  // string literal
  let strlit = Regex::new(r"^\x22(?s)(.*)\x22$").unwrap();

  println!("{}",decuint.is_match("1023"));
  println!("{}",hexnum.is_match("0x22"));
  println!("{}",hexnum.is_match("0x7Ef6"));
  println!("{}",strlit.is_match("\"abc\""));
  println!("{}",strlit.is_match("\"hi \"ok\" th
ere\""));
  
}//main
