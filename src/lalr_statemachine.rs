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
//use std::rc::Rc;
use std::hash::{Hash,Hasher};
use std::mem;
use crate::grammar_processor::*;
use crate::lr_statemachine::{Stateaction,Statemachine,printrulela,add_action,sr_resolve};
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
  kernel: HashSet<LALRitem>,
  lookaheads: HashMap<LALRitem,RefCell<HashSet<usize>>>, //las for each item
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
      kernel:HashSet::with_capacity(16),      
      lookaheads:HashMap::with_capacity(hint),
      propagation:HashMap::new(),
      lhss: BTreeSet::new(),
    }
  }

  fn insert(&mut self,item:LALRitem,lhs:usize) -> bool
  {
     let inserted = self.items.insert(item);
     if inserted {
       self.lhss.insert(lhs);     
       if self.lookaheads.get(&item).is_none()
        {self.lookaheads.insert(item,RefCell::new(HashSet::new()));}
       if self.propagation.get(&item).is_none()
        {self.propagation.insert(item,HashSet::new());}
     }
    inserted
  }
  /*
  fn additem(&mut self,item:LALRitem, lhs:usize) -> bool
  {
     let inserted = self.items.insert(item);
     self.lhss.insert(lhs);
     inserted
  }  
  */
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
     for kitem in self.kernel.iter() {
       if !state2.kernel.contains(kitem) {return false;}
     }// for each item
     true
  }// state_eq
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
  pub fn to_statemachine(self) -> Statemachine // transfer before writeparser
  {
     Statemachine {
       Gmr: self.Gmr,
       States: Vec::new(),
       Statelookup:HashMap::new(),
       FSM: self.FSM,
       lalr:true,
       Open:Vec::new(),
       sr_conflicts:HashMap::new(),
     }
  }//to_statemachine
  
// generate the GOTO sets of a state with index si, creates new states.
// this verions does not add reduce actions since lookaheads needs to be
// propagated first.  PURE LR(0);
  fn makegotos(&mut self, si:usize)
  {
     let ref state = self.States[si];
     // key to following hashmap is the next symbol's index after pi (the dot)
     // the values of the map are the "kernels" of the next state to generate
     let mut newstates:HashMap<usize,LALRState> = HashMap::with_capacity(64);
     let mut keyvec:Vec<usize> = Vec::new(); //keys of newstates
     for item in &state.items
     {
       let (itemri,itempi) = (item.0,item.1);
       let rule = &self.Gmr.Rules[itemri]; // rule ref
       if itempi<rule.rhs.len() { // can goto (dot before end of rule)
          let nextsymi = rule.rhs[itempi].index;
          if let None = newstates.get(&nextsymi) {
             newstates.insert(nextsymi,LALRState::new(self.Gmr.Symbols.len()));
             keyvec.push(nextsymi);
          }
          let symstate = newstates.get_mut(&nextsymi).unwrap();
          // add new item to states associated with nextsymi
          let newitem = LALRitem(itemri,itempi+1); //kernel item in new state
          symstate.kernel.insert(newitem);
/*
          // handled during propagation phase?
          if symstate.lookaheads.get(&newitem).is_none() {
            symstate.lookaheads.insert(newitem,RefCell::new(HashSet::new()));
          }
          let las = state.lookaheads.get(&item).unwrap().borrow();
          let mut slas = symstate.lookaheads.get(&newitem).unwrap().borrow_mut();
          for la in las.iter() {
            if *la<self.Gmr.Symbols.len() { slas.insert(*la); }
          }
          // lookahead propagation
*/
          // SHIFT/GOTONEXT actions added by addstate function
       }//can goto
       /*  NO REDUCE/ACCEPT UNTIL LATER */
     }// for each item 
     // form closures for all new states and add to self.States list
     for key in keyvec //keyvec must be separate to avoid borrow error
     {
        let mut kernelstate = newstates.remove(&key).unwrap();
        closure0(&mut kernelstate,&self.Gmr);
        self.addstate(kernelstate,si,key); //only place addstate called
     }
  }//makegotos

  // addstate *** now pure LR(0)
  // psi is previous state index, nextsym is next symbol
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
     for i in indices.iter() //compares only LR(0) kernel items
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
     let gsymbol = &self.Gmr.Symbols[nextsymi];
     let newaction = if gsymbol.terminal {Stateaction::Shift(toadd)}
        else {Stateaction::Gotonext(toadd)};
     add_action(&mut self.FSM, &self.Gmr, newaction,psi,nextsymi,&mut self.sr_conflicts);
  }  //addstate


fn set_propagations(&mut self)
{
 for state in self.States.iter_mut()
 {
   // only do for kernel items - but must regenerate the closure
   let dummy = self.Gmr.Symbols.len()+1; // distinguish from real symbols (#)
  for kitem in state.kernel.iter()
  {
   state.lookaheads.get(kitem).unwrap().borrow_mut().insert(dummy);
   let mut interior = HashSet::new();
   let mut closure = vec![*kitem];
   let mut closed = 0;
   while closed<closure.len()
   {
     let item = closure[closed];
     let (ri,pi) = (item.0,item.1);
     let rulei = &self.Gmr.Rules[ri];
     let lhsi = rulei.lhs.index;
     interior.insert(item);
     closed += 1;
     if pi<rulei.rhs.len() {// find .X , X terminal or non-terminal
       let Xsym = &rulei.rhs[pi]; // symbol X of dragon book
       let propagate = state.lookaheads.get(&item).unwrap().borrow().contains(&dummy);
       if propagate {
          // use existing FSM to find nextstate:
          let mut nsi = self.FSM.len();//invlalid default
          match self.FSM[state.index].get(&Xsym.index) {
            Some(Shift(nexts)) | Some(Gotonext(nexts)) => {nsi=*nexts;},
            _ => {panic!("THIS SHOULD NOT HAPPEN!");},
          }
          let kpropagation = state.propagation.get_mut(&kitem).unwrap();
          kpropagation.insert((nsi,LALRitem(ri,pi+1)));
       }// insert into propagation table
       if !Xsym.terminal {  // X is non-terminal, add closure items
         // adds spontaneous lookaheads, plus #.
         let mut Xlookaheads = self.Gmr.Firstseq(&rulei.rhs[pi+1..],dummy);
         if Xlookaheads.remove(&dummy) {  // local use of dummy
           let itemlas = state.lookaheads.get(&item).unwrap().borrow();
           // this could be the kernel item with dummy
           for ila in itemlas.iter() {Xlookaheads.insert(*ila);}
         }
         for rulent in self.Gmr.Rulesfor.get(&Xsym.sym).unwrap() {
           let newitem = LALRitem(*rulent,0);
           if !interior.contains(&newitem) {
             closure.push(newitem); // add to "frontier"
           }
           let mut slas = state.lookaheads.get(&newitem).unwrap().borrow_mut();
           for xla in Xlookaheads.iter() {slas.insert(*xla);}
         }// for each rule of this non-terminal X
       }//X nonterminal
     }// pi not at right end
   }//while !closed
  }//for each kernel item
 }//for each state
}//set_propagations

 // called after all states created, propagations marked.
 fn propagate_lookaheads(&mut self)
 {
   let mut changed = true;
   while changed
   {
      changed = false;
      for si in 0..self.States.len()
      {
         /*let mut items = Vec::with_capacity(self.States[si].items.len());
         for item in &self.States[si].items {
           items.push(*item);
         }*/
         for item in &self.States[si].kernel { //-borrow checker!
           let (ri,pi) = (item.0, item.1);
           //println!("propagate_lookahead looking at kernel item {:?}, state {}",&item,si);
           //if pi>0 || ri == self.Gmr.Rules.len()-1 {  // kernel item
             let itemlas = self.States[si].lookaheads.get(&item).unwrap().borrow();
             let props = self.States[si].propagation.get(&item).unwrap();
//println!("props len {}", props.len());
             for (nextstate,nextitem) in props.iter() {
//println!("{}: found nextitem ({},{}) in next state {}",self.States[*nextstate].items.contains(nextitem),nextitem.0,nextitem.1,nextstate);
                let nextlasopt = self.States[*nextstate].lookaheads.get(nextitem);
                if let Some(rfcell)=nextlasopt {
                  if *nextstate!=si || nextitem!=item { //runtime borrow check
                    let mut nextlas = rfcell.borrow_mut();
                    for la in itemlas.iter() {
                      if *la>=self.Gmr.Symbols.len() {continue;} //no #
println!("propagating lookahead {} to state {} from state {}",&self.Gmr.Symbols[*la].sym, nextstate, si);
                      changed =nextlas.insert(*la)||changed;
                    }
                  }
                }
                else {panic!("NO lookahead ENTRY!");}
             } // each propagation mark
           //}//kernel item
         }//for each item in state
      }//for each state
   } // which changed
 }//propagate_lookaheads

 // set reduce actions and check for conflicts
 fn set_reduce(&mut self)
 {
    for si in 0..self.States.len()
    {
       for item in &self.States[si].items
       {
         let (ri,pi) = (item.0,item.1);
         if pi==self.Gmr.Rules[ri].rhs.len() { //dot at end of rhs
           let itemlas = self.States[si].lookaheads.get(item).unwrap().borrow();
           for la in itemlas.iter() { // for each lookahead
             if *la>=self.Gmr.Symbols.len() {continue;} // skip the dummy
             //println!("adding reduce/accept rule {}, la {}",ri,&self.Gmr.Symbols[*la].sym);
             
             let isaccept = (ri == self.Gmr.Rules.len()-1 && la==&(self.Gmr.Symbols.len()-1));
             if isaccept {
               add_action(&mut self.FSM,&self.Gmr,Accept,si,*la,&mut self.sr_conflicts);
             }
             else {
               add_action(&mut self.FSM,&self.Gmr,Reduce(ri),si,*la,&mut self.sr_conflicts);
             }
           } // for each la
         }//if reduce situation
       } // for each item
   } // for each state
 }//set_reduce

fn reclose(&mut self) // set remaining lookaheads
{
  for state in self.States.iter_mut() {
    let mut closure = Vec::new();
    for kitem in state.kernel.iter() { closure.push(*kitem); }
    let mut interior =HashSet::new();
    let mut closed = 0;
     while closed<closure.len()
     {
       let item = closure[closed];
       let (ri,pi) = (item.0,item.1);
       let rulei = &self.Gmr.Rules[ri];
       let lhsi = rulei.lhs.index;
       interior.insert(item);
       closed += 1;
       if pi<rulei.rhs.len() && !rulei.rhs[pi].terminal {
       let Xsym = &rulei.rhs[pi]; // symbol X of dragon book
       // adds remaining lookaheads, plus #.
       // spontaneously generated lookaheads already there .. only need
       // to add from kernels.
       let mut Xlookaheads = HashSet::new();
       let itemlas = state.lookaheads.get(&item).unwrap().borrow();
       for ila in itemlas.iter() {Xlookaheads.insert(*ila);}
       for rulent in self.Gmr.Rulesfor.get(&Xsym.sym).unwrap() {
         let newitem = LALRitem(*rulent,0);
         if !interior.contains(&newitem) {
           closure.push(newitem); // add to "frontier"
         }
         if newitem!=item { //runtime borrow check!
           let mut slas = state.lookaheads.get(&newitem).unwrap().borrow_mut();
           for xla in Xlookaheads.iter() {slas.insert(*xla);}
         }
       }// for each rule of this non-terminal X
      }//X nonterminal
     }//while !closed
  } //for each state
}//reclose

pub fn generatefsm(&mut self)
  { 
    // create initial state, closure from initial item: 
    // START --> .topsym EOF
    let mut startstate=LALRState::new(self.Gmr.Rules.len());
    let EOFi = self.Gmr.Symbols.len()-1;
    //let STARTi = self.Gmr.Symbols.len()-2;
    let startrule = self.Gmr.Rules.len()-1;
    let startitem = LALRitem(startrule,0);
    startstate.kernel.insert(startitem);
    closure0(&mut startstate,&self.Gmr);
    self.States.push(startstate); // add start state, first state
    self.FSM.push(HashMap::with_capacity(128)); // row for state
    // now generate closure for state machine (not individual states)
    let mut closed:usize = 0;
    //println!("before makegotos");    
    while closed<self.States.len()
    {
       self.makegotos(closed);
       closed += 1;
    }//while not closed
    //println!("after makegotos");
    println!("states generated: {}",self.States.len());
    self.States[0].lookaheads.get(&startitem).unwrap().borrow_mut().insert(EOFi);  // initial lookahead
    self.set_propagations();
    //println!("set_propagations");    
    self.propagate_lookaheads();
    //println!("after propagate_lookaheads");    
    self.reclose(); // set lookaheads for non-kernel items
    //println!("after reclose");        
    self.set_reduce();
  }//generatefsm
  
}//impl LALRMachine



//AXIOM: if item exists, an entry must exist for it in lookaheads

// purel LR0 state closure
fn closure0(state: &mut LALRState,Gmr:&Grammar)
{// assuming kernel is a kernel item and not? in state
assert!(state.items.len()==0);
   let mut closure = Vec::new();
   // start with kernel items
   for kitem in state.kernel.iter() {closure.push(*kitem);} // copy!
   let mut closed = 0;
   while closed < closure.len()
   {
     let item = closure[closed];
     let (ri,pi) = (item.0,item.1);
     let rulei = &Gmr.Rules[ri];
     let lhsi = rulei.lhs.index;
     state.insert(item,lhsi); // insert into state.items here
     closed += 1;
     if pi<rulei.rhs.len() && !rulei.rhs[pi].terminal {// add closure items
       let sympi = &rulei.rhs[pi].sym;
       for rulent in Gmr.Rulesfor.get(sympi).unwrap() {
         let newitem = LALRitem(*rulent,0); // can't be kernel again
         if !state.items.contains(&newitem) {
           closure.push(newitem); // add to "frontier"
         }
       }// for each rule of this non-terminal X
     }// not .X situation, no closure items added
   }//while !closed
}//closure0 - pure LR(0)

