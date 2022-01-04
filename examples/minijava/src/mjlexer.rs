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
