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

#[derive(Copy,Clone,PartialEq,Eq,Hash,Debug,PartialOrd,Ord)]
pub struct LRitem
{
   pub ri: usize, // rule index
   pub pi: usize, // position of dot
   pub la: usize, // lookahead symbol index
   //interior : bool,  // can't have this here if deriving Eq
}

pub fn printrulela(ri:usize, Gmr:&Grammar, la:usize)
{
     if ri>=Gmr.Rules.len() {println!("printing invalid rule number {}",ri); return;}
     let ref lhs_sym = Gmr.Rules[ri].lhs.sym;
     let ref rhs = Gmr.Rules[ri].rhs;
     print!("  (Rule {}) {} --> ",ri,lhs_sym);
     for gsym in rhs  { print!("{} ",gsym.sym); }
     println!(" , lookahead {}",Gmr.symref(la));
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
     println!(", {}",Gmr.symref(item.la));  
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

// works for all but initial kernel START --> topsym, EOF,
// However, this state will never be encountered again.
pub fn extract_kernel(items:&Itemset) -> HashSet<(usize,usize)> // for lalr
{
   let mut kernel0 = HashSet::with_capacity(256);
   for LRitem{ri:r, pi:p, la} in items  {
     if *p>0 {kernel0.insert((*r,*p));}
   }
   kernel0
}

#[derive(Clone,Debug)]
pub struct LR1State
{
   index: usize, // index into vector
   items:Itemset,
   lhss: BTreeSet<usize>,  // set of left-side non-terminal indices
   kernel: HashSet<(usize,usize)>, // used only by lalr
   //expected : HashSet<String>, // expected lookaheads for error reporting
}
impl LR1State
{
  pub fn new() -> LR1State
  {
     LR1State {
        index : 0,   // need to change
        items : HashSet::with_capacity(512),
        lhss: BTreeSet::new(), // for quick lookup
        kernel : HashSet::with_capacity(64),
        //expected : HashSet::with_capacity(32),
     }
  }
  pub fn insert(&mut self, item:LRitem, lhs:usize) -> bool
  {
     let inserted = self.items.insert(item);
     self.lhss.insert(lhs);
     inserted
  }
  
  pub fn hashval(&self) -> usize  // note: NOT UNIQUE
  { 
    let mut key=self.items.len()+ self.lhss.len()*10000;
    let limit = usize::MAX/1000 -1;
    let mut cx = 8;
    for s in &self.lhss {key+=s*1000; cx-=1; if cx==0  || key>=limit {break;}}
    key 
    //self.items.len() + self.lhss.len()*10000
  } // 
  pub fn hashval_lalr(&mut self) -> usize  // note: NOT UNIQUE
  {
    if self.kernel.len()==0 {self.kernel = extract_kernel(&self.items); }
    let mut key=self.kernel.len() + self.lhss.len()*1000000;    
    let limit = usize::MAX/1000 -1;
    let mut cx = 8;
    for s in &self.lhss {key+=1000*s; cx-=1; if cx==0 || key>=limit {break;}}
    key
  }
    
  pub fn contains(&self, x:&LRitem) -> bool {self.items.contains(x)}

  fn kernel_eq(&mut self, state2:&mut LR1State) -> bool // for LALR
  {
     //if self.kernel.len()==0 {self.kernel = extract_kernel(&self.items);}
     //if state2.kernel.len()==0 {state2.kernel = extract_kernel(&state2.items);}
     //if self.kernel.len()!=state2.kernel.len() {return false;}
     if self.hashval_lalr() != state2.hashval_lalr() || (self.kernel.len()!=state2.kernel.len()) {return false;}
     for item_kernel in &self.kernel
     {
      if !state2.kernel.contains(item_kernel) {return false; }
     }
     return true;
  }//kernel_eq

  // two states being merged must have same core
  fn merge_states(&mut self, state2:&LR1State) // used by lalr
  {
      for item in &state2.items {self.items.insert(*item);}
  }//merge_states

/* won't work because new lookaheads also afters other states from this one.
 // FOR LALR, returns false if no additions where added, will also
  // augment action table with new reduce actions. - destination is state2
  fn state_merge(FSM: &mut Vec<HashMap<usize,Stateaction>>, Gmr:&Grammar, state1:&LR1State, state2:&mut LR1State) -> bool
  {  let mut answer = false;
     for item in &state1.items {
       let res = state2.items.insert(*item); // returns true if proper add
       if res {
          answer = true;
          let newaction = Stateaction::Reduce(item.ri);
          add_action(FSM,Gmr,newaction,state2.index,item.la,&mut self.sr_conflicts,true);
       }// proper addtion, meaning only the lookahead was not there before
     }//for each item in state2
     answer
  }//state_merge
*/


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
  let mut lamap:HashMap<(usize,usize),Vec<usize>> = HashMap::with_capacity(Gmr.Rules.len()*4);
  for item in &state.items
  {
     let laset:&mut Vec<usize> = match lamap.get_mut(&(item.ri,item.pi)) {
        Some(x) => x,
        None => {
           let mut newset = Vec::<usize>::with_capacity(16);
           lamap.insert((item.ri,item.pi),newset);
           lamap.get_mut(&(item.ri,item.pi)).unwrap()
        },
     };//match
     laset.push(item.la);
  }
  for (ri,pi) in lamap.keys()
  {
    let ref lhs_sym = Gmr.Rules[*ri].lhs.sym;
     let ref rhs = Gmr.Rules[*ri].rhs;
     print!("  ({}) {} --> ",ri,lhs_sym);
     let mut position = 0;
     for gsym in rhs 
     {
       if &position==pi {print!(".");}
       print!("{} ",gsym.sym);
       position+=1;
     }
     if &position==pi {print!(". ");}
     print!(" {{ ");
     for la in lamap.get(&(*ri,*pi)).unwrap()
     {
       print!("{},",Gmr.symref(*la));
     }
     println!(" }}");
  }//for key
}//printstate
pub fn printstate_raw(state:&LR1State,Gmr:&Grammar) 
{
  for item in &state.items
  { printitem(item,Gmr); }
}


pub fn stateclosure0(state:&mut LR1State, Gmr:&Grammar)
{
  //algorithm is like that of a spanning tree
  let mut closed = 0;
  let mut closure = Vec::new();
  let mut onclosure = HashSet::new();
  for item in state.items.iter() {
     let lhsi = Gmr.Rules[item.ri].lhs.index;
     closure.push(*item);
     onclosure.insert(*item);
  } // cover over to new hashmap from items to lhsi
  while closed < closure.len()
  {
     let item = closure[closed];
     closed+=1;
     let (ri,pi,la) = (item.ri,item.pi,item.la);
     let rulei = &Gmr.Rules[ri]; //.get(ri).unwrap();
     let lhsi = rulei.lhs.index;
     if pi<rulei.rhs.len() && !rulei.rhs[pi].terminal {
        let nti = &rulei.rhs[pi]; // non-terminal after dot (Gsym)
        let nti_lhsi = nti.index;
        let lookaheads=&Gmr.Firstseq(&rulei.rhs[pi+1..],la);
        for rulent in Gmr.Rulesfor.get(&nti.index).unwrap()
        {
          for lafollow in lookaheads 
          { 
            let newitem = LRitem {
               ri: *rulent,
               pi: 0,
               la: *lafollow,
            };
            if !state.items.contains(&newitem) && !onclosure.contains(&newitem) {
             closure.push(newitem);
             onclosure.insert(newitem);
             state.insert(newitem,nti_lhsi);
            } 
          }//for each possible lookahead following non-terminal
        }// for each rule with this non-terminal on lhs (rulent loop)
     } // if candidate for add (dot before nonterminal)
  }  // loop until closed
}//stateclosure0 generation



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
     let (ri,pi,la) = (item.ri,item.pi,item.la);
     let rulei = &Gmr.Rules[ri]; //.get(ri).unwrap();
     let lhsi = rulei.lhs.index; //*Gmr.Symhash.get(&rulei.lhs.sym).unwrap();
     closed.insert(nextitem,lhsi); // place item in interior
     if pi<rulei.rhs.len() && !rulei.rhs[pi].terminal {
       let nti = &rulei.rhs[pi]; // non-terminal after dot (Gsym)
       let nti_lhsi = nti.index; //*Gmr.Symhash.get(&nti.sym).unwrap();
       let lookaheads=&Gmr.Firstseq(&rulei.rhs[pi+1..],la);
       for rulent in Gmr.Rulesfor.get(&nti.index).unwrap()
       {
          for lafollow in lookaheads 
          { 
            //if TRACE>2 {println!("adding new item for la {}",&lafollow);}
            let newitem = LRitem {
               ri: *rulent,
               pi: 0,
               la: *lafollow, //*Gmr.Symhash.get(lafollow).unwrap(),
            };
            if !closed.items.contains(&newitem)  {
              state.insert(newitem,nti_lhsi); // add to "frontier"
            }
          }//for each possible lookahead following non-terminal
       }// for each rule in this non-terminal
     } // add items to closure for this item
  }  // while not closed
  closed
}//stateclosure generation


////// Contruction of the FSM, which is a Vec<HashMap<usize,stateaction>>

/// this enum is only exported because it's used by the generated parsers.
/// There is no reason to use it in other programs.
#[derive(Copy,Clone,PartialEq,Eq,Debug)]
pub enum Stateaction {
  Shift(usize),     // shift then go to state index
  Reduce(usize),    // reduce by rule index
  Gotonext(usize),  // folded into same table, only for non-terminals
  Accept,
  /// note: this has been changed after version 0.1.1 from String to
  /// &'static str for increased efficiency. Error action entries are
  /// not generated by rustlr: they can only be added with the parser's
  /// training capability.  Parsers already trained can be hand-modified
  /// by removing all instances of ".to_string()" from the load_extras function.
  Error(&'static str),
}

/*
// for keeping track of conflicts
#[derive(Hash,PartialEq,Eq,Debug)]
enum Conflict
{
   //rule,lookahead Symbol index,clearly-resolved, resolution: true=reduce
   SR(usize,usize,bool,bool),
   // rule number, rule number : always resolved in favor of lower number
   RR(usize,usize),
}//Conflict
*/

// abstract parser struct
pub struct Statemachine  // Consumes Grammar
{
   pub Gmr: Grammar,
   pub States: Vec<LR1State>, 
   pub Statelookup: HashMap<usize,LookupSet<usize>>,
   pub FSM: Vec<HashMap<usize,Stateaction>>,
   pub lalr: bool,
   pub Open: Vec<usize>, // for LALR only, vector of unclosed states
   pub sr_conflicts:HashMap<(usize,usize),(bool,bool)>,
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
          sr_conflicts:HashMap::new(),
       }
  }//new

  // psi is previous state index, nextsym is next symbol (may do lalr)
  fn addstate(&mut self, mut state:LR1State, psi:usize, nextsymi:usize)
  {  let nextsym = &self.Gmr.Symbols[nextsymi].sym;
     let newstateindex = self.States.len(); // index of new state
     state.index = newstateindex;
     let lookupkey = if self.lalr {state.hashval_lalr()} else {state.hashval()};
     if let None=self.Statelookup.get(&lookupkey) {
        self.Statelookup.insert(lookupkey,LookupSet::new());
     }
     let indices = self.Statelookup.get_mut(&lookupkey).unwrap();
     let mut toadd = newstateindex; // defaut is add new state (will push)
     if self.lalr {
        for i in indices.iter()
        { 
           if state.kernel_eq(&mut self.States[*i]) {
             toadd=*i; // toadd changed to index of existing state
             let mut stateclone = LR1State {
                index : toadd,
                items : state.items.clone(),
                lhss: BTreeSet::new(), //state.lhss.clone(), //BTreeSet::new(), // will set by stateclosure
                //expected : state.expected.clone(),
                kernel: state.kernel.clone(),
             };
             stateclone.merge_states(&self.States[toadd]);
             //self.state_merge(&self.States[toadd],&mut stateclone);
             if stateclone.items.len() > self.States[toadd].items.len() {
                self.States[toadd] = stateclosure(stateclone,&self.Gmr);
                // now need to call makegotos again on this state - add
                // to end of open vector.
                self.Open.push(toadd);
                //if TRACE>3 { print!("===> MERGED STATE: ");
                //    printstate(&self.States[toadd],&self.Gmr);
                //}
             } // existing state extended, re-closed, but ...
             break;
           } // kernel_eq with another state  
        } // for each index in Statelookup to look at
     }// if lalr
     else {   // lr1
       for i in indices.iter()
       {
         if &state==&self.States[*i] {toadd=*i; break; } // state i exists
       }
     }// lalr or lr1

     if self.Gmr.tracelev>3 {println!("Transition to state {} from state {}, symbol {}..",toadd,psi,nextsym);}
     if toadd==newstateindex {  // add new state
       //if TRACE>2 {printstate(&state,&self.Gmr);}
       indices.insert(newstateindex); // add to StateLookup index hashset
       self.States.push(state);
       self.FSM.push(HashMap::with_capacity(128)); // always add row to fsm at same time
       if self.lalr {self.Open.push(newstateindex)}
     }// add new state

     // add to- or change FSM TABLE ...  only Shift or Gotnext added here.
//     let nextsymi = *self.Gmr.Symhash.get(nextsym).expect("GRAMMAR CORRUPTION, UNKOWN SYMBOL");
     let gsymbol = &self.Gmr.Symbols[nextsymi]; //self.Gmr.getsym(nextsym).
     let newaction = if gsymbol.terminal {Stateaction::Shift(toadd)}
        else {Stateaction::Gotonext(toadd)};
     add_action(&mut self.FSM, &self.Gmr, newaction,psi,nextsymi,&mut self.sr_conflicts,true);
     // reduce rules are only added with . at end, nextsymbol terminal,
     // so a "reduce-gotonext" conflict is not possible
  }  //addstate


// generate the GOTO sets of a state with index si, creates new states
  fn makegotos(&mut self, si:usize)
  {
     let ref /*mut*/ state = self.States[si];
     // key to following hashmap is the next symbol's index after pi (the dot)
     // the values of the map are the "kernels" of the next state to generate
     let mut newstates:HashMap<usize,LR1State> = HashMap::with_capacity(128);
     let mut keyvec:Vec<usize> = Vec::with_capacity(128); //keys of newstates
     for item in &state.items
     {
       let rule = self.Gmr.Rules.get(item.ri).unwrap();
       if item.pi<rule.rhs.len() { // can goto (dot before end of rule)
          let nextsymi = rule.rhs[item.pi].index;
          if let None = newstates.get(&nextsymi) {
             newstates.insert(nextsymi,LR1State::new());
             keyvec.push(nextsymi); // push only if unqiue
          }
          let symstate = newstates.get_mut(&nextsymi).unwrap();
          let newitem = LRitem { // this will be a kernel item in new state
             ri : item.ri,
             pi : item.pi+1,
             la : item.la, //.clone(),
          };
          //let lhssym = &self.Gmr.Rules[item.ri].lhs.sym;
          let lhssymi = self.Gmr.Rules[item.ri].lhs.index; //*self.Gmr.Symhash.get(lhssym).unwrap();
          symstate.insert(newitem,lhssymi);
          // SHIFT/GOTONEXT actions added by addstate function
       }//can goto
       else // . at end of production, this is a reduce situation
       {
          let isaccept = (item.ri == self.Gmr.startrulei && self.Gmr.symref(item.la)=="EOF");
          if isaccept {
            add_action(&mut self.FSM,&self.Gmr,Accept,si,item.la,&mut self.sr_conflicts,true);
          }
          else {
            add_action(&mut self.FSM, &self.Gmr,Reduce(item.ri),si,item.la,&mut self.sr_conflicts,true);
          }
          // only place addreduce is called
       } // set reduce action
     }// for each item 
     // form closures for all new states and add to self.States list
     for key in keyvec // keyvec must be separate to avoid borrow error
     {
        let mut kernel = newstates.remove(&key).unwrap();
        //let fullstate = stateclosure(kernel,&self.Gmr);
        //self.addstate(fullstate,si,key); //only place addstate called        
        stateclosure0(&mut kernel,&self.Gmr);
        self.addstate(kernel,si,key); //only place addstate called
     }
  }//makegotos

   pub fn generatefsm(&mut self)
  { 
    // create initial state, closure from initial item: 
    // START --> .topsym EOF
    let mut startstate=LR1State::new();
    let STARTi = self.Gmr.startnti; //*self.Gmr.Symhash.get("START").unwrap();
    startstate.insert( LRitem {
         ri : self.Gmr.startrulei, //self.Gmr.Rules.len()-1, 
         pi : 0,
         la : self.Gmr.eoftermi, //self.Gmr.Symbols.len()-1, //*self.Gmr.Symhash.get("EOF").unwrap(),   // must have this in grammar
       },STARTi);       
    startstate = stateclosure(startstate,&self.Gmr);
    startstate.kernel.insert((self.Gmr.startrulei,0)); //special core (lalr)
    self.States.push(startstate); // add start state, first state
    self.FSM.push(HashMap::with_capacity(128)); // row for state
    // now generate closure for state machine (not individual states)
    let mut closed:usize = 0;
    if !self.lalr {
      while closed<self.States.len()
      {
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
      (4,x)  => Error("shouldn't be here"),
      _      => Error("unrecognized action in TABLE"),
    }
}//decode - must be independent function seen by parsers


  // add_action unifies elements of previous addstate and addreduce 3/22
pub  fn add_action(FSM: &mut Vec<HashMap<usize,Stateaction>>, Gmr:&Grammar, newaction:Stateaction, si:usize, la:usize, conflicts:&mut HashMap<(usize,usize),(bool,bool)>, checkconflict:bool)
  {
     if !checkconflict {
       FSM[si].insert(la,newaction);
       return;
     }
     let currentaction = FSM[si].get(&la);
     let mut changefsm = true; // add or keep current
     match (currentaction, &newaction) {
       //(_ Accept) | (_,Gotonext(_)) => {},  //part of default
       (None,_) => {},  // most likely: just add
       (Some(Reduce(rsi)), Shift(_)) => { //doesn't apply for newlalr
         if Gmr.tracelev>4 {
           println!("Shift-Reduce Conflict between rule {} and lookahead {} in state {}",rsi,Gmr.symref(la),si);
         }       
         if sr_resolve(Gmr,rsi,la,si,conflicts) {changefsm = false; }
       },
       (Some(Reduce(cri)),Reduce(nri)) if cri==nri => { changefsm=false; },
       (Some(Reduce(cri)),Reduce(nri)) if cri!=nri => { // RR conflict
         let winner = if (cri<nri) {cri} else {nri};
         println!("Reduce-Reduce conflict between rules {} and {} resolved in favor of {} ",cri,nri,winner);
//         printrule(&Gmr.Rules[*cri]);
//         printrule(&Gmr.Rules[*nri]);
         printrulela(*cri,Gmr,la);
         printrulela(*nri,Gmr,la);
         if winner==cri {changefsm=false;}
       },
       (Some(Accept),_) => { changefsm = false; },
       (Some(Shift(_)), Reduce(rsi)) => {
         if Gmr.tracelev>4 {
           println!("Shift-Reduce Conflict between rule {} and lookahead {} in state {}",rsi,Gmr.symref(la),si);
         }
         // look in state to see which item caused the conflict...
         if !sr_resolve(Gmr,rsi,la,si,conflicts) {changefsm = false; }
       },
       _ => {}, // default add newstate
     }// match currentaction
     if changefsm { FSM[si].insert(la,newaction); }
  }//add_action


  // reslove shift-reduce conflict, returns true if reduce, but defaults
  // to false (shift) so parsing will always continue and terminate.
pub fn sr_resolve(Gmr:&Grammar, ri:&usize, la:usize, si:usize,conflicts:&mut HashMap<(usize,usize),(bool,bool)>) -> bool
  {
     if let Some((c,r)) = conflicts.get(&(*ri,la)) {
        return *r;
     }
     let mut clearly_resolved = true;
     let mut resolution = false; // shift
     let lasym = &Gmr.Symbols[la];
     let lapred = lasym.precedence;
     let rulepred = Gmr.Rules[*ri].precedence;
     if (lapred==rulepred) && rightassoc(lapred) { 
       /* default */
     } // right-associative lookahead, return shift
     else
     if (lapred==rulepred) && leftassoc(lapred) { // left associative
        resolution = true;
     } // right-associative lookahead, return shift     
     else if (prec_level(lapred).abs()>prec_level(rulepred).abs() && rulepred!=0) {/*default*/}
     else if prec_level(lapred).abs()<prec_level(rulepred).abs() {
       resolution = true;
       if lapred==0 {
          clearly_resolved = false;
          println!("Shift-Reduce conflict between lookahead {} and rule {} in state {} not clearly resolved, defaulting to Reduce because the rule has positive precedence.",&Gmr.Symbols[la].sym,ri,si);
          printrulela(*ri,Gmr,la);
       }
     } // reduce
     else {
       clearly_resolved=false;
       // report unclear case
         if conflicts.len() >= Gmr.expect {
             println!("Shift-Reduce conflict between lookahead {} and rule {} in state {} not clearly resolved, defaulting to Shift", &Gmr.Symbols[la].sym, ri, si);
             printrulela(*ri, Gmr, la);
         }
     }
     conflicts.insert((*ri,la),(clearly_resolved,resolution));
     resolution
  }//sr_resolve

