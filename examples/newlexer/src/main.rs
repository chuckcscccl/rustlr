use regex::Regex;
use std::collections::{HashSet};
use crate::RawToken::*;

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


#[derive(Debug)]
pub enum RawToken<'t>
{
  Num(i64),
  Hex(u64),
  Float(f64),
  Strlit(&'t str),
  Alphanum(&'t str),
  Symbol(&'t str),
  Newline,
  Verbatim(&'t str),
  Whitespace(usize), // counts number of non-newline whitespaces
}//RawToken

pub struct StrTokenizer<'t>
{
   decuint:Regex,
   hexnum:Regex,
   floatp:Regex,
   strlit:Regex,
   alphan:Regex,
   //nonalph:Regex,
   singletons:HashSet<char>,
   symbols: Vec<&'static str>,
   input: &'t str,
   position: usize,
   pub keep_whitespace:bool,
   pub keep_newline:bool,
   pub line:usize,
   line_comment:&'static str,
   pub keep_comments:bool,
}
impl<'t> StrTokenizer<'t>
{
  pub fn new() -> StrTokenizer<'t>
  {
    let decuint = Regex::new(r"\d+").unwrap();
    let hexnum = Regex::new(r"0x[\dA-Fa-f]+").unwrap();
    let floatp = Regex::new(r"\d*.\d+").unwrap();
    let strlit = Regex::new(r"\x22(?s)(.*)\x22").unwrap();
    let alphan = Regex::new(r"[_a-zA-Z][_\da-zA-Z]*").unwrap();
    //let nonalph=Regex::new(r"[!@#$%^&*?-+*/.,<>?=~`';:\\|]").unwrap();
    let mut singletons = HashSet::with_capacity(16);
    for c in ['(',')','[',']','{','}'] {singletons.insert(c);}
    let mut symbols = Vec::with_capacity(32);
    let input = "";
    let position = 0;
    let keep_whitespace=false;
    let keep_newline=false;
    let line = 1;
    let line_comment = "#";
    let keep_comments=false;
    StrTokenizer{decuint,hexnum,floatp,strlit,alphan,singletons,symbols,input,position,keep_whitespace,keep_newline,line,line_comment,keep_comments}
  }// new
  pub fn add_single(&mut self, c:char) { self.singletons.insert(c);}
  pub fn add_symbol(&mut self, s:&'static str) {self.symbols.push(s); }
  pub fn set_input(&mut self, inp:&'t str) {self.input=inp; self.position=0; self.line=1;}
  pub fn set_line_comment(&mut self,cm:&'static str) {
    if cm.len()>0 {self.line_comment=cm;}
  }
  pub fn next_token(&mut self) -> Option<RawToken<'t>>
  {
    let mut pi = self.position;
    if pi>=self.input.len() {return None;}

    // skip/keep whitespaces
    let mut nextchars = self.input[pi..].chars();
    let mut c = nextchars.next().unwrap();
    let mut i = pi;
    
    while c.is_whitespace() && c!='\n' && i <self.input.len() 
    {
       i+= 1; 
       if i<self.input.len() {c = nextchars.next().unwrap();}
    }
    if (i>pi && self.keep_whitespace) {
      self.position = i;
      return Some(Whitespace(i-pi));}
    else {pi=i;}
    let nextchar=self.input[pi..pi+1].chars().next().unwrap();
    // look fo newline
    if nextchar=='\n' {
      self.line+=1; pi+=1;
      if self.keep_newline {
        self.position=pi; return Some(Newline);
      }
    }//newline
    // look for singleton:
    if self.singletons.contains(&nextchar) {
      self.position=pi+1;
      return Some(Symbol(&self.input[pi..pi+1]));
    }
    let mut minpos = self.input.len();
    // look for alphanum
    if let Some(mat) = self.strlit.find(&self.input[pi..]) {
      minpos = std::cmp::min(minpos,mat.start()+pi);
      if minpos==pi {
        self.position = mat.end()+pi;  
        return Some(Strlit(&self.input[pi..self.position]));
      }      
    }//string lits are matched first, so other's aren't part of strings
    if let Some(mat) = self.alphan.find(&self.input[pi..]) {
      minpos = std::cmp::min(minpos,mat.start()+pi);
      if minpos==pi {
        self.position = mat.end()+pi;  
        return Some(Alphanum(&self.input[pi..self.position]));
      }
    }//alphanums
    if let Some(mat) = self.decuint.find(&self.input[pi..]) {
      minpos = std::cmp::min(minpos,mat.start()+pi);
      if minpos==pi {
        self.position = mat.end()+pi;  
        return Some(Num(self.input[pi..self.position].parse::<i64>().unwrap()));
      }
    }//decuint
    if let Some(mat) = self.floatp.find(&self.input[pi..]) {
      minpos = std::cmp::min(minpos,mat.start()+pi);
      if minpos==pi {
        self.position = mat.end()+pi;
	println!("HEERRE: {}",&self.input[pi..self.position]);
        return Some(Float(self.input[pi..self.position].parse::<f64>().unwrap()));
      }
    }//floatp
    if let Some(mat) = self.hexnum.find(&self.input[pi..]) {
      minpos = std::cmp::min(minpos,mat.start()+pi);
      if minpos==pi {
        self.position = mat.end()+pi;  
        return Some(Hex(self.input[pi..self.position].parse::<u64>().unwrap()));
      }
    }//hexnum
    //at this point, minpos is still > pi, so what's left must be a symbol
    for sym in &self.symbols
    {
      println!("LOOKING FOR sym {}, pi {}, minpos {}",sym,pi,minpos);
      if let Some(0) = self.input[pi..minpos].find(sym) {
         self.position = pi+sym.len();
	 return Some(Symbol(&self.input[pi..self.position]));	 
      }
    }
    self.position=minpos;
    return Some(Verbatim(&self.input[pi..minpos]));
  }//next  
}//impl StrTokenizer

impl<'t> Iterator for StrTokenizer<'t>
{
  type Item = RawToken<'t>;
  fn next(&mut self) -> Option<RawToken<'t>>
  {  self.next_token() }
}//Iterator

//////////////////////////
fn main()
{
  let re = Regex::new(r"\d{4}-\d{2}-\d{2}\sxyz$").unwrap();
  //println!("{}",re.is_match("abc2014-01-01 xyz"));

  // Regex for common token types

  // decimal unsigned int
  let decuint = Regex::new(r"^\d+").unwrap();
  // hexadecimal number
  let hexnum = Regex::new(r"^0x[\dA-Fa-f]+").unwrap();
  // string literal
  let strlit = Regex::new(r"^\x22(?s)(.*)\x22").unwrap();
  // alpha-numeric with _:
  let alphan = Regex::new(r"^[_a-zA-Z][_\da-zA-Z]*").unwrap();
  let nonalph=Regex::new(r"[!@#$%\^&\*-\+/.,<>?~`';:\\|]+").unwrap();  
  let floatp = Regex::new(r"\d*.\d+").unwrap();
    
  println!("{}",decuint.is_match("1023"));
  println!("{}",hexnum.is_match("0x22"));
  println!("{}",hexnum.is_match("0x7Ef6"));
  println!("{}",strlit.is_match("\"abc\""));
  println!("{}",strlit.is_match("\"hi \"ok\" th
ere\""));
  println!("alphanum: {}",alphan.is_match("_a8b_c_9"));
  println!("alphanum: {}",alphan.is_match("0_a8b_c_9"));
  println!("sym {}",nonalph.is_match("*&^"));
  println!("sym {}",nonalph.is_match("?!@$"));
  println!("sym {}",nonalph.is_match("avbc?_!@$"));
  println!("float {}",floatp.is_match("2.3"));

  // testing for overloading:
  let st = St(8);
  st.f(8);
  //st.f("abc");
  let mut stk = StrTokenizer::new();
  for x in ["==","=","+",";",",",":","!","*","/","-","<=","<"] {
    stk.add_symbol(x);
  }
  stk.set_input("while (1==3-2) fork(x_y);");
  while let Some(token) = stk.next_token()
  {  
     println!("Token: {:?}",&token);
  }
  
}//main

//// overloading test

struct St(i32);
impl St
{
   fn f(&self,i:i32) {println!("i32 {}",i==self.0);}
}
trait STT
{
   fn f(&self,i:&str);
}
impl STT for St
{
  fn f(&self, i:&str) {println!("str, {} and {}",i,self.0);}
}


impl<'t> Tokenizer<i64> for StrTokenizer<'t>
{
   fn nextsym(&mut self) -> Option<LexToken<'t,i64>>
   {
     match self.next_token() {
       Some(Alphanum(s)) => Some(LexToken::new(s,0)),
       Some(Num(x)) => Some(LexToken::new("num",x)),
       Some(Strlit(s)) => Some(LexToken::new("strlit",2)),
       Some(Symbol("@")) => Some(LexToken::new("ATSYM",3)),       
       Some(Symbol(s)) => Some(LexToken::new(s,3)),
       _ => Some(LexToken::new("EOF",0)),
     }//match
   }
   fn current_line(&self) -> &str {self.input}
   fn linenum(&self) -> usize {self.line}
}
