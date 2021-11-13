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
use crate::{TRACE,Lexer,Lextoken,Stateaction,Statemachine};
use crate::Stateaction::*;

/// this structure is only exported because it is required by the generated parsers.
/// There is no reason to use it in other programs.
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

pub struct Stackelement<AT:Default>
{
   pub si : usize, // state index
   pub value : AT,  // semantic value (don't clone grammar symbols)
}

/// this is the structure created by the generated parser.  The generated parser
/// program will contain a make_parser function that returns this structure.
/// Most of the pub items are, however, only exported to support the operation
/// of the parser, and should not be accessed directly.  Only the functions
/// [RuntimeParser::parse], [RuntimeParser::report], [RuntimeParser::abort]
/// and [RuntimeParser::error_occurred] should be called directly 
/// from user programs.  Only the field [RuntimeParser::exstate] should be accessed
/// by user programs.
pub struct RuntimeParser<AT:Default,ET:Default>  
{
  /// this is the "external state" structure, with type ET defined by the grammar.
  /// The semantic actions associated with each grammar rule, which are written
  /// in the grammar, have ref mut access to the RuntimeParser structure, which
  /// allows them to read and change the external state object.  This gives
  /// the parsers greater flexibility and capability, including the ability to
  /// parse some non-context free languages.  See the sample grammar at
  /// <https://cs.hofstra.edu/~cscccl/rustlr_project/ncf.grammar>.
  /// The exstate is initialized to ET::default().
  pub exstate : ET,  // external state structure, usage optional
  /// used only by generated parser: do not reference
  pub RSM : Vec<HashMap<&'static str,Stateaction>>,  // runtime state machine
  /// do not reference
  pub Rules : Vec<RProduction<AT,ET>>, //rules with just lhs and delegate function
  ////// this value should be set through abort or report
  stopparsing : bool,
  /// do not reference  
  pub stack :  Vec<Stackelement<AT>>, // parse stack
//  pub recover : HashSet<&'static str>, // for error recovery
  pub resynch : HashSet<&'static str>,
  pub Errsym : &'static str,
  err_occured : bool,
  pub linenum : usize,
  pub column : usize,  // not used for now
}//struct RuntimeParser

impl<AT:Default,ET:Default> RuntimeParser<AT,ET>
{
    /// this is only called by the make_parser function in the machine-generated
    /// parser program.  *Do not call this function in other places* as it
    /// only generates a skeleton.
    pub fn new(rlen:usize, slen:usize) -> RuntimeParser<AT,ET>
    {  // given number of rules and number states
       let mut p = RuntimeParser {
         RSM : Vec::with_capacity(slen),
         Rules : Vec::with_capacity(rlen),
         stopparsing : false,
         exstate : ET::default(),
         stack : Vec::with_capacity(1024),
         Errsym : "",
         err_occured : false,
         linenum : 0,
         column : 0,
//         recover : HashSet::new(),
         resynch : HashSet::new(),
       };
       for _ in 0..slen {p.RSM.push(HashMap::new());}
       p
    }//new

    /// this function can be called from with the "semantic" actions attached
    /// to grammar production rules that are executed for each
    /// "reduce" action of the parser.
    pub fn abort(&mut self, msg:&str)
    {
       println!("!!!Parsing Aborted: {}",msg);
       self.err_occured = true;
       self.stopparsing=true;
    }

    /// may be called from grammar semantic actions to report error
    pub fn report(&mut self, errmsg:&str)  // linenum must be set prior to call
    {
       println!("ERROR on line {}, column {}: {}",self.linenum,self.column,errmsg);
       self.err_occured = true;
    }

    fn reduce(&mut self, ri:&usize)
    {
              let rulei = &self.Rules[*ri];
              let ruleilhs = rulei.lhs; // &'static : Copy
              let val = (rulei.Ruleaction)(self); // calls delegate function
              let newtop = self.stack[self.stack.len()-1].si; 
              let goton = self.RSM[newtop].get(ruleilhs).unwrap();
//              if TRACE>1 {println!(" ..performing Reduce({}), new state {}, action on {}: {:?}..",ri,newtop,ruleilhs,goton);}
              if let Stateaction::Gotonext(nsi) = goton {
                self.stack.push(Stackelement{si:*nsi,value:val});
                // DO NOT CHANGE LOOKAHEAD AFTER REDUCE!
              }// goto next state after reduce
              else {
                self.report("no suitable action can be taken");
                self.stopparsing=true;
              }
    }//reduce

    /// can be called to determine if an error occurred during parsing.  The parser
    /// will not panic.
    pub fn error_occurred(&self) -> bool {self.err_occured}

    fn nexttoken(&self, tokenizer:&mut dyn Lexer<AT>) -> Lextoken<AT>
    {
       if let Some(tok) = tokenizer.nextsym() {tok}
        else { Lextoken{sym:"EOF".to_owned(),  value:AT::default()} } 
    }
    // parse does not reset state stack
    
    /// this function is used to invoke the generated parser returned by
    /// the generated parser program's make_parser function.
    pub fn parse(&mut self, tokenizer:&mut dyn Lexer<AT>) -> AT
    {
       self.err_occured = false;
       self.stack.clear();
//       self.exstate = ET::default(); ???
       let mut result = AT::default();
       // push state 0 on stack:
       self.stack.push(Stackelement {si:0, value:AT::default()});
       let unexpected = Stateaction::Error(String::from("unexpected end of input"));
       let mut action = unexpected; //Stateaction::Error(String::from("get started"));
       self.stopparsing = false;
       let mut lookahead = Lextoken{sym:"EOF".to_owned(),value:AT::default()}; 
       if let Some(tok) = tokenizer.nextsym() {lookahead=tok;}
       else {self.stopparsing=true;}

       while !self.stopparsing
       {
         self.linenum = tokenizer.linenum(); self.column=tokenizer.column();
         let currentstate = self.stack[self.stack.len()-1].si;
//         if TRACE>1 {print!(" current state={}, lookahead={}, ",&currentstate,&lookahead.sym);}
         let mut actionopt = self.RSM[currentstate].get(lookahead.sym.as_str());//.unwrap();
//         if TRACE>1 {println!("RSM action : {:?}",actionopt);}
//println!("actionopt: {:?}, current state {}",actionopt,self.stack[self.stack.len()-1].si);            
///// Do error recovery
         if let None = actionopt {
            self.report(&format!("unexpected symbol {}, attempting recovery ...",&lookahead.sym));
            let mut erraction = None;
            // skip ahead until a resync symbol is found

            ///// prefer to use Errsym method
            if self.Errsym.len()>0 {
               let errsym = self.Errsym;
               //lookdown stack for "shift" action on errsym
               let mut k = self.stack.len()-1; // offset by 1 because of usize
               let mut spos = k+1;
               while k>0 && spos>k
               {
                  let ksi = self.stack[k-1].si;
                  erraction = self.RSM[ksi].get(errsym);
                  if let Some(Shift(_)) = erraction { spos=k;}
                  else {k-=1;}
               }//while k>0
               if spos==k { self.stack.truncate(k); }
               if let Some(Shift(i)) = erraction { // simulate shift errsym 
                 self.stack.push(Stackelement{si:*i,value:AT::default()});
                 // now keep lookahead until action is found that transitions from
                 // current state (i).  since only terminals may follow errsym,
                 // this would have to be a shift rule
                 while let None = self.RSM[*i].get(&lookahead.sym[..]) {
                    if &lookahead.sym[..]=="EOF" {break;}
                    lookahead = self.nexttoken(tokenizer);
                 }//while let
                 // either at end of input or found action on next symbol
                 erraction = self.RSM[*i].get(&lookahead.sym[..]);
               } // if shift action found down under stack
               else {erraction = None; }// don't reduce
            }//errsym exists

            // at this point, if erraction is None, then Errsym failed to recover,
            // try the resynch symbol method...
            
            if erraction==None && self.resynch.len()>0 {
               while &lookahead.sym!="EOF" &&
                      !self.resynch.contains(&lookahead.sym[..]) {
                 lookahead = self.nexttoken(tokenizer);
               }
             if &lookahead.sym!="EOF" {
              // look for state on stack that has action defined on next symbol
              lookahead = self.nexttoken(tokenizer); // skipp err-causing symbol
             }
              let mut k = self.stack.len()-1; // offset by 1 because of usize
              let mut position = 0;
              while k>0 && erraction==None
               {
                  let ksi = self.stack[k-1].si;
                  erraction = self.RSM[ksi].get(&lookahead.sym[..]);
                  if let None=erraction {k-=1;}
               }//while k>0 && erraction==None
              match erraction {
                 None => {}, // do nothing, whill shift next symbol
                 _ => { self.stack.truncate(k);},//pop stack
              }//match
            }// there are resync symbols

            // at this point, if erraction is None, then resynch recovery failed too.
            // only action left is to skip ahead...
            if let None = erraction { //skip input, loop back
                lookahead = self.nexttoken(tokenizer);
                if &lookahead.sym=="EOF" {
                  self.abort("error recovery failed before end of input");
                }
            }

/* /////           
            while let Some(Reduce(ri)) = erraction // keep reducing
            {
              //self.reduce(ri); // borrow error- only need mut self.stack
              let rulei = &self.Rules[*ri];
              let ruleilhs = rulei.lhs; // &'static : Copy
              let val = (rulei.Ruleaction)(self); // calls delegate function
              let newtop = self.stack[self.stack.len()-1].si; 
              let gotonopt = self.RSM[newtop].get(ruleilhs);
              match gotonopt {
                Some(Gotonext(nsi)) => { 
                  self.stack.push(Stackelement{si:*nsi,value:val});
                },// goto next state after reduce
                _ => {self.abort("recovery failed"); },
              }//match
              // end reduce
              let tos=self.stack[self.stack.len()-1].si;
              erraction = self.RSM[tos].get(self.Errsym);
            } // while let erraction is reduce
            //println!("erraction: {:?}, current state {}",erraction,self.stack[self.stack.len()-1].si);

///// */

         }//error recovery
         
         else {
          action = actionopt.unwrap().clone();  // cloning stateaction is ok
          match &action {
            Stateaction::Shift(i) => { // shift to state si
                self.stack.push(Stackelement{si:*i,value:mem::replace(&mut lookahead.value,AT::default())});
                lookahead = self.nexttoken(tokenizer);
             }, //shift
            Stateaction::Reduce(ri) => { //reduce by rule i
               self.reduce(ri);
            /*
              let rulei = &self.Rules[*ri];
              let ruleilhs = rulei.lhs; // &'static : Copy
              let val = (rulei.Ruleaction)(self); // calls delegate function
              let newtop = self.stack[self.stack.len()-1].si; 
              let goton = self.RSM[newtop].get(ruleilhs).unwrap();
              if let Stateaction::Gotonext(nsi) = goton {
                self.stack.push(Stackelement{si:*nsi,value:val});
                // DO NOT CHANGE LOOKAHEAD AFTER REDUCE!
              }// goto next state after reduce
              else { self.stopparsing=true; }
             */
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
         }// else not in error recovery mode
       } // main parser loop
       if let Stateaction::Error(msg) = &action {
          //panic!("!!!Parsing failed on line {}, next symbol {}: {}",tokenizer.linenum(),&lookahead.sym,msg);
          self.report(&format!("failure with next symbol {}",tokenizer.linenum()));
       }
       //if self.err_occured {result = AT::default(); }
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
    write!(fd,"//Parser generated by rustlr\n
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
extern crate rustlr;
use rustlr::{{RuntimeParser,RProduction,decode_action}};\n")?;

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
    write!(fd," parser1.Errsym = \"{}\";\n",&self.Gmr.Errsym)?;
    // resynch vector
    for s in &self.Gmr.Resynch {write!(fd," parser1.resynch.insert(\"{}\");\n",s)?;}

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
    write!(fd,"//Parser generated by rustlr\n
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
extern crate rustlr;
use rustlr::{{RuntimeParser,RProduction,Stateaction}};\n")?;

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
    write!(fd," parser1.Errsym = \"{}\";\n",&self.Gmr.Errsym)?;
    // resynch vector
    for s in &self.Gmr.Resynch {write!(fd," parser1.resynch.insert(\"{}\");\n",s)?;}
    
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

