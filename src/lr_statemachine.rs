// module for generating the LR finite state machine
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
#![allow(unused_doc_comments)]
#![allow(unused_imports)]
use std::collections::{HashMap,HashSet,BTreeSet};
use std::cell::{RefCell,Ref,RefMut};
use std::hash::{Hash,Hasher};
use std::mem;
use crate::grammar_processor::*;
use crate::Stateaction::*;

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

pub type LookupSet<T> = BTreeSet<T>;

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
   let mut core0 = HashSet::with_capacity(256);
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
   //expected : HashSet<String>, // expected lookaheads for error reporting
}
impl LR1State
{
  pub fn new() -> LR1State
  {
     LR1State {
        index : 0,   // need to change
        items : HashSet::with_capacity(256),
        lhss: BTreeSet::new(), // for quick lookup
        //expected : HashSet::with_capacity(32),
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
     /*
     // insert terminals into expected set for error reporting
     if pi<rulei.rhs.len() && rulei.rhs[pi].terminal { // add to expected
       closed.expected.insert(rulei.rhs[pi].sym.clone());
     }
     else if pi==rulei.rhs.len() {closed.expected.insert(la.clone());}
     */
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


////// Contruction of the FSM, which is a Vec<HashMap<String,stateaction>>

/// this enum is only exported because it's used by the generated parsers.
/// There is no reason to use it in other programs.
#[derive(Clone,PartialEq,Eq,Debug)]
pub enum Stateaction {
  Shift(usize),     // shift then go to state index
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
   pub Statelookup: HashMap<String,LookupSet<usize>>,
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
        self.Statelookup.insert(lookupkey.clone(),LookupSet::new());
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
                //expected : state.expected.clone(),
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
       self.FSM.push(HashMap::with_capacity(64)); // always add row to fsm at same time
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
     let ref /*mut*/ state = self.States[si];
     // key to following hashmap is the next symbol after pi (the dot)
     let mut newstates:HashMap<String,LR1State> = HashMap::with_capacity(64);
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
    self.FSM.push(HashMap::with_capacity(64)); // row for state
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

}//impl Statemachine

// encode a state transition: FSM[i].get(key)=action as u64 numbers
/// this function is only exported because it's used by the generated parsers.
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
}//decode - must be independent function seen by parsers

