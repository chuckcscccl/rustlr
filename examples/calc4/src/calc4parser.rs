//Parser generated by rustlr for grammar calc4

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
use crate::exprtrees::*; /* ! lines are injected verbatim into parser */
use crate::exprtrees::Expr::*;
use rustlr::{LBox,makelbox};

const SYMBOLS:[&'static str;16] = ["E","ES","+","-","*","/","(",")","=",";","let","in","int","var","START","EOF"];

const TABLE:[u64;177] = [55834902528,51540066304,42949869568,4295032833,25770196992,262145,12885032960,281526516776960,281517926580224,281474977234945,281500746907648,281539401220099,281530811613184,281487861743616,562992903290880,562949954011137,563001493487616,562975723618304,563005788323840,562962838454272,844467880001536,844437815164928,844476470198272,844450700328960,844480765034496,844424930787329,1125917087432704,1125908497760256,1125921382531072,1125912792662016,1125938562334720,1407396358455298,1407409243357186,1407383473553410,1407392063488002,1407387768520706,1407404948389890,1407413538324482,1407422128259074,1688862745296896,1688901400330240,1688892810133504,1688875630460928,1688905695166464,1688849861312513,1970346311811074,1970342016843778,1970337721876482,1970359196712962,1970363491680258,1970372081614850,1970333426909186,1970354901745666,2251838469505024,2251808404602880,2251812699504640,2251821289373696,2251816994275328,2533283380789250,2533287675756546,2533291970985984,2533304855625730,2533322035494914,2533313445560322,2533309150593026,2533296266084352,2814758358024192,2814784128024576,2814766947696640,2814762652925952,2814771242795008,3096224745062401,3096276283883520,3096267693686784,3096250514014208,3096280578719744,3096237628850176,3377751260725250,3377755555692546,3377712606019586,3377742670790658,3377764145627138,3377725490921474,3659230532141056,3659217647108096,3659174698549249,3659226237304832,3659200467435520,3659187582271488,3940675444146176,3940662558982144,3940692623818752,3940701214015488,3940705508851712,3940649675325441,4222167600529408,4222176190726144,4222150420856832,4222124652101633,4222137535692800,4222180485562368,4503629693648896,4503608218288128,4503612513189888,4503616807960576,4503621103058944,4785100374540290,4785087489638402,4785126144344066,4785130439311362,4785117554409474,4785139029245954,5066592530661376,5066549582364673,5066605415694336,5066575350988800,5066601120858112,5066562465824768,5348063212601346,5348058917634050,5348033147830274,5348037442797570,5348046032732162,5348041737764866,5348071802535938,5348054622666754,5629508124475394,5629529599311874,5629516714409986,5629521009377282,5629512419442690,5629538189246466,5629533894279170,5629546779181058,5911013165891586,5910987396087810,5911021755826178,5910991691513856,5911008870924290,5911004575956994,5910983101120514,5910995986612224,6192483847569410,6192470963322880,6192462372732930,6192458077765634,6192496732471298,6192479552602114,6192466668224512,6192488142536706,6473941644738562,6473937349771266,6473933054803970,6473945939705858,6473971709509634,6473958824607746,6473963119575042,6473954529640450,6755408031973376,6755416621645824,6755446687334400,6755412326875136,6755420916744192,7036900187963392,7036925957832704,7036930252668928,7036887302799360,7036917367635968,7036874419470337,7318388049313794,7318357985394688,7318370870165504,7318383754346498,7318362280296448,7318366575067136,7318379459379202,7318396639248386,];

pub fn make_parser<'src_lt>() -> ZCParser<Expr<'src_lt>,i64>
{
 let mut parser1:ZCParser<Expr<'src_lt>,i64> = ZCParser::new(12,27);
 let mut rule = ZCRProduction::<Expr<'src_lt>,i64>::new_skeleton("start");
 rule = ZCRProduction::<Expr<'src_lt>,i64>::new_skeleton("E");
 rule.Ruleaction = |parser|{ let mut m = parser.popstack();  m.value };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<Expr<'src_lt>,i64>::new_skeleton("E");
 rule.Ruleaction = |parser|{ let mut s = parser.popstack(); 
  if let (Var(v),)=(&mut s.value,) {  s.value }  else {parser.bad_pattern("(Var(v),)")} };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<Expr<'src_lt>,i64>::new_skeleton("E");
 rule.Ruleaction = |parser|{ let mut b = parser.popstack(); let mut _item4_ = parser.popstack(); let mut e = parser.popstack(); let mut _item2_ = parser.popstack(); let mut _item1_ = parser.popstack(); let mut _item0_ = parser.popstack(); 
  if let (Var(x),)=(_item1_.value,) { Letexp(x,e.lbox(),b.lbox())}  else {parser.bad_pattern("(Var(x),)")} };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<Expr<'src_lt>,i64>::new_skeleton("E");
 rule.Ruleaction = |parser|{ let mut e2 = parser.popstack(); let mut _item1_ = parser.popstack(); let mut e1 = parser.popstack();  Plus(e1.lbox(), e2.lbox()) };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<Expr<'src_lt>,i64>::new_skeleton("E");
 rule.Ruleaction = |parser|{ let mut e2 = parser.popstack(); let mut _item1_ = parser.popstack(); let mut e1 = parser.popstack();  Minus(e1.lbox(), parser.lbx(2,e2.value))};
 parser1.Rules.push(rule);
 rule = ZCRProduction::<Expr<'src_lt>,i64>::new_skeleton("E");
 rule.Ruleaction = |parser|{ let mut e2 = parser.popstack(); let mut _item1_ = parser.popstack(); let mut e1 = parser.popstack();  Divide(e1.lbox(), e2.lbox())};
 parser1.Rules.push(rule);
 rule = ZCRProduction::<Expr<'src_lt>,i64>::new_skeleton("E");
 rule.Ruleaction = |parser|{ let mut e2 = parser.popstack(); let mut _item1_ = parser.popstack(); let mut e1 = parser.popstack();  Times(e1.lbox(), e2.lbox())};
 parser1.Rules.push(rule);
 rule = ZCRProduction::<Expr<'src_lt>,i64>::new_skeleton("E");
 rule.Ruleaction = |parser|{ let mut e = parser.popstack(); let mut _item0_ = parser.popstack();  Negative(e.lbox()) };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<Expr<'src_lt>,i64>::new_skeleton("E");
 rule.Ruleaction = |parser|{ let mut _item2_ = parser.popstack(); let mut e = parser.popstack(); let mut _item0_ = parser.popstack();  e.value };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<Expr<'src_lt>,i64>::new_skeleton("ES");
 rule.Ruleaction = |parser|{ let mut _item1_ = parser.popstack(); let mut n = parser.popstack();  Seq(vec![n.lbox()]) };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<Expr<'src_lt>,i64>::new_skeleton("ES");
 rule.Ruleaction = |parser|{ let mut _item2_ = parser.popstack(); let mut e = parser.popstack(); let mut _item0_ = parser.popstack(); 
  if let (Seq(mut v),)=(_item0_.value,) { 
   v.push(e.lbox());
   Seq(v)
   }  else {parser.bad_pattern("(Seq(mut v),)")} };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<Expr<'src_lt>,i64>::new_skeleton("START");
 rule.Ruleaction = |parser|{ let mut _item0_ = parser.popstack(); <Expr<'src_lt>>::default()};
 parser1.Rules.push(rule);
 parser1.Errsym = "";
 parser1.resynch.insert(";");

 for i in 0..177 {
   let symi = ((TABLE[i] & 0x0000ffff00000000) >> 32) as usize;
   let sti = ((TABLE[i] & 0xffff000000000000) >> 48) as usize;
   parser1.RSM[sti].insert(SYMBOLS[symi],decode_action(TABLE[i]));
 }

 for s in SYMBOLS { parser1.Symset.insert(s); }

 load_extras(&mut parser1);
 return parser1;
} //make_parser


// Lexical Scanner using RawToken and StrTokenizer
pub struct calc4lexer<'t> {
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
  pub fn new(mut stk:StrTokenizer<'t>) -> calc4lexer<'t> {
    let mut keywords = HashSet::with_capacity(16);
    for kw in ["let","in",] {keywords.insert(kw);}
    for c in ['+','-','*','/','(',')','=',';',] {stk.add_single(c);}
    for d in [] {stk.add_double(d);}
    stk.set_line_comment("#");
    calc4lexer {stk,keywords}
  }
}
impl<'src_lt> Tokenizer<'src_lt,Expr<'src_lt>> for calc4lexer<'src_lt>
{
   fn nextsym(&mut self) -> Option<TerminalToken<'src_lt,Expr<'src_lt>>> {
    let tokopt = self.stk.next_token();
    if let None = tokopt {return None;}
    let token = tokopt.unwrap();
    match token.0 {
      RawToken::Alphanum(sym) if self.keywords.contains(sym) => Some(TerminalToken::from_raw(token,sym,<Expr<'src_lt>>::default())),
      RawToken::Num(n) => Some(TerminalToken::from_raw(token,"int",Val(n))),
      RawToken::Alphanum(x) => Some(TerminalToken::from_raw(token,"var",Var(x))),
      RawToken::Symbol(s) => Some(TerminalToken::from_raw(token,s,<Expr<'src_lt>>::default())),
      RawToken::Alphanum(s) => Some(TerminalToken::from_raw(token,s,<Expr<'src_lt>>::default())),
      _ => Some(TerminalToken::from_raw(token,"<LexicalError>",<Expr<'src_lt>>::default())),
    }
  }
   fn linenum(&self) -> usize {self.stk.line()}
   fn column(&self) -> usize {self.stk.column()}
   fn position(&self) -> usize {self.stk.current_position()}
}//impl Tokenizer

fn load_extras<'src_lt>(parser:&mut ZCParser<Expr<'src_lt>,i64>)
{
}//end of load_extras: don't change this line as it affects augmentation
