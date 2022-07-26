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
use crate::lr_statemachine::{Stateaction,printrulela,add_action,sr_resolve};
use crate::lr_statemachine::Stateaction::*;

#[derive(Copy,Clone,PartialEq,Eq,Hash,Debug)]
pub struct LALRitem(usize,usize);   // rule index, position of the dot

// use Grammar.Symbols.len()+1 for the hash mark

//propagation stores for each item if lookaheads should be propagated to
//(symindex,item). symindex identifies nextstate: FSM[si][symindex] lookup.
// don't know index yet because it's not pre-generated.
//Only kernel items will have entries (?)
pub struct LALRState
{
  index: usize, // index into FSM vector
  items: HashSet<LALRitem>,
  lookaheads: HashMap<LALRitem,HashSet<usize>>, //looheads for each item
  propagation: HashMap<LALRitem,HashSet<(usize,LALRitem)>>,
  lhss: BTreeSet<usize>, // set of lhs of rules in the state
}
impl LALRState
{
  pub fn new(hint:usize) -> Self
  {
    LALRState {
      index:0,
      items:HashSet::with_capacity(hint),
      lookaheads:HashMap::with_capacity(hint),
      propagation:HashMap::new(),
      lhss: BTreeSet::new(),
    }
  }

  fn insert(&mut self,item:LALRitem,/*las:&HashSet<usize>,*/ lhs:usize) -> bool
  {
     let inserted = self.items.insert(item);
     if inserted {
       self.lookaheads.insert(item,HashSet::new());
       self.propagation.insert(item,HashSet::new());       
     }
//     let lahs = self.lookaheads.get_mut(&item).unwrap();
//     let mut lainserted = false;
//     for la in las.iter() { lainserted = lahs.insert(*la) || lainserted; }
//     lainserted || inserted
    inserted
  }

  fn additem(&mut self,item:LALRitem, lhs:usize) -> bool
  {
     let inserted = self.items.insert(item);
     self.lhss.insert(lhs);
     inserted
  }  

  pub fn hashval_lalr(&mut self) -> usize  // note: NOT UNIQUE
  {
    let mut key=self.items.len() + self.lhss.len()*1000000;    
    let limit = usize::MAX/1000 -1;
    let mut cx = 8;
    for s in &self.lhss {key+=1000*s; cx-=1; if cx==0 || key>=limit {break;}}
    key
  }

  pub fn state_eq(&self, state2:&LALRState) -> bool
  {
     // ignore index - set later
     if self.items.len() != state2.items.len() {return false;}
     for item in self.items.iter() {
       if !state2.items.contains(item) {return false;}
       /*  ignore lookaheads - there can only be one state with same core
       let las2 = state2.lookaheads.get(item);
       let las1 = self.lookaheads.get(item);
       if las1.is_some() && las2.is_some() {
         let la2set = las2.unwrap();
         for la in las1.unwrap() {
           if !la2set.contains(la) {return false;}
         }
       } else if las1.is_some() || las2.is_some() {return false;}
       */
     }// for each item
     true
  }// state_eq
  // no need for core_eq?  when adding state, propagate immediately.
  // but how do we know what to propagate?
}//impl LALRState
impl PartialEq for LALRState
{
   fn eq(&self, other:&LALRState) -> bool
   {  self.state_eq(&other) }
   fn ne(&self, other:&LALRState) ->bool
   {  !self.state_eq(&other) }
}
impl Eq for LALRState {}

impl Grammar  // Grammar additions
{
  // First set of a sequence of symbols given set of lookaheads
  pub fn Firstseqla(&self, Gs:&[Gsym], las:&HashSet<usize>) -> HashSet<usize>
  {
     let mut Fseq = HashSet::with_capacity(2);
     let mut i = 0;
     let mut nullable = true;
     while nullable && i<Gs.len() 
     {
         if (Gs[i].terminal) {Fseq.insert(Gs[i].index); nullable=false; }
	 else  // Gs[i] is non-terminal
         {
            //println!("symbol {}, index {}", &Gs[i].sym, Gs[i].index);
            let firstgsym = self.First.get(&Gs[i].index).unwrap();
	    for s in firstgsym { Fseq.insert(*s); }
	    if !self.Nullable.contains(&Gs[i].sym) {nullable=false;}
         }
	 i += 1;
     }//while
     if nullable {
      for la in las {Fseq.insert(*la);}
     }
     Fseq
  }//FirstSeqb
  //determine if a sequence of symbols is nullable
  
  pub fn Nullableseq(&self, Gs:&[Gsym]) -> bool
  {
     for g in Gs {
       if g.terminal || !self.Nullable.contains(&g.sym) {return false;}
     }
     return true;
  }
}//impl Grammar

pub struct LALRMachine
{
   pub Gmr: Grammar,
   pub States: Vec<LALRState>, 
   pub Statelookup: HashMap<usize,BTreeSet<usize>>,
   pub FSM: Vec<HashMap<usize,Stateaction>>,
   sr_conflicts:HashMap<(usize,usize),(bool,bool)>,
}
impl LALRMachine
{
  pub fn new(gram:Grammar) -> Self
  { 
       LALRMachine {
          Gmr: gram,
          States: Vec::with_capacity(1024), // reserve 1K states
          Statelookup: HashMap::with_capacity(1024),
          FSM: Vec::with_capacity(8*1024),
          sr_conflicts:HashMap::new(),
       }
  }//new

// generate the GOTO sets of a state with index si, creates new states.
// this verions does not add reduce actions since lookaheads needs to be
// propagated first.
  fn makegotos(&mut self, si:usize)
  {
     let ref state = self.States[si];
     // key to following hashmap is the next symbol's index after pi (the dot)
     // the values of the map are the "kernels" of the next state to generate
     let mut newstates:HashMap<usize,LALRState> = HashMap::with_capacity(128);
     let mut keyvec:Vec<usize> = Vec::new(); //keys of newstates
     for item in &state.items
     {
       let (itemri,itempi) = (item.0,item.1);
       let rule = self.Gmr.Rules.get(itemri).unwrap(); // rule ref
       if itempi<rule.rhs.len() { // can goto (dot before end of rule)
          let nextsymi = rule.rhs[itempi].index;
          if let None = newstates.get(&nextsymi) {
             newstates.insert(nextsymi,LALRState::new(self.Gmr.Symbols.len()));
             keyvec.push(nextsymi);
          }
          let symstate = newstates.get_mut(&nextsymi).unwrap();
          // add new item to states associated with nextsymi
          
          let newitem = LALRitem(itemri,itempi+1); //kernel item in new state
          
          let lhssymi = self.Gmr.Rules[itemri].lhs.index;
          symstate.additem(newitem,lhssymi);
          let las = state.lookaheads.get(&item).unwrap().clone();
          symstate.lookaheads.insert(newitem,las.clone()); //clone by alg
          //if let Some((psi,pitem)) = state.propagation.get(&item) {
          //  symstate.propagation.insert(newitem,(*psi,*pitem));
          //}
          // SHIFT/GOTONEXT actions added by addstate function
       }//can goto
       /*  NO REDUCE/ACCEPT UNTIL LATER
       else // . at end of production, this is a reduce situation
       {
          let isaccept = (item.ri == self.Gmr.Rules.len()-1 && self.Gmr.symref(item.la)=="EOF");
          if isaccept {
            Statemachine::add_action(&mut self.FSM,&self.Gmr,Accept,si,item.la,&mut self.sr_conflicts);
          }
          else {
            Statemachine::add_action(&mut self.FSM, &self.Gmr,Reduce(item.ri),si,item.la,&mut self.sr_conflicts);
          }
          // only place addreduce is called
          //Statemachine::addreduce(&mut self.FSM,&self.Gmr,item,si);
       } // set reduce action
       */
     }// for each item 
     // form closures for all new states and add to self.States list
     for key in keyvec //keyvec must be separate to avoid borrow error
     {
        let kernel = newstates.remove(&key).unwrap();
        let fullstate = lalrclosure(kernel,&self.Gmr);
        //self.addstate(fullstate,si,key); //only place addstate called
     }
  }//makegotos

  // addstate ***
  // psi is previous state index, nextsym is next symbol (may do lalr)
  // state already closed by makegotos
  fn addstate(&mut self, mut state:LALRState, psi:usize, nextsymi:usize)
  {  let nextsym = &self.Gmr.Symbols[nextsymi].sym;
     let newstateindex = self.States.len(); // index of new state
     state.index = newstateindex;
     let lookupkey = state.hashval_lalr();
     if let None=self.Statelookup.get(&lookupkey) {
        self.Statelookup.insert(lookupkey,BTreeSet::new());
     }
     let indices = self.Statelookup.get_mut(&lookupkey).unwrap();
     let mut toadd = newstateindex; // defaut is add new state (will push)
     for i in indices.iter() //lalr version only compares core items
     {
         if &state==&self.States[*i] {toadd=*i; break; } // state i exists
     }
     if self.Gmr.tracelev>3 {println!("Transition to state {} from state {}, symbol {}..",toadd,psi,nextsym);}
     if toadd==newstateindex {  // add new state
       indices.insert(newstateindex); // add to StateLookup index hashset
       self.States.push(state);
       self.FSM.push(HashMap::with_capacity(128)); // always add row to fsm at same time
     }// add new state

     // add to- or change FSM TABLE ...  only Shift or Gotnext added here.
     let gsymbol = &self.Gmr.Symbols[nextsymi]; //self.Gmr.getsym(nextsym).
     let newaction = if gsymbol.terminal {Stateaction::Shift(toadd)}
        else {Stateaction::Gotonext(toadd)};
     add_action(&mut self.FSM, &self.Gmr, newaction,psi,nextsymi,&mut self.sr_conflicts);
     // reduce rules are only added with . at end, nextsymbol terminal,
     // so a "reduce-gotonext" conflict is not possible
  }  //addstate

// need function propagate_lookaheads

}//impl LALRMachine



//AXIOM: if item exists, an entry must exist for it in lookaheads

fn lalrclosure(mut state:LALRState, Gmr:&Grammar) -> LALRState
{
  let mut closed =LALRState::new(Gmr.Symbols.len());
  closed.index = state.index;
  std::mem::swap(&mut state.lookaheads, &mut closed.lookaheads);
  std::mem::swap(&mut state.propagation, &mut closed.propagation);
  let dummy = Gmr.Symbols.len()+1; // distinguish from real symbols (#)
  while state.items.len()>0
  {
     let nextitem = *state.items.iter().next().unwrap();
     let item = state.items.take(&nextitem).unwrap();
     let (ri,pi) = (item.0,item.1);
     let rulei = &Gmr.Rules[ri]; //.get(ri).unwrap();
     let lhsi = rulei.lhs.index; // *Gmr.Symhash.get(&rulei.lhs.sym).unwrap();
     closed.insert(item,lhsi); // place item in interior
     // this will also initialize the lookaheads and propagation entries.
     //let itemlas = closed.lookaheads.get_mut(&item).unwrap(); //see axiom
     let itempropagation = closed.propagation.get_mut(&item).unwrap();
     let mut propagate = false;
     let kernlookaheads = &mut Gmr.Firstseq(&rulei.rhs[pi+1..],dummy);
     if (pi>0) || ri==Gmr.Rules.len()-1 {  // kernel item
       propagate = kernlookaheads.remove(&dummy);
     }
     else { kernlookaheads.remove(&dummy); };
     // rule itself is always in it's own closure...
     if propagate && pi < rulei.rhs.len() { // set propagation mark
         let sympi = rulei.rhs[pi].index;
         itempropagation.insert((sympi,LALRitem(ri,pi+1)));
     }
     // construct rest of closure for this item
     if pi<rulei.rhs.len() && !rulei.rhs[pi].terminal { //find .nonterminal
       let nti = &rulei.rhs[pi]; // non-terminal after dot (Gsym)
       for rulent in Gmr.Rulesfor.get(&nti.sym).unwrap()
       {
          let newitem = LALRitem(*rulent,0);
          if !closed.items.contains(&newitem)  {
             state.additem(newitem,nti.index); // add to "frontier"
          }
          if propagate && Gmr.Rules[*rulent].rhs.len()>0 {
            itempropagation.insert((nti.index,LALRitem(*rulent,1)));
          }
          // clone is needed algorithmically since there are multiple newitems
          closed.lookaheads.insert(newitem,kernlookaheads.clone());
          // these are just the "spontaneously generated" la's, since there's
          // only a dummy representing the lookaheads of item at top
       }// for each rule in this non-terminal
     } // add items to closure for this item
  }  // while not closed
  closed
}//lalrclosure generation

