//Parser generated by rustlr for grammar brackets

#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(unused_assignments)]
#![allow(dead_code)]
#![allow(irrefutable_let_patterns)]
#![allow(unreachable_patterns)]
extern crate rustlr;
use rustlr::{Tokenizer,TerminalToken,ZCParser,ZCRProduction,Stateaction,decode_action};
use rustlr::{StrTokenizer,RawToken,LexSource};
use std::collections::{HashMap,HashSet};
//anything on a ! line is injected verbatim into the generated parser

fn main() {
  let argv:Vec<String> = std::env::args().collect(); // command-line args
  let mut parser1 = make_parser();
  let mut lexer1 = bracketslexer::from_str(&argv[1]);
  let result = parser1.parse(&mut lexer1);
  if !parser1.error_occurred() {
    println!("parsed successfully with result {:?}",&result);
  }
  else {println!("parsing failed; partial result is {:?}",&result);}
}//main

const SYMBOLS:[&'static str;12] = ["E","S","WS","(",")","[","]","LBRACE","RBRACE","Whitespace","START","EOF"];

const TABLE:[u64;92] = [4295098369,8590000129,21475164162,47244967938,12885229570,30065098754,38655033346,281509336645634,281505041678338,281522221547522,281500746711042,281492156776450,281487861809154,281496451743746,281513631612928,562997198061571,562980018585600,562971428519936,562949953748993,562962838781952,844454995296258,844450700328962,844459290263554,844472175165442,844437815427074,844463585230850,844442110394370,844446405361666,1125908496842753,1125921382006786,1125938561875970,1125904202334209,1125929971941378,1125912792072194,1125925676974082,1407392063684610,1407400653619202,1407422128455682,1407387768717314,1407396358651906,1407409243553794,1407404948586498,1688884220329986,1688879925362690,1688854155821057,1688888515297282,1688862745493506,1688858450264065,1688871335428098,1970333426974721,1970337722204162,1970354902073346,1970342017171458,1970329132597249,1970346312138754,1970363492007938,2251799814012929,2251812699045888,2251821288783872,2251825584209920,2251829878849536,2533296265494528,2533287675756544,2533274790723585,2533309150920704,2533304855560192,2814779832270848,2814766947827712,2814771242205184,2814762652467200,2814749767434241,3096237628784642,3096250513686530,3096246218719234,3096254808653826,3096241923751938,3096271988523010,3096259103621122,3377734080397314,3377712605560834,3377721195495426,3377725490462722,3377746965299202,3377716900528130,3377729785430018,3659221941878786,3659191877107714,3659200467042306,3659196172075010,3659204762009602,3659209056976898,3659187582140418,];

pub fn make_parser() -> ZCParser<(u32,u32,u32),(u32,u32,u32)>
{
 let mut parser1:ZCParser<(u32,u32,u32),(u32,u32,u32)> = ZCParser::new(8,14);
 let mut rule = ZCRProduction::<(u32,u32,u32),(u32,u32,u32)>::new_skeleton("start");
 rule = ZCRProduction::<(u32,u32,u32),(u32,u32,u32)>::new_skeleton("E");
 rule.Ruleaction = |parser|{ let mut _item2_ = parser.popstack(); let mut _item1_ = parser.popstack(); let mut _item0_ = parser.popstack(); 
  if let ((a,b,c),)=(_item1_.value,) { (a+1,b,c)}  else {parser.bad_pattern("((a,b,c),)")} };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<(u32,u32,u32),(u32,u32,u32)>::new_skeleton("E");
 rule.Ruleaction = |parser|{ let mut _item2_ = parser.popstack(); let mut _item1_ = parser.popstack(); let mut _item0_ = parser.popstack(); 
  if let ((a,b,c),)=(_item1_.value,) { (a,b+1,c)}  else {parser.bad_pattern("((a,b,c),)")} };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<(u32,u32,u32),(u32,u32,u32)>::new_skeleton("E");
 rule.Ruleaction = |parser|{ let mut _item2_ = parser.popstack(); let mut _item1_ = parser.popstack(); let mut _item0_ = parser.popstack(); 
  if let ((a,b,c),)=(_item1_.value,) { (a,b,c+1)}  else {parser.bad_pattern("((a,b,c),)")} };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<(u32,u32,u32),(u32,u32,u32)>::new_skeleton("S");
 rule.Ruleaction = |parser|{ let mut _item0_ = parser.popstack();  (0,0,0) };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<(u32,u32,u32),(u32,u32,u32)>::new_skeleton("S");
 rule.Ruleaction = |parser|{ let mut _item1_ = parser.popstack(); let mut _item0_ = parser.popstack(); 
  if let ((p,q,r),(a,b,c),)=(_item1_.value,_item0_.value,) { (a+p,b+q,c+r)}  else {parser.bad_pattern("((p,q,r),(a,b,c),)")} };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<(u32,u32,u32),(u32,u32,u32)>::new_skeleton("WS");
 rule.Ruleaction = |parser|{ <(u32,u32,u32)>::default()};
 parser1.Rules.push(rule);
 rule = ZCRProduction::<(u32,u32,u32),(u32,u32,u32)>::new_skeleton("WS");
 rule.Ruleaction = |parser|{ let mut _item1_ = parser.popstack(); let mut _item0_ = parser.popstack(); <(u32,u32,u32)>::default()};
 parser1.Rules.push(rule);
 rule = ZCRProduction::<(u32,u32,u32),(u32,u32,u32)>::new_skeleton("START");
 rule.Ruleaction = |parser|{ let mut _item0_ = parser.popstack(); <(u32,u32,u32)>::default()};
 parser1.Rules.push(rule);
 parser1.Errsym = "";
 parser1.resynch.insert("Whitespace");

 for i in 0..92 {
   let symi = ((TABLE[i] & 0x0000ffff00000000) >> 32) as usize;
   let sti = ((TABLE[i] & 0xffff000000000000) >> 48) as usize;
   parser1.RSM[sti].insert(SYMBOLS[symi],decode_action(TABLE[i]));
 }

 for s in SYMBOLS { parser1.Symset.insert(s); }

 load_extras(&mut parser1);
 return parser1;
} //make_parser


// Lexical Scanner using RawToken and StrTokenizer
pub struct bracketslexer<'t> {
   stk: StrTokenizer<'t>,
   keywords: HashSet<&'static str>,
}
impl<'t> bracketslexer<'t> 
{
  pub fn from_str(s:&'t str) -> bracketslexer<'t>  {
    Self::new(StrTokenizer::from_str(s))
  }
  pub fn from_source(s:&'t LexSource<'t>) -> bracketslexer<'t>  {
    Self::new(StrTokenizer::from_source(s))
  }
  pub fn new(mut stk:StrTokenizer<'t>) -> bracketslexer<'t> {
    let mut keywords = HashSet::with_capacity(16);
    for kw in [] {keywords.insert(kw);}
    for c in ['(',')','[',']','{','}',] {stk.add_single(c);}
    for d in [] {stk.add_double(d);}
    stk.keep_whitespace = true;
    bracketslexer {stk,keywords}
  }
}
impl<'t> Tokenizer<'t,(u32,u32,u32)> for bracketslexer<'t>
{
   fn nextsym(&mut self) -> Option<TerminalToken<'t,(u32,u32,u32)>> {
    let tokopt = self.stk.next_token();
    if let None = tokopt {return None;}
    let token = tokopt.unwrap();
    match token.0 {
      RawToken::Whitespace(_) => Some(TerminalToken::from_raw(token,"Whitespace",(0,0,0))),
      RawToken::Symbol(r"{") => Some(TerminalToken::from_raw(token,"LBRACE",<(u32,u32,u32)>::default())),
      RawToken::Symbol(r"}") => Some(TerminalToken::from_raw(token,"RBRACE",<(u32,u32,u32)>::default())),
      RawToken::Symbol(s) => Some(TerminalToken::from_raw(token,s,<(u32,u32,u32)>::default())),
      RawToken::Alphanum(s) => Some(TerminalToken::from_raw(token,s,<(u32,u32,u32)>::default())),
      _ => Some(TerminalToken::from_raw(token,"<LexicalError>",<(u32,u32,u32)>::default())),
    }
  }
   fn linenum(&self) -> usize {self.stk.line()}
   fn column(&self) -> usize {self.stk.column()}
   fn position(&self) -> usize {self.stk.current_position()}
}//impl Tokenizer

fn load_extras(parser:&mut ZCParser<(u32,u32,u32),(u32,u32,u32)>)
{
}//end of load_extras: don't change this line as it affects augmentation
