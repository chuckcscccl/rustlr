#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
#![allow(unused_doc_comments)]
#![allow(unused_imports)]
use std::fmt::Display;
use std::default::Default;
use std::collections::{HashMap,HashSet,BTreeSet};
use std::io::{self,Read,Write,BufReader,BufRead};
use std::cell::{RefCell,Ref,RefMut};
use std::hash::{Hash,Hasher};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::mem;
use crate::bunch::*;
use crate::bunch::Stateaction::*;

#[derive(Clone)]
pub struct RProduction<AT:Default,ET:Default>  // runtime rep of grammar rule
{
  pub lhs: &'static str, // left-hand side nonterminal of rule
  pub Ruleaction : fn(&mut RuntimeParser<AT,ET>) -> AT, //parser as arg
}
impl<AT:Default,ET:Default> RProduction<AT,ET>
{
  pub fn new_skeleton(lh:&'static str) -> RProduction<AT,ET>
  {
     RProduction {
       lhs : lh,
       Ruleaction : |p|{AT::default()},
     }
  }
}//impl RProduction


pub struct RuntimeParser<AT:Default,ET:Default>  
{
  pub RSM : Vec<HashMap<&'static str,Stateaction>>,  // runtime state machine
  pub Rules : Vec<RProduction<AT,ET>>, //rules with just lhs and delegate function
  stopparsing : bool,
  pub exstate : ET,  // external state structure, usage optional
  pub stack :  Vec<Stackelement<AT>>, // parse stack
}

impl<AT:Default,ET:Default> RuntimeParser<AT,ET>
{
    pub fn new(rlen:usize, slen:usize) -> RuntimeParser<AT,ET>
    {  // given number of rules and number states
       let mut p = RuntimeParser {
         RSM : Vec::with_capacity(slen),
         Rules : Vec::with_capacity(rlen),
         stopparsing : false,
         exstate : ET::default(),
         stack : Vec::with_capacity(1024),
       };
       for _ in 0..slen {p.RSM.push(HashMap::new());}
       p
    }//new

    pub fn abort(&mut self, msg:&str)
    {
       println!("!!!Parsing Aborted: {}",msg);
       self.stopparsing=true;
    }

    // parse does not reset state stack
    pub fn parse(&mut self, tokenizer:&mut dyn Lexer<AT>) -> AT
    { 
       let mut result = AT::default();
       // push state 0 on stack:
       self.stack.push(Stackelement {si:0, value:AT::default()});
       let unexpected = Stateaction::Error(String::from("unexpected end of input"));
       let mut action = unexpected; //Stateaction::Error(String::from("get started"));
       self.stopparsing = false;
//       if !tokenizer.has_next() { self.stopparsing=true; }
//       let mut lookahead = tokenizer.nextsym(); // initial, this is a Lextoken
       let mut lookahead = Lextoken{sym:"EOF".to_owned(),value:AT::default()}; 
       if let Some(tok) = tokenizer.nextsym() {lookahead=tok;}
       else {self.stopparsing=true;}

       while !self.stopparsing
       {  
         let currentstate = self.stack[self.stack.len()-1].si;
         if TRACE>1 {print!(" current state={}, lookahead={}, ",&currentstate,&lookahead.sym);}
         let actionopt = self.RSM[currentstate].get(lookahead.sym.as_str());//.unwrap();
         if TRACE>1 {println!("RSM action : {:?}",actionopt);}
         if let None = actionopt {
            panic!("!!PARSE ERROR: no action at state {}, lookahead {}, line {}",currentstate,&lookahead.sym,tokenizer.linenum());
         }
         action = actionopt.unwrap().clone();  // cloning stateaction is ok
         match &action {
            Stateaction::Shift(i) => { // shift to state si
//              self.stack.push(Stackelement{si:*i,value:lookahead.value.clone()});
                self.stack.push(Stackelement{si:*i,value:mem::replace(&mut lookahead.value,AT::default())});
              // cloning here ok because it's just a token, like an int or string
//              if !tokenizer.has_next() { self.stopparsing=true; }
//              else {lookahead = tokenizer.nextsym();} // ADVANCE LOOKAHEAD HERE ONLY!
                if let Some(tok) = tokenizer.nextsym() {lookahead=tok;}
                else {
                  lookahead=Lextoken{sym:"EOF".to_owned(),  value:AT::default()};
                }
             }, //shift
            Stateaction::Reduce(ri) => { //reduce by rule i
              let rulei = &self.Rules[*ri];
              let ruleilhs = rulei.lhs; // &'static : Copy
              let val = (rulei.Ruleaction)(self); // calls delegate function
              let newtop = self.stack[self.stack.len()-1].si; 
              let goton = self.RSM[newtop].get(ruleilhs).unwrap();
              if TRACE>1 {println!(" ..performing Reduce({}), new state {}, action on {}: {:?}..",ri,newtop,ruleilhs,goton);}
              if let Stateaction::Gotonext(nsi) = goton {
                self.stack.push(Stackelement{si:*nsi,value:val});
                // DO NOT CHANGE LOOKAHEAD AFTER REDUCE!
              }// goto next state after reduce
              else { self.stopparsing=true; }
             },
            Stateaction::Accept => {
              result = self.stack.pop().unwrap().value;
              self.stopparsing = true;
             },
            Stateaction::Error(msg) => {
              self.stopparsing = true;
             },
            Stateaction::Gotonext(_) => { //should not see this here
              self.stopparsing = true;
             },
         }//match & action
       } // main parser loop
       if let Stateaction::Error(msg) = &action {
          panic!("!!!Parsing failed on line {}, next symbol {}: {}",tokenizer.linenum(),&lookahead.sym,msg);
       }
       return result;
    }//parse
}// impl RuntimeParser


////////////////////////////////////////////////////////////////
//// new version of write_fsm:

impl Statemachine
{
  pub fn writeparser(&self, filename:&str)->Result<(),std::io::Error>
  {
    let mut fd = File::create(filename)?;
    write!(fd,"//Parser generated by RustLr\n
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
extern crate RustLr;
use RustLr::{{RuntimeParser,RProduction,Stateaction,decode_action}};\n")?;

    write!(fd,"{}\n",&self.Gmr.Extras)?; // use clauses

    // write static array of symbols
    write!(fd,"const SYMBOLS:[&'static str;{}] = [",self.Gmr.Symbols.len())?;
    for i in 0..self.Gmr.Symbols.len()-1
    {
      write!(fd,"\"{}\",",&self.Gmr.Symbols[i].sym)?;
    }
    write!(fd,"\"{}\"];\n\n",&self.Gmr.Symbols[self.Gmr.Symbols.len()-1].sym)?;
    // position of symbols must be inline with self.Gmr.Symhash

    // record table entries in a static array
    let mut totalsize = 0;
    for i in 0..self.FSM.len() { totalsize+=self.FSM[i].len(); }
    write!(fd,"const TABLE:[u64;{}] = [",totalsize)?;
    // generate table to represent FSM
    let mut encode:u64 = 0;
    for i in 0..self.FSM.len() // for each state index i
    {
      let row = &self.FSM[i];
      for key in row.keys()
      { // see function decode for opposite translation
        let k = *self.Gmr.Symhash.get(key).unwrap(); // index of symbol
        encode = ((i as u64) << 48) + ((k as u64) << 32);
        match row.get(key) {
          Some(Shift(statei)) => { encode += (*statei as u64) << 16; },
          Some(Gotonext(statei)) => { encode += ((*statei as u64) << 16)+1; },
          Some(Reduce(rulei)) => { encode += ((*rulei as u64) << 16)+2; },
          Some(Accept) => {encode += 3; },
          _ => {encode += 4; },  // 4 indicates Error
        }//match
        write!(fd,"{},",encode)?;
      } //for symbol index k
    }//for each state index i
    write!(fd,"];\n\n")?;

    // must know what absyn type is when generating code.
    let ref absyn = self.Gmr.Absyntype;
    let ref extype = self.Gmr.Externtype;
    write!(fd,"pub fn make_parser() -> RuntimeParser<{},{}>",absyn,extype)?; 
    write!(fd,"\n{{\n")?;
    // write code to pop stack, assign labels to variables.
    write!(fd," let mut parser1:RuntimeParser<{},{}> = RuntimeParser::new({},{});\n",absyn,extype,self.Gmr.Rules.len(),self.States.len())?;
    // generate rules and Ruleaction delegates, must pop values from runtime stack
    write!(fd," let mut rule = RProduction::<{},{}>::new_skeleton(\"{}\");\n",absyn,extype,"start")?;
    for i in 0..self.Gmr.Rules.len() 
    {
      write!(fd," rule = RProduction::<{},{}>::new_skeleton(\"{}\");\n",absyn,extype,self.Gmr.Rules[i].lhs.sym)?;      
      write!(fd," rule.Ruleaction = |parser|{{ ")?;
      let mut k = self.Gmr.Rules[i].rhs.len();
      while k>0
      {
        let gsym = &self.Gmr.Rules[i].rhs[k-1];
        if gsym.label.len()>0 && &gsym.rusttype[0..3]=="mut"
          { write!(fd," let mut {}:{}=",gsym.label,absyn)?; }        
        else if gsym.label.len()>0
          { write!(fd," let {}:{}=",gsym.label,absyn)?; }
        write!(fd,"parser.stack.pop()")?; 
        if gsym.label.len()>0 { write!(fd,".unwrap().value;  ")?;}
        else {write!(fd,";  ")?;}
        k -= 1;
      } // for each symbol on right hand side of rule  
      let mut semaction = &self.Gmr.Rules[i].action; //this is a string
      //if semaction.len()<1 {semaction = "}}";}
      //if al>1 {semaction = semaction.substring(0,al-1);}
      if semaction.len()>1 {write!(fd,"{};\n",semaction.trim_end())?;}
      else {write!(fd," return {}::default();}};\n",absyn)?;}
      write!(fd," parser1.Rules.push(rule);\n")?;
    }// for each rule

    // generate code to load RSM from TABLE
    write!(fd,"\n for i in 0..{} {{\n",totalsize)?;
    write!(fd,"   let symi = ((TABLE[i] & 0x0000ffff00000000) >> 32) as usize;\n")?;
    write!(fd,"   let sti = ((TABLE[i] & 0xffff000000000000) >> 48) as usize;\n")?;
    write!(fd,"   parser1.RSM[sti].insert(SYMBOLS[symi],decode_action(TABLE[i]));\n }}\n\n")?;
//    write!(fd,"\n for i in 0..{} {{for k in 0..{} {{\n",rows,cols)?;
//    write!(fd,"   parser1.RSM[i].insert(SYMBOLS[k],decode_action(TABLE[i*{}+k]));\n }}}}\n\n",cols)?;

    write!(fd," return parser1;\n")?;
    write!(fd,"}} //make_parser\n")?;
    Ok(())
  }//writeparser



///////////////// non-binary version //////////////////
pub fn write_verbose(&self, filename:&str)->Result<(),std::io::Error>
  {
    let mut fd = File::create(filename)?;
    write!(fd,"//Parser generated by RustLr\n
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
extern crate RustLr;
use RustLr::{{RuntimeParser,RProduction,Stateaction}};\n")?;

    write!(fd,"{}\n",&self.Gmr.Extras)?; // use clauses
    let ref absyn = self.Gmr.Absyntype;
    let ref extype = self.Gmr.Externtype;
    write!(fd,"pub fn make_parser() -> RuntimeParser<{},{}>",absyn,extype)?; 
    write!(fd,"\n{{\n")?;
    // write code to pop stack, assign labels to variables.
    write!(fd," let mut parser1:RuntimeParser<{},{}> = RuntimeParser::new({},{});\n",absyn,extype,self.Gmr.Rules.len(),self.States.len())?;
    // generate rules and Ruleaction delegates, must pop values from runtime stack
    write!(fd," let mut rule = RProduction::<{},{}>::new_skeleton(\"{}\");\n",absyn,extype,"start")?;
    for i in 0..self.Gmr.Rules.len() 
    {
      write!(fd," rule = RProduction::<{},{}>::new_skeleton(\"{}\");\n",absyn,extype,self.Gmr.Rules[i].lhs.sym)?;      
      write!(fd," rule.Ruleaction = |parser|{{ ")?;
      let mut k = self.Gmr.Rules[i].rhs.len();
      while k>0
      {
        let gsym = &self.Gmr.Rules[i].rhs[k-1];
        if gsym.label.len()>0 && &gsym.rusttype[0..3]=="mut"
          { write!(fd," let mut {}:{}=",gsym.label,absyn)?; }        
        else if gsym.label.len()>0
          { write!(fd," let {}:{}=",gsym.label,absyn)?; }
        write!(fd,"parser.stack.pop()")?; 
        if gsym.label.len()>0 { write!(fd,".unwrap().value;  ")?;}
        else {write!(fd,";  ")?;}
        k -= 1;
      } // for each symbol on right hand side of rule  
      let mut semaction = &self.Gmr.Rules[i].action; //this is a string
      //if semaction.len()<1 {semaction = "}}";}
      //if al>1 {semaction = semaction.substring(0,al-1);}
      if semaction.len()>1 {write!(fd,"{};\n",semaction.trim_end())?;}
      else {write!(fd," return {}::default();}};\n",absyn)?;}
      write!(fd," parser1.Rules.push(rule);\n")?;
    }// for each rule


    for i in 0..self.FSM.len()
    {
      let row = &self.FSM[i];
      for key in row.keys()
      {
        write!(fd," parser1.RSM[{}].insert(\"{}\",Stateaction::{:?});\n",i,key,row.get(key).unwrap())?;
      } //for each string key in row
    }//for each state index i

    write!(fd," return parser1;\n")?;
    write!(fd,"}} //make_parser\n")?;
    Ok(())
  }//write_verbose

} // impl Statemachine
