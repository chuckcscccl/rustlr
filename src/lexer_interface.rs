//! Rustlr allows the use of any lexical analyzer (tokenizer) that satisfies
//! the [Tokenizer] trait.  However, a basic tokenizer, [StrTokenizer] is
//! provided that suffices for many examples.  This tokenizer is not
//! maximally efficient (not single-pass) as it uses [regex](https://docs.rs/regex/latest/regex/).
//!
//! The main contents of this module are [TerminalToken], [Tokenizer],
//! [RawToken], [StrTokenizer] and [LexSource].
//! For backwards compatibility with Rustlr version 0.1, [Lexer], [Lextoken]
//! and [charlexer] are retained, for now.

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
use regex::Regex;
use std::collections::{HashSet,BTreeMap};
use crate::RawToken::*;
use crate::{LBox,LRc,lbup};
use std::any::Any;
//use std::rc::Rc;
//use std::cell::RefCell;

/// **This structure is deprecated by [TerminalToken]**.
/// The structure is expected to be returned by the lexical analyzer ([Lexer] objects).
/// Furthermore, the .sym field of a Lextoken *must* match the name of a terminal
/// symbol specified in the grammar.
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

/// **This trait is deprecated by [Tokenizer]** and is only retained for
/// compatibility.
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
  /// **not** called automatically by [crate::RuntimeParser::parse], and it is
  /// up to the implementor of [Lexer::nextsym] to call it.
  fn modify(t:Lextoken<AT>)->Lextoken<AT> { t }

  /// this function takes a functional argument intended to change the
  /// [Lexer::modify] function.  The default implementation does nothing.
  fn set_modify(&mut self,fn(Lextoken<AT>)->Lextoken<AT>) {}
*/  
}//trait Lexer


/// **This struct is deprecated by [charscanner]**.  It is compatible with
/// [Lexer] and [Lextoken], which are also deprecated.
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




//////////////////////////////////////////////////////////////////////
//////////////////////// new stuff, needs regex
//////////////////////////////////////////////////////////////////////

/// This the token type required by Rustlr while parsing.  A TerminalToken must correspond
/// to a terminal symbol of the grammar being parsed.  The **sym** field of
/// the struct must correspond to the name of the terminal as defined by the
/// grammar and the **value** must be of type AT, which the is abstract syntax
/// type (*absyntype*) of the grammar.
/// It also includes the starting line and column positions of the token.
/// These tokens are generated by implementing [Tokenizer::nextsym].
///
/// This struct is intended to replace Lextoken, and does not use owned strings.
/// Currently this structure lives side-by side with Lextoken for compatibility.
pub struct TerminalToken<'t,AT:Default>
{
  pub sym: &'t str,
  pub value: AT,
  pub line:usize,
  pub column:usize,
}
impl<'t,AT:Default> TerminalToken<'t,AT>
{
  ///creates new lexical token with sym s, value v, line ln and column cl
  pub fn new(s:&'t str, v:AT, ln:usize, cl:usize) -> TerminalToken<'t,AT>
  { TerminalToken{sym:s, value:v, line:ln, column:cl} }
  /// transfers lexical information (line/column) to new TerminalToken
  pub fn transfer(&self, s:&'t str, v:AT) -> TerminalToken<'t,AT>
  {  TerminalToken{sym:s, value:v, line:self.line, column:self.column} }
  /// transfers lexical information from a (RawToken,line,column) triple
  /// returned by [StrTokenizer::next_token] to a new TerminalToken with
  /// sym s and value v.
  pub fn from_raw(rt:(RawToken<'t>,usize,usize),s:&'t str,v:AT) -> TerminalToken<'t,AT>
  { TerminalToken{sym:s, value:v, line:rt.1, column:rt.2} }

    /// creates an [LBox] vale  using lexical information contained in
    /// the token.
    pub fn lb<T>(&self,e:T) -> LBox<T> { LBox::new(e,self.line,self.column /*,self.src_id*/) }
    /// creates a `LBox<dyn Any>`, which allows attributes of different types to
    /// be associated with grammar symbols.
    pub fn lba<T:'static>(&self,e:T) -> LBox<dyn Any> { LBox::upcast(LBox::new(e,self.line,self.column /*,self.src_id*/)) }
    /// similar to [crate::ZCParser::lb], but creates a [LRc] instead of [LBox]
    pub fn lrc<T>(&self,e:T) -> LRc<T> { LRc::new(e,self.line,self.column /*,self.src_id*/) }
    /// similar to [crate::ZCParser::lba] but creates a [LRc]
    pub fn lrca<T:'static>(&self,e:T) -> LRc<dyn Any> { LRc::upcast(LRc::new(e,self.line,self.column /*,self.src_id*/)) }
}// impl TerminalToken

impl<'t,AT:Default+'static> TerminalToken<'t,AT>
{
  /// creates a [TerminalToken] from a [RawToken] with value of type
  /// `LBox<dyn Any>`
  pub fn raw_to_lba(rt:(RawToken<'t>,usize,usize),s:&'t str,v:AT) -> TerminalToken<'t,LBox<dyn Any>> {
   TerminalToken {
     sym:s,
     value: lbup!(LBox::new(v,rt.1,rt.2)),
     line:rt.1, column:rt.2,
   }
 }
}//impl for AT:'static

/* useless
impl<'t,AT:Default+std::fmt::Debug> TerminalToken<'t,AT>
{
   pub fn make_string(&self)->String
   { format!("{}({:?})",self.sym,&self.value) }
}
*/

///////////
/// This is the trait that repesents an abstract lexical scanner for
/// any grammar.  Any tokenizer must be adopted to implement this trait.
/// The default implementations of functions such as [Tokenizer::linenum] do not return correct values
/// and should be replaced: they're only given defaults for easy compatibility
/// with prototypes that may not have their own implementations.
/// This trait replaced LexToken used in earlier versions of Rustlr.
pub trait Tokenizer<'t,AT:Default> 
{
  /// retrieves the next [TerminalToken], or None at end-of-stream. 
  fn nextsym(&mut self) -> Option<TerminalToken<'t,AT>>;
  /// returns the current line number.  The default implementation
  /// returns 0.
  fn linenum(&self) -> usize { 0 } // line number
  /// returns the current column (character position) on the current line.
  /// The default implementation returns 0;
  fn column(&self) -> usize { 0 }
  /// returns the absolute character position of the tokenizer.  The
  /// default implementation returns 0;
  fn position(&self) -> usize { 0 }
  // returns (line, column) information based on given position
  //fn get_line_column(&self, position:usize) -> (usize,usize) { (0,0) }
  /*
  /// returns the previous position (before as opposed to after the current token).
  /// The default implementation returns 0.
  fn previous_position(&self) -> usize { 0 }
  */
  /// returns the current line being tokenized.  The
  /// default implementation returns the empty string.
  fn current_line(&self) -> &str  { "" }
  /// Retrieves the ith line of the raw input, if line index i is valid.
  /// This function should be called after the tokenizer has
  /// completed its task of scanning and tokenizing the entire input,
  /// when generating diagnostic messages when evaluating the AST post-parsing.

  /// The default implementation returns None.
  fn get_line(&self,i:usize) -> Option<&str> {None}

  /// Retrieves the source string slice at the indicated indices; returns
  /// the empty string if indices are invalid.  The default implementation
  /// returns the empty string.
  fn get_slice(&self,start:usize,end:usize) -> &str {""}
  
  /// retrieves the source (such as filename or URL) of the tokenizer.  The
  /// default implementation returns the empty string.
  fn source(&self) -> &str {""}

  /// For internal use only unless not using StrTokenizer.  This is a call-back
  /// function from the parser and can only be implemented when the grammar
  /// and token types are known.  It transforms a token to a token representing
  /// the wildcard "_", with semantic value indicating its position in the text.
  /// The default implementation returns the same TerminalToken.
  /// This function is automatically overridden by the generated lexer when
  /// using the -genlex option.
  fn transform_wildcard(&self,t:TerminalToken<'t,AT>) -> TerminalToken<'t,AT>
  {t}
  
  /// returns next [TerminalToken].  This provided function calls nextsym but
  /// will return a TerminalToken with sym="EOF" at end of stream, with
  /// value=AT::default().  The is the only provided function that should *not*
  /// be re-implemented.
  fn next_tt(&mut self) -> TerminalToken<'t,AT>
  {
    match self.nextsym() {
      Some(tok) => tok,
      None => TerminalToken::new("EOF",AT::default(),self.linenum(),self.column()),
    }//match
  }//next_tt
}// Trait Tokenizer

///////////////// Basic Tokenizer

/// structure produced by [StrTokenizer].  [TerminalToken]s must be
/// created from RawTokens (in the [Tokenizer::nextsym] function)
/// once the grammar's terminal symbols and abstract syntax type are known.
#[derive(Debug)]
pub enum RawToken<'t>
{
  /// an unsigned integer, though for convenience it is interpreted as
  /// a signed number.  Negative numbers must be recognized by higher-level
  /// parser.  Both decimal and hexadecimal numbers prefixed by 0x are
  /// recognized.
  Num(i64),
//  Hex(u64),
  /// floating point number
  Float(f64),
  /// Number too large for i64 or f64 - hex numbers will include 0x prefix
  BigNumber(&'t str),
  /// single character inside single quotes.
  Char(char), 
  /// String literal, allows for nested quotes.  **String literals always contain the enclosing double quotes**
  Strlit(&'t str),
  /// Alphanumeric sequence, staring with an alphabetical character or '_',
  /// and followed by arbitrary numbers of alphabetical, numeric or _.
  Alphanum(&'t str),
  /// non-alphanumeric characters, either identified as triples, doubles,
  /// singles, or unrecognized sequences (refering to length of symbol)
  Symbol(&'t str),
  /// a single byte (intended for binary data): this variant is currently
  /// not recognized by [StrTokenizer]
  Byte(u8),
  /// slice of bytes (intended for binary data): this variant is currently
  /// not recognized by [StrTokenizer]
  Bytes(&'t [u8]),
  /// newline, returned optionally
  Newline,
  /// number of consecutive whitespaces, returned optionally
  Whitespace(usize), // counts number of non-newline whitespaces
  /// usually used to represent comments, if returned optionally
  Verbatim(&'t str),
  /// Custom token type, allows for user extension.  The first string should
  /// define the type of the token while the second should carry raw text.
  /// This token type is intended to be enabled with **`lexattribute add_custom`** directives,
  /// which correspond to the function [StrTokenizer::add_custom]
  Custom(&'static str, &'t str),
  /// tokenizer error
  LexError,
}//RawToken

/// General-purpose, zero-copy lexical analyzer that produces [RawToken]s from an str.  This tokenizer uses
/// [regex](https://docs.rs/regex/latest/regex), although not for everything.  For
/// example, to allow for string literals that contain escaped quotations,
/// a direct loop is implemented.
/// The tokenizer gives the option of returning newlines, whitespaces (with
/// count) and comments as special tokens.  It recognizes mult-line
/// string literals, multi-line as well as single-line comments, and returns
/// the starting line and column positions of each token.
///
///Example:
///```ignore
///  let mut scanner = StrTokenizer::from_str("while (1) fork();//run at your own risk");
///  scanner.set_line_comment("//");
///  scanner.keep_comment=true;
///  scanner.add_single(';'); // separates ; from following symbols
///  while let Some(token) = scanner.next() {
///     println!("Token,line,column: {:?}",&token);
///  }
///```
/// this code produces output
///```ignore
///  Token,line,column: (Alphanum("while"), 1, 1)
///  Token,line,column: (Symbol("("), 1, 7)
///  Token,line,column: (Num(1), 1, 8)
///  Token,line,column: (Symbol(")"), 1, 9)
///  Token,line,column: (Alphanum("fork"), 1, 11)
///  Token,line,column: (Symbol("("), 1, 15)
///  Token,line,column: (Symbol(")"), 1, 16)
///  Token,line,column: (Symbol(";"), 1, 17)
///  Token,line,column: (Verbatim("//run at your own risk"), 1, 18)
///```

pub struct StrTokenizer<'t>
{
   decuint:Regex,
   hexnum:Regex,
   floatp:Regex,
   //strlit:Regex,
   alphan:Regex,
   nonalph:Regex,
   //custom_defined:BTreeMap<&'static str, Regex>, // added for 0.2.95
   custom_defined:Vec<(&'static str,Regex)>,
   doubles:HashSet<&'t str>,   
   singles:HashSet<char>,
   triples:HashSet<&'t str>,
   //other_syms: Vec<&'t str>,
   input: &'t str,
   position: usize,
   prev_position: usize,
   /// flag to toggle whether whitespaces should be returned as Whitespace tokens,
   /// default is false.
   pub keep_whitespace:bool,
   /// flag to toggle whether newline characters ('\n') are returned as Newline
   /// tokens. Default is false.  Note that if this flag is set to true then
   /// newline characters are treated differently from other whitespaces.
   /// For example, when parsing languages like Python, both keep_whitespace
   /// and keep_newline should be set to true.  
   pub keep_newline:bool,
   line:usize,
   line_comment:&'t str,
   ml_comment_start:&'t str,
   ml_comment_end:&'t str,
   /// flag to determine if comments are kept and returned as Verbatim tokens,
   /// default is false.
   pub keep_comment:bool,
   line_start:usize, // keep starting position of line, for column info
   src:&'t str, // source name
   /// vector of starting byte position of each line, position 0 not used.
   pub line_positions:Vec<usize>, // starting position of each line
   //pub shared_state: dyn for <ET:Default> Rc<RefCell<ET>>,
}
impl<'t> StrTokenizer<'t>
{
  /// creats a new tokenizer with defaults, *does not* set input.
  pub fn new() -> StrTokenizer<'t>
  {
    let decuint = Regex::new(r"^\d+").unwrap();
    let hexnum = Regex::new(r"^0x[\dABCDEFabcdef]+").unwrap();
    let floatp = Regex::new(r"^\d*\x2E\d+([eE][+-]?\d+)?").unwrap();
    // not allowing +- before numbers is right decision.  How to tokenize 1-2?
    //let floatp = Regex::new(r"^\d*\x2E\d+").unwrap();
    //let strlit = Regex::new(r"^\x22(?s)(.*?)\x22").unwrap();
    let alphan = Regex::new(r"^[_a-zA-Z][_\da-zA-Z]*").unwrap();
    let nonalph=Regex::new(r"^[!@#$%\^&*\?\-\+\*/\.,<>=~`';:\|\\]+").unwrap();
    let custom_defined = Vec::new(); //BTreeMap::new();
    let mut doubles = HashSet::with_capacity(16);
    let mut triples = HashSet::with_capacity(16);        
    let mut singles = HashSet::with_capacity(16);
    for c in ['(',')','[',']','{','}'] {singles.insert(c);}
    //let mut other_syms = Vec::with_capacity(32);
    let input = "";
    let position = 0;
    let prev_position = 0;
    let keep_whitespace=false;
    let keep_newline=false;
    let line = 1;
    let line_comment = "//";
    let ml_comment_start="/*";
    let ml_comment_end="*/";    
    let keep_comment=false;
    let line_start=0;
    let src = "";
    let line_positions = vec![0,0];
    StrTokenizer{decuint,hexnum,floatp,/*strlit,*/alphan,nonalph,custom_defined,doubles,singles,triples,input,position,prev_position,keep_whitespace,keep_newline,line,line_comment,ml_comment_start,ml_comment_end,keep_comment,line_start,src,line_positions}
  }// new
  /// adds a symbol of exactly length two. If the length is not two the function
  /// has no effect.  Note that these symbols override all other types except for
  /// leading whitespaces and comments markers, e.g. "//" will have precedence
  /// over "/" and "==" will have precedence over "=".
  pub fn add_double(&mut self, s:&'t str)
  {
    if s.len()==2 { self.doubles.insert(s); }
  }
  /// add a single-character symbol.  The type of the symbol overrides other
  /// types except for whitespaces, comments and double-character symbols.
  pub fn add_single(&mut self, c:char) { self.singles.insert(c);}
  /// add a 3-character symbol
  pub fn add_triple(&mut self, s:&'t str)
  { if s.len()==3 {self.triples.insert(s);} }

  /*
  /// add symbol of length greater than two. Symbols that are prefixes of
  /// other symbols should be added after the longer symbols.
  pub fn add_symbol(&mut self, s:&'t str) {
    if s.len()>2 {self.other_syms.push(s); }
  }
  */

  /// add custom defined regex, will correspond to [RawToken::Custom] variant.
  /// Custom regular expressions should not start with whitespaces and will
  /// override all others.  Multiple Custom types will be matched by the
  /// order in which they where declared in the grammar file.
  pub fn add_custom(&mut self, tkind:&'static str, reg_expr:&str)
  {
    let reg = if reg_expr.starts_with('^') || reg_expr.starts_with("(?m") {reg_expr.to_owned()} else {format!("^{}",reg_expr)};
    let re = Regex::new(&reg).expect(&format!("Error compiling custom regular expression \"{}\"",reg_expr));
    //self.custom_defined.insert(tkind,re);
    self.custom_defined.push((tkind,re));
  }//add_custom

  /// sets the input str to be parsed, resets position information.  Note:
  /// trailing whitespaces are always trimmed from the input.
  pub fn set_input(&mut self, inp:&'t str)
  {
    self.input=inp.trim_end(); self.position=0; self.line=1; self.line_start=0;
    self.line_positions = vec![0,0];
  }
  /// sets the symbol that begins a single-line comment. The default is
  /// "//".  If this is set to the empty string then no line-comments are
  /// recognized.
  pub fn set_line_comment(&mut self,cm:&'t str) {
    self.line_comment=cm;
  }
  /// sets the symbols used to delineate multi-line comments using a
  /// whitespace separated string such as "/* */".  These symbols are
  /// also the default.  Set this to the empty string to disable
  /// multi-line comments.
  pub fn set_multiline_comments(&mut self,cm:&'t str)
  {
    if cm.len()==0 {
      self.ml_comment_start=""; self.ml_comment_end=""; return;
    }
    let split:Vec<_> = cm.split_whitespace().collect();
    if split.len()!=2 {return;}
    self.ml_comment_start = split[0].trim();
    self.ml_comment_end = split[1].trim();
  }
  /// the current line that the tokenizer is on
  pub fn line(&self)->usize {self.line}
  /// the current column of the tokenizer
  pub fn column(&self)->usize {self.position-self.line_start+1}
  /// returns the current absolute byte position of the Tokenizer
  pub fn current_position(&self)-> usize {self.position}
  /// returns the previous absolute byte position of the Tokenizer
  pub fn previous_position(&self)-> usize {self.prev_position}  
  /// returns the source of the tokenizer such as URL or filename
  pub fn get_source(&self) -> &str {self.src}
  pub fn set_source<'u:'t>(&mut self, s:&'u str) {self.src=s;}

  /// gets the current line of the source input
  pub fn current_line(&self) -> &str
  {
     let startl = self.line_start;
     let max = self.input.len() - startl;
     let endl = self.input[startl..].find('\n').unwrap_or(max);
     &self.input[startl..startl+endl]
  }

  /// Retrieves the ith line of the raw input, if line index i is valid.
  /// This function is intended to be called once the tokenizer has
  /// completed its task of scanning and tokenizing the entire input.
  /// Otherwise, it may return None if the tokenizer has not yet scanned
  /// up to the line indicated.
  /// That is, it is intended for error message generation when evaluating
  /// the AST post-parsing.
  pub fn get_line(&self,i:usize) -> Option<&str>
  {
     if i<1 || i>=self.line_positions.len() {return None;}
     let startl = self.line_positions[i];
     let endl = *self.line_positions.get(i+1).unwrap_or(&self.input.len());
     Some(&self.input[startl..endl])
  }

  /// Retrieves the source string slice at the indicated indices; returns
  /// the empty string if indices are invalid.  The default implementation
  /// returns the empty string.
  pub fn get_slice(&self,start:usize,end:usize) -> &str
  {
    if start<end && end<=self.input.len() {&self.input[start..end]} else {""}
  }

  /// reset tokenizer to parse from beginning of input
  pub fn reset(&mut self) {
   self.position=0; self.prev_position=0; self.line=0; self.line_start=0;
   self.line_positions = vec![0,0];
  }

  // transform_wildcard cannot be implemented here: need to know RetTypeEnum

  /// returns next token, along with starting line and column numbers.
  /// This function will return None at end of stream or LexError along
  /// with a message printed to stderr if a tokenizer error occured.
  pub fn next_token(&mut self) -> Option<(RawToken<'t>,usize,usize)>
  {
   let mut pi = 0;
   self.prev_position = self.position;
   let clen = self.line_comment.len();
   let (cms,cme) = (self.ml_comment_start,self.ml_comment_end);
   while self.position<self.input.len()
   {
    pi = self.position;
    //if pi>=self.input.len() {return None;}
    let mut column0 = self.column();
    let mut line0 = self.line;
    let mut lstart0 = self.line_start;
    
    // skip/keep whitespaces
    let mut nextchars = self.input[pi..].chars();
    let mut c = nextchars.next().unwrap();
    //println!("NEXTCHAR is ({}), position {}",c,self.position);
    let mut i = pi;
    while c.is_whitespace() && i < self.input.len() 
    {
       if c=='\n' {
         self.line+=1; lstart0=self.line_start; self.line_start=i+1; line0=self.line;
         self.line_positions.push(i+1);
         if self.keep_newline { self.position = i+1; return Some((Newline,self.line-1,pi-lstart0+1)); }
       }
       i+= 1; 
       if i<self.input.len() {c = nextchars.next().unwrap();}
    } //c.is_whitespace
    self.position = i;
    if (i>pi && self.keep_whitespace) {
      return Some((Whitespace(i-pi),line0,self.column()-(i-pi)));}
    else if i>pi {continue;}
    //if pi>=self.input.len() {return None;}

    // look for custom-defined regular expressions
    for (ckey,cregex) in self.custom_defined.iter()
    {
       //println!("custom token type {}, regex {}",ckey,cregex);
       if let Some(mat) = cregex.find(&self.input[pi..]) {
         self.position = mat.end()+pi;
         let rawtext = &self.input[pi..self.position];
         let oldline = self.line;  let oldstart = self.line_start;
         let endls:Vec<_>=rawtext.match_indices('\n').collect();
         for (x,y) in &endls
         {
           self.line+=1;
           self.line_start += x+1;  
           self.line_positions.push(self.line_start);
         } // record new lines
         return Some((Custom(ckey,rawtext),oldline,pi-oldstart+1));
       } // match to cregex found
    }//for each custom key

    // look for line comment
    if clen>0 && pi+clen<=self.input.len() && self.line_comment==&self.input[pi..pi+clen] {
      if let Some(nlpos) = self.input[pi+clen..].find("\n") {
        self.position = nlpos+pi+clen;
        if self.keep_comment {
          return Some((Verbatim(&self.input[pi..pi+clen+nlpos]),self.line,pi-self.line_start+1));
        }
        else {continue;}
      } else { // no newline fould
        self.position = self.input.len(); 
        if self.keep_comment {return Some((Verbatim(&self.input[pi..]),self.line,pi-self.line_start+1));}
        else {break;}
      }
    }// line comment

    // look for multi-line comment (similar to string literals)
    if cms.len()>0 && pi+cms.len()<=self.input.len() && &self.input[pi..pi+cms.len()] == cms {
       if let Some(endpos) = self.input[pi+cms.len()..].find(cme) {
         self.position = pi+cms.len()+endpos+cme.len();
       } else {
         self.position = self.input.len();
         eprintln!("Tokenizer error: unclosed multi-line comment starting on line {}, column {}",line0,pi-self.line_start+1);
         return Some((LexError,line0,pi-self.line_start+1));
       }
       // find newline chars
       let mut ci = pi;
       while let Some(nli) = self.input[ci..self.position].find('\n')
       {
          self.line+=1; ci += nli+1;  self.line_start=ci;
          self.line_positions.push(ci);
          // Newline token is never returned if inside string literal
       }
       if self.keep_comment {
         return Some((Verbatim(&self.input[pi..self.position]),line0,pi-lstart0+1));
       }
       else {continue;}
    }//multi-line comments

    // look for triples
    if self.triples.len()>0 && pi+2<self.input.len() && self.triples.contains(&self.input[pi..pi+3]) {
      self.position = pi+3;
      return Some((Symbol(&self.input[pi..pi+3]),self.line,self.column()-3));
    }
    
    // look for doubles
    if pi+1<self.input.len() && self.doubles.contains(&self.input[pi..pi+2]) {
      self.position = pi+2;
      return Some((Symbol(&self.input[pi..pi+2]),self.line,self.column()-2));
    }

    // look for singles:
    //c=self.input[pi..pi+1].chars().next().unwrap();
    if self.singles.contains(&c) {
     // println!("ADDING SINGLE {}",c);
      self.position=pi+1;
      return Some((Symbol(&self.input[pi..pi+1]),self.line,self.column()-1));
    }

    // look for char literal
    if c=='\'' && pi+2<self.input.len() && &self.input[pi+2..pi+3]=="\'" {
      self.position = pi+3;
      let thechar = self.input[pi+1..pi+2].chars().next().unwrap();
      return Some((Char(thechar),self.line,self.column()-3));
    }

    // look for string literal, keep track of newlines
    if c=='\"' {
      let mut ci = pi+1;
      while ci<self.input.len()
      {
         if &self.input[ci..ci+1]=="\"" {
            self.position = ci+1;
            return Some((Strlit(&self.input[pi..self.position]),line0,pi-lstart0+1));
         }
         else if &self.input[ci..ci+1] == "\n" {
           self.line+=1; self.line_start=ci+1;
           self.line_positions.push(self.line_start);
         }
         // else need to try again!
         else if &self.input[ci..ci+1] == "\\" {ci+=1}; // extra skip
         ci+=1;
      }// while ci < input.len()
      // terminated without finding end of string
      self.position = self.input.len();
//        eprintln!("Tokenizer error: unclosed string starting on line {}, column {}",line0,pi-self.line_start+1);
        eprintln!("Tokenizer error: unclosed string, line {}, possibly starting earlier",line0);
        let errposition = if (lstart0-1)<pi {pi-lstart0+1} else {0};
        return Some((LexError,line0,pi-lstart0+1)); 
    }//strlit
    
    // look for hex, with possibility of overflow
    if let Some(mat) = self.hexnum.find(&self.input[pi..]) {
        self.position = mat.end()+pi;
        let tryparse = i64::from_str_radix(&self.input[pi+2..self.position],16);
        if let Ok(hn) = tryparse {return Some((Num(hn),self.line,pi+3-self.line_start));}
        else {return Some((BigNumber(&self.input[pi..self.position]),self.line,pi-self.line_start+1));}
        //return Some((Num(i64::from_str_radix(&self.input[pi+2..self.position],16).unwrap()),self.line,pi+3-self.line_start));        
    }//hexnum
    // look for alphanum    
    if let Some(mat) = self.alphan.find(&self.input[pi..]) {
        self.position = mat.end()+pi;  
        return Some((Alphanum(&self.input[pi..self.position]),self.line,pi-self.line_start+1));
    }//alphanums
    // floats
    if let Some(mat) = self.floatp.find(&self.input[pi..]) {
        self.position = mat.end()+pi;
        let tryparse = self.input[pi..self.position].parse::<f64>();
        if let Ok(n)=tryparse {return Some((Float(n),self.line,pi-self.line_start+1));}
        else {return Some((BigNumber(&self.input[pi..self.position]),self.line,pi-self.line_start+1));}
        //return Some((Float(self.input[pi..self.position].parse::<f64>().unwrap()),self.line,pi-self.line_start+1));
    }//floatp
    // decimal ints
    if let Some(mat) = self.decuint.find(&self.input[pi..]) {
        self.position = mat.end()+pi;
        let tryparse = self.input[pi..self.position].parse::<i64>();
        if let Ok(n)=tryparse {return Some((Num(n),self.line,pi-self.line_start+1));}
        else {return Some((BigNumber(&self.input[pi..self.position]),self.line,pi-self.line_start+1));}        
//        return Some((Num(self.input[pi..self.position].parse::<i64>().unwrap()),self.line,pi-self.line_start+1));
    }//decuint

    //check for unclosed string
    if pi<self.input.len() && &self.input[pi..pi+1]=="\"" {
        self.position = self.input.len();
        eprintln!("Tokenizer error: unclosed string starting on line {}, column {}",line0,pi-self.line_start+1);
        return Some((LexError,line0,pi-self.line_start+1));        
    }//unclosed string
      
    // at this point, what remains must be a recognized, non-alphanumeric symbol
    if let Some(mat) = self.nonalph.find(&self.input[pi..]) {
        self.position = mat.end()+pi;
        return Some((Symbol(&self.input[pi..self.position]),self.line,pi-self.line_start+1));	 
    }

    // at this point, must be error
    self.position = self.input.len();
    if pi<self.position {
      eprintln!("Tokenizer error: unrecognized symbols starting on line {}, column {}",line0,pi-self.line_start+1);
     return Some((LexError,line0,pi-self.line_start+1));
    }
    //else { return None; }
   } //while
   return None;
  }//next_token
  
}//impl StrTokenizer

impl<'t> Iterator for StrTokenizer<'t>
{
  type Item = (RawToken<'t>,usize,usize);
  fn next(&mut self) -> Option<(RawToken<'t>,usize,usize)>
  {
     if let Some(tok) = self.next_token() {Some(tok)} else {None}
  }
}//Iterator

/// Structure to hold contents of a source (such as contents of file).  A
/// [StrTokenizer] can be created from such a struct.  It reads the contents
/// of a file using [std::fs::read_to_string] and stores it locally.
pub struct LexSource<'t>
{
   pathname:&'t str,
   contents:String,
   //id:usize,
}
impl<'t> LexSource<'t>
{
  /// creates a new LexSource struct with given source path,
  /// reads contents into struct using [std::fs::read_to_string]
  pub fn new(path:&'t str) -> std::io::Result<LexSource<'t>>
  {
     let tryread=std::fs::read_to_string(path);
     //println!("READTHIS: {:?}",&tryread);
     match tryread {
       Ok(st) => {
         Ok(LexSource {
           pathname:path,
           //id:0,
           contents:st,
         })
       },
       Err(e) => {Err(e)}
     }//match
  }//new
  /// retrieves entire contents of lexsource
  pub fn get_contents(&self)->&str {&self.contents}
  /// retrieves original path (such as filename) of this source
  pub fn get_path(&self)->&str {self.pathname}  
}//impl LexSource
impl<'t> StrTokenizer<'t>
{
   /// creates a StrTokenizer from a [LexSource] structure that contains
   /// a string representing the contents of the source, and
   /// calls [StrTokenizer::set_input] to reference that string.
   /// To create a tokenizer that reads from, for example, a file is:
   ///   ```ignore
   ///   let source = LexSource::new(source_path).unwrap();
   ///   let mut tokenizer = StrTokenizer::from_source(&source);
   ///   ```
   pub fn from_source(ls:&'t LexSource<'t>) ->StrTokenizer<'t>
   {
      let mut stk = StrTokenizer::new();
      stk.set_source(ls.get_path());
      stk.set_input(ls.contents.as_str());
      let res=stk.line_positions.try_reserve(stk.input.len()/40);
      stk
   }
   /// creates a string tokenizer and sets input to give str.
   pub fn from_str(s:&'t str) -> StrTokenizer<'t>
   {
      let mut stk = StrTokenizer::new();
      stk.set_input(s);
      stk
   }   
}// impl StrTokenizer

/*
// testing an example
impl<'t> Tokenizer<'t,i64> for StrTokenizer<'t>
{
   fn nextsym(&mut self) -> Option<TerminalToken<'t,i64>>
   {
     let tokopt = self.next_token();
     if let None=tokopt {return None;}
     let tok = tokopt.unwrap();
     match tok.0 {
       Alphanum(s) => Some(TerminalToken::new(s,0,tok.1,tok.2)),
       Num(x) => Some(TerminalToken::from_raw(tok,"num",x)),
       Strlit(s) => Some(TerminalToken::from_raw(tok,"strlit",2)),
       Symbol("@") => Some(TerminalToken::from_raw(tok,"ATSYM",3)),
       Symbol(s) => Some(TerminalToken::from_raw(tok,s,3)),
       _ => Some(TerminalToken::new("EOF",0,0,0)),
     }//match
   }
   fn linenum(&self) -> usize {self.line}
   fn position(&self) -> usize {self.position}
   fn source(&self) -> &str {self.get_source()}
   fn current_line(&self) -> &str {self.input}
}
*/


/// This is a sample Lexer implementation designed to return every character in a
/// string as a separate token, and is used in small grammars for testing and
/// illustration purposes.  It is assumed that the characters read are defined as
/// terminal symbols in the grammar.  This replaces [charlexer] using
/// [Tokenizer] and [RawToken].  
pub struct charscanner<'t>
{
   contents: &'t str,
   index: usize,
   line:usize,
   keep_ws: bool,  // keep whitespace chars
   /// function to modify char returned by nextsym, can be changed.
   /// [charscanner::new] sets this function
   /// initially to `|x|{x.to_str()}`.  For example, some characters such
   /// as '{' and '}' cannot be used as terminal symbols of a grammar and must
   /// be translated into something like "LBRACE" and "RBRACE"
   pub modify: fn(&'t str)->&'t str
}
impl<'t> charscanner<'t>
{
  /// creates a charscanner with the option of keeping whitespace chars if kws=true
  pub fn new(input:&'t str, kws:bool) -> charscanner<'t>
  { charscanner {
       contents:input,
       index:0,
       line:1,
       keep_ws:kws,
       modify:|x|{x},
     }
  }
}

/// The source code of this implementation of the [Tokenizer] trait also serves
/// as an illustration of how the trait should be implemented.
impl<'t, AT:Default> Tokenizer<'t,AT> for charscanner<'t>
{
   fn nextsym(&mut self) -> Option<TerminalToken<'t,AT>>
   {
      let mut res = None;
      let mut stop = false;
      let mut i = self.index;
      while !stop && i<self.contents.len()
      {
        let c = self.contents[i..i+1].chars().next().unwrap();
        if c=='\n' {self.line+=1;}
        if c.is_whitespace() && !self.keep_ws {
            i+=1; continue;
        }
        else if c.is_whitespace() && self.keep_ws {
          stop = true;
          res = Some(TerminalToken::new(&self.contents[i..i+1],AT::default(),self.line,i));
        }
        else {
            stop=true;
            let mc = (self.modify)(&self.contents[i..i+1]);
            res =Some(TerminalToken::new(mc,AT::default(),self.line,i));
          }
      }//while
      self.index = i+1;
      return res;
   }//nextsym
   /// returns current line number starting from 1
   fn linenum(&self) -> usize { self.line }
   /// returns the index of the current char, starting from 1
   fn column(&self) -> usize { self.index }
   /// returns slice of underlying data using [std::str::Chars::as_str]
   fn current_line(&self) -> &str  {self.contents}   
}//impl Tokenizer for charscanner

// binary range search
// given position, return line number and starting position of that line
fn brsearch(ps:&[usize], p:usize) -> (usize,usize) // returns (0,0) none
{
   let mut min = 1;
   let mut max = ps.len();
   while min<max
   {
      let mid = (min+max)/2;
      if ps[mid]>p {max = mid;}
      else { // possible., p>=ps[mid]
        if mid == ps.len()-1 || p<ps[mid+1] {return (mid,ps[mid]);}
        else {min = mid+1;}
      }
   }
   (0,0)  // not found, there's no line 0
}// determine line based on position
// given (l,s), column is calculated py p-s+1.
