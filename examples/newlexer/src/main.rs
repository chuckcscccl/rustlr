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
   nonalph:Regex,
   singletons:HashSet<char>,
   symbols: Vec<&'t str>,
   input: &'t str,
   position: usize,
   pub keep_whitespace:bool,
   pub keep_newline:bool,
   line:usize,
   line_comment:&'t str,
   pub keep_comments:bool,
   line_start:usize, // keep starting position of line, for column info
}
impl<'t> StrTokenizer<'t>
{
  pub fn new() -> StrTokenizer<'t>
  {
    let decuint = Regex::new(r"^\d+").unwrap();
    let hexnum = Regex::new(r"^0x[\dABCDEFabcdef]+").unwrap();
    let floatp = Regex::new(r"^\d*\x2E\d+").unwrap();
    let strlit = Regex::new(r"^\x22(?s)(.*)\x22").unwrap();
    let alphan = Regex::new(r"^[_a-zA-Z][_\da-zA-Z]*").unwrap();
    let nonalph=Regex::new(r"^[!@#$%\^&*\?\-\+\*/\.,<>=~`';:\|\\]").unwrap();
    let mut singletons = HashSet::with_capacity(16);
    for c in ['(',')','[',']','{','}'] {singletons.insert(c);}
    let mut symbols = Vec::with_capacity(32);
    let input = "";
    let position = 0;
    let keep_whitespace=false;
    let keep_newline=false;
    let line = 1;
    let line_comment = "//";
    let keep_comments=false;
    let line_start=0;
    StrTokenizer{decuint,hexnum,floatp,strlit,alphan,nonalph,singletons,symbols,input,position,keep_whitespace,keep_newline,line,line_comment,keep_comments,line_start}
  }// new
  pub fn add_single(&mut self, c:char) { self.singletons.insert(c);}
  pub fn add_symbol(&mut self, s:&'t str) {self.symbols.push(s); }
  pub fn set_input(&mut self, inp:&'t str)
  {
    self.input=inp; self.position=0; self.line=1; self.line_start=0;
  }
  pub fn set_line_comment(&mut self,cm:&'t str) {
    if cm.len()>0 {self.line_comment=cm;}
  }
  pub fn line(&self)->usize {self.line}
  pub fn column(&self)->usize {self.position-self.line_start+1}


  /// returns next token, along with starting line and column numbers
  fn next_token(&mut self) -> Option<(RawToken<'t>,usize,usize)>
  {
    let mut pi = self.position; // should be set to start of token
    if pi>=self.input.len() {return None;}

    let mut column0 = self.column();
    let mut line0 = self.line;
    let mut lstart0 = self.line_start;
    
    // skip/keep whitespaces
    let mut nextchars = self.input[pi..].chars();
    let mut c = nextchars.next().unwrap();
    let mut i = pi;

    while c.is_whitespace() && i < self.input.len() 
    {
       if c=='\n' {
         self.line+=1; lstart0=self.line_start; self.line_start=i+1; line0=self.line;
         if self.keep_newline { self.position = i+1; return Some((Newline,self.line-1,pi-lstart0+1)); }
       }
       i+= 1; 
       if i<self.input.len() {c = nextchars.next().unwrap();}
    }
    if (i>pi && self.keep_whitespace) {
      self.position = i;
      return Some((Whitespace(i-pi),line0,self.column()-(i-pi)));}
    else {pi=i;}
    c=self.input[pi..pi+1].chars().next().unwrap();

    // look for singleton:
    if self.singletons.contains(&c) {
      self.position=pi+1;
      return Some((Symbol(&self.input[pi..pi+1]),self.line(),self.column()-1));
    }
    // look for string literal, keep track of newlines
    if let Some(mat) = self.strlit.find(&self.input[pi..]) {
       self.position = mat.end()+pi;
       // find newline chars
       let mut ci = pi;
       while let Some(nli) = self.input[ci..self.position].find('\n')
       {
          self.line+=1; ci += nli+1;  self.line_start=ci;
          // Newline token is never returned if inside string literal
       }
       return Some((Strlit(&self.input[pi..self.position]),line0,pi-lstart0+1));
    }//string lits are matched first, so other's aren't part of strings
    // look for line comment
    
    let clen = self.line_comment.len();
    if clen>0 && pi+clen<=self.input.len() && self.line_comment==&self.input[pi..pi+clen] {
      if let Some(nlpos) = self.input[pi+clen..].find("\n") {
        self.position = nlpos+pi+clen;
        if self.keep_comments {
          return Some((Verbatim(&self.input[pi..pi+clen+nlpos]),self.line,pi-self.line_start+1));
        }
        else {pi=self.position;}
      } else { // no newline fould
        self.position = self.input.len(); 
        if self.keep_comments {return Some((Verbatim(&self.input[pi..]),self.line,pi-self.line_start+1));}
        else {pi=self.position;}
      }
    }// line comment
    
    // hex
    if let Some(mat) = self.hexnum.find(&self.input[pi..]) {
        self.position = mat.end()+pi;
        return Some((Hex(u64::from_str_radix(&self.input[pi+2..self.position],16).unwrap()),self.line,pi+3-self.line_start));        
    }//hexnum
    // look for alphanum    
    if let Some(mat) = self.alphan.find(&self.input[pi..]) {
        self.position = mat.end()+pi;  
        return Some((Alphanum(&self.input[pi..self.position]),self.line,pi-self.line_start+1));
    }//alphanums
    // decimal ints
    if let Some(mat) = self.decuint.find(&self.input[pi..]) {
        self.position = mat.end()+pi;  
        return Some((Num(self.input[pi..self.position].parse::<i64>().unwrap()),self.line,pi-self.line_start+1));
    }//decuint
    // floats
    if let Some(mat) = self.floatp.find(&self.input[pi..]) {
        self.position = mat.end()+pi;
        return Some((Float(self.input[pi..self.position].parse::<f64>().unwrap()),self.line,pi-self.line_start+1));
    }//floatp
    // at this point, what remains must be a symbol, match longest symbols first
    for sym in &self.symbols
    {
      let symlen = sym.len();
      if pi+symlen<=self.input.len() && &&self.input[pi..pi+symlen] == sym {
         self.position = pi+symlen;
	 return Some((Symbol(&self.input[pi..self.position]),self.line,pi-self.line_start+1));	 
      }
    }
    self.position = self.input.len();
    if pi<self.position {return Some((Verbatim(&self.input[pi..]),self.line,pi-self.line_start+1));}
    else {return None;}
  }//next  
}//impl StrTokenizer

impl<'t> Iterator for StrTokenizer<'t>
{
  type Item = RawToken<'t>;
  fn next(&mut self) -> Option<RawToken<'t>>
  {
     if let Some((tok,_,_)) = self.next_token() {Some(tok)} else {None}
  }
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
  stk.keep_comments=true;
  stk.keep_newline=true;
  stk.keep_whitespace=true;
  stk.set_input("{while (1==3.5-.7101*0x7E6) fork(x_y); //don't run
printf(\"%d hello
 there!
hello!\");
x = x==      y;
return 0;
}");
  let mut coln = stk.column();
  let mut linen = stk.line();
  while let Some(token) = stk.next_token()
  {
     println!("Token: {:?}",&token);
     coln = stk.column();
     linen = stk.line();
  }
}//main


//// overloading test

struct St(i32);
trait STS
{
  fn f(&self, i:i32);
}
impl STS for St
{
   fn f(&self,i:i32) {println!("i32 {}",i==self.0);}
}
trait STT // need different module for overloading, can't be in same scope
{
   fn f1(&self,i:&str);
}
impl STT for St
{
  fn f1(&self, i:&str) {println!("str, {} and {}",i,self.0);}
}

impl<'t> Tokenizer<i64> for StrTokenizer<'t>
{
   fn nextsym(&mut self) -> Option<LexToken<'t,i64>>
   {
     match self.next() {
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

// to do, recognize doublesyms before single sims. using hashset
// doublesyms
// single syms
// other syms
