#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
extern crate rustlr;
use rustlr::*;
use std::fmt::Display;
use std::default::Default;
use std::collections::{HashMap,HashSet,BTreeSet};
mod ncfparser;
use crate::ncfparser::*;

pub struct tokenizer<'t>
{
  content:&'t str,
  index:usize,
}
impl<'t> Tokenizer<'t,bool> for tokenizer<'t>
{
  fn nextsym(&mut self) -> Option<TerminalToken<'t,bool>>
  {
    if self.index >=self.content.len() {None}
    else {
      self.index+=1;
      Some(TerminalToken::new(&self.content[self.index-1..self.index],true,1,self.index))
    }
  }//nextsym
  fn linenum(&self) -> usize {1}
  fn column(&self) -> usize {self.index+1} // columns should start at 1
}


fn main()
{
  let mut input = "aaabbbccc";
  let args:Vec<String> = std::env::args().collect(); // command-line args
  if args.len()>1 {input = &args[1];}
  let mut lexer1 = tokenizer{content:input,index:0};
  let mut parser1 = make_parser();
  let result = parser1.parse(&mut lexer1);
  println!("counters after parse: {:?}", parser1.exstate);  
  println!("result: {}",result);
}//main
