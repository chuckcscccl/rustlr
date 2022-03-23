#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(non_upper_case_globals)]
extern crate rustlr;
use rustlr::{LBox,TerminalToken,Tokenizer,RawToken,StrTokenizer,LexSource};
use std::collections::{HashSet};

use crate::Expr;
use crate::Expr::*;
use rustlr::RawToken::*;

// add everything that's in the extras section of grammar

pub struct calc4lexer<'t>
{
   stk: StrTokenizer<'t>,
   keywords: HashSet<&'static str>,
}
impl<'t> calc4lexer<'t>
{
  pub fn from_str(s:&'t str) -> calc4lexer<'t>  {
    Self::new(StrTokenizer::from_str(s))
  }
  pub fn from_source(s:&'t LexSource<'t>) -> calc4lexer<'t>  {
    Self::new(StrTokenizer::from_source(s))
  }
  
  pub fn new(mut stk:StrTokenizer<'t>) -> calc4lexer<'t>  {
    let mut keywords = HashSet::with_capacity(10);
    for kw in ["let", "in"] {keywords.insert(kw);}
    for x in ['+','-','*','/','=',';'] {stk.add_single(x)}
    stk.set_line_comment("#");
    calc4lexer { stk,keywords }
  }//new

/*
  ////// function to be partially completed by user:
  fn convert_<'t>(&self,rw:RawToken<'t>,ln:usize,col:usize)-> TerminalToken<'t>
  {
    match rw {

      /*
       ...
       Complete the cases for tokens carrying values where appropriate:
          Num
          Float
          Char
          Strlit
       as well as special cases not covered by defaults
       ...
      */
      RawToken::Num(n) => TerminalToken::new("int",Val(n),ln,col),
      //
      RawToken::Alphanum(kw) if self.keywords.contains(kw) => TerminalToken::new(kw,<Expr>::default(),ln,col),
      RawToken::Symbol("||") =>TerminalToken::new("OROR",<Expr>::default(),ln,col),
      RawToken::Symbol(sym) => TerminalToken::new(sym,<Expr>::default(),ln,col),
      _ => TerminalToken::new("lexerror",<Expr>::default(),ln,col),
    }//match
  }//convert
*/
}//impl calc4lexer

impl<'t> Tokenizer<'t,Expr<'t>> for calc4lexer<'t>
{
   fn nextsym(&mut self) -> Option<TerminalToken<'t,Expr<'t>>>
   {
      let tokopt = self.stk.next_token();
      if let None = tokopt {return None;}
      let token = tokopt.unwrap();
      match token.0 {
       RawToken::Num(n) => Some(TerminalToken::from_raw(token,"int",Val(n))),
       RawToken::Alphanum("let")  => Some(TerminalToken::from_raw(token,"let",Nothing)),
       RawToken::Alphanum(s) if s=="in" => Some(TerminalToken::from_raw(token,"in",Nothing)),       
       RawToken::Alphanum(a) => Some(TerminalToken::from_raw(token,"var",Var(a))),
       RawToken::Symbol(s) => Some(TerminalToken::from_raw(token,s,Nothing)),
       _ => Some(TerminalToken::from_raw(token,"<<LexicalError>>",Nothing)),
      }//match      
   }
   fn linenum(&self) -> usize {self.stk.line()}
   fn column(&self) -> usize {self.stk.column()}
   fn position(&self) -> usize {self.stk.current_position()}
}//impl Tokenizer

/*
On the grammar side,

lexname OROR ||   // hash map || --> OROR
lexname LBRACE {
lexval int Num(n) Val(n)    // vector of triples (string,string,string)

lex val -name of terminal-  -form of Rawtoken-  -form of TT.value-
generate from above 
*/



