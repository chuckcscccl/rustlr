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
use crate::Stateaction::*;
pub const DEFAULTPRECEDENCE:i32 = 20;

pub const TRACE:usize = 0;   // 0 means don't trace

#[derive(Clone)]
pub struct Gsym // struct for a grammar symbol
{
  pub sym : String,
  pub rusttype : String,  //used only to indicate "mut"
  pub terminal : bool,
  pub label : String,  // object-level variable holding value
  pub precedence : i32,   // negatives indicate right associativity
}

impl Gsym
{
  pub fn new(s:&str,isterminal:bool) -> Gsym // compile time
  {
    Gsym {
      sym : s.to_owned(),
      terminal : isterminal,
      label : String::default(),
      rusttype : String::from("String"),
      precedence : DEFAULTPRECEDENCE, // + means left, - means right
    }
  }
  pub fn setlabel(&mut self, la:&str)
  { self.label = String::from(la); }
  pub fn settype(&mut self, rt:&str)
  { self.rusttype = String::from(rt); }
  pub fn setprecedence(&mut self, p:i32)
  { self.precedence = p; }
}// impl for Gsym


//Grammar Rule structure
// This will be used only statically: the action is a string.
// The Gsym structures are repeated on the right-hand side because each
// one can have a different label
pub struct Grule  // struct for a grammar rule
{
  pub lhs : Gsym,  // left-hand side of rule
  pub rhs : Vec<Gsym>, // right-hand side symbols (cloned from Symbols)
  pub action : String, //string representation of Ruleaction
  pub precedence : i32, // set to rhs symbol with highest |precedence|
//  pub Ruleaction : fn(&mut Vec<Stackelement<AT>>) -> AT, //takes stack as arg
}
impl Grule
{
  pub fn new_skeleton(lh:&str) -> Grule
  {
     Grule {
       lhs : Gsym::new(lh,false),
       rhs : Vec::new(),
       action : String::default(),
       precedence : 0,   
     }
  }
}//impl Grule

pub fn printrule(rule:&Grule)  //independent function
{
   print!("PRODUCTION: {} --> ",rule.lhs.sym);
   for s in &rule.rhs {
      print!("{}",s.sym);
      if s.label.len()>0 {print!(":{}",s.label);}
      print!(" ");
   }
   println!("{{ {}, preclevel {}",rule.action,rule.precedence);  // {{ is \{
}

/////main global class, roughly corresponds to "metaparser"
pub struct Grammar
{
  pub name : String,
  pub Symbols : Vec<Gsym>,
  pub Symhash : HashMap<String,usize>,
  pub Rules: Vec<Grule>,
  pub topsym : String,
  pub Nullable : HashSet<String>,
  pub First : HashMap<String,HashSet<String>>,
  pub Rulesfor: HashMap<String,HashSet<usize>>,  //rules for a non-terminal
  pub Absyntype : String,     // string name of abstract syntax type
  pub Externtype : String,    // type of external structure
  pub Extras : String,        // indicated by {% .. %}, mostly  use ...
}

/* 
Metaparser: Parses grammar spec consistent with 2014 'Myocc' program
*/

impl Grammar
{
  pub fn new() -> Grammar
  {
     Grammar {
       name : String::from(""),       // name of grammar
       Symbols: Vec::new(),           // grammar symbols
       Symhash: HashMap::new(),       
       Rules: Vec::new(),                 // production rules
       topsym : String::default(),        // top symbol
       Nullable : HashSet::new(),
       First : HashMap::new(),
       Rulesfor: HashMap::new(),
       Absyntype:String::from("i64"), //default(),
       Externtype:String::from(""),   // default unused field
       Extras: String::new(),
     }
  }//new grammar

  pub fn nonterminal(&self,s:&str) -> bool
  {
     match self.Symhash.get(s) {
        Some(symi) => !self.Symbols[*symi].terminal,
	_ => false,
     }
  }
  pub fn terminal(&self,s:&str) -> bool
  {
     match self.Symhash.get(s) {
        Some(symi) => self.Symbols[*symi].terminal,
	_ => false,
     }
  }

////// meta (grammar) parser
/* does not recognize {%  and %}, or multi-line comments */
  pub fn parse_grammar(&mut self, filename:&str)
  {
     let mut reader =  match File::open(filename) {
       Ok(f) => { Some(BufReader::new(f)) },
       _ => { println!("cannot open file, reading from stdin..."); None},
     };//match

     let mut line=String::from("");
     let mut atEOF = false;
     let mut linenum = 0;
     let mut linelen = 0;
     let mut stage = 0;
     let mut multiline = false;  // multi-line mode with ==>, <==
     let mut foundeol = false;
     while !atEOF
     {
       if !multiline {line = String::new();}
       if foundeol { multiline=false;} //use current line
       else {
         let result = if let Some(br)=&mut reader {br.read_line(&mut line)}
                      else {std::io::stdin().read_line(&mut line)};
         match result {
            Ok(0) | Err(_) => { line = String::from("EOF"); },
  	    Ok(n) => {linenum+=1;},
         }//match
       }// did not find line
       
       linelen = line.len();
       
       if multiline && linelen>1 && &line[0..1]!="#" {
          // keep reading until <== found
          if linelen==3 && &line[0..3]=="EOF" {
            panic!("MULTI-LINE GRAMMAR PRODUCTION DID NOT END WITH <==");
          }
          match line.rfind("<==") {
            None => {}, // keep reading, add to line buffer
            Some(eoli) => {
               line.truncate(eoli);
               foundeol = true;
            }
          }//match
       }
       else if linelen>1 && &line[0..1]=="!" {
           self.Extras.push_str(&line[1..]);
           //self.Extras.push_str("\n");                                      
       }
       else if linelen>1 && &line[0..1]!="#" {
         let toksplit = line.split_whitespace();
         let stokens:Vec<&str> = toksplit.collect();
         match stokens[0] {
            "use" => {
              self.Extras.push_str("use ");
              self.Extras.push_str(stokens[1]);
              self.Extras.push_str("\n");
            },
            "extern" if stokens.len()>2 && stokens[1]=="crate" => {
              self.Extras.push_str("extern crate ");
              self.Extras.push_str(stokens[2]);
              self.Extras.push_str("\n");              
            },
            "!" => {
               let pbi = line.find('!').unwrap();
               self.Extras.push_str(&line[pbi+1..]);
               self.Extras.push_str("\n");                             
            },
            "gramname" | "grammarname" | "grammar" => {
               self.name = String::from(stokens[1]);
            },
            "EOF" => {atEOF=true},
            ("terminal" | "terminals") if stage==0 => {
               for i in 1..stokens.len() {
	          let newterm = Gsym::new(stokens[i],true);
                  self.Symhash.insert(stokens[i].to_owned(),self.Symbols.len());
                  self.Symbols.push(newterm);
                  //self.Symbols.insert(stokens[i].to_owned(),newterm);
		  if TRACE>2 {println!("terminal {}",stokens[i]);}
               }
            }, //terminals
	    "typedterminal" if stage==0 => {
	       let mut newterm = Gsym::new(stokens[1],true);
	       if stokens.len()>2 {newterm.settype(stokens[2]);}
               self.Symhash.insert(stokens[1].to_owned(),self.Symbols.len());
               self.Symbols.push(newterm);               
	       //self.Symbols.insert(stokens[1].to_owned(),newterm);
   	       //if TRACE>2 {println!("typedterminal {}:{}",stokens[1],stokens[2]);}
	    }, //typed terminals
	    "nonterminal" if stage==0 => {
	       let mut newterm = Gsym::new(stokens[1],false);
	       if stokens.len()>2 {newterm.settype(stokens[2]);}
               self.Symhash.insert(stokens[1].to_owned(),self.Symbols.len());
               self.Symbols.push(newterm);                              
	    }, //nonterminals
            "nonterminals" if stage==0 => {
               for i in 1..stokens.len() {
	          let newterm = Gsym::new(stokens[i],false);
                  self.Symhash.insert(stokens[i].to_owned(),self.Symbols.len());
                  self.Symbols.push(newterm);
		  if TRACE>2 {println!("nonterminal {}",stokens[i]);}
               }
            },
	    "topsym" | "startsymbol" if stage==0 => {
               match self.Symhash.get(stokens[1]) {
                 Some(tsi) if *tsi<self.Symbols.len() && !self.Symbols[*tsi].terminal => {
              	    self.topsym = String::from(stokens[1]);
                 },
                 _ => { panic!("top symbol {} not found in declared non-terminals; check ordering of declarations, line {}",stokens[1],linenum);
                 },
               }//match
	       //if TRACE>4 {println!("top symbol is {}",stokens[1]);}
	    }, //topsym
            "absyntype" | "valuetype" if stage==0 => {
               self.Absyntype = String::from(stokens[1]);
	       if TRACE>2 {println!("abstract syntax type is {}",stokens[1]);}
            },
            "externtype" | "externaltype" if stage==0 => {
               self.Externtype = String::from(stokens[1]);
	       if TRACE>2 {println!("external structure type is {}",stokens[1]);}
            },            
	    "left" | "right" if stage<2 => {
               if stage==0 {stage=1;}
	       let mut preclevel:i32 = 0;
	       if let Ok(n)=stokens[2].parse::<i32>() {preclevel = n;}
               else {panic!("did not read precedence level on line {}",linenum);}
	       if stokens[0]=="right" && preclevel>0 {preclevel = -1 * preclevel;}
               if let Some(index) = self.Symhash.get(stokens[1]) {
	         //let gsym = self.Symbols.get_mut(index);
	         //if let Some(sym)=gsym { sym.precedence = preclevel; }
                 self.Symbols[*index].precedence = preclevel;
               }
	    }, // precedence and associativity
	    "recover" | "flexname" | "resync" => {}, //not covered
	    LHS if (stokens[1]=="-->" || stokens[1]=="::=" || stokens[1]=="==>") => {
              if !foundeol && stokens[1]=="==>" {multiline=true; continue;}
              else if foundeol {foundeol=false;}
              // println!("RULE {}",&line); 
              if stage<2 {stage=2;}
	    // construct lhs symbol
              let symindex = match self.Symhash.get(LHS) {
                Some(smi) if *smi<self.Symbols.len() && !self.Symbols[*smi].terminal => smi,
                _ => {panic!("unrecognized non-terminal symbol {}, line {}",LHS,linenum);},
              };
              let lhsym = self.Symbols[*symindex].clone();
	      //let lhsym = self.Symbols.get(LHS).unwrap().clone();
              let mut rhsyms:Vec<Gsym> = Vec::new();
	      let mut semaction = "}";
	      let mut i:usize = 2;
              let mut maxprec:i32 = 0;
              while i<stokens.len() {
	        let strtok = stokens[i];
		i+=1;
		if strtok == "{"  {
                   //let ll:Vec<&str> = line.split('{').collect();
		   //semaction = ll[1];
                   let position = line.find('{').unwrap();
                   semaction = line.split_at(position+1).1;
		   break;
                }
		let toks:Vec<&str> = strtok.split(':').collect();
if TRACE>2&&toks.len()>1 {println!("see labeled token {}",strtok);}		
		match self.Symhash.get(toks[0]) {
		   None => {panic!("unrecognized grammar symbol {}, line {}",toks[0],linenum); },
		   Some(symi) => {
                     let sym = &self.Symbols[*symi];
		     let mut newsym = sym.clone();
		     if toks.len()>1 { newsym.setlabel(toks[1]); }
                     if maxprec.abs() < newsym.precedence.abs()  {
                        maxprec=newsym.precedence;
                     }
		     rhsyms.push(newsym);
                   }
                }//match
	      } // while there are tokens on rhs
	      // form rule
	      let rule = Grule {
	        lhs : lhsym,
		rhs : rhsyms,
		action: semaction.to_owned(),
		precedence : maxprec,
//                Ruleaction : |p|{AT::default()}, //will be changed when writing parser
	      };
	      if TRACE>2 {printrule(&rule);}
	      self.Rules.push(rule);
            }, //production            	    	    	    
            _ => {panic!("error parsing grammar on line {}, grammar stage {}",linenum,stage);},  
         }//match first word
       }// not an empty or comment line
     } // while !atEOF
     if self.Symhash.contains_key("START") || self.Symhash.contains_key("EOF")
     {
        panic!("Error in grammar: START and EOF are reserved symbols");
     }
     // add start,eof and starting rule:
     let startnt = Gsym::new("START",false);
     let eofterm =  Gsym::new("EOF",true);
     self.Symhash.insert(String::from("START"),self.Symbols.len());
     self.Symhash.insert(String::from("EOF"),self.Symbols.len()+1);     
     self.Symbols.push(startnt.clone());
     self.Symbols.push(eofterm.clone());
     //self.Symbols.insert(String::from("START"),startnt.clone());
     //self.Symbols.insert(String::from("EOF"),eofterm.clone());
     let topgsym = &self.Symbols[*self.Symhash.get(&self.topsym).unwrap()];
     let startrule = Grule {  // START-->topsym EOF
        lhs:startnt,
        rhs:vec![topgsym.clone()], //,eofterm],  //eofterm is lookahead
        action: String::default(),
        precedence : 0,
//        Ruleaction: |p|{AT::default()}, //{p.Parsestack.pop().unwrap().value},
     };
     self.Rules.push(startrule);  // last rule is start rule
     if TRACE>0 {println!("{} rules in grammar",self.Rules.len());}
     if self.Externtype.len()<1 {self.Externtype = self.Absyntype.clone();} ////***
  }//parse_grammar
}// impl Grammar
// last rule is always start rule and first state is start state

//////////////////////  Nullable

//// also sets the RulesFor map for easy lookup of all the rules for
//// a non-terminal
impl Grammar
{
  pub fn compute_NullableRf(&mut self)
  {
     let mut changed = true;
     let mut rulei:usize = 0;
     while changed 
     {
       changed = false;
       rulei = 0;
       for rule in &self.Rules 
       {
          let mut addornot = true;
          for gs in &rule.rhs 
          {
             if gs.terminal || !self.Nullable.contains(&gs.sym) {addornot=false;}
          } // for each rhs symbol
	  if (addornot) {
             changed = self.Nullable.insert(rule.lhs.sym.clone()) || changed;
             if TRACE>3 {println!("{} added to Nullable",rule.lhs.sym);}
          }
          // add rule index to Rulesfor map:
          if let None = self.Rulesfor.get(&rule.lhs.sym) {
             self.Rulesfor.insert(rule.lhs.sym.clone(),HashSet::new());
          }
          let ruleset = self.Rulesfor.get_mut(&rule.lhs.sym).unwrap();
          ruleset.insert(rulei);
          rulei += 1;
       } // for each rule
     } //while changed
  }//nullable

  // calculate the First set of each non-terminal  (not used- use compute_FirstIM)
// with interior mutability, no need to clone HashSets. // USE THIS ONE!
  pub fn compute_FirstIM(&mut self)
  {
     let mut FIRST:HashMap<String,RefCell<HashSet<String>>> = HashMap::new();
     let mut changed = true;
     while changed 
     {
       changed = false;
       for rule in &self.Rules
       {
         let ref nt = rule.lhs.sym; // left symbol of rule is non-terminal
	 if !FIRST.contains_key(nt) {
            changed = true;
	    FIRST.insert(String::from(nt),RefCell::new(HashSet::new()));
         } // make sure set exists for this non-term
	 let mut Firstnt = FIRST.get(nt).unwrap().borrow_mut();
	 // now look at rhs
	 let mut i = 0;
	 let mut isnullable = true;
 	 while i< rule.rhs.len() && isnullable
         {
            let gs = &rule.rhs[i];
	    if gs.terminal {
	      changed=Firstnt.insert(gs.sym.clone()) || changed;
//if TRACE>2 {println!("{} added to First set of {}",gs.sym,nt);}
              isnullable = false;
            }
            else if &gs.sym!=nt {   // non-terminal
              if let Some(firstgs) = FIRST.get(&gs.sym) {
                  let firstgsb = firstgs.borrow();
                  for sym in firstgsb.iter() {
                    changed=Firstnt.insert(sym.clone())||changed;
                  }
              } // if first set exists for gs
            } // non-terminal 
           if gs.terminal || !self.Nullable.contains(&gs.sym) {isnullable=false;}
	    i += 1;
         } // while loop look at rhs until not nullable
       } // for each rule
     } // while changed
     // Eliminate RefCells and place in self.First
     for nt in FIRST.keys() {
        if let Some(rcell) = FIRST.get(nt) {
          self.First.insert(nt.to_owned(),rcell.take());
        }
     }
  }//compute_FirstIM


  // First set of a sequence of symbols
  pub fn Firstseq(&self, Gs:&[Gsym], la:&str) -> HashSet<String>
  {
     let mut Fseq = HashSet::new();
     let mut i = 0;
     let mut nullable = true;
     while nullable && i<Gs.len() 
     {
         if (Gs[i].terminal) {Fseq.insert(Gs[i].sym.clone()); nullable=false; }
	 else  // Gs[i] is non-terminal
         {
            let firstgsym = self.First.get(&Gs[i].sym).unwrap();
	    for s in firstgsym { Fseq.insert(s.to_owned()); }
	    if !self.Nullable.contains(&Gs[i].sym) {nullable=false;}
         }
	 i += 1;
     }//while
     if nullable {Fseq.insert(la.to_owned());}
     Fseq
  }//FirstSeq

}//impl Grammar continued


/////////////// LR state machine

//actions are: shift, reduce, accept, gotonext

#[derive(Clone,PartialEq,Eq,Hash,Debug)]
pub struct LRitem
{
   ri: usize, // rule index
   pi: usize, // position of dot
   la: String, // lookahead
   //interior : bool,  // can't have this here if deriving Eq
}
pub fn printrulela(ri:usize, Gmr:&Grammar, la:&str)
{
     if ri>=Gmr.Rules.len() {println!("printing invalid rule number {}",ri); return;}
     let ref lhs_sym = Gmr.Rules[ri].lhs.sym;
     let ref rhs = Gmr.Rules[ri].rhs;
     print!("  (Rule {}) {} --> ",ri,lhs_sym);
     for gsym in rhs  { print!("{} ",gsym.sym); }
     println!(" , lookahead {}",la);
}
pub fn printitem(item:&LRitem, Gmr:&Grammar)
{
     let ref lhs_sym = Gmr.Rules[item.ri].lhs.sym;
     let ref rhs = Gmr.Rules[item.ri].rhs;
     print!("  ({}) {} --> ",item.ri,lhs_sym);
     let mut position = 0;
     for gsym in rhs 
     {
       if &position==&item.pi {print!(".");}
       print!("{} ",gsym.sym);
       position+=1;
     }
     if &position==&item.pi {print!(". ");}
     println!(", {}",&item.la);  
}// printitem

// representation of each LR1 state
pub type Itemset = HashSet<LRitem>;
// check if two states are the same
pub fn stateeq(s1:&Itemset, s2:&Itemset) -> bool
{
   if s1.len()!=s2.len() { return false; }
   for s in s1 {
      if !s2.contains(s) {return false;}
   }
   return true;
}//stateeq

fn extract_core(items:&Itemset) -> HashSet<(usize,usize)> // for lalr
{
   let mut core0 = HashSet::new();
   for LRitem{ri:r, pi:p, la} in items  { core0.insert((*r,*p)); }
   core0
}

// checks if every item core in s1 is also in s2, for LALR
fn sub_core(s1:&Itemset, s2:&Itemset) -> bool // not used
{
   for LRitem{ri:r1,pi:p1,la:la1} in s1
   {
      let mut bx = false;
      for LRitem{ri:r2,pi:p2,la} in s2
      {
         if r1==r2 && p1==p2 {bx=true; break;}
      }
      if !bx {return false;}
   }
   return true;
}//sub_core

fn eq_core(s1:&Itemset, s2:&Itemset) -> bool 
{
   let (core1,core2) = (extract_core(s1),extract_core(s2));
   if core1.len()!=core2.len() {return false;}
   for item_core in &core1
   {
      if !core2.contains(item_core) {return false; }
   }
   return true;
}//eq_core

#[derive(Clone,Debug)]
pub struct LR1State
{
   index: usize, // index into vector
   items:Itemset,
   lhss: BTreeSet<String>,  // set of left-side non-terminals
}
impl LR1State
{
  pub fn new() -> LR1State
  {
     LR1State {
        index : 0,   // need to change
        items : HashSet::new(),
        lhss: BTreeSet::new(),
     }
  }
  pub fn insert(&mut self, item:LRitem, lhs:&str) -> bool
  {
     let inserted = self.items.insert(item);
     self.lhss.insert(String::from(lhs));
     inserted
  }
  pub fn hashval(&self) -> String  // note: NOT UNIQUE
  {
    let mut key=self.items.len().to_string(); // better for lr1
    for s in &self.lhss {key.push_str(s);}
    key    
  }  
  pub fn hashval_lalr(&self) -> String  // note: NOT UNIQUE
  {
    let mut key = extract_core(&self.items).len().to_string(); // lr1/lalr
    for s in &self.lhss {key.push_str(s);}
    key    
  }
  pub fn contains(&self, x:&LRitem) -> bool {self.items.contains(x)}

  fn core_eq(&self, state2:&LR1State) -> bool // for LALR
  { eq_core(&self.items,&state2.items) }
    //{ sub_core(&self.items,&state2.items) && sub_core(&state2.items,&self.items) }

  fn merge_states(&mut self, state2:&LR1State) // not used
  {
      for item in &state2.items {self.items.insert(item.clone());}
  }//merge_states

}// basics ofr LR1State

impl PartialEq for LR1State
{
   fn eq(&self, other:&LR1State) -> bool
   {stateeq(&self.items,&other.items)}
   fn ne(&self, other:&LR1State) ->bool
   {!stateeq(&self.items,&other.items)}
}
impl Eq for LR1State {}
// Hash for LR1 state no longer implemented

// independent function for tracing
pub fn printstate(state:&LR1State,Gmr:&Grammar) 
{
  println!("state {}:",state.index);
  for item in &state.items
  { printitem(item,Gmr); }
}//printstate


pub fn stateclosure(mut state:LR1State, Gmr:&Grammar)
  -> LR1State // consumes and returns new state
{
  //algorithm is like that of a spanning tree
  let mut closed =LR1State::new();  // closed set,
  closed.index = state.index;
  while state.items.len()>0
  {  
     //if TRACE>2 {printstate(&state,Gmr);}
     let nextitem = state.items.iter().next().unwrap().clone();
     let item = state.items.take(&nextitem).unwrap();
     let (ri,pi,la) = (item.ri,item.pi,&item.la);
     let rulei = &Gmr.Rules[ri]; //.get(ri).unwrap();
     let lhs = &rulei.lhs.sym;
     closed.insert(nextitem,lhs); // place item in interior
     if pi<rulei.rhs.len() && !rulei.rhs[pi].terminal {
       let nti = &rulei.rhs[pi]; // non-terminal after dot (Gsym)
       let lookaheads=&Gmr.Firstseq(&rulei.rhs[pi+1..],la);  
       for rulent in Gmr.Rulesfor.get(&nti.sym).unwrap() //rulent:usize
       {
          for lafollow in lookaheads 
          { 
            //if TRACE>2 {println!("adding new item for la {}",&lafollow);}
            let newitem = LRitem {
               ri: *rulent,
               pi: 0,
               la: lafollow.clone(),
            };
            if !closed.items.contains(&newitem)  {
              state.insert(newitem,&nti.sym); // add to "frontier"
//if TRACE>2 {println!("added new item of rule {}, la {}",&rulent,&lafollow);}
            }
          }//for each possible lookahead following non-terminal
       }// for each rule in this non-terminal                 
     } // add items to closure for this item
  }  // while not closed
  closed
}//stateclosure generation


//// Contruction of the FSM, which is a Vec<HashMap<String,stateaction>>

#[derive(Clone,PartialEq,Eq,Debug)]
pub enum Stateaction {
  Shift(usize),     // shift then got to state index
  Reduce(usize),    // reduce by rule index
  Gotonext(usize),  // folded into same table, only for non-terminals
  Accept,
  Error(String),
}

// abstract parser struct
pub struct Statemachine  // AT is abstract syntax (enum) type
{
   pub Gmr: Grammar,
   pub States: Vec<LR1State>, 
   pub Statelookup: HashMap<String,BTreeSet<usize>>,
   pub FSM: Vec<HashMap<String,Stateaction>>,
   pub lalr: bool,
   pub Open: Vec<usize>, // for LALR only, vector of unclosed states
}

impl Statemachine
{
  pub fn new(gram:Grammar) -> Statemachine
  { 
       Statemachine {
          Gmr: gram,
          States: Vec::with_capacity(8*1024), // reserve 8K states
          Statelookup: HashMap::with_capacity(1024),
          FSM: Vec::with_capacity(8*1024),
          lalr: false,
          Open: Vec::new(), // not used for lr1, set externally if lalr
       }
  }//new

  // psi is previous state index, nextsym is next symbol (may do lalr)
  fn addstate(&mut self, mut state:LR1State, psi:usize, nextsym:String)
  {  
     let newstateindex = self.States.len(); // index of new state
     state.index = newstateindex;
     let lookupkey = if self.lalr {state.hashval_lalr()} else {state.hashval()};
     if let None=self.Statelookup.get(&lookupkey) {
        self.Statelookup.insert(lookupkey.clone(),BTreeSet::new());
     }
     let indices = self.Statelookup.get_mut(&lookupkey).unwrap();
     let mut toadd = newstateindex; // defaut is add new state (will push)
     if self.lalr {
        for i in indices.iter()
        { 
           if state.core_eq(&self.States[*i]) {
             toadd=*i;
             let mut stateclone = LR1State {
                index : toadd,
                items : state.items.clone(),
                lhss: BTreeSet::new(),
             };
             stateclone.merge_states(&self.States[toadd]);
             if stateclone.items.len() > self.States[toadd].items.len() {
                self.States[toadd] = stateclosure(stateclone,&self.Gmr);
                // now need to call makegotos again on this state - add
                // to end of open vector.
                self.Open.push(toadd);
                if TRACE>3 { print!("===> MERGED STATE: ");
                    printstate(&self.States[toadd],&self.Gmr);
                }
             } // existing state extended, re-closed, but ...
             break;
           } // core_eq with another state  
        } // for each index in Statelookup to look at
     }// if lalr
     else {   // lr1
       for i in indices.iter()
       {
         if &state==&self.States[*i] {toadd=*i; break; } // state i exists
       }
     }// lalr or lr1

     if TRACE==2 {println!("transition to state {} from state {}, symbol {}..",toadd,psi,&nextsym);}
     if toadd==newstateindex {  // add new state
       if TRACE>2 {printstate(&state,&self.Gmr);}
       indices.insert(newstateindex); // add to StateLookup index hashset
       self.States.push(state);
       self.FSM.push(HashMap::new()); // always add row to fsm at same time
       if self.lalr {self.Open.push(newstateindex)}
     }// add new state

     // add to- or change FSM TABLE ...  only Shift or Gotnext added here.
     let gsymbol = &self.Gmr.Symbols[*self.Gmr.Symhash.get(&nextsym).unwrap()];
     let mut newaction = Stateaction::Gotonext(toadd);
     if gsymbol.terminal {newaction=Stateaction::Shift(toadd);}
     let currentaction = self.FSM[psi].get(&nextsym);
     let mut changefsm = true;
     match currentaction {   // detect shift-reduce conflict
       Some(Reduce(ri2)) =>  {
         let prec2 = self.Gmr.Rules[*ri2].precedence;
         let prec1 = gsymbol.precedence;
         if prec1==prec2 && prec1>0 {changefsm=false;} // assume left-associative
         else if prec2.abs()>prec1.abs() {changefsm=false;} // still reduce
         if TRACE>0 {println!("shift-reduce conflict resolved by operator precedence/associativity:"); printrulela(*ri2,&self.Gmr,&nextsym); /*printstate(&self.States[psi],&self.Gmr);*/}
       },
       Some(Accept) => {changefsm=false;},
       _ => {},
     }// match for conflict detection
     if changefsm {self.FSM[psi].insert(nextsym,newaction);}
     // set fsm
  }  //addstate

/*
 // LALR only: si is from makegoto&addstate, fsi is state to merge into 
    fn merge_states(FSM: &mut Vec<HashMap<String,Stateaction>>, States:&mut Vec<LR1State>, Gmr:&Grammar, si:usize, state2:&LR1State)
    {
       for item in &state2.items
       {
          //print!("LALR-checking if state {} contains {:?}: ",si,item);
          if !States[si].items.contains(item) {
              //println!("NO");
              // determine if this is a reduce item
              if item.pi >= Gmr.Rules[item.ri].rhs.len() {
                 if TRACE>1 {print!("LALR MERGE: ");}
                 Statemachine::addreduce(FSM,Gmr,item,si);
              }
              States[si].items.insert(item.clone());
          }// new item needs to be inserted
          //else {println!("yes");}
       }
       //for item in &state2.items {self.items.insert(item.clone());}
    }//merge_states
*/    

  // called by addstate and makegotos, only for reduce/accept situation
  // it assumes that the . is at the right end of the rule
  fn addreduce(FSM: &mut Vec<HashMap<String,Stateaction>>, Gmr:&Grammar, item:&LRitem, si:usize)
  {
     let currentaction = FSM[si].get(&item.la);
     let mut changefsm = true;
     let ri1 = &item.ri;
     /// detect CONFLICT HERE
     match currentaction {
        Some(Reduce(ri2)) if ri2<ri1 => {
           changefsm=false;
           println!("Reduce-Reduce Conflict conflicted detected between rules {} and {}, resolved in favor of {}",ri2,ri1,ri2);
           printrulela(*ri1,Gmr,&item.la);  printrulela(*ri2,Gmr,&item.la);
           //printstate(&self.States[si],Gmr);
        },
        Some(Reduce(ri2)) if ri2>ri1 => {
           println!("Reduce-Reduce Conflict conflicted detected between rules {} and {}, resolved in favor of {}",ri2,ri1,ri1);
           printrulela(*ri1,Gmr,&item.la);  printrulela(*ri2,Gmr,&item.la); 
           //printstate(&self.States[si],Gmr);            
        },
        Some(Reduce(ri2)) if ri2==ri1 => {changefsm=false;},
        Some(Accept) => {changefsm = false;},
        Some(Shift(_)) => {   // shift-reduce conflict
           let prec1 = Gmr.Rules[item.ri].precedence;
           let prec2 = Gmr.Symbols[*Gmr.Symhash.get(&item.la).unwrap()].precedence;

           if prec1==prec2 && prec1<0 {changefsm=false;} // assume right-associative
           else if prec2.abs()>prec1.abs() {changefsm=false;} // still shift 
           if TRACE>0 {println!("Shift-Reduce conflict resolved by operator precedence/associativity:"); printrulela(*ri1,Gmr,&item.la); }
        },
       _ => {},
     }//match to detect conflict
     if changefsm {   // only Reduce/Accept added here
        // accept or reduce
        if item.ri==Gmr.Rules.len()-1 && item.la=="EOF"  { // start rule
           FSM[si].insert(item.la.clone(),Stateaction::Accept);
        }
        else {
           if TRACE>1 {println!("++adding Reduce({}) at state {}, lookahead {}",item.ri,si,&item.la);}
        
           FSM[si].insert(item.la.clone(),Stateaction::Reduce(item.ri));
        }
     }// add reduce action
  }//addreduce

  // generate the GOTO sets of a state with index si, creates new states
  fn makegotos(&mut self, si:usize)
  {
     let ref state = self.States[si];
     // key to following hashmap is the next symbol after pi (the dot)
     let mut newstates:HashMap<String,LR1State> = HashMap::new();
     let mut keyvec:Vec<String> = Vec::new(); //keys of newstates
     for item in &state.items
     {
       let rule = self.Gmr.Rules.get(item.ri).unwrap();
       if item.pi<rule.rhs.len() { // can goto (dot before end of rule)
          let ref nextsym = rule.rhs[item.pi].sym;
          if let None = newstates.get(nextsym) {
             newstates.insert(nextsym.to_owned(),LR1State::new());
             keyvec.push(nextsym.clone());
          }
          let symstate = newstates.get_mut(nextsym).unwrap();
          let newitem = LRitem {
             ri : item.ri,
             pi : item.pi+1,
             la : item.la.clone(),
          };
          let lhssym = &self.Gmr.Rules[item.ri].lhs.sym;
          symstate.insert(newitem,lhssym);
          // SHIFT/GOTONEXT actions added by addstate function
       }//can goto
       else // . at end of production, this is a reduce situation
       {
          Statemachine::addreduce(&mut self.FSM,&self.Gmr,item,si);
           /*
          let currentaction = self.FSM[si].get(&item.la);
          let mut changefsm = true;
          let ri1 = &item.ri;
          /// detect CONFLICT HERE
          match currentaction {
            Some(Reduce(ri2)) if ri2<ri1 => {
              changefsm=false;
              println!("Reduce-Reduce Conflict conflicted detected between rules {} and {}, resolved in favor of {}",ri2,ri1,ri2);
              printstate(&self.States[si],&self.Gmr);
            },
            Some(Reduce(ri2)) if ri2>ri1 => {
              println!("Reduce-Reduce Conflict conflicted detected between rules {} and {}, resolved in favor of {}",ri2,ri1,ri1);
              printstate(&self.States[si],&self.Gmr);            
            },
            Some(Accept) => {changefsm = false;},
            Some(Shift(_)) => {   // shift-reduce conflict
              let prec1 = self.Gmr.Rules[item.ri].precedence;
              let prec2 = self.Gmr.Symbols[*self.Gmr.Symhash.get(&item.la).unwrap()].precedence;
              //let prec2 = self.Gmr.Symbols.get(&item.la).unwrap().precedence;
              if prec1==prec2 && prec1<0 {changefsm=false;} // assume right-associative
              else if prec2.abs()>prec1.abs() {changefsm=false;} // still shift 
              if TRACE>4 {println!("shift-reduce conflict resolved by operator precedence/associativity:"); printstate(&self.States[si],&self.Gmr);}
            },
            _ => {},
          }//match to detect conflict

          if changefsm {   // only Reduce/Accept added here
             // accept or reduce
             if item.ri==self.Gmr.Rules.len()-1 && item.la=="EOF"  { // start rule
               self.FSM[si].insert(item.la.clone(),Stateaction::Accept);
             }
             else {
               self.FSM[si].insert(item.la.clone(),Stateaction::Reduce(item.ri));
             }
          }// add reduce action
          */
       } // set reduce action
     }// for each item 
     // form closures for all new states and add to self.States list
     for key in keyvec
     {
        let kernel = newstates.remove(&key).unwrap();
        let fullstate = stateclosure(kernel,&self.Gmr);
        self.addstate(fullstate,si,key);
     }
  }//makegotos

  pub fn generatefsm(&mut self)
  { 
    // create initial state, closure from initial item: 
    // START --> .topsym EOF
    let mut startstate=LR1State::new();
    startstate.insert( LRitem {
         ri : self.Gmr.Rules.len()-1, // last rule is start
         pi : 0,
         la : "EOF".to_owned(),   // must have this in grammar
       },"START");       
    startstate = stateclosure(startstate,&self.Gmr);
    //setRactions(startstate); //???????
    self.States.push(startstate); // add start state
    self.FSM.push(HashMap::new()); // row for state
    // now generate closure for state machine (not individual states)
    let mut closed:usize = 0;
    if !self.lalr {
      while closed<self.States.len()
      {
         //if TRACE>2 {println!("closed states: {}",closed);}
         self.makegotos(closed);
         closed += 1;
      }//while not closed
    } // lr1
    else { //lalr
      self.Open.push(0);
      while closed<self.Open.len()
      {
         let si = self.Open[closed]; // state index to close
         self.makegotos(si);
         closed += 1;
      }
    }// lalr
  }//generate


////////////// write parser to .rs file
  pub fn writefsm(&self, filename:&str)->Result<(),std::io::Error>
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
use RustLr::{{Parser,RGrule,Stateaction}};\n")?;

// taken out the following from written parser:
//extern crate scanlex;
//use scanlex::{{Scanner,Token,ScanError}};
//use bunch::Stateaction::*;  // didn't write
//use std::fmt::Display;
//use std::default::Default;
//use std::collections::{{HashMap,HashSet,BTreeSet}};

    write!(fd,"{}\n",&self.Gmr.Extras)?; // use clauses
    // must know what absyn type is when generating code.
    let ref absyn = self.Gmr.Absyntype;
    write!(fd,"pub fn make_parser() -> Parser<{}>",absyn)?; 
    write!(fd,"\n{{\n")?;
    // write code to pop stack, assign labels to variables.
    write!(fd," let mut parser1:Parser<{}> = Parser::new({},{});\n",absyn,self.Gmr.Rules.len(),self.States.len())?;
    // generate rules and Ruleaction delegates, must pop values from runtime stack
    write!(fd," let mut rule = RGrule::<{}>::new_skeleton(\"{}\");\n",absyn,"start")?;
    for i in 0..self.Gmr.Rules.len() 
    {
      //write!(fd," let mut rule = RGrule::<{}>::new_skeleton(\"{}\");\n",absyn,self.Gmr.Rules[i].lhs.sym)?;
      write!(fd," rule = RGrule::<{}>::new_skeleton(\"{}\");\n",absyn,self.Gmr.Rules[i].lhs.sym)?;      
      write!(fd," rule.Ruleaction = |pstack|{{ ")?;
      let mut k = self.Gmr.Rules[i].rhs.len();
      while k>0
      {
        let gsym = &self.Gmr.Rules[i].rhs[k-1];
        if gsym.label.len()>0 && &gsym.rusttype[0..3]=="mut"
          { write!(fd," let mut {}:{}=",gsym.label,absyn)?; }        
        else if gsym.label.len()>0
          { write!(fd," let {}:{}=",gsym.label,absyn)?; }
        write!(fd,"pstack.pop()")?; //.unwrap().value;  ")?;
        if gsym.label.len()>0 { write!(fd,".unwrap().value;  ")?;}
        else {write!(fd,";  ")?;}
        k -= 1;
      } // for each symbol on right hand side of rule  
      let mut semaction = &self.Gmr.Rules[i].action; //this is a string
      //if semaction.len()<1 {semaction = "}}";}
      //if al>1 {semaction = semaction.substring(0,al-1);}
      if semaction.len()>1 {write!(fd,"{};\n",semaction)?;}
      else {write!(fd," return {}::default();}};\n",absyn)?;}
      write!(fd," parser1.Rules.push(rule);\n")?;
    }// for each rule

    // generate code to load fsm quickly:
    let mut linecx = 0; // number of lines written
    let cxmax = 512; // number of lines before creating a new function
    for i in 0..self.FSM.len()
    {
      let row = &self.FSM[i];
      for key in row.keys()
      {
        write!(fd," parser1.RSM[{}].insert(\"{}\",Stateaction::{:?});\n",i,key,row.get(key).unwrap())?;
        linecx += 1;
        if linecx%cxmax==0 {
          write!(fd," return make_parser{}(parser1);\n}}\n\n",(linecx/cxmax))?;
          write!(fd,"fn make_parser{}(mut parser1:Parser<{}>) -> Parser<{}>\n{{\n",(linecx/cxmax),absyn,absyn)?;
        } //max function size reached, start new function       

        //write!(fd," parser1.RSM[{}].insert(\"{}\",Stateaction::{:?});\n",i,key,row.get(key).unwrap())?;
      } //for each string key in row
    }//for each state index i

    write!(fd," return parser1;\n")?;
    write!(fd,"}} //make_parser\n")?;
    Ok(())
  }//writefsm

/// Generate compact binary table rowsxcols = statesxsymbols, each entry
/// is a 16-bit value with lower 3 bits indicating shift/goto/reduce/accept/error
/// and higher 13 bits indicating state number or rule number (or error?)
  pub fn writefsm_bin(&self, filename:&str)->Result<(),std::io::Error>
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
use RustLr::{{Parser,RGrule,Stateaction,decode_action}};\n")?;

    write!(fd,"{}\n",&self.Gmr.Extras)?; // use clauses

    // write static array of symbols
    write!(fd,"const SYMBOLS:[&'static str;{}] = [",self.Gmr.Symbols.len())?;
    for i in 0..self.Gmr.Symbols.len()-1
    {
      write!(fd,"\"{}\",",&self.Gmr.Symbols[i].sym)?;
    }
    write!(fd,"\"{}\"];\n\n",&self.Gmr.Symbols[self.Gmr.Symbols.len()-1].sym)?;
    // position of symbols must be inline with self.Gmr.Symhash

    // create simulated 2d array representing table
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
//          _ => {panic!("invalid state entry");},
        }//match
        write!(fd,"{},",encode)?;
      } //for symbol index k
    }//for each state index i
    write!(fd,"];\n\n")?;

    // must know what absyn type is when generating code.
    let ref absyn = self.Gmr.Absyntype;
    write!(fd,"pub fn make_parser() -> Parser<{}>",absyn)?; 
    write!(fd,"\n{{\n")?;
    // write code to pop stack, assign labels to variables.
    write!(fd," let mut parser1:Parser<{}> = Parser::new({},{});\n",absyn,self.Gmr.Rules.len(),self.States.len())?;
    // generate rules and Ruleaction delegates, must pop values from runtime stack
    write!(fd," let mut rule = RGrule::<{}>::new_skeleton(\"{}\");\n",absyn,"start")?;
    for i in 0..self.Gmr.Rules.len() 
    {
      //write!(fd," let mut rule = RGrule::<{}>::new_skeleton(\"{}\");\n",absyn,self.Gmr.Rules[i].lhs.sym)?;
      write!(fd," rule = RGrule::<{}>::new_skeleton(\"{}\");\n",absyn,self.Gmr.Rules[i].lhs.sym)?;      
      write!(fd," rule.Ruleaction = |pstack|{{ ")?;
      let mut k = self.Gmr.Rules[i].rhs.len();
      while k>0
      {
        let gsym = &self.Gmr.Rules[i].rhs[k-1];
        if gsym.label.len()>0 && &gsym.rusttype[0..3]=="mut"
          { write!(fd," let mut {}:{}=",gsym.label,absyn)?; }        
        else if gsym.label.len()>0
          { write!(fd," let {}:{}=",gsym.label,absyn)?; }
        write!(fd,"pstack.pop()")?; //.unwrap().value;  ")?;
        if gsym.label.len()>0 { write!(fd,".unwrap().value;  ")?;}
        else {write!(fd,";  ")?;}
        k -= 1;
      } // for each symbol on right hand side of rule  
      let mut semaction = &self.Gmr.Rules[i].action; //this is a string
      //if semaction.len()<1 {semaction = "}}";}
      //if al>1 {semaction = semaction.substring(0,al-1);}
      if semaction.len()>1 {write!(fd,"{};\n",semaction)?;}
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
  }//writefsm_bin

}//impl Statemachine

pub fn decode_action(code:u64) -> Stateaction
{
    let actiontype =   code & 0x000000000000ffff;
    let actionvalue = (code & 0x00000000ffff0000) >> 16;
    //let symboli =     (code & 0x0000ffff00000000) >> 32;
    //let statei =      (code & 0xffff000000000000) >> 48;    
    match (actiontype,actionvalue) {
      (0,si) => Shift(si as usize),
      (1,si) => Gotonext(si as usize),
      (2,ri) => Reduce(ri as usize),
      (3,_)  => Accept,
      (4,x)  => Error(x.to_string()),
      _      => Error("unrecognized action in TABLE".to_owned()),
    }
}//decode - must be independent function seen by use bunch::*;



////////////RUNTIME PARSER/////////////////////

pub struct Stackelement<AT:Default>
{
   pub si : usize, // state index
   pub value : AT,  // semantic value (don't clone grammar symbols)
}


///// new structures for runtime representation of Grule, Gsym:
// should not require AT to be clonable
pub struct Lextoken<AT:Default> // now separated from Gsym
{
   pub sym: String, // must correspond to terminal symbol
   //pub symindex : usize, // index into Symbols array, must be terminal
   pub value: AT,         // value of terminal symbol, if any
}
impl<AT:Default> Lextoken<AT>
{
  pub fn new(name:String, val:AT) -> Lextoken<AT>   // runtime
  {
     Lextoken {
       sym : name,
       value : val,
     }
  }//new Lextoken
}//impl Lextoken

/////// trait for abstract lexer
pub trait Lexer<AT:Default>
{
  fn nextsym(&mut self) -> Option<Lextoken<AT>>; //assum .sym matches terminal
  fn linenum(&self) -> usize; // line number
}//trait Lexer
// lexer must translate tokens into Gsyms


pub struct RGrule<AT:Default>  // runtime rep of grammar rule
{
  pub lhs: &'static str,
  pub Ruleaction : fn(&mut Vec<Stackelement<AT>>) -> AT, //takes stack as arg
}
// compile time version of Grule no longer requires Ruleaction
impl<AT:Default> RGrule<AT>
{
  pub fn new_skeleton(lh:&'static str) -> RGrule<AT>
  {
     RGrule {
       lhs : lh,
       Ruleaction : |p|{AT::default()},
     }
  }
}//impl RGrule


/////////////////// used as Actual Parser (Runtime State Machine to be written) 
// Change runtime representation of Gsym, Grule?
pub struct Parser<AT:Default>  
{
  pub RSM : Vec<HashMap<&'static str,Stateaction>>, //runtime version of state machine
  pub Rules : Vec<RGrule<AT>>, // rules with just lhs and delegate function
}
// parse stack 
//////////// Parsing algorithm.
impl<AT:Default> Parser<AT>
{
    pub fn new(rlen:usize, slen:usize) -> Parser<AT>
    {  // given number of rules and number states
       let mut p = Parser {
         RSM : Vec::with_capacity(slen),
         Rules : Vec::with_capacity(rlen),
       };
       for _ in 0..slen {p.RSM.push(HashMap::new());}
       p
    }//new

    // parse does not reset state stack
    pub fn parse(&self, tokenizer:&mut dyn Lexer<AT>) -> AT
    { 
       let mut result = AT::default();
       let mut stack:Vec<Stackelement<AT>> = Vec::with_capacity(1024);
       // push state 0 on stack:
       stack.push(Stackelement {si:0, value:AT::default()});
       let unexpected = Stateaction::Error(String::from("unexpected end of input"));
       let mut action = &unexpected; //Stateaction::Error(String::from("get started"));
       let mut stopparsing = false;
//       if !tokenizer.has_next() { stopparsing=true; }
//       let mut lookahead = tokenizer.nextsym(); // initial, this is a Lextoken
       let mut lookahead = Lextoken{sym:"EOF".to_owned(),value:AT::default()}; 
       if let Some(tok) = tokenizer.nextsym() {lookahead=tok;}
       else {stopparsing=true;}
       while !stopparsing
       {  
         let currentstate = stack[stack.len()-1].si;
         if TRACE>1 {print!(" current state={}, lookahead={}, ",&currentstate,&lookahead.sym);}
         let actionopt = self.RSM[currentstate].get(lookahead.sym.as_str());//.unwrap();
         if TRACE>1 {println!("RSM action : {:?}",actionopt);}
         if let None = actionopt {
            panic!("!!PARSE ERROR: no action at state {}, lookahead {}, line {}",currentstate,&lookahead.sym,tokenizer.linenum());
         }
         action = actionopt.unwrap();
         match action {
            Stateaction::Shift(i) => { // shift to state si
                stack.push(Stackelement{si:*i,value:mem::replace(&mut lookahead.value,AT::default())});
//              if !tokenizer.has_next() { stopparsing=true; }
//              else {lookahead = tokenizer.nextsym();} // ADVANCE LOOKAHEAD HERE ONLY!
                if let Some(tok) = tokenizer.nextsym() {lookahead=tok;}
                else {
                  lookahead=Lextoken{sym:"EOF".to_owned(),  value:AT::default()};
                }
             }, //shift
            Stateaction::Reduce(ri) => { //reduce by rule i
              let rulei = &self.Rules[*ri];
              let val = (rulei.Ruleaction)(&mut stack); // calls delegate function
              let newtop = stack[stack.len()-1].si; 
//              let goton = self.RSM[newtop].get(rulei.lhs.sym.as_str()).unwrap();
              let goton = self.RSM[newtop].get(rulei.lhs).unwrap();
              if TRACE>1 {println!(" ..performing Reduce({}), new state {}, action on {}: {:?}..",ri,newtop,&rulei.lhs,goton);}
              if let Stateaction::Gotonext(nsi) = goton {
                stack.push(Stackelement{si:*nsi,value:val});
                // DO NOT CHANGE LOOKAHEAD AFTER REDUCE!
              }// goto next state after reduce
              else { stopparsing=true; }
             },
            Stateaction::Accept => {
              result = stack.pop().unwrap().value;
              stopparsing = true;
             },
            Stateaction::Error(msg) => {
              stopparsing = true;
             },
            Stateaction::Gotonext(_) => { //should not see this here
              stopparsing = true;
             },
         }//match action
       } // main parser loop
       if let Stateaction::Error(msg) = action {
          panic!("!!!Parsing failed on line {}, next symbol {}: {}",tokenizer.linenum(),&lookahead.sym,msg);
       }
       return result;
    }//parse
}// parsing algorithm

// the start rule START -> topsym. EOF, is never reduced, so its Ruleaction
// is never called: the parser will enter Accept state and return the value
// associated with topsym.


// takes grammar file prefix as command line arg
pub fn rustler(grammarname:&str, option:&str) {
  let mut gram1 = Grammar::new();
  let grammarfile = format!("{}.grammar",&grammarname);

  let lalr =  match option {
    "lalr" | "LALR" => true,   
    "lr1" | "LR1" => false,
    _ => {println!("Option {} not supported, defaulting to full LR1 generation",option); false},
  };
  
  if TRACE>1 {println!("parsing grammar from {}",grammarfile);}
  gram1.parse_grammar(&grammarfile);
  if TRACE>2 {println!("computing Nullable set");}
  gram1.compute_NullableRf();
  if TRACE>2 {println!("computing First sets");}
  gram1.compute_FirstIM();
  if gram1.name.len()<2 {gram1.name = grammarname.to_owned(); }
  let gramname = gram1.name.clone();
  /*
  for nt in gram1.First.keys() {
     print!("First({}): ",nt);
     let firstnt = gram1.First.get(nt).unwrap();
     for tt in firstnt { print!("{} ",tt); }
     println!();
  }//print first set
  */
  let mut fsm0 = Statemachine::new(gram1);
  fsm0.lalr = lalr;
  if lalr {fsm0.Open = Vec::with_capacity(1024); }
  println!("Generating {} state machine for grammar...",if lalr {"LALR"} else {"LR1"});
  fsm0.generatefsm();
  if TRACE>1 { for state in &fsm0.States {printstate(state,&fsm0.Gmr);} }
  else if TRACE>0 {   printstate(&fsm0.States[0],&fsm0.Gmr); }//print state
  let parserfile = format!("{}parser.rs",&gramname);
  let write_result = 
  if fsm0.Gmr.Externtype.len()>0 {
    if fsm0.States.len()<=16 {fsm0.write_verbose(&parserfile)}
    else if fsm0.States.len()<=65536 {fsm0.writeparser(&parserfile)}
    else {panic!("too many states: {}",fsm0.States.len())}
  }
  else if fsm0.States.len()<=16 {  fsm0.writefsm(&parserfile) }
  else if fsm0.States.len()<=65536 { fsm0.writefsm_bin(&parserfile) }
  else {panic!("too many states: {}",fsm0.States.len());};
  println!("{} total states",fsm0.States.len());
  if let Ok(_) = write_result {println!("written parser to {}",&parserfile);}
}//rustler
