#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
#![allow(unused_doc_comments)]
#![allow(unused_imports)]
#![allow(non_upper_case_globals)]
//use std::fmt::Display;
//use std::default::Default;
use std::collections::{HashMap,HashSet,BTreeSet};
use std::cell::{RefCell,Ref,RefMut};
use std::hash::{Hash,Hasher};
use std::io::{self,Read,Write,BufReader,BufRead};
use crate::grammar_processor::*;


// implemented marked delaying transformations.
impl Grammar
{
  // this must be called before start symbol, eof and startrule added to grammar!
  pub fn delay_transform(&mut self)
  {
  /*
    if self.delaymarkers.iter().next().is_none() {return;}
    self.Rulesfor.remove(&(self.Symbols.len()-2)); // start rule for START
    self.Symhash.remove("START");
    self.Symhash.remove("EOF");
    let mut eofterm = self.Symbols.pop().unwrap();
    let mut startnt = self.Symbols.pop().unwrap();
    let startrule = self.Rules.pop().unwrap();
  */
    let mut ntcx = self.ntcxmax+1;
    for (ri, delaymarks) in self.delaymarkers.iter() {
     for (dbegin,dend) in delaymarks.iter() {
       // check if first symbol at marker is a nonterminal
       let NT1 = &self.Rules[*ri].rhs[*dbegin];
       if NT1.terminal {
         eprintln!("WARNING: STARTING DELAY MARKER MUST PRECEED NONTERMINAL SYMBOL, RULE {} IN GRAMMAR.  MARKERS IGNORED",ri); continue;
       }// NT1 is non-terminal
       // construct suffix delta to be added to each rule
       let mut delta = Vec::new();
//println!("!!!!dbegin:{}, dend:{}, ri:{}",dbegin,dend,ri);
//printrule(&self.Rules[*ri],*ri);
       for i in dbegin+1..*dend {
         delta.push(self.Rules[*ri].rhs[i].clone());
       }
       // construct new nonterminal name ([Mdelta])
       let mut newntname = format!("NEWDELAYNT_{}",&NT1.sym);
       for s in &delta {newntname.push_str(&format!("_{}",&s.index));}
       // check that no such name already exists
       // construct new nonterminal
       let mut newnt = Gsym::new(&newntname,false);
       if let Some(nti) = self.Symhash.get(&newntname) {
          newnt = self.Symbols[*nti].clone();
       } else { // really new

         if self.sametype || !self.genabsyn {newnt.rusttype = self.Absyntype.clone();}  else {
            let mut nttype = String::from("(");
            for i in *dbegin .. *dend {
             let rsymi = self.Rules[*ri].rhs[i].index;
             nttype.push_str(&format!("{},",&self.Symbols[rsymi].rusttype));
            }
            nttype.push(')');
            self.enumhash.insert(nttype.clone(),ntcx); ntcx+=1;
            newnt.rusttype = nttype;
        }// form type of newnt
//println!("newnttype for {} is {}",&newnt.sym, &newnt.rusttype);         
         newnt.index = self.Symbols.len();
         self.Symbols.push(newnt.clone());
         self.Symhash.insert(newntname.clone(),self.Symbols.len()-1);

         let NTrules:Vec<_> = self.Rulesfor.get(&NT1.index).unwrap().iter().collect();
         let mut rset = HashSet::new(); // rules set for newnt (delayed nt)
         for ntri in NTrules {
           // create new rule
           let mut newrule = Grule::from_lhs(&newnt);
           newrule.rhs = self.Rules[*ntri].rhs.clone();
           for d in &delta { newrule.rhs.push(d.clone()); } //extend

           //////// set semantic action for new rule.
           // need to call/refer action for original rule for NT1
           // need to form a tuple.
           // internal variable symbol.
           let newvar = format!("_del_{}_{}_",&newnt.index,dbegin);
// check for return at end of last action.

           let mut actionri = format!(" let {} = {{ {}; ",&newvar,self.Rules[*ntri].action); // retrieves value from original action.
           // need to assign values to new items added to delta
           // they will be popped off of the stack by parser_writer as
           // item2, item1 item0...  because parser writer will write an action
           // for the extended rule. [Mc] --> abc
           
           let mut dtuple = format!("({},",&newvar);
           let mut labi = self.Rules[*ntri].rhs.len(); // original rule rhs len
           for sym in &delta {
             let defaultlabel =format!("_item{}_",&labi); 
             let slabel = if sym.label.len()>0 {checkboxlabel(&sym.label)}
               else {&defaultlabel};
             dtuple.push_str(&format!("{},",slabel));
             labi+=1;
           }
           actionri.push_str(&format!("{}) }}",&dtuple));  //rparen added here.
           newrule.action = actionri;

           if self.tracelev>1 {
             print!("COMBINED DELAY RULE: ");
             printrule(&newrule,self.Rules.len());
           }

           self.Rules.push(newrule);
           rset.insert(self.Rules.len()-1);
         }// for each rule for this NT1 to be delayed, add suffix
         self.Rulesfor.insert(newnt.index,rset);
       } // newnt is actually a new symbol, else it and its rules exist
       // change original rule ri to refer to newnt
       let mut newrhs = Vec::with_capacity(self.Rules[*ri].rhs.len()-1);
       if *dbegin>0 {
         for i in 0..*dbegin {newrhs.push(self.Rules[*ri].rhs[i].clone());}
       }
       let mut clonenewnt = newnt.clone();
       let ntlabel = format!("_delayeditem{}_",dbegin);
       clonenewnt.label = ntlabel.clone();
       newrhs.push(clonenewnt); // newnt added to rule!
       for i in *dend .. self.Rules[*ri].rhs.len() {
         newrhs.push(self.Rules[*ri].rhs[i].clone());
       }

       /////// change semantic action of original rule.
       let mut newaction = String::from(" ");
       // break up tuple
       //let mut labi = 0;
       for i in *dbegin..*dend {
          let defaultlab = format!("_item{}_",i);
          let symi = &self.Rules[*ri].rhs[i]; // original rule
          let labeli = if symi.label.len()>0 {checkboxlabel(&symi.label)}
            else {&defaultlab};
          newaction.push_str(&format!("let mut {} = {}.{}; ",labeli,&ntlabel,i-dbegin));
          //labi+=1;
       }// break up tuple
       // anything to do with the other values?  they have labels, but indexes
       // may be off - but original actions will refer to them as-is.
       newaction.push_str(&self.Rules[*ri].action);
       self.Rules[*ri].rhs = newrhs; // change rhs of rule
       self.Rules[*ri].action = newaction;
       
       if self.tracelev>1 {
         print!("TRANSFORMED RULE FOR DELAY: ");
         printrule(&self.Rules[*ri],*ri);
       }
       
     } // for each pair of delay marks assume dend>dbegin+1
    }//for each rule

  }// delay_transform
} // transformation



//// generic structure bijective hashmap
#[derive(Default,Debug)]
pub struct Bimap<TA:Hash+Default+Eq+Clone, TB:Hash+Default+Eq+Clone>
{
  pub forward: HashMap<TA,TB>,
  pub backward: HashMap<TB,TA>,
}
impl<TA:Hash+Default+Eq+Clone, TB:Hash+Default+Eq+Clone> Bimap<TA,TB>
{
  pub fn new() -> Self {
    Bimap { forward:HashMap::new(), backward:HashMap::new() }
  }
  pub fn with_capacity(cap:usize) -> Self {
    Bimap { forward:HashMap::with_capacity(cap), backward:HashMap::with_capacity(cap) }  
  }
  pub fn insert(&mut self, x:TA,y:TB) -> bool {
    let (x2,y2) = (x.clone(),y.clone());
    let fopt = &self.forward.remove(&x);
    if let Some(y0) = fopt {
      self.backward.remove(y0);
    }
    let bopt = &self.backward.remove(&y);
    if let Some(x0) = bopt {
      self.forward.remove(x0);
    }
    self.forward.insert(x,y);
    self.backward.insert(y2,x2);
    fopt.is_none() && bopt.is_none()
  }//insert
  pub fn get(&self,x:&TA) -> Option<&TB> { self.forward.get(x) }
  pub fn get_mut(&mut self,x:&TA) -> Option<&mut TB> { self.forward.get_mut(x) }
  pub fn rget(&self,x:&TB) -> Option<&TA> { self.backward.get(x) }
  pub fn rget_mut(&mut self,x:&TB) -> Option<&mut TA> { self.backward.get_mut(x) }
  pub fn len(&self)->usize {self.forward.len()}
  pub fn delete(&mut self, x:&TA) -> Option<TB> {
    if let Some(y) = self.forward.remove(x) {
      self.backward.remove(&y);
      Some(y)
    } else {None}
  }
  pub fn rdelete(&mut self, x:&TB) -> Option<TA> {
    if let Some(y) = self.backward.remove(x) {
      self.forward.remove(&y);
      Some(y)
    } else {None}
  }
  pub fn keys(&self) -> std::collections::hash_map::Keys<'_,TA,TB> {
    self.forward.keys()
  }
  pub fn rkeys(&self) -> std::collections::hash_map::Keys<'_,TB,TA> {
    self.backward.keys()
  }
  pub fn iter(&self) -> std::collections::hash_map::Iter<'_,TA,TB> {
    self.forward.iter()
  }
}//impl Bimap
// will be used to map nonterminal symbols to vectors of symbols


////////////////////////////////////////////////////////////////////////////
// Experimental module to implement selML(k,1) parsers introduced roughly by
// Bertsch, Nederhof and Schmitz.

// nonterminals consists of a symbol plus a fixed k-size array of symbols.
// symbol unused represents nothing and allows us to use fixed arrays.

// usize is the type of grammar symbols (as an index)
/*
use crate::grammar_processor::*;
use crate::selmlk::GSymbol::*;

//pub struct Nonterminal<const K:usize>(usize,[usize;K]);
#[derive(Copy,Clone,Debug,Hash,Ord,PartialOrd,Eq,PartialEq)]
pub enum GSymbol<const K:usize> {
   Terminal(usize),
   Nonterminal(usize,[usize;K]),
}
impl<const K:usize> GSymbol<K>
{
   fn tostr(&self, Gmr:&Grammar) -> String
   {
      match self {
        Terminal(ti) => Gmr.Symbols[*ti].sym.clone(),
        Nonterminal(ni,D) => {
           let mut s = format!("[{},",&Gmr.Symbols[*ni].sym);
           for ti in D {
             if *ti == Hash {s.push('#');}
             else { s.push_str(&Gmr.Symbols[*ti].sym); s.push(','); }
           }
           s.push(']'); s
        },
      }//match
   }//tostr
}
// a special usize index, perhaps 0 or usize::MAX, will represent a dummy
// filler so we can have fixed size arrays and const generics.

const Hash:usize = usize::MAX;
//const HASH:GSymbol = GSymbol::Terminal(Hash);
//static Hashes<const K:usize> = [Hash;K];

//compile time production
pub struct Production<const K:usize> {
  pub lhs: GSymbol<K>, 
  pub rhs: Vec<GSymbol<K>>,
}

// use these on top of grammar_processor constructs

// semantic values
#[derive(Copy,Clone,Debug)]
pub struct Values<AT:Default, const K:usize>([AT;K]);
*/