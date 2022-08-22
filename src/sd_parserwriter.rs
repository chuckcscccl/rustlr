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
use crate::{Statemachine,checkboxlabel};
use crate::Stateaction::*;

/////////////////////LRSD VERSION//////////////////////////////////////
   ///// semantic action fn is _rrsemaction_rule_{rule index}
////////////////////////////////////////////////
impl Statemachine
{
  pub fn writelrsdparser(&self, filename:&str)->Result<(),std::io::Error>
  {
    let ref absyn = self.Gmr.Absyntype;

    if self.Gmr.sametype || is_lba(absyn){
       return self.writelbaparser(filename);
    }
    
    let ref extype = self.Gmr.Externtype;
    let ref lifetime = self.Gmr.lifetime;
    let has_lt = lifetime.len()>0 && (absyn.contains(lifetime) || extype.contains(lifetime));
    let ltopt = if has_lt {format!("<{}>",lifetime)} else {String::from("")};
    let rlen = self.Gmr.Rules.len();
    
    // generate action fn's from strings stored in gen-time grammar
    // these are the _semaction_rule_ri functions.  move function to
    // pop stack to the closures attached to each runtime rule.
    // make this a pure function on types defined.
    let mut actions:Vec<String> = Vec::with_capacity(rlen);
    
    for ri in 0..rlen
    {
      let lhs = &self.Gmr.Rules[ri].lhs.sym;
      let lhsi = self.Gmr.Rules[ri].lhs.index; //self.Gmr.Symhash.get(lhs).expect("GRAMMAR REPRESENTATION CORRUPTED");
      let rettype = &self.Gmr.Symbols[lhsi].rusttype; // return type=rusttype
      let ltoptr = if has_lt || (lifetime.len()>0 && rettype.contains(lifetime))
        {format!("<{}>",lifetime)} else {String::from("")};

// first arg to semaction is parser itself. - this is a must.
      let mut fndef = format!("\nfn _rrsemaction_{}_{}(parser:&mut ZCParser<RetTypeEnum{},{}>",ri,&ltoptr,&ltopt,extype);
      // now for other arguments
      // inside actions, can still bind labels to patterns
      let mut patternactions = String::new();                  
      for k in 0..self.Gmr.Rules[ri].rhs.len() {
        let symk= &self.Gmr.Rules[ri].rhs[k]; 
        let symktype = &self.Gmr.Symbols[symk.index].rusttype;
        let(labelkind,label) = decode_label(&symk.label,k);
        let mut fargk = match labelkind {
          0 => {format!(", mut {}:{}",&label,symktype)},
          1 => {format!(", mut {}:LBox<{}>",&label,symktype)},
          2 => {   // label is a e@..@ pattern
            let ati = symk.label.find('@').unwrap();
            patternactions.push_str(&format!("let {} = {}; ",
                                     &symk.label[ati+1..],&label));
            format!(", {}:&mut {}",&label,symktype)
          },
          3 => {   // label is a [e]@..@ pattern
            let ati = symk.label.find('@').unwrap();          
            patternactions.push_str(&format!("let {} = &mut *{}; ",
                                     &symk.label[ati+1..],&label));
            format!(", mut {}:LBox<{}>",&label,symktype)
          },
          _ => {
            let ati = symk.label.find('@').unwrap();          
            patternactions.push_str(&format!("let {} = _item{}_; ",
                                     &symk.label[ati+1..],k));
            format!(", mut _item{}_:{}",k,symktype)
          },
        };//match
        fndef.push_str(&fargk);
      }// for each symbol on rhs
      fndef.push_str(&format!(") -> {} {{ ",rettype));
      let defaultaction = format!("<{}>::default()}}",rettype);
      let mut semaction = &self.Gmr.Rules[ri].action; //string that ends w/rbr
      if semaction.len()>1  {fndef.push_str(&patternactions);}
      if semaction.len()<=1 {semaction = &defaultaction;}
      fndef.push_str(&semaction); 
      actions.push(fndef);
    } //for ri

    ////// write to file, create Ruleaction closures for each rule

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
if true || self.Gmr.tracelev>1 {println!("{} total state table entries",totalsize);}
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
    for deffn in &actions { write!(fd,"{}\n",deffn)?; }

    // must know what absyn type is when generating code.
    write!(fd,"\npub fn make_parser{}() -> ZCParser<RetTypeEnum{},{}>",&ltopt,&ltopt,extype)?; 
    write!(fd,"\n{{\n")?;
    write!(fd," let mut parser1:ZCParser<RetTypeEnum{},{}> = ZCParser::new({},{});\n",&ltopt,extype,self.Gmr.Rules.len(),self.FSM.len())?;


    // generate rules and Ruleaction delegates to call action fns, cast
//     write!(fd," let mut rule = ZCRProduction::<RetTypeEnum{},{}>::new_skeleton(\"{}\");\n",&ltopt,extype,"start")?; // dummy for init
    write!(fd," let mut rule;\n")?; // dummy for init
    for ri in 0..self.Gmr.Rules.len() 
    {
      write!(fd," rule = ZCRProduction::<RetTypeEnum{},{}>::new_skeleton(\"{}\");\n",&ltopt,extype,self.Gmr.Rules[ri].lhs.sym)?;
      write!(fd," rule.Ruleaction = |parser|{{ ")?;

    // write code to pop stack, decode labels into args. /////////
      let mut k = self.Gmr.Rules[ri].rhs.len(); //k=len of rhs of rule ri
      //form if-let labels and patterns as we go...
      let mut actualargs = Vec::new();
      while k>0 // k is length of right-hand side, use k-1
      {
        let gsym = &self.Gmr.Rules[ri].rhs[k-1]; // rhs syms right to left
        let (lbtype,poppedlab) = decode_label(&gsym.label,k-1);
        let symtype=&self.Gmr.Symbols[gsym.index].rusttype;
        let emsg = format!("FATAL ERROR: '{}' IS NOT A TYPE IN THIS GRAMMAR. DID YOU INTEND TO USE THE -auto OPTION TO GENERATE TYPES?",&symtype);
        let eindex = self.Gmr.enumhash.get(symtype).expect(&emsg);
        actualargs.push(format!("{}",&poppedlab));           
        let stat = match lbtype {
           0 => {
             format!("let {0} = if let RetTypeEnum::Enumvariant_{1}(_rr_{1})=parser.popstack().value {{ _rr_{1} }} else {{<{2}>::default()}}; ",&poppedlab,&eindex,symtype)
           },
           1  | 3 => {
             format!("let _rr{0}_ = if let RetTypeEnum::Enumvariant_{1}(_rr_{1})=parser.popstack().value {{ _rr_{1} }} else {{<{2}>::default()}}; let mut {0} = parser.lbx({3},_rr{0}_); ",&poppedlab,&eindex,symtype,k-1)
           },
           2 => {
             format!("let ref mut {0} = if let RetTypeEnum::Enumvariant_{1}(_rr_{1})=parser.popstack().value {{ _rr_{1} }} else {{<{2}>::default()}}; ",poppedlab,&eindex,symtype)
           },
           _ => {
             format!("let {0} = if let RetTypeEnum::Enumvariant_{1}(_rr_{1})=parser.popstack().value {{ _rr_{1} }} else {{<{2}>::default()}}; ",poppedlab,&eindex,symtype)
           },
        };
        write!(fd,"{}",&stat)?;
        k-=1;
      } // while k>0
      // form args
      let mut aargs = String::new();
      k = actualargs.len();
      while k>0
      {
        aargs.push(',');
        aargs.push_str(&actualargs[k-1]);
        k-=1;
      }
      /// formed actual arguments
    // write code to call action function, then convert to RetTypeEnum
      let lhsi = self.Gmr.Symhash.get(&self.Gmr.Rules[ri].lhs.sym).expect("GRAMMAR REPRESENTATION CORRUPTED");
      let fnname = format!("_rrsemaction_{}_",ri);
      let typei = &self.Gmr.Symbols[*lhsi].rusttype;
      let enumindex = self.Gmr.enumhash.get(typei).expect("FATAL ERROR: TYPE {typei} NOT USED IN GRAMMAR");
      write!(fd," RetTypeEnum::Enumvariant_{}({}(parser{})) }};\n",enumindex,&fnname,aargs)?;
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
      write!(fd,"pub fn parse_with{}(parser:&mut ZCParser<RetTypeEnum{},{}>, lexer:&mut {}) -> Result<{},{}>\n{{\n",lexerlt,&ltopt,extype,&lexername,absyn,absyn)?;
      write!(fd,"  lexer.shared_state = Rc::clone(&parser.shared_state);\n")?;
      write!(fd,"  if let RetTypeEnum::Enumvariant_{}(_xres_) = parser.parse(lexer) {{\n",abindex)?;
      write!(fd,"     if !parser.error_occurred() {{Ok(_xres_)}} else {{Err(_xres_)}}\n  }} ")?;
      write!(fd,"else {{ Err(<{}>::default())}}\n}}//parse_with public function\n",absyn)?;
      // training version
      write!(fd,"\npub fn parse_train_with{}(parser:&mut ZCParser<RetTypeEnum{},{}>, lexer:&mut {}, parserpath:&str) -> Result<{},{}>\n{{\n",lexerlt,&ltopt,extype,&lexername,absyn,absyn)?;
      write!(fd,"  lexer.shared_state = Rc::clone(&parser.shared_state);\n")?;
      write!(fd,"  if let RetTypeEnum::Enumvariant_{}(_xres_) = parser.parse_train(lexer,parserpath) {{\n",abindex)?;
      write!(fd,"     if !parser.error_occurred() {{Ok(_xres_)}} else {{Err(_xres_)}}\n  }} ")?;
      write!(fd,"else {{ Err(<{}>::default())}}\n}}//parse_train_with public function\n",absyn)?;


      ////// WRITE ENUM (test)
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

  fn is_lba(t:&str) -> bool {
   t.trim().starts_with("LBox") && t.contains("Any") && t.contains('<') && t.contains('>')
  }//is_lba to check type

// decode a grammar label, first return value is type of the label
// 0=direct
// 1=boxed
// 2= &mut   like in e@..@
// 3= &mut box  like in [e]@..@
// 4= no distinct label, @..@ without name
// k = position of argument of rhs 0 = first
pub fn decode_label(label:&str,k:usize) -> (u8,String)
{
  let mut plab = format!("_item{}_",k);
  if label.len()==0 {return (0,plab);}
  let mut boxedlabel = false;  // see if named label is of form [x]
  let findat = label.find('@');
  let mut ltype = 0;
  match &findat {
     None if label.len()>0 /*&& !gsym.label.contains('(')*/ => {
            let truelabel = checkboxlabel(label);
            boxedlabel = truelabel != label; 
            plab = String::from(truelabel);
            if boxedlabel {ltype=1;} /* else {ltype=0;} */
          },
    Some(ati) if *ati==0 => { ltype=4; },
    Some(ati) if *ati>0 => {
            let rawlabel = label[0..*ati].trim();
            let truelabel = checkboxlabel(rawlabel);
            boxedlabel = truelabel != rawlabel;
            if boxedlabel {ltype=3;} else {ltype=2;}
            plab = String::from(truelabel);
          },
    _ => {},
  }//match
  if ltype>1
    {eprintln!("\nWARNING: @..@ PATTERNS MUST BE IRREFUTABLE WITH THE -lrsd OPTION\n");}
  //if plab.starts_with("NEW") {plab=format!("_item{}_",k);}
  (ltype,plab)
}//decode_label
