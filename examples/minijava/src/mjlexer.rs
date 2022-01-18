#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
use rustlr::{Lextoken,Lexer};
use basic_lexer::*;  // needs pub so can be used by main
use crate::absyntax::*;
use crate::absyntax::Construct::*;
use crate::absyntax::Expr::*;
use crate::absyntax::Stat::*;
use std::collections::{HashSet};

//////////////////////////////////// lexical scanner

pub struct Mjscanner(File_tokenizer);
impl Mjscanner {
  pub fn new(file:&str)->Mjscanner
  {
     let mut ft = File_tokenizer::new(file);
     ft.add_keywords("class public static void main String extends return length new this boolean int if else while");
     Mjscanner(ft)
     // other defaults for File_tokenizer suffice
  }
}


impl Lexer<Construct> for Mjscanner
{
   fn nextsym(&mut self) -> Option<Lextoken<Construct>>
   {
      let tok = self.0.next();
      if let None = tok {return None;}
      let token = tok.unwrap();
      let retval = 
      match token {
        Token::Symbol(s) if &s=="{" => { Lextoken::new(String::from("LBR"),Exp(Nothing)) },
        Token::Symbol(s) if &s=="}" => { Lextoken::new(String::from("RBR"),Exp(Nothing)) },
        Token::Symbol(s) if &s=="%" => { Lextoken::new(String::from("MOD"),Exp(Nothing)) },
        Token::Symbol(s) if &s=="." => { Lextoken::new(String::from("DOT"),Exp(Nothing)) },			
        Token::Symbol(s) if &s=="||" => { Lextoken::new(String::from("OROR"),Exp(Nothing)) },        
        Token::Keyword(c) => { Lextoken::new(c,Exp(Nothing)) },
        Token::Symbol(d) => {Lextoken::new(d,Exp(Nothing)) },
        Token::Alphanum(x) => { Lextoken::new(String::from("ID"),Id(x)) },
        Token::Integer(n) =>  {Lextoken::new(String::from("INTEGER"),Exp(Int(n as i32))) },
        Token::Stringlit(s) => {
          let slen = s.len();
          let s2 = s[1..slen-1].replace("\\n","\n");
          Lextoken::new(String::from("STRING"),Exp(Strlit(s2)))
        },
        _ => Lextoken::new(format!("symbol {:?}",&token),Exp(Nothing)),
      };//match
      Some(retval)
   }
   fn linenum(&self) -> usize {self.0.line_number()}
   fn column(&self) -> usize {self.0.column_number()}
}//impl Lexer<Construct> for exprscanner


/////////////////// ZC version
use rustlr::{RawToken,Tokenizer,TerminalToken,StrTokenizer,LexSource};

// keywords are no longer distinguished from alphanums by StrTokenizer
pub struct Mjlexer<'t>
{
  stk:StrTokenizer<'t>,
  keywords:HashSet<&'static str>,
}
impl<'t> Mjlexer<'t>
{
  pub fn new(s:StrTokenizer<'t>) -> Mjlexer<'t>
  {
    let mut kwh = HashSet::with_capacity(16);
    for kw in ["class", "public", "static", "void", "main", "String", "extends", "return", "length", "new", "this", "boolean", "int", "if", "else", "while"]
    { kwh.insert(kw);}
    Mjlexer {
      stk: s,
      keywords : kwh,
    }
  }//new
}//impl Mjlexer
impl<'t> Tokenizer<'t,Construct> for Mjlexer<'t>
{
   fn linenum(&self) -> usize {self.stk.line()}
   fn column(&self) -> usize {self.stk.column()}
   fn position(&self) -> usize {self.stk.current_position()}
   fn nextsym(&mut self) -> Option<TerminalToken<'t,Construct>>
   {
      let tokopt = self.stk.next_token();
      if let None = tokopt { return None; }
      let tok = tokopt.unwrap();
      let tt =  match tok.0 {
        RawToken::Symbol("{") => TerminalToken::from_raw(tok,"LBR",Exp(Nothing)),
        RawToken::Symbol("}") => TerminalToken::from_raw(tok,"RBR",Exp(Nothing)),
        RawToken::Symbol("%") => TerminalToken::from_raw(tok,"MOD",Exp(Nothing)),
        RawToken::Symbol(".") => TerminalToken::from_raw(tok,"DOT",Exp(Nothing)),
        RawToken::Symbol("||") => TerminalToken::from_raw(tok,"OROR",Exp(Nothing)),
        RawToken::Symbol(s) => TerminalToken::from_raw(tok,s,Exp(Nothing)),
        RawToken::Alphanum(a) if self.keywords.contains(a) => {
          TerminalToken::from_raw(tok,a,Exp(Nothing))
        },
        RawToken::Alphanum(a) => TerminalToken::from_raw(tok,"ID",Id(a.to_owned())),
        RawToken::Num(n) => TerminalToken::from_raw(tok,"INTEGER",Exp(Int(n as i32))),
        RawToken::Strlit(s) => {
          let slen = s.len()-1;
          let s2 = s[1..slen].replace("\\n","\n"); //makes owned string
          TerminalToken::from_raw(tok,"STRING",Exp(Strlit(s2)))
        },
        _ => TerminalToken::from_raw(tok,"<<UNRECOGNIZED>>",Exp(Nothing)),
      };//match
      Some(tt)
   }//nextsym
}
