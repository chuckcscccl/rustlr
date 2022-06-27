//Parser generated by rustlr for grammar re
    
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(unused_assignments)]
#![allow(dead_code)]
#![allow(unreachable_patterns)]
#![allow(irrefutable_let_patterns)]
extern crate rustlr;
use rustlr::{Tokenizer,TerminalToken,ZCParser,ZCRProduction,Stateaction,decode_action};
use rustlr::{StrTokenizer,RawToken,LexSource};
use std::collections::{HashMap,HashSet};
use rustlr::LBox;
use crate::re_ast;
use crate::re_ast::*;

static SYMBOLS:[&'static str;9] = ["_WILDCARD_TOKEN_","ID","VAL","#","E","NID0","N#2","START","EOF"];

static TABLE:[u64;17] = [4295098368,17179934721,21475033089,281509336449027,562954248388610,562958543355906,562962838323202,844433520197634,844450700263425,844437815427072,844429225361408,1125904201875458,1125908496842754,1125912791810050,1407383473946624,1688858450395138,1970359196975106,];


fn _semaction_rule_0_(parser:&mut ZCParser<RetTypeEnum,()>) -> Vec<LBox<String>> {
let mut _item0_ = if let RetTypeEnum::Enumvariant_2(_x_2)=parser.popstack().value { _x_2 } else {<String>::default()};  vec![parser.lbx(0,_item0_)] }

fn _semaction_rule_1_(parser:&mut ZCParser<RetTypeEnum,()>) -> Vec<LBox<String>> {
let mut _item1_ = if let RetTypeEnum::Enumvariant_2(_x_2)=parser.popstack().value { _x_2 } else {<String>::default()}; let mut _item0_ = if let RetTypeEnum::Enumvariant_5(_x_5)=parser.popstack().value { _x_5 } else {<Vec<LBox<String>>>::default()};  _item0_.push(parser.lbx(1,_item1_)); _item0_ }

fn _semaction_rule_2_(parser:&mut ZCParser<RetTypeEnum,()>) -> () {
<()>::default()}

fn _semaction_rule_3_(parser:&mut ZCParser<RetTypeEnum,()>) -> () {
let mut _item0_ = if let RetTypeEnum::Enumvariant_1(_x_1)=parser.popstack().value { _x_1 } else {<()>::default()}; <()>::default()}

fn _semaction_rule_4_(parser:&mut ZCParser<RetTypeEnum,()>) -> E {
let mut _item2_ = if let RetTypeEnum::Enumvariant_3(_x_3)=parser.popstack().value { _x_3 } else {<i64>::default()}; let mut _item1_ = if let RetTypeEnum::Enumvariant_1(_x_1)=parser.popstack().value { _x_1 } else {<()>::default()}; let mut _item0_ = if let RetTypeEnum::Enumvariant_5(_x_5)=parser.popstack().value { _x_5 } else {<Vec<LBox<String>>>::default()}; E::SeqNum(parser.lbx(0,_item0_),_item2_) }

fn _semaction_rule_5_(parser:&mut ZCParser<RetTypeEnum,()>) -> E {
let mut _item0_ = if let RetTypeEnum::Enumvariant_0(_x_0)=parser.popstack().value { _x_0 } else {<E>::default()}; <E>::default()}

pub fn make_parser() -> ZCParser<RetTypeEnum,()>
{
 let mut parser1:ZCParser<RetTypeEnum,()> = ZCParser::new(6,8);
 let mut rule = ZCRProduction::<RetTypeEnum,()>::new_skeleton("start");
 rule = ZCRProduction::<RetTypeEnum,()>::new_skeleton("NID0");
 rule.Ruleaction = |parser|{  RetTypeEnum::Enumvariant_5(_semaction_rule_0_(parser)) };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<RetTypeEnum,()>::new_skeleton("NID0");
 rule.Ruleaction = |parser|{  RetTypeEnum::Enumvariant_5(_semaction_rule_1_(parser)) };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<RetTypeEnum,()>::new_skeleton("N#2");
 rule.Ruleaction = |parser|{  RetTypeEnum::Enumvariant_1(_semaction_rule_2_(parser)) };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<RetTypeEnum,()>::new_skeleton("N#2");
 rule.Ruleaction = |parser|{  RetTypeEnum::Enumvariant_1(_semaction_rule_3_(parser)) };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<RetTypeEnum,()>::new_skeleton("E");
 rule.Ruleaction = |parser|{  RetTypeEnum::Enumvariant_0(_semaction_rule_4_(parser)) };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<RetTypeEnum,()>::new_skeleton("START");
 rule.Ruleaction = |parser|{  RetTypeEnum::Enumvariant_0(_semaction_rule_5_(parser)) };
 parser1.Rules.push(rule);
 parser1.Errsym = "";

 for i in 0..17 {
   let symi = ((TABLE[i] & 0x0000ffff00000000) >> 32) as usize;
   let sti = ((TABLE[i] & 0xffff000000000000) >> 48) as usize;
   parser1.RSM[sti].insert(SYMBOLS[symi],decode_action(TABLE[i]));
 }

 for s in SYMBOLS { parser1.Symset.insert(s); }

 load_extras(&mut parser1);
 return parser1;
} //make_parser

pub fn parse_with<'t>(parser:&mut ZCParser<RetTypeEnum,()>, lexer:&mut relexer<'t>) -> Result<E,E>
{
  if let RetTypeEnum::Enumvariant_0(_xres_) = parser.parse(lexer) {
     if !parser.error_occurred() {Ok(_xres_)} else {Err(_xres_)}
  } else { Err(<E>::default())}
}//parse_with public function

pub fn parse_train_with<'t>(parser:&mut ZCParser<RetTypeEnum,()>, lexer:&mut relexer<'t>, parserpath:&str) -> Result<E,E>
{
  if let RetTypeEnum::Enumvariant_0(_xres_) = parser.parse_train(lexer,parserpath) {
     if !parser.error_occurred() {Ok(_xres_)} else {Err(_xres_)}
  } else { Err(<E>::default())}
}//parse_train_with public function

//Enum for return values 
pub enum RetTypeEnum {
  Enumvariant_0(E),
  Enumvariant_1(()),
  Enumvariant_3(i64),
  Enumvariant_2(String),
  Enumvariant_5(Vec<LBox<String>>),
}
impl Default for RetTypeEnum { fn default()->Self {RetTypeEnum::Enumvariant_0(<E>::default())} }


// Lexical Scanner using RawToken and StrTokenizer
pub struct relexer<'t> {
   stk: StrTokenizer<'t>,
   keywords: HashSet<&'static str>,
   lexnames: HashMap<&'static str,&'static str>,
}
impl<'t> relexer<'t> 
{
  pub fn from_str(s:&'t str) -> relexer<'t>  {
    Self::new(StrTokenizer::from_str(s))
  }
  pub fn from_source(s:&'t LexSource<'t>) -> relexer<'t>  {
    Self::new(StrTokenizer::from_source(s))
  }
  pub fn new(mut stk:StrTokenizer<'t>) -> relexer<'t> {
    let mut lexnames = HashMap::with_capacity(64);
    let mut keywords = HashSet::with_capacity(64);
    for kw in ["_WILDCARD_TOKEN_",] {keywords.insert(kw);}
    for c in ['#',] {stk.add_single(c);}
    for d in [] {stk.add_double(d);}
    for d in [] {stk.add_triple(d);}
    for (k,v) in [] {lexnames.insert(k,v);}
    relexer {stk,keywords,lexnames}
  }
}
impl<'t> Tokenizer<'t,RetTypeEnum> for relexer<'t>
{
   fn nextsym(&mut self) -> Option<TerminalToken<'t,RetTypeEnum>> {
    let tokopt = self.stk.next_token();
    if let None = tokopt {return None;}
    let token = tokopt.unwrap();
    match token.0 {
      RawToken::Alphanum(sym) if self.keywords.contains(sym) => {
        let truesym = self.lexnames.get(sym).unwrap_or(&sym);
        Some(TerminalToken::from_raw(token,truesym,<RetTypeEnum>::default()))
      },
      RawToken::Num(n) => Some(TerminalToken::from_raw(token,"VAL",RetTypeEnum::Enumvariant_3(n))),
      RawToken::Alphanum(s) => Some(TerminalToken::from_raw(token,"ID",RetTypeEnum::Enumvariant_2(s.to_owned()))),
      RawToken::Symbol(s) if self.lexnames.contains_key(s) => {
        let tname = self.lexnames.get(s).unwrap();
        Some(TerminalToken::from_raw(token,tname,<RetTypeEnum>::default()))
      },
      RawToken::Symbol(s) => Some(TerminalToken::from_raw(token,s,<RetTypeEnum>::default())),
      RawToken::Alphanum(s) => Some(TerminalToken::from_raw(token,s,<RetTypeEnum>::default())),
      _ => Some(TerminalToken::from_raw(token,"<LexicalError>",<RetTypeEnum>::default())),
    }
  }
   fn linenum(&self) -> usize {self.stk.line()}
   fn column(&self) -> usize {self.stk.column()}
   fn position(&self) -> usize {self.stk.current_position()}
   fn current_line(&self) -> &str {self.stk.current_line()}
   fn get_line(&self,i:usize) -> Option<&str> {self.stk.get_line(i)}
}//impl Tokenizer

fn load_extras(parser:&mut ZCParser<RetTypeEnum,()>)
{
}//end of load_extras: don't change this line as it affects augmentation