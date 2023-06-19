//Parser generated by rustlr for grammar calc
    
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
use std::rc::Rc;
use std::cell::RefCell;
extern crate rustlr;
use rustlr::{Tokenizer,TerminalToken,ZCParser,ZCRProduction,Stateaction,decode_action};
use rustlr::{StrTokenizer,RawToken,LexSource};
use std::collections::{HashMap,HashSet};
use crate::ll1calcast::*;
use rustlr::LBox;
use crate::calc_ast::*;

static SYMBOLS:[&'static str;18] = ["_WILDCARD_TOKEN_","+","-","*","/","(",")","POWER","Int","E","T","F","G","E1","T1","F1","START","EOF"];

static TABLE:[u64;142] = [8590393344,47245033473,38654771201,21475033088,34359869440,51539869697,42950000641,281547991154691,562958544076802,562980018913282,562967134011394,562975723945986,562954249109506,562962839044098,563022968586242,844433520525312,844476470001665,844472175165441,844467880132609,844463585361921,844459290001408,844446405165056,1125964332007425,1125917087301634,1125972921876482,1125904202399746,1125929972203520,1125908497367042,1125912792334338,1125925677236226,1407379179307008,1407447898062850,1407383474208768,1407430718980097,1407400653422594,1688858450526210,1688922875035650,1688867041181696,1688862746083328,1688875630395394,1688909990789121,1688854155558914,1970333427367936,1970376377696257,1970359196844032,1970346312007680,2251825584668672,2533326330265601,2533322036281345,2533309150265344,2533296265428992,2533283380789248,2814822782074882,2814754062598146,2814762652532738,2814766947500034,2814775537434626,2814758357565442,3096259103686656,3096233334210560,3096271988850689,3096267694800897,3096246218850304,3096276283686913,3377734080397312,3377746965561345,3377751260397569,3377742671577089,3377708310921216,3377721195560960,3659200467042306,3659247711682562,3940701213818881,3940684033818624,3940696920031233,3940658264342528,3940671148982272,4222133240856578,4222128945889282,4222197665366018,4222150420725762,4503633987239936,4503621102403584,4503608217763840,4503646873518081,4503651167240193,4785091784736770,4785104669638658,4785087489769474,4785100374671362,4785078899834882,4785083194802178,4785147619311618,5066562466545666,5066622596087810,5066579646414850,5066575351447554,5066553876611074,5066566761512962,5066558171578370,5348028853125122,5348050327961602,5348037443059714,5348033148092418,5348097572601858,5348041738027010,5629508124868608,5629572548722690,5629555370360833,5629503829966848,5629525304082434,5910983101579264,5911047525433346,5911030347137025,5910978806677504,5911000280793090,6192453782929410,6192462373453824,6192466668552192,6192509618880513,6192522502406146,6192475257765890,6192458077896706,6473950234476546,6473984595656705,6473933054607362,6473928759640066,6473997479116802,6473937350164480,6473941645262848,6755472455696386,6755425211056130,7036900187701250,7036947432341506,7318375164674050,7318357984804866,7318422409314306,7318353689837570,7599850141450242,7599897386090498,7599832961581058,7599828666613762,];


fn _semaction_rule_0_<'lt>(parser:&mut ZCParser<RetTypeEnum<'lt>,()>) -> i32 {
let mut f = if let RetTypeEnum::Enumvariant_17(_x_17)=parser.popstack().value { _x_17 } else {<Continuation<'lt,i32>>::default()}; let mut x = if let RetTypeEnum::Enumvariant_0(_x_0)=parser.popstack().value { _x_0 } else {<i32>::default()};  f.apply(x) }

fn _semaction_rule_1_<'lt>(parser:&mut ZCParser<RetTypeEnum<'lt>,()>) -> Continuation<'lt,i32> {
 Continuation::default() }

fn _semaction_rule_2_<'lt>(parser:&mut ZCParser<RetTypeEnum<'lt>,()>) -> Continuation<'lt,i32> {
let mut f = if let RetTypeEnum::Enumvariant_17(_x_17)=parser.popstack().value { _x_17 } else {<Continuation<'lt,i32>>::default()}; let mut y = if let RetTypeEnum::Enumvariant_0(_x_0)=parser.popstack().value { _x_0 } else {<i32>::default()}; let mut _item0_ = if let RetTypeEnum::Enumvariant_13(_x_13)=parser.popstack().value { _x_13 } else {<()>::default()};  Continuation::make(move |x|f.apply(x+y)) }

fn _semaction_rule_3_<'lt>(parser:&mut ZCParser<RetTypeEnum<'lt>,()>) -> Continuation<'lt,i32> {
let mut f = if let RetTypeEnum::Enumvariant_17(_x_17)=parser.popstack().value { _x_17 } else {<Continuation<'lt,i32>>::default()}; let mut y = if let RetTypeEnum::Enumvariant_0(_x_0)=parser.popstack().value { _x_0 } else {<i32>::default()}; let mut _item0_ = if let RetTypeEnum::Enumvariant_13(_x_13)=parser.popstack().value { _x_13 } else {<()>::default()};  Continuation::make(move |x|f.apply(x-y)) }

fn _semaction_rule_4_<'lt>(parser:&mut ZCParser<RetTypeEnum<'lt>,()>) -> i32 {
let mut g = if let RetTypeEnum::Enumvariant_17(_x_17)=parser.popstack().value { _x_17 } else {<Continuation<'lt,i32>>::default()}; let mut z = if let RetTypeEnum::Enumvariant_0(_x_0)=parser.popstack().value { _x_0 } else {<i32>::default()};  g.apply(z) }

fn _semaction_rule_5_<'lt>(parser:&mut ZCParser<RetTypeEnum<'lt>,()>) -> Continuation<'lt,i32> {
 Continuation::default() }

fn _semaction_rule_6_<'lt>(parser:&mut ZCParser<RetTypeEnum<'lt>,()>) -> Continuation<'lt,i32> {
let mut g = if let RetTypeEnum::Enumvariant_17(_x_17)=parser.popstack().value { _x_17 } else {<Continuation<'lt,i32>>::default()}; let mut z = if let RetTypeEnum::Enumvariant_0(_x_0)=parser.popstack().value { _x_0 } else {<i32>::default()}; let mut _item0_ = if let RetTypeEnum::Enumvariant_13(_x_13)=parser.popstack().value { _x_13 } else {<()>::default()};  Continuation::make(move |y|g.apply(y*z)) }

fn _semaction_rule_7_<'lt>(parser:&mut ZCParser<RetTypeEnum<'lt>,()>) -> Continuation<'lt,i32> {
let mut g = if let RetTypeEnum::Enumvariant_17(_x_17)=parser.popstack().value { _x_17 } else {<Continuation<'lt,i32>>::default()}; let mut z = if let RetTypeEnum::Enumvariant_0(_x_0)=parser.popstack().value { _x_0 } else {<i32>::default()}; let mut _item0_ = if let RetTypeEnum::Enumvariant_13(_x_13)=parser.popstack().value { _x_13 } else {<()>::default()};  Continuation::make(move |y|g.apply(y/z)) }

fn _semaction_rule_8_<'lt>(parser:&mut ZCParser<RetTypeEnum<'lt>,()>) -> i32 {
let mut f = if let RetTypeEnum::Enumvariant_17(_x_17)=parser.popstack().value { _x_17 } else {<Continuation<'lt,i32>>::default()}; let mut y = if let RetTypeEnum::Enumvariant_0(_x_0)=parser.popstack().value { _x_0 } else {<i32>::default()};  f.apply(y) }

fn _semaction_rule_9_<'lt>(parser:&mut ZCParser<RetTypeEnum<'lt>,()>) -> Continuation<'lt,i32> {
 Continuation::default() }

fn _semaction_rule_10_<'lt>(parser:&mut ZCParser<RetTypeEnum<'lt>,()>) -> Continuation<'lt,i32> {
let mut z = if let RetTypeEnum::Enumvariant_0(_x_0)=parser.popstack().value { _x_0 } else {<i32>::default()}; let mut _item0_ = if let RetTypeEnum::Enumvariant_13(_x_13)=parser.popstack().value { _x_13 } else {<()>::default()};  Continuation::make(move |y:i32|y.pow(z as u32)) }

fn _semaction_rule_11_<'lt>(parser:&mut ZCParser<RetTypeEnum<'lt>,()>) -> i32 {
let mut x = if let RetTypeEnum::Enumvariant_0(_x_0)=parser.popstack().value { _x_0 } else {<i32>::default()};  x }

fn _semaction_rule_12_<'lt>(parser:&mut ZCParser<RetTypeEnum<'lt>,()>) -> i32 {
let mut x = if let RetTypeEnum::Enumvariant_0(_x_0)=parser.popstack().value { _x_0 } else {<i32>::default()}; let mut _item0_ = if let RetTypeEnum::Enumvariant_13(_x_13)=parser.popstack().value { _x_13 } else {<()>::default()};  -1*x }

fn _semaction_rule_13_<'lt>(parser:&mut ZCParser<RetTypeEnum<'lt>,()>) -> i32 {
let mut _item2_ = if let RetTypeEnum::Enumvariant_13(_x_13)=parser.popstack().value { _x_13 } else {<()>::default()}; let mut x = if let RetTypeEnum::Enumvariant_0(_x_0)=parser.popstack().value { _x_0 } else {<i32>::default()}; let mut _item0_ = if let RetTypeEnum::Enumvariant_13(_x_13)=parser.popstack().value { _x_13 } else {<()>::default()};  x }

fn _semaction_rule_14_<'lt>(parser:&mut ZCParser<RetTypeEnum<'lt>,()>) -> () {
let mut _item0_ = if let RetTypeEnum::Enumvariant_0(_x_0)=parser.popstack().value { _x_0 } else {<i32>::default()}; <()>::default()}

pub fn make_parser<'lt>() -> ZCParser<RetTypeEnum<'lt>,()>
{
 let mut parser1:ZCParser<RetTypeEnum<'lt>,()> = ZCParser::new(15,28);
 let mut rule = ZCRProduction::<RetTypeEnum<'lt>,()>::new_skeleton("start");
 rule = ZCRProduction::<RetTypeEnum<'lt>,()>::new_skeleton("E");
 rule.Ruleaction = |parser|{  RetTypeEnum::Enumvariant_0(_semaction_rule_0_(parser)) };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<RetTypeEnum<'lt>,()>::new_skeleton("E1");
 rule.Ruleaction = |parser|{  RetTypeEnum::Enumvariant_17(_semaction_rule_1_(parser)) };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<RetTypeEnum<'lt>,()>::new_skeleton("E1");
 rule.Ruleaction = |parser|{  RetTypeEnum::Enumvariant_17(_semaction_rule_2_(parser)) };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<RetTypeEnum<'lt>,()>::new_skeleton("E1");
 rule.Ruleaction = |parser|{  RetTypeEnum::Enumvariant_17(_semaction_rule_3_(parser)) };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<RetTypeEnum<'lt>,()>::new_skeleton("T");
 rule.Ruleaction = |parser|{  RetTypeEnum::Enumvariant_0(_semaction_rule_4_(parser)) };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<RetTypeEnum<'lt>,()>::new_skeleton("T1");
 rule.Ruleaction = |parser|{  RetTypeEnum::Enumvariant_17(_semaction_rule_5_(parser)) };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<RetTypeEnum<'lt>,()>::new_skeleton("T1");
 rule.Ruleaction = |parser|{  RetTypeEnum::Enumvariant_17(_semaction_rule_6_(parser)) };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<RetTypeEnum<'lt>,()>::new_skeleton("T1");
 rule.Ruleaction = |parser|{  RetTypeEnum::Enumvariant_17(_semaction_rule_7_(parser)) };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<RetTypeEnum<'lt>,()>::new_skeleton("F");
 rule.Ruleaction = |parser|{  RetTypeEnum::Enumvariant_0(_semaction_rule_8_(parser)) };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<RetTypeEnum<'lt>,()>::new_skeleton("F1");
 rule.Ruleaction = |parser|{  RetTypeEnum::Enumvariant_17(_semaction_rule_9_(parser)) };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<RetTypeEnum<'lt>,()>::new_skeleton("F1");
 rule.Ruleaction = |parser|{  RetTypeEnum::Enumvariant_17(_semaction_rule_10_(parser)) };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<RetTypeEnum<'lt>,()>::new_skeleton("G");
 rule.Ruleaction = |parser|{  RetTypeEnum::Enumvariant_0(_semaction_rule_11_(parser)) };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<RetTypeEnum<'lt>,()>::new_skeleton("G");
 rule.Ruleaction = |parser|{  RetTypeEnum::Enumvariant_0(_semaction_rule_12_(parser)) };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<RetTypeEnum<'lt>,()>::new_skeleton("G");
 rule.Ruleaction = |parser|{  RetTypeEnum::Enumvariant_0(_semaction_rule_13_(parser)) };
 parser1.Rules.push(rule);
 rule = ZCRProduction::<RetTypeEnum<'lt>,()>::new_skeleton("START");
 rule.Ruleaction = |parser|{  RetTypeEnum::Enumvariant_13(_semaction_rule_14_(parser)) };
 parser1.Rules.push(rule);
 parser1.Errsym = "";

 for i in 0..142 {
   let symi = ((TABLE[i] & 0x0000ffff00000000) >> 32) as usize;
   let sti = ((TABLE[i] & 0xffff000000000000) >> 48) as usize;
   parser1.RSM[sti].insert(SYMBOLS[symi],decode_action(TABLE[i]));
 }

 for s in SYMBOLS { parser1.Symset.insert(s); }

 load_extras(&mut parser1);
 return parser1;
} //make_parser

pub fn parse_with<'lt>(parser:&mut ZCParser<RetTypeEnum<'lt>,()>, lexer:&mut calclexer<'lt>) -> Result<i32,i32>
{
  lexer.shared_state = Rc::clone(&parser.shared_state);
  if let RetTypeEnum::Enumvariant_0(_xres_) = parser.parse(lexer) {
     if !parser.error_occurred() {Ok(_xres_)} else {Err(_xres_)}
  } else { Err(<i32>::default())}
}//parse_with public function

pub fn parse_train_with<'lt>(parser:&mut ZCParser<RetTypeEnum<'lt>,()>, lexer:&mut calclexer<'lt>, parserpath:&str) -> Result<i32,i32>
{
  lexer.shared_state = Rc::clone(&parser.shared_state);
  if let RetTypeEnum::Enumvariant_0(_xres_) = parser.parse_train(lexer,parserpath) {
     if !parser.error_occurred() {Ok(_xres_)} else {Err(_xres_)}
  } else { Err(<i32>::default())}
}//parse_train_with public function

//Enum for return values 
pub enum RetTypeEnum<'lt> {
  Enumvariant_0(i32),
  Enumvariant_2((usize,usize)),
  Enumvariant_13(()),
  Enumvariant_17(Continuation<'lt,i32>),
}
impl<'lt> Default for RetTypeEnum<'lt> { fn default()->Self {RetTypeEnum::Enumvariant_0(<i32>::default())} }


// Lexical Scanner using RawToken and StrTokenizer
pub struct calclexer<'lt> {
   stk: StrTokenizer<'lt>,
   keywords: HashSet<&'static str>,
   lexnames: HashMap<&'static str,&'static str>,
   shared_state: Rc<RefCell<()>>,
}
impl<'lt> calclexer<'lt> 
{
  pub fn from_str(s:&'lt str) -> calclexer<'lt>  {
    Self::new(StrTokenizer::from_str(s))
  }
  pub fn from_source(s:&'lt LexSource<'lt>) -> calclexer<'lt>  {
    Self::new(StrTokenizer::from_source(s))
  }
  pub fn new(mut stk:StrTokenizer<'lt>) -> calclexer<'lt> {
    let mut lexnames = HashMap::with_capacity(64);
    let mut keywords = HashSet::with_capacity(64);
    let shared_state = Rc::new(RefCell::new(<()>::default()));
    for kw in ["_WILDCARD_TOKEN_",] {keywords.insert(kw);}
    for c in ['+','-','*','/','(',')',] {stk.add_single(c);}
    for d in ["**",] {stk.add_double(d);}
    for d in [] {stk.add_triple(d);}
    for (k,v) in [(r"**","POWER"),] {lexnames.insert(k,v);}
    calclexer {stk,keywords,lexnames,shared_state,}
  }
}
impl<'lt> Tokenizer<'lt,RetTypeEnum<'lt>> for calclexer<'lt>
{
   fn nextsym(&mut self) -> Option<TerminalToken<'lt,RetTypeEnum<'lt>>> {
    let tokopt = self.stk.next_token();
    if let None = tokopt {return None;}
    let token = tokopt.unwrap();
    match token.0 {
      RawToken::Alphanum(sym) if self.keywords.contains(sym) => {
        let truesym = self.lexnames.get(sym).unwrap_or(&sym);
        Some(TerminalToken::from_raw(token,truesym,<RetTypeEnum<'lt>>::default()))
      },
      RawToken::Num(_tt) => Some(TerminalToken::from_raw(token,"Int",RetTypeEnum::Enumvariant_0(_tt as i32))),
      RawToken::Symbol(s) if self.lexnames.contains_key(s) => {
        let tname = self.lexnames.get(s).unwrap();
        Some(TerminalToken::from_raw(token,tname,<RetTypeEnum<'lt>>::default()))
      },
      RawToken::Symbol(s) => Some(TerminalToken::from_raw(token,s,<RetTypeEnum<'lt>>::default())),
      RawToken::Alphanum(s) => Some(TerminalToken::from_raw(token,s,<RetTypeEnum<'lt>>::default())),
      _ => { let _rrodb=token.0.to_staticstr(); Some(TerminalToken::from_raw(token,_rrodb,<RetTypeEnum<'lt>>::default())) },
    }
  }
   fn linenum(&self) -> usize {self.stk.line()}
   fn column(&self) -> usize {self.stk.column()}
   fn position(&self) -> usize {self.stk.current_position()}
   fn current_line(&self) -> &str {self.stk.current_line()}
   fn get_line(&self,i:usize) -> Option<&str> {self.stk.get_line(i)}
   fn get_slice(&self,s:usize,l:usize) -> &str {self.stk.get_slice(s,l)}
   fn transform_wildcard(&self,t:TerminalToken<'lt,RetTypeEnum<'lt>>) -> TerminalToken<'lt,RetTypeEnum<'lt>> { TerminalToken::new(t.sym,RetTypeEnum::Enumvariant_2((self.stk.previous_position(),self.stk.current_position())),t.line,t.column) }
}//impl Tokenizer

fn load_extras<'lt>(parser:&mut ZCParser<RetTypeEnum<'lt>,()>)
{
}//end of load_extras: don't change this line as it affects augmentation