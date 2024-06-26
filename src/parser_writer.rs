#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
#![allow(unused_doc_comments)]
#![allow(unused_imports)]
use std::io::{self,Read,Write,BufReader,BufRead};
use std::collections::HashSet;
//use std::hash::{Hash,Hasher};
//use std::any::Any;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
//use std::mem;
use crate::{Statemachine,Stateaction,checkboxexp};
use crate::{StandardReporter};
use crate::zc_parser::*;
use crate::lexer_interface::*;
use crate::generic_absyn::*;
use crate::Stateaction::*;


  fn is_lba(t:&str) -> bool {
   t.trim().starts_with("LBox") && t.contains("Any") && t.contains('<') && t.contains('>')
  }//is_lba to check type


// function to remove lifetime, <'t>, non-alphanums from string
fn remove_lt(s:&str, lt:&str) -> String
{
   let mut ax = String::from(s);
   if lt.len()==0 {return ax;}
   let mut ltform = format!("{} ",lt);
   let mut ln = ltform.len();
   while let Some(p) = ax.find(&ltform) {ax.replace_range(p..(p+ln),"");}
   ltform = format!("<{}>",lt); ln = ltform.len();
   while let Some(p) = ax.find(&ltform) {ax.replace_range(p..(p+ln),"");}   
   ln = lt.len();
   while let Some(p) = ax.find(lt) {ax.replace_range(p..(p+ln),"");}
   while let Some(p) = ax.find("<") {ax.replace_range(p..(p+1),"_");}
   while let Some(p) = ax.find(">") {ax.replace_range(p..(p+1),"_");}   
   ax
}//remove_lt

/////////////////////ENUM VERSION//////////////////////////////////////
   ///// semantic acition fn is _semaction_rule_{rule index}
////////////////////////////////////////////////
impl Statemachine
{
  pub fn writeenumparser(&self, filename:&str)->Result<(),std::io::Error>
  {
    let ref absyn = self.Gmr.Absyntype;

    if self.Gmr.sametype || is_lba(absyn){
       return self.writelbaparser(filename);
    }
    
    let ref extype = self.Gmr.Externtype;
    let ref lifetime = self.Gmr.lifetime;
    let has_lt = lifetime.len()>0 /*&& (absyn.len()==0 || (absyn.contains(lifetime) || extype.contains(lifetime)))*/;
    let ltopt = if has_lt {format!("<{}>",lifetime)} else {String::from("")};
    let lbc = if self.Gmr.bumpast {"lc"} else {"lbx"};

    let rlen = self.Gmr.Rules.len();
    // generate action fn's from strings stored in gen-time grammar
    let mut actions:Vec<String> = Vec::with_capacity(rlen);    
    for ri in 0..rlen
    {
      let lhs = &self.Gmr.Rules[ri].lhs.sym;
      let lhsi = self.Gmr.Rules[ri].lhs.index;
      let rettype = &self.Gmr.Symbols[lhsi].rusttype; // return type=rusttype
      let ltoptr = if has_lt || (lifetime.len()>0 && rettype.contains(lifetime))
        {format!("<{}>",lifetime)} else {String::from("")};
      let mut fndef = format!("\nfn _semaction_rule_{}_{}(parser:&mut ZCParser<RetTypeEnum{},{}>) -> {} {{\n",ri,&ltoptr,&ltopt,extype,rettype);
      let mut k = self.Gmr.Rules[ri].rhs.len(); //k=len of rhs of rule ri
      //form if-let labels and patterns as we go...
      let mut labels = String::from("(");
      let mut patterns = String::from("(");
      while k>0 // k is length of right-hand side
      {
        let mut boxedlabel = false;  // see if named label is of form [x]
        let gsym = &self.Gmr.Rules[ri].rhs[k-1]; // rhs syms right to left
        //let gsymi = *self.Gmr.Symhash.get(&gsym.sym).unwrap();
        let findat = gsym.label.find('@');
        let mut plab = format!("_item{}_",k-1);
        match &findat {
          None if gsym.label.len()>0 /*&& !gsym.label.contains('(')*/ => {
            let truelabel = checkboxexp(&gsym.label,&plab);
            boxedlabel = gsym.label.starts_with('[') && (truelabel != &gsym.label);
            plab = String::from(truelabel);
          },
          Some(ati) if *ati>0 => {
            let rawlabel = gsym.label[0..*ati].trim();
            let truelabel = checkboxexp(rawlabel,&plab);
            boxedlabel = gsym.label.starts_with('[') && (truelabel != rawlabel);
            plab = String::from(truelabel);
          },
          _ => {},
        }//match
        let poppedlab = plab.as_str();
        let symtype=&self.Gmr.Symbols[gsym.index].rusttype;
        let emsg = format!("FATAL ERROR: '{}' IS NOT A TYPE IN THIS GRAMMAR. DID YOU INTEND TO USE THE -auto OPTION TO GENERATE TYPES?",&symtype);
        let eindex = self.Gmr.enumhash.get(symtype).expect(&emsg);
        //form RetTypeEnum::Enumvariant_{eindex}(popped value)
        let stat;
        
//        if !boxedlabel { // not a [x] label -
// NEW APPROACH 0.4.13: let ast_writer decide if it's an lbox - always

        if self.Gmr.bumpast && boxedlabel {  // if bumpast and [x] label
           stat = format!("let mut _{0}_ = if let RetTypeEnum::Enumvariant_{1}(_x_{1})=parser.popstack().value {{ _x_{1} }} else {{<{2}>::default()}};  let mut {0} = parser.exstate.make(parser.lc({3},_{0}_));  ",poppedlab,&eindex,symtype,k-1);
        }  // special case to correspond with bumpast_writer
        else {
           if self.Gmr.Rules[ri].autogenerated || !boxedlabel { //auto-generated type/action
             stat = format!("let mut {0} = if let RetTypeEnum::Enumvariant_{1}(_x_{1})=parser.popstack().value {{ _x_{1} }} else {{<{2}>::default()}}; ",poppedlab,&eindex,symtype);
           }
           else {  // action not auto with box label, must create LBox
             stat = format!("let mut _{0}_ = if let RetTypeEnum::Enumvariant_{1}(_x_{1})=parser.popstack().value {{ _x_{1} }} else {{<{2}>::default()}};  let mut {0} = parser.lbx({3},_{0}_);  ",poppedlab,&eindex,symtype,k-1);           
           }
        } // not bumpast

/*          
        } else {   // is boxedlabel  (new: do same)
          if self.Gmr.bumpast {
            stat = format!("let mut _{0}_ = if let RetTypeEnum::Enumvariant_{1}(_x_{1})=parser.popstack().value {{ _x_{1} }} else {{<{2}>::default()}};  let mut {0} = parser.exstate.make(parser.lc({3},_{0}_));  ",poppedlab,&eindex,symtype,k-1);
          } else {
            stat = format!("let mut _{0}_ = if let RetTypeEnum::Enumvariant_{1}(_x_{1})=parser.popstack().value {{ _x_{1} }} else {{<{2}>::default()}};  let mut {0} = parser.lbx({3},_{0}_);  ",poppedlab,&eindex,symtype,k-1);
          }//no bump
        }// is a boxed [x] label
*/

        fndef.push_str(&stat);

        if gsym.label.len()>1 && findat.is_some() { // if-let pattern @@
	  let atindex = findat.unwrap();
          if atindex>0 { // label like es:@Exp(..)@
            labels.push_str("&mut "); // for if-let
            if boxedlabel {labels.push('*');} // &mut *Lbox gets the value
            labels.push_str(poppedlab); labels.push(',');
          }
          else { // non-labeled pattern: E:@..@
            labels.push_str(poppedlab); labels.push(',');
          }
	  patterns.push_str(&gsym.label[atindex+1..]); patterns.push(',');
	} // @@ pattern exists, with or without label

        k -= 1;      
      }// for each symbol on right hand side of rule (while k)
      // form if let pattern=labels ...
      let defaultaction = format!("<{}>::default()}}",rettype);
      let mut semaction = &self.Gmr.Rules[ri].action; //string that ends w/ rbr
      if semaction.len()<=1 {semaction = &defaultaction;}
      if labels.len()<2 {
        fndef.push_str(semaction.trim_end()); fndef.push_str("\n");
      } //empty pattern
      else { // write an if-let
        labels.push(')');  patterns.push(')');
	let pat2= format!("\n  if let {}={} {{ {}  else {{parser.report(\"{}\"); <{}>::default()}} }}\n",&patterns,&labels,semaction.trim_end(),&patterns,rettype);
        fndef.push_str(&pat2);
      }// if-let semantic action
      actions.push(fndef);
    }// generate action function for each rule  (for ri..

    ////// write to file

    let mut fd = File::create(filename)?;
    write!(fd,"//Parser generated by rustlr for grammar {}",&self.Gmr.name)?;
    write!(fd,"\n    
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
use rustlr::{{Tokenizer,TerminalToken,ZCParser,ZCRProduction,Stateaction,decode_action}};\n")?;
    if self.Gmr.genlex {
      write!(fd,"use rustlr::{{StrTokenizer,RawToken,LexSource}};
use std::collections::{{HashMap,HashSet}};\n")?;
    }

    write!(fd,"{}\n",&self.Gmr.Extras)?; // use clauses and such

    // write static array of symbols
    write!(fd,"static SYMBOLS:[&'static str;{}] = [",self.Gmr.Symbols.len())?;
    for i in 0..self.Gmr.Symbols.len()-1
    {
      write!(fd,"\"{}\",",&self.Gmr.Symbols[i].sym)?;
    }
    write!(fd,"\"{}\"];\n\n",&self.Gmr.Symbols[self.Gmr.Symbols.len()-1].sym)?;
    // position of symbols must be inline with self.Gmr.Symhash

    // record table entries in a static array
    let mut totalsize = 0;
    for i in 0..self.FSM.len() { totalsize+=self.FSM[i].len(); }
    if self.Gmr.tracelev>1 {println!("{} total state table entries",totalsize);}
    write!(fd,"static TABLE:[u64;{}] = [",totalsize)?;
    // generate table to represent FSM
    let mut encode:u64 = 0;
    for i in 0..self.FSM.len() // for each state index i
    {
      let row = &self.FSM[i];        
      for key in row.keys()
      { // see function decode for opposite translation
        let k = *key; //*self.Gmr.Symhash.get(key).unwrap(); // index of symbol
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

    // write action functions fn _semaction_rule_{} ..
    for deffn in &actions { write!(fd,"{}",deffn)?; }

    // must know what absyn type is when generating code.
    write!(fd,"\npub fn make_parser{}() -> ZCParser<RetTypeEnum{},{}>",&ltopt,&ltopt,extype)?; 
    write!(fd,"\n{{\n")?;
    // write code to pop stack, assign labels to variables.
    write!(fd," let mut parser1:ZCParser<RetTypeEnum{},{}> = ZCParser::new({},{});\n",&ltopt,extype,self.Gmr.Rules.len(),self.FSM.len())?;
    // generate rules and Ruleaction delegates to call action fns, cast
     write!(fd," let mut rule = ZCRProduction::<RetTypeEnum{},{}>::new_skeleton(\"{}\");\n",&ltopt,extype,"start")?; // dummy for init
    for i in 0..self.Gmr.Rules.len() 
    {
      write!(fd," rule = ZCRProduction::<RetTypeEnum{},{}>::new_skeleton(\"{}\");\n",&ltopt,extype,self.Gmr.Rules[i].lhs.sym)?;
      write!(fd," rule.Ruleaction = |parser|{{ ")?;

    // write code to call action function, then convert to RetTypeEnum
      let lhsi = self.Gmr.Symhash.get(&self.Gmr.Rules[i].lhs.sym).expect("GRAMMAR REPRESENTATION CORRUPTED");
      let fnname = format!("_semaction_rule_{}_",i);
      let typei = &self.Gmr.Symbols[*lhsi].rusttype;
      let enumindex = self.Gmr.enumhash.get(typei).expect("FATAL ERROR: TYPE {typei} NOT USED IN GRAMMAR");
      write!(fd," RetTypeEnum::Enumvariant_{}({}(parser)) }};\n",enumindex,&fnname)?;
      write!(fd," parser1.Rules.push(rule);\n")?;
    }// write each rule action
    
    
    write!(fd," parser1.Errsym = \"{}\";\n",&self.Gmr.Errsym)?;
    // resynch vector
    for s in &self.Gmr.Resynch {write!(fd," parser1.resynch.insert(\"{}\");\n",s)?;}

    // generate code to load RSM from TABLE
    write!(fd,"\n for i in 0..{} {{\n",totalsize)?;
    write!(fd,"   let symi = ((TABLE[i] & 0x0000ffff00000000) >> 32) as usize;\n")?;
    write!(fd,"   let sti = ((TABLE[i] & 0xffff000000000000) >> 48) as usize;\n")?;
    write!(fd,"   parser1.RSM[sti].insert(SYMBOLS[symi],decode_action(TABLE[i]));\n }}\n\n")?;
//    write!(fd,"\n for i in 0..{} {{for k in 0..{} {{\n",rows,cols)?;
//    write!(fd,"   parser1.RSM[i].insert(SYMBOLS[k],decode_action(TABLE[i*{}+k]));\n }}}}\n",cols)?;
    write!(fd," for s in SYMBOLS {{ parser1.Symset.insert(s); }}\n\n")?;

    /* // took out 0.2.97
    if self.Gmr.transform_function.len()>0 {
      write!(fd," parser1.set_transform_token({});\n\n",&self.Gmr.transform_function)?;
    }
    */
    
    write!(fd," load_extras(&mut parser1);\n")?;
    write!(fd," return parser1;\n")?;
    write!(fd,"}} //make_parser\n\n")?;

/* // took out 0.2.97
    // write special value extraction functions for transform_function
    //if self.Gmr.transform_function.len()>0 {
      let mut already:HashSet<&str> = HashSet::new();
      for sym in &self.Gmr.Symbols
      {
         if sym.terminal && &sym.rusttype!="()" && !already.contains(&sym.rusttype[..]) && &sym.sym!="_WILDCARD_TOKEN_" {
//println!("processing for {}, type {}",&sym.sym, &sym.rusttype);         
            already.insert(&sym.rusttype);
            let ei = self.Gmr.enumhash.get(&sym.rusttype).expect("GRAMMAR CORRUPTED");
//            let ltm = &self.Gmr.lifetime;
//            let refform = format!("&{} ",ltm);
            let needclone = ".clone()"; //if sym.rusttype.starts_with("&") {""} else {".clone()"};
            
            write!(fd," fn extract_value_{}{}(x:&RetTypeEnum{}) -> {} {{
    if let RetTypeEnum::Enumvariant_{}(_v_) = x {{_v_{}}} else {{<{}>::default()}}
 }}\n",&sym.sym,&ltopt,&ltopt,&sym.rusttype,ei,needclone,&sym.rusttype)?;
            write!(fd," fn encode_value_{}{}(x:{}) -> RetTypeEnum{} {{ RetTypeEnum::Enumvariant_{}(x) }}\n",&sym.sym,&ltopt,&sym.rusttype,&ltopt,ei)?;
         }
      }//for each terminal symbol
    //}//transform-related
*/

    //if !self.Gmr.sametype {  // checked at first

      ////// WRITE parse_with and parse_train_with
      let lexerlt = if has_lt {&ltopt} else {"<'t>"};
      let lexername = format!("{}lexer{}",&self.Gmr.name,lexerlt);
      let abindex = *self.Gmr.enumhash.get(absyn).unwrap();
      write!(fd,"pub fn parse_with{}(parser:&mut ZCParser<RetTypeEnum{},{}>, lexer:&mut {}) -> Result<{},{}>\n{{\n",lexerlt,lexerlt,extype,&lexername,absyn,absyn)?;
      if self.Gmr.bumpast {
        write!(fd,"  if lexer.bump.is_some() {{parser.exstate.set(lexer.bump.unwrap());}}\n")?;
      }//bump
      
      write!(fd,"  lexer.shared_state = Rc::clone(&parser.shared_state);\n")?;
      write!(fd,"  if let RetTypeEnum::Enumvariant_{}(_xres_) = parser.parse(lexer) {{\n",abindex)?;
      write!(fd,"     if !parser.error_occurred() {{Ok(_xres_)}} else {{Err(_xres_)}}\n  }} ")?;
      write!(fd,"else {{ Err(<{}>::default())}}\n}}//parse_with public function\n",absyn)?;
      // training version
      write!(fd,"\npub fn parse_train_with{}(parser:&mut ZCParser<RetTypeEnum{},{}>, lexer:&mut {}, parserpath:&str) -> Result<{},{}>\n{{\n",lexerlt,&ltopt,extype,&lexername,absyn,absyn)?;
      if self.Gmr.bumpast {
        write!(fd,"  if lexer.bump.is_some() {{parser.exstate.set(lexer.bump.unwrap());}}\n")?;
      }//bump
      write!(fd,"  lexer.shared_state = Rc::clone(&parser.shared_state);\n")?;
      write!(fd,"  if let RetTypeEnum::Enumvariant_{}(_xres_) = parser.parse_train(lexer,parserpath) {{\n",abindex)?;
      write!(fd,"     if !parser.error_occurred() {{Ok(_xres_)}} else {{Err(_xres_)}}\n  }} ")?;
      write!(fd,"else {{ Err(<{}>::default())}}\n}}//parse_train_with public function\n",absyn)?;


      ////// WRITE ENUM
      self.Gmr.gen_enum(&mut fd)?;
    // }// !sametype
    
    ////// WRITE LEXER
    if self.Gmr.genlex { self.Gmr.genlexer(&mut fd,"from_raw")?; }

    ////// Augment!
    write!(fd,"fn load_extras{}(parser:&mut ZCParser<RetTypeEnum{},{}>)\n{{\n",&ltopt,&ltopt,extype)?;
    write!(fd,"}}//end of load_extras: don't change this line as it affects augmentation\n")?;
    Ok(())
  }//writeenumparser

}//impl statemachine




/////////////////////////////////////// for new base_parser
////////////////////////////////////////////////////////////////////////

/////// version that can write table to file.

impl Statemachine
{
  pub fn writebaseenumparser(&self, filename:&str)->Result<(),std::io::Error>
  {
    let ref absyn = self.Gmr.Absyntype;
    let ref extype = self.Gmr.Externtype;
    let ref lifetime = self.Gmr.lifetime;
    let has_lt = lifetime.len()>0 /*&& (absyn.len()==0 || (absyn.contains(lifetime) || extype.contains(lifetime)))*/;
    let ltopt = if has_lt {format!("<{}>",lifetime)} else {String::from("")};
//println!("abysn: {}, ltopt: {}",absyn,&ltopt);    
    let lbc = if self.Gmr.bumpast {"lc"} else {"lbx"};

      ////// WRITE parse_with and parse_train_with
      let lexerlt = if has_lt {&ltopt} else {"<'t>"};
      let lexerlife = if has_lt {lifetime} else {"'t"};
      let lexername = format!("{}lexer{}",&self.Gmr.name,lexerlt);
      let abindex = *self.Gmr.enumhash.get(absyn).unwrap();
      
    let rlen = self.Gmr.Rules.len();
    // generate action fn's from strings stored in gen-time grammar
    let mut actions:Vec<String> = Vec::with_capacity(rlen);    
    for ri in 0..rlen
    {
      let lhs = &self.Gmr.Rules[ri].lhs.sym;
      let lhsi = self.Gmr.Rules[ri].lhs.index;
      let rettype = &self.Gmr.Symbols[lhsi].rusttype; // return type=rusttype
      let ltoptr = if has_lt || (lifetime.len()>0 && rettype.contains(lifetime))
        {format!("<{}>",lifetime)} else {String::from("")};
      let mut fndef = format!("\nfn _semaction_rule_{}_<{},TT:Tokenizer<{},RetTypeEnum{}>>(parser:&mut BaseParser<{},RetTypeEnum{},{},TT>) -> {} {{\n",ri,lexerlife,lexerlife, &ltopt,lexerlife,&ltopt, extype,rettype);
      let mut k = self.Gmr.Rules[ri].rhs.len(); //k=len of rhs of rule ri
      //form if-let labels and patterns as we go...
      let mut labels = String::from("(");
      let mut patterns = String::from("(");
      while k>0 // k is length of right-hand side
      {
        let mut boxedlabel = false;  // see if named label is of form [x]
        let gsym = &self.Gmr.Rules[ri].rhs[k-1]; // rhs syms right to left
        //let gsymi = *self.Gmr.Symhash.get(&gsym.sym).unwrap();
        let findat = gsym.label.find('@');
        let mut plab = format!("_item{}_",k-1);
        match &findat {
          None if gsym.label.len()>0 /*&& !gsym.label.contains('(')*/ => {
            let truelabel = checkboxexp(&gsym.label,&plab);
            boxedlabel = gsym.label.starts_with('[') && (truelabel != &gsym.label);
            plab = String::from(truelabel);
          },
          Some(ati) if *ati>0 => {
            let rawlabel = gsym.label[0..*ati].trim();
            let truelabel = checkboxexp(rawlabel,&plab);
            boxedlabel = gsym.label.starts_with('[') && (truelabel != rawlabel);
            plab = String::from(truelabel);
          },
          _ => {},
        }//match
        let poppedlab = plab.as_str();
        let symtype=&self.Gmr.Symbols[gsym.index].rusttype;
        let emsg = format!("FATAL ERROR: '{}' IS NOT A TYPE IN THIS GRAMMAR. DID YOU INTEND TO USE THE -auto OPTION TO GENERATE TYPES?",&symtype);
        let eindex = self.Gmr.enumhash.get(symtype).expect(&emsg);
        //form RetTypeEnum::Enumvariant_{eindex}(popped value)
        let stat;
        
//        if !boxedlabel { // not a [x] label -
// NEW APPROACH 0.4.13: let ast_writer decide if it's an lbox - always

        if self.Gmr.bumpast && boxedlabel {  // if bumpast and [x] label
           stat = format!("let mut _{0}_ = if let RetTypeEnum::Enumvariant_{1}(_x_{1})=parser.popstack().value {{ _x_{1} }} else {{<{2}>::default()}};  let mut {0} = parser.exstate.make(parser.lc({3},_{0}_));  ",poppedlab,&eindex,symtype,k-1);
        }  // special case to correspond with bumpast_writer
        else {
           if self.Gmr.Rules[ri].autogenerated || !boxedlabel { //auto-generated type/action
             stat = format!("let mut {0} = if let RetTypeEnum::Enumvariant_{1}(_x_{1})=parser.popstack().value {{ _x_{1} }} else {{<{2}>::default()}}; ",poppedlab,&eindex,symtype);
           }
           else {  // action not auto with box label, must create LBox
             stat = format!("let mut _{0}_ = if let RetTypeEnum::Enumvariant_{1}(_x_{1})=parser.popstack().value {{ _x_{1} }} else {{<{2}>::default()}};  let mut {0} = parser.lbx({3},_{0}_);  ",poppedlab,&eindex,symtype,k-1);           
           }
        } // not bumpast

        fndef.push_str(&stat);

        if gsym.label.len()>1 && findat.is_some() { // if-let pattern @@
	  let atindex = findat.unwrap();
          if atindex>0 { // label like es:@Exp(..)@
            labels.push_str("&mut "); // for if-let
            if boxedlabel {labels.push('*');} // &mut *Lbox gets the value
            labels.push_str(poppedlab); labels.push(',');
          }
          else { // non-labeled pattern: E:@..@
            labels.push_str(poppedlab); labels.push(',');
          }
	  patterns.push_str(&gsym.label[atindex+1..]); patterns.push(',');
	} // @@ pattern exists, with or without label

        k -= 1;      
      }// for each symbol on right hand side of rule (while k)
      // form if let pattern=labels ...
      let defaultaction = format!("<{}>::default()}}",rettype);
      let mut semaction = &self.Gmr.Rules[ri].action; //string that ends w/ rbr
      if semaction.len()<=1 {semaction = &defaultaction;}
      if labels.len()<2 {
        fndef.push_str(semaction.trim_end()); fndef.push_str("\n");
      } //empty pattern
      else { // write an if-let
        labels.push(')');  patterns.push(')');
	let pat2= format!("\n  if let {}={} {{ {}  else {{parser.report(\"{}\"); <{}>::default()}} }}\n",&patterns,&labels,semaction.trim_end(),&patterns,rettype);
        fndef.push_str(&pat2);
      }// if-let semantic action
      actions.push(fndef);
    }// generate action function for each rule  (for ri..

    ////// write to file

    let mut fd = File::create(filename)?;
    write!(fd,"//Parser generated by rustlr for grammar {}",&self.Gmr.name)?;
    write!(fd,"\n    
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
use rustlr::{{Tokenizer,TerminalToken,BaseParser,BaseProduction,Stateaction,decode_action}};\n")?;
    if self.Gmr.genlex {
      write!(fd,"use rustlr::{{StrTokenizer,RawToken,LexSource}};
use std::collections::{{HashMap,HashSet}};\n")?;
    }

    write!(fd,"{}\n",&self.Gmr.Extras)?; // use clauses and such

    // write static array of symbols
    write!(fd,"static SYMBOLS:[&'static str;{}] = [",self.Gmr.Symbols.len())?;
    for i in 0..self.Gmr.Symbols.len()-1
    {
      write!(fd,"\"{}\",",&self.Gmr.Symbols[i].sym)?;
    }
    write!(fd,"\"{}\"];\n\n",&self.Gmr.Symbols[self.Gmr.Symbols.len()-1].sym)?;
    // position of symbols must be inline with self.Gmr.Symhash

    // record table entries in a static array
    let mut totalsize = 0;
    for i in 0..self.FSM.len() { totalsize+=self.FSM[i].len(); }
    if self.Gmr.tracelev>1 {println!("{} total state table entries",totalsize);}


    let mut tfdopt = None;
    if !self.Gmr.inlinetable {
      write!(fd,"static TABLE:[u64;{}] = [0;{}];\n",totalsize,totalsize)?;
      let tablefile = format!("{}_table.fsm",&self.Gmr.name);
      let mut tfd1 = File::create(tablefile)?;
      tfdopt = Some(tfd1);    
    }
    else {  // default behavior: write large table inline
      write!(fd,"static TABLE:[u64;{}] = [",totalsize)?;
    }
    // generate table to represent FSM
    let mut encode:u64 = 0;
    for i in 0..self.FSM.len() // for each state index i
    {
      let row = &self.FSM[i];        
      for key in row.keys()
      { // see function decode for opposite translation
        let k = *key; //*self.Gmr.Symhash.get(key).unwrap(); // index of symbol
        encode = ((i as u64) << 48) + ((k as u64) << 32);
        match row.get(key) {
          Some(Shift(statei)) => { encode += (*statei as u64) << 16; },
          Some(Gotonext(statei)) => { encode += ((*statei as u64) << 16)+1; },
          Some(Reduce(rulei)) => { encode += ((*rulei as u64) << 16)+2; },
          Some(Accept) => {encode += 3; },
          _ => {encode += 4; },  // 4 indicates Error
        }//match
        tfdopt.as_mut().map_or_else(||{write!(fd,"{},",encode)},
	                   |tfd|{tfd.write_all(&encode.to_be_bytes())})?;
      } //for symbol index k
    }//for each state index i
    if self.Gmr.inlinetable { write!(fd,"];\n\n")?; }
    else { // generate code to read from file
      let tablefile = format!("{}_table.fsm",&self.Gmr.name);
      write!(fd,"let mut tfd = File::open(tablefile).expect(\"Parse Table file {} Not Found\");\n",&tablefile)?;
      panic!("THIS FEATURE IS NOT YET SUPPORTED");
    }


    // write action functions fn _semaction_rule_{} ..
    for deffn in &actions { write!(fd,"{}",deffn)?; }

    // must know what absyn type is when generating code.
    write!(fd,"\npub fn make_parser<{},TT:Tokenizer<{},RetTypeEnum{}>>(tk:TT) -> BaseParser<{},RetTypeEnum{},{},TT>",lexerlife,lexerlife,&ltopt,lexerlife,&ltopt,extype)?; 
    write!(fd,"\n{{\n")?;
    // write code to pop stack, assign labels to variables.
    write!(fd," let mut parser1:BaseParser<{},RetTypeEnum{},{},TT> = BaseParser::new({},{},tk);\n",lexerlife,&ltopt,extype,self.Gmr.Rules.len(),self.FSM.len())?;
    // generate rules and Ruleaction delegates to call action fns, cast
     write!(fd," let mut rule = BaseProduction::<{},RetTypeEnum{},{},TT>::new_skeleton(\"{}\");\n",lexerlife,&ltopt,extype,"start")?; // dummy for init
    for i in 0..self.Gmr.Rules.len() 
    {
      write!(fd," rule = BaseProduction::<{},RetTypeEnum{},{},TT>::new_skeleton(\"{}\");\n",lexerlife,&ltopt,extype,self.Gmr.Rules[i].lhs.sym)?;
      write!(fd," rule.Ruleaction = |parser|{{ ")?;

    // write code to call action function, then convert to RetTypeEnum
      let lhsi = self.Gmr.Symhash.get(&self.Gmr.Rules[i].lhs.sym).expect("GRAMMAR REPRESENTATION CORRUPTED");
      let fnname = format!("_semaction_rule_{}_",i);
      let typei = &self.Gmr.Symbols[*lhsi].rusttype;
      let enumindex = self.Gmr.enumhash.get(typei).expect("FATAL ERROR: TYPE {typei} NOT USED IN GRAMMAR");
      write!(fd," RetTypeEnum::Enumvariant_{}({}(parser)) }};\n",enumindex,&fnname)?;
      write!(fd," parser1.Rules.push(rule);\n")?;
    }// write each rule action
    
    
    write!(fd," parser1.Errsym = \"{}\";\n",&self.Gmr.Errsym)?;
    // resynch vector
    for s in &self.Gmr.Resynch {write!(fd," parser1.resynch.insert(\"{}\");\n",s)?;}

    // generate code to load RSM from TABLE
    write!(fd,"\n for i in 0..{} {{\n",totalsize)?;
    write!(fd,"   let symi = ((TABLE[i] & 0x0000ffff00000000) >> 32) as usize;\n")?;
    write!(fd,"   let sti = ((TABLE[i] & 0xffff000000000000) >> 48) as usize;\n")?;
    write!(fd,"   parser1.RSM[sti].insert(SYMBOLS[symi],decode_action(TABLE[i]));\n }}\n\n")?;
//    write!(fd,"\n for i in 0..{} {{for k in 0..{} {{\n",rows,cols)?;
//    write!(fd,"   parser1.RSM[i].insert(SYMBOLS[k],decode_action(TABLE[i*{}+k]));\n }}}}\n",cols)?;
    write!(fd," for s in SYMBOLS {{ parser1.Symset.insert(s); }}\n\n")?;

    /* // took out 0.2.97
    if self.Gmr.transform_function.len()>0 {
      write!(fd," parser1.set_transform_token({});\n\n",&self.Gmr.transform_function)?;
    }
    */
    
    write!(fd," load_extras(&mut parser1);\n")?;
    write!(fd," return parser1;\n")?;
    write!(fd,"}} //make_parser\n\n")?;

      write!(fd,"pub fn parse_with{}(parser:&mut BaseParser<{},RetTypeEnum{},{},{}>) -> Result<{},{}>\n{{\n",lexerlt,lexerlife,&ltopt,extype,&lexername,absyn,absyn)?;
            
      if self.Gmr.bumpast {
        write!(fd,"  if parser.tokenizer.bump.is_some() {{let bb = parser.tokenizer.bump.unwrap(); parser.exstate.set(bb);}}\n")?;
      }//bump
      
      write!(fd,"  parser.tokenizer.shared_state = Rc::clone(&parser.shared_state);\n")?;
      write!(fd,"  if let RetTypeEnum::Enumvariant_{}(_xres_) = parser.parse() {{\n",abindex)?;
      write!(fd,"     if !parser.error_occurred() {{Ok(_xres_)}} else {{Err(_xres_)}}\n  }} ")?;
      write!(fd,"else {{ Err(<{}>::default())}}\n}}//parse_with public function\n",absyn)?;
      
      // training version

      write!(fd,"\npub fn parse_train_with{}(parser:&mut BaseParser<{},RetTypeEnum{},{},{}>, parserpath:&str) -> Result<{},{}>\n{{\n",lexerlt,lexerlife,&ltopt,extype,&lexername,absyn,absyn)?;
      
      if self.Gmr.bumpast {
        write!(fd,"  if parser.tokenizer.bump.is_some() {{let bb = parser.tokenizer.bump.unwrap(); parser.exstate.set(bb);}}\n")?;
      }//bump
      write!(fd,"  parser.tokenizer.shared_state = Rc::clone(&parser.shared_state);\n")?;
      write!(fd,"  if let RetTypeEnum::Enumvariant_{}(_xres_) = parser.parse_train(parserpath) {{\n",abindex)?;
      write!(fd,"     if !parser.error_occurred() {{Ok(_xres_)}} else {{Err(_xres_)}}\n  }} ")?;
      write!(fd,"else {{ Err(<{}>::default())}}\n}}//parse_train_with public function\n",absyn)?;


      ////// WRITE ENUM
      self.Gmr.gen_enum(&mut fd)?;
    // }// !sametype
    
    ////// WRITE LEXER
    if self.Gmr.genlex { self.Gmr.genlexer(&mut fd,"from_raw")?; }

    ////// Augment!
    write!(fd,"fn load_extras<{},TT:Tokenizer<{},RetTypeEnum{}>>(parser:&mut BaseParser<{},RetTypeEnum{},{},TT>)\n{{\n",lexerlife,lexerlife,&ltopt,lexerlife,&ltopt,extype)?;
    write!(fd,"}}//end of load_extras: don't change this line as it affects augmentation\n")?;
    Ok(())
  }//writebaseenumparser

}//impl statemachine
