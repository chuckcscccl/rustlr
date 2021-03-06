//Parser generated by rustlr

#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
extern crate rustlr;
use rustlr::{RuntimeParser,RProduction,decode_action};
 // still need to put into a crate
use rustlr::{Lextoken,Lexer};
use std::io::{Write};

fn readln()-> String {
  let mut s = String::new();
  std::io::stdin().read_line(&mut s);
  s
}

struct Slex<'t> {
 split : std::str::SplitWhitespace<'t>,
}
impl<'t> Lexer<String> for Slex<'t> {
  fn nextsym(&mut self) -> Option<Lextoken<String>> {
    match self.split.next() {
     None => None,
     Some(sym) => Some(Lextoken::new(sym.trim().to_string(), sym.to_string())),
    }//match
  }//nextsym
  fn linenum(&self) -> usize {0}
  fn column(&self) -> usize {0} // not accurate, just filler
}

fn main() {
   print!("Write something in C+- : ");
   std::io::stdout().flush().unwrap();
   let input = readln();
   let mut lexer1 =  Slex{split:input.split_whitespace()};
   let mut parser1 = make_parser();
   parser1.parse( &mut lexer1);
   println!("parsing success: {}",!parser1.error_occurred());
}//main

const SYMBOLS:[&'static str;17] = ["STAT","STATLIST","EXPR","EXPRLIST","x","y","z","cin","cout",";","(",")","<<",">>","ERROR","START","EOF"];

const TABLE:[u64;74] = [4295098369,60129804288,34359803904,327681,30064967680,281526516711424,563018672898051,562980018388992,562984313225216,563010083225600,562949953880065,844480765231104,1125938562138112,1407404948324354,1407409243291650,1407443603030018,1407435013095426,1688892810854400,1688871336083456,1688862745821185,1688875630788608,1688858450984961,1688867040985088,1970384966582274,1970354901811202,1970393556516866,1970359196778498,2251816994406400,2251808404668417,2251842764275712,2251825584209920,2251821289504768,2533309150789634,2533343510528002,2533304855822338,2533334920593410,2814801307828224,2814788422991872,3096276283817986,3096263398916098,3096271988850690,3377738375757826,3377751260659714,3659221942140930,3659213352206338,3659226237108226,3940666854670336,3940671149768704,3940692624539648,3940675444473856,3940658265128961,4222171895627778,4222176190595074,4222163305693186,4503638283386880,4785083195392001,4785117554671616,4785100374605824,4785091784802304,4785096079900672,5066579645759490,5066583940726786,5066609710530562,5066618300465154,5348071803584512,5629559663886338,5629533894082562,5629568253820930,5629529599115266,5911026051121154,5911013166219266,6192488142798850,6192501027700738,6192496732733442,];

pub fn make_parser() -> RuntimeParser<String,String>
{
 let mut parser1:RuntimeParser<String,String> = RuntimeParser::new(12,23);
 let mut rule = RProduction::<String,String>::new_skeleton("start");
 rule = RProduction::<String,String>::new_skeleton("STATLIST");
 rule.Ruleaction = |parser|{ parser.stack.pop();   return String::default();};
 parser1.Rules.push(rule);
 rule = RProduction::<String,String>::new_skeleton("STATLIST");
 rule.Ruleaction = |parser|{ parser.stack.pop();  parser.stack.pop();   return String::default();};
 parser1.Rules.push(rule);
 rule = RProduction::<String,String>::new_skeleton("STAT");
 rule.Ruleaction = |parser|{ parser.stack.pop();  parser.stack.pop();  parser.stack.pop();  parser.stack.pop();  readln()};
 parser1.Rules.push(rule);
 rule = RProduction::<String,String>::new_skeleton("STAT");
 rule.Ruleaction = |parser|{ parser.stack.pop();   let mut s:String=parser.stack.pop().unwrap().value;  parser.stack.pop();  parser.stack.pop();  println!(": {}",&s); String::new()};
 parser1.Rules.push(rule);
 rule = RProduction::<String,String>::new_skeleton("EXPR");
 rule.Ruleaction = |parser|{ parser.stack.pop();  "x".to_string()};
 parser1.Rules.push(rule);
 rule = RProduction::<String,String>::new_skeleton("EXPR");
 rule.Ruleaction = |parser|{ parser.stack.pop();  "y".to_string()};
 parser1.Rules.push(rule);
 rule = RProduction::<String,String>::new_skeleton("EXPR");
 rule.Ruleaction = |parser|{ parser.stack.pop();  "z".to_string()};
 parser1.Rules.push(rule);
 rule = RProduction::<String,String>::new_skeleton("EXPR");
 rule.Ruleaction = |parser|{ parser.stack.pop();   let s:String=parser.stack.pop().unwrap().value;  parser.stack.pop();  s};
 parser1.Rules.push(rule);
 rule = RProduction::<String,String>::new_skeleton("EXPRLIST");
 rule.Ruleaction = |parser|{  let s:String=parser.stack.pop().unwrap().value;  s};
 parser1.Rules.push(rule);
 rule = RProduction::<String,String>::new_skeleton("EXPRLIST");
 rule.Ruleaction = |parser|{  let s:String=parser.stack.pop().unwrap().value;  parser.stack.pop();   let mut sl:String=parser.stack.pop().unwrap().value;   format!("{} {}",sl,s) };
 parser1.Rules.push(rule);
 rule = RProduction::<String,String>::new_skeleton("STAT");
 rule.Ruleaction = |parser|{ parser.stack.pop();  parser.stack.pop();   parser.report("invalid statement, skipping to ;"); String::new()};
 parser1.Rules.push(rule);
 rule = RProduction::<String,String>::new_skeleton("START");
 rule.Ruleaction = |parser|{ parser.stack.pop();   return String::default();};
 parser1.Rules.push(rule);
 parser1.Errsym = "ERROR";

 for i in 0..74 {
   let symi = ((TABLE[i] & 0x0000ffff00000000) >> 32) as usize;
   let sti = ((TABLE[i] & 0xffff000000000000) >> 48) as usize;
   parser1.RSM[sti].insert(SYMBOLS[symi],decode_action(TABLE[i]));
 }

 return parser1;
} //make_parser
