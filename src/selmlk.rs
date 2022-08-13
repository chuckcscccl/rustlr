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
use crate::lr_statemachine::*;
use crate::Stateaction::*;

pub const MAXK:usize = 2;

pub struct MLState // emulates LR1/oldlalr engine
{
   index: usize, // index into vector
   items:Itemset,
   lhss: BTreeSet<usize>,  // set of left-side non-terminal indices
   kernel: HashSet<(usize,usize)>, // used only by lalr
   conflicts:Itemset,
   deprecated:Itemset,
}
impl MLState
{
  pub fn new() -> MLState
  {
     MLState {
        index : 0,   // need to change
        items : HashSet::with_capacity(512),
        lhss: BTreeSet::new(), // for quick lookup
        kernel : HashSet::with_capacity(64),
        conflicts: HashSet::new(),
        deprecated:HashSet::new(),
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

  fn contains(&self, x:&LRitem) -> bool {self.items.contains(x)}

  fn kernel_eq(&mut self, state2:&mut MLState) -> bool // for LALR
  {
     if self.hashval_lalr() != state2.hashval_lalr() || (self.kernel.len()!=state2.kernel.len()) {return false;}
     for item_kernel in &self.kernel
     {
      if !state2.kernel.contains(item_kernel) {return false; }
     }
     return true;
  }//kernel_eq

  // two states being merged must have same core
  fn merge_states(&mut self, state2:&MLState) // used by lalr
  {
      for item in &state2.items {self.items.insert(*item);}
  }//merge_states

//// put here for now: affects both items and conflicts sets,
  // conflict detection is done by mladd_action.
  //returns continue for true, failure for false
  fn conflict_close(&mut self, Gmr:&mut Grammar, combing:&mut Bimap<usize,Vec<usize>>) -> bool // close state si
  {
     let mut open = true;
     let mut moreconflicts = false;
     while open
     { open = false;
       let mut newitems:HashSet<LRitem> = HashSet::new();
       let mut newconflicts = HashSet::new();
//       let mut reduce_candidates = HashSet::new();
       for item@LRitem{ri,pi,la} in self.items.iter() {
//         if *pi==Gmr.Rules[*ri].rhs.len() {reduce_candidates.insert(*item);}
         for LRitem{ri:cri,pi:cpi,la:cla} in self.conflicts.iter() {
            if *cpi==0 && pi+1==Gmr.Rules[*ri].rhs.len() && Gmr.Rules[*cri].lhs.index==Gmr.Rules[*ri].rhs[*pi].index && cla==la{ //conflict propagation
              newconflicts.insert(*item);
            }
            else if *cpi==0 && pi+1<Gmr.Rules[*ri].rhs.len() && Gmr.Rules[*cri].lhs.index==Gmr.Rules[*ri].rhs[*pi].index && la==cla { //conflict extension *****

              let nti = Gmr.Rules[*ri].rhs[*pi].index;
              let defaultcomb = vec![nti];
              let comb = combing.get(&nti).unwrap_or(&defaultcomb);
              if comb.len()>MAXK {return false;}

              // deprecate "shorter" item
              self.deprecated.insert(*item);
              
              /////// got to create new symbol, rule, then insert into
              /////// items (closed), and deprecate others.
              // maybe dynamically create new symbol, change rule here...***
              // extend one at a time              
              let eri=Gmr.delay_extend(*ri,*pi,pi+1,combing);
              // this return index of new rule with longer delay
              newitems.insert(LRitem{ri:eri, pi:*pi, la:*la});

              //newitems.insert...
            }
         }//inner for each conflict item
         // look for new items
         if *pi<Gmr.Rules[*ri].rhs.len() && !Gmr.Rules[*ri].rhs[*pi].terminal {
             let lookaheads = &Gmr.Firstseq(&Gmr.Rules[*ri].rhs[pi+1..],*la);
             for rulent in
                 Gmr.Rulesfor.get(&Gmr.Rules[*ri].rhs[*pi].index).unwrap() {
               for lafollow in lookaheads {
                 let newitem = LRitem {
                   ri: *rulent,
                   pi: 0,
                   la: *lafollow,
                 };
                 newitems.insert(newitem);
               } //for each possible la
             }//for rulent
          }// add newitem
          // detect conflict
//          for rc in reduce_candidates.iter() {
//            if rc!=item && ((item.pi==Gmr.Rules[*ri].rhs.len() &&rc.la==*la) || (item.pi<Gmr.Rules[*ri].rhs.len() && Gmr.Firstseq(&Gmr.Rules[*ri].rhs[*pi..],*la).contains(&rc.la))) {
//              newconflicts.insert(*rc);
//            }
//          }
       }//for each conflict and closed item
       // add to current state
       for c in newconflicts {
         let inserted = self.conflicts.insert(c);
         moreconflicts = moreconflicts || inserted;
         open = inserted || open;
       }
       for n in newitems {open=self.items.insert(n)||open; } //MUST RECLOSE!
     }//while more
     true
  }//conflict_close
  // incorporates mlclosure into same loop!
}// impl MLState


impl PartialEq for MLState
{
   fn eq(&self, other:&MLState) -> bool
   {stateeq(&self.items,&other.items)}
   fn ne(&self, other:&MLState) ->bool
   {!stateeq(&self.items,&other.items)}
}
impl Eq for MLState {}


pub struct MLStatemachine  // Consumes Grammar
{
   pub Gmr: Grammar,
   pub States: Vec<MLState>, 
   pub Statelookup: HashMap<usize,LookupSet<usize>>,
   pub FSM: Vec<HashMap<usize,Stateaction>>,
   pub lalr: bool,
   pub Open: Vec<usize>, // for LALR only, vector of unclosed states
   pub sr_conflicts:HashMap<(usize,usize),(bool,bool)>,
   pub prev_states : HashMap<usize,HashSet<(usize,usize)>>,
   pub combing: Bimap<usize,Vec<usize>>,
   pub maxK:usize,   //max combing size
}
impl MLStatemachine
{
  pub fn new(gram:Grammar,k:usize) -> Self
  { 
       MLStatemachine {
          Gmr: gram,
          States: Vec::with_capacity(8*1024), // reserve 8K states
          Statelookup: HashMap::with_capacity(1024),
          FSM: Vec::with_capacity(8*1024),
          lalr: false, 
          Open: Vec::new(), // not used for lr1, set externally if lalr
          sr_conflicts:HashMap::new(),
          prev_states:HashMap::new(), //state --> symbol,state parent
          combing: Bimap::new(), //stores what each nt really represents
          maxK: k,
       }
  }//new

  fn simplemakegotos(&mut self,si:usize,agenda:&mut Vec<usize>) //LR1
  {
     // key to following hashmap is the next symbol's index after pi (the dot)
     // the values of the map are the "kernels" of the next state to generate
     let mut newstates:HashMap<usize,MLState> = HashMap::with_capacity(128);
     let mut keyvec:Vec<usize> = Vec::with_capacity(128); //keys of newstates
     for item in &self.States[si].items
     {
       let rule = self.Gmr.Rules.get(item.ri).unwrap();
       if item.pi<rule.rhs.len() { // can goto (dot before end of rule)
          let nextsymi = rule.rhs[item.pi].index;
          if let None = newstates.get(&nextsymi) {
             newstates.insert(nextsymi,MLState::new());
             keyvec.push(nextsymi); // push only if unqiue
          }
          let symstate = newstates.get_mut(&nextsymi).unwrap();
          let newitem = LRitem { // this will be a kernel item in new state
             ri : item.ri,
             pi : item.pi+1,
             la : item.la, 
          };
          //let lhssym = &self.Gmr.Rules[item.ri].lhs.sym;
          let lhssymi = self.Gmr.Rules[item.ri].lhs.index; //*self.Gmr.Symhash.get(lhssym).unwrap();
          symstate.insert(newitem,lhssymi);
          // SHIFT/GOTONEXT actions added by addstate function
       }//can goto
     }// for each item

     // form closures for all new states and add to self.States list
     for key in keyvec // keyvec must be separate to avoid borrow error
     {
        let mut kernel = newstates.remove(&key).unwrap();
        //kernel.conflict_close(&self.Gmr); //mlclosure(kernel,&self.Gmr);
        self.mladdstate(kernel,si,key,agenda); //only place addstate called
        // don't know if this will tobe new state or previous state
        
     }
  }//simplemakegotos

// psi is previous state index, nextsym is next symbol
  fn mladdstate(&mut self, mut state:MLState, psi:usize, nextsymi:usize, agenda:&mut Vec<usize>) ->bool
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
           if state.kernel_eq(&mut self.States[*i]) { //found existing state
             toadd=*i; // toadd changed to index of existing state
             let mut stateclone = MLState {
                index : toadd,
                items : state.items.clone(),
                lhss: BTreeSet::new(), //state.lhss.clone(), //BTreeSet::new(), // will set by stateclosure
                kernel: state.kernel.clone(),
                conflicts: state.conflicts.clone(),
                deprecated: state.deprecated.clone(),
             };
             stateclone.merge_states(&self.States[toadd]);
             //self.state_merge(&self.States[toadd],&mut stateclone);
             if stateclone.items.len() > self.States[toadd].items.len() {
                //stateclone.conflict_close(&self.Gmr,&self.combing); //** closed here
                //closure called only when agenda is popped
                self.States[toadd] = stateclone;
                //self.States[toadd] = mlclosure(stateclone,&self.Gmr);
                // now need to call makegotos again on this state - add
                // to end of open vector.
                self.Open.push(toadd);
                
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

//     if self.Gmr.tracelev>3 {println!("Transition to state {} from state {}, symbol {}..",toadd,psi,nextsym);}

     // toadd is either a new stateindex or an existing one

     if toadd==newstateindex {  // add new state
       indices.insert(newstateindex); // add to StateLookup index hashset
       self.States.push(state);
       self.FSM.push(HashMap::with_capacity(128)); // always add row to fsm at same time
       let mut prev_set = HashSet::new();
       prev_set.insert((nextsymi,psi));
       self.prev_states.insert(newstateindex,prev_set);
       //if self.lalr {self.Open.push(newstateindex)}  //lalr
       agenda.push(newstateindex);
     }// add new state
     else { // add to prev_states
       self.prev_states.get_mut(&toadd).unwrap().insert((nextsymi,psi));
       // propagate conflicts backwards
       let mut backconfs = HashSet::new();
       for LRitem{ri,pi,la} in self.States[toadd].conflicts.iter() {
         if *pi>0 && self.Gmr.Rules[*ri].rhs[pi-1].index==nextsymi {
            backconfs.insert(LRitem{ri:*ri,pi:pi-1,la:*la});
         }
       }//for
       let mut bchanged = false;
       for bc in backconfs {bchanged=self.States[psi].conflicts.insert(bc)||bchanged;}
       if bchanged {agenda.push(psi);}   
       
     }// existing state

     // add to- or change FSM TABLE ...  only Shift or Gotnext added here.
//     let nextsymi = *self.Gmr.Symhash.get(nextsym).expect("GRAMMAR CORRUPTION, UNKOWN SYMBOL");
     let gsymbol = &self.Gmr.Symbols[nextsymi]; //self.Gmr.getsym(nextsym).
     let newaction = if gsymbol.terminal {Stateaction::Shift(toadd)}
        else {Stateaction::Gotonext(toadd)};

// toadd is index of next state, new or old

     let mut newconflicts = mladd_action(&mut self.FSM, &self.Gmr, newaction,psi,nextsymi,&mut self.sr_conflicts,true);
     // append conflicts to the state just added.
//     for c in newconflicts { self.States[psi].conflicts.insert(c);}

/*     
     // maybe shouldn't do this here: not enough!
     newconflicts = vec![];
     for LRitem{ri,pi,la} in self.States[toadd].conflicts.iter() {
       if *pi>0 {newconflicts.push(LRitem{ri:*ri,pi:pi-1,la:*la});}
     }
     // Addconflict to previous state, according to selml alg
     let mut reschedule = false;
     for nc in newconflicts {reschedule=self.States[psi].conflicts.insert(nc)||reschedule;}
     // once conflicts added to psi state, must compute other conflicts in
     // same state - call conflict_close, which then invokes itself on
     // previous state?
*/     
     false
  }  //mladdstate


// set reduce/accept actions at the end, starting with startrule
  fn mlset_reduce(&mut self)
  {
     let mut interior:HashSet<usize> = HashSet::new();
     let mut frontier = vec![self.Gmr.startrulei];
     while frontier.len()>0
     {
       let si = frontier.pop().unwrap();
       interior.insert(si);
       // expand frontier
       for (_,action) in self.FSM[si].iter() {
         match action {
           Shift(nsi) | Gotonext(nsi) => {
             if !interior.contains(nsi) {frontier.push(*nsi);} 
           },
           _ => {},
         }//match
       } // expand frontier
       // process this item - insert actions
       for item in &self.States[si].items
       {
         let (ri,pi,la) = (item.ri,item.pi,item.la);
         if pi==self.Gmr.Rules[ri].rhs.len() { //dot at end of rhs
             //println!("adding reduce/accept rule {}, la {}",ri,&self.Gmr.Symbols[*la].sym);
             let isaccept = (ri== self.Gmr.startrulei && la==self.Gmr.eoftermi);
             if isaccept {
               add_action(&mut self.FSM,&self.Gmr,Accept,si,la,&mut self.sr_conflicts,false);  // don't check conflicts here
             }
             else {
               add_action(&mut self.FSM,&self.Gmr,Reduce(ri),si,la,&mut self.sr_conflicts,true);  // check conflicts here
             }
         }//if reduce situation
       } // for each item
     }// while frontier exists
     // eliminate extraneous states
     for si in 0..self.FSM.len() {
       if !interior.contains(&si) { self.FSM[si] = HashMap::new(); }
     }
  }//mlset_reduce


// replaces genfsm procedure
  pub fn selml(&mut self, k:usize) // algorithm according to paper (k=max delay)
  {

     // modify startrule
     let sri = self.Gmr.startrulei;
     for i in 1..MAXK {
      self.Gmr.Rules[sri].rhs.push(self.Gmr.Symbols[self.Gmr.eoftermi].clone());
     }
     // agenda is a state index, possibly indicating just a kernel

     // construct initial state for START --> . topsym EOF^k, EOF
     // place in agenda
     let mut startstate = MLState::new();
     startstate.insert( LRitem {
         ri : self.Gmr.startrulei, //self.Gmr.Rules.len()-1, 
         pi : 0,
         la : self.Gmr.eoftermi,
       },self.Gmr.startnti);
     self.States.push(startstate); //index always 0
     self.FSM.push(HashMap::with_capacity(128)); // row for state
     self.prev_states.insert(0,HashSet::new());
     let mut agenda = vec![0]; // start with start state
     while agenda.len()>0
     {
        let si:usize = agenda.pop().unwrap();
        self.States[si].conflict_close(&mut self.Gmr,&mut self.combing);
        for ((symi,psi)) in self.prev_states.get(&si).unwrap().iter() {
          let mut newconfs = HashSet::new(); //backwards propagation
          for LRitem{ri,pi,la} in self.States[si].conflicts.iter() {
            if *pi>0 && *symi == self.Gmr.Rules[*ri].rhs[pi-1].index {
                 newconfs.insert(LRitem{ri:*ri,pi:pi-1,la:*la});
            }// symi matches
          } // for each conflict in si
          let mut added = false;
          for nc in newconfs {
            if self.States[*psi].conflicts.insert(nc) {added=true;}
          }
          if added {agenda.push(*psi);}
        } // for each conflict and previous state
        // for loop will not run at first for start state

        // compute qmax of si - take out deprecated items
        let depitems = self.States[si].deprecated.clone();
        for ditem in depitems.iter() {
           self.States[si].items.remove(ditem);
        }
        
        // now call makegotos, create kernels of new states, call addstate...
        self.simplemakegotos(si,&mut agenda); //won't detect conflicts
        // create version that does not detect conflicts.  But then
        // when should reduce actions be added?  at the end.
     }//while agenda exists
     self.mlset_reduce()
  }//selml

}//impl MLStatemachine


///// mladd_action must detect conflicts and build .conflicts set.
pub  fn mladd_action(FSM: &mut Vec<HashMap<usize,Stateaction>>, Gmr:&Grammar, newaction:Stateaction, si:usize, la:usize, conflicts:&mut HashMap<(usize,usize),(bool,bool)>, checkconflict:bool) -> Vec<LRitem> //return conflict items
  {
     let mut answer = Vec::new();
     if !checkconflict {
       FSM[si].insert(la,newaction);
       return answer;
     }
     let currentaction = FSM[si].get(&la);
     let mut changefsm = true; // add or keep current
     match (currentaction, &newaction) {
       (None,_) => {},  // most likely: just add
       (Some(Reduce(rsi)), Shift(_)) => {
         if Gmr.tracelev>4 {
           println!("Shift-Reduce Conflict between rule {} and lookahead {} in state {}",rsi,Gmr.symref(la),si);
         }

         //// construct conflict item as return value
         answer.push(LRitem {ri:*rsi, pi:Gmr.Rules[*rsi].rhs.len(), la:la});
         //// does it matter if we change the FSM here?  it should

         //maybe force changfsm to true to keep shift action.
         changefsm = true;
         if !sr_resolve(Gmr,rsi,la,si,conflicts) {changefsm = false; }
       },
       (Some(Reduce(cri)),Reduce(nri)) if cri==nri => { changefsm=false; },
       (Some(Reduce(cri)),Reduce(nri)) if cri!=nri => { // RR conflict
         let winner = if (cri<nri) {cri} else {nri};
         println!("Reduce-Reduce conflict between rules {} and {} resolved in favor of {} ",cri,nri,winner);
//         printrule(&Gmr.Rules[*cri]);
//         printrule(&Gmr.Rules[*nri]);
         printrulela(*cri,Gmr,la);
         printrulela(*nri,Gmr,la);

         answer.push(LRitem{ri:*cri,pi:Gmr.Rules[*cri].rhs.len(),la:la});
         answer.push(LRitem{ri:*nri,pi:Gmr.Rules[*nri].rhs.len(),la:la});

         changefsm = false; // dont change if conflict detected        
         //if winner==cri {changefsm=false;}
       },
       (Some(Accept),_) => { changefsm = false; },
       (Some(Shift(_)), Reduce(rsi)) => {
         if Gmr.tracelev>4 {
           println!("Shift-Reduce Conflict between rule {} and lookahead {} in state {}",rsi,Gmr.symref(la),si);
         }
         // look in state to see which item caused the conflict...

         answer.push(LRitem {ri:*rsi, pi:Gmr.Rules[*rsi].rhs.len(), la:la});
         // maybe force changefsm to false to keep shift action.
         changefsm = false;
         //if !sr_resolve(Gmr,rsi,la,si,conflicts) {changefsm = false; }
       },
       _ => {}, // default add newstate
     }// match currentaction
     if changefsm { FSM[si].insert(la,newaction); }
     answer
  }//mladd_action

// startstate may get transformed to something else, which changes where
// the Accept action should be inserted.  Tracing is important


//whenever a state's item set has been expanded, we need to put it back
//on the Open list - LR or LALR - to call closure and makegotos again.
// As soon as step "extension" is applied, adding new NT, new rules and
// new item to a state, we should recompute the state's closure  -- bad idea,
// Better to completely compute the conflicts set and deprecate set first before
// calling it a state!   but later it could change.

/*
// closes items as well as conflicts
pub fn mlclosure(mut state:MLState, Gmr:&Grammar) -> MLState
{
  let mut closed =MLState::new();  // closed set,
  closed.index = state.index;
  while state.items.len()>0
  {  
     let nextitem = state.items.iter().next().unwrap().clone();
     let item = state.items.take(&nextitem).unwrap();
     let (ri,pi,la) = (item.ri,item.pi,item.la);
     let rulei = &Gmr.Rules[ri]; 
     let lhsi = rulei.lhs.index; 
     closed.insert(nextitem,lhsi); // place item in interior
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
            if !closed.items.contains(&newitem)  {
              state.insert(newitem,nti_lhsi); // add to "frontier"
            }
          }//for each possible lookahead following non-terminal
       }// for each rule in this non-terminal
     } // add items to closure for this item
     // find conflicts -- do it here or later?
     // much better to detect conflicts on the fly.. forget about
     // operator precedence for now.
  }  // while not closed  // closed complete
  // conflicts are calculated when we add state and find conflict.
  closed.conflicts = state.conflicts; // transfer over
  closed.deprecated = state.deprecated;
  closed
}//stateclosure generation
*/
/* -- in MLStatemachine
// generate the GOTO sets of a state with index si, creates new states
  fn mlmakegotos(&mut self, si:usize)
  {
     // key to following hashmap is the next symbol's index after pi (the dot)
     // the values of the map are the "kernels" of the next state to generate
     let mut newstates:HashMap<usize,MLState> = HashMap::with_capacity(128);
     let mut keyvec:Vec<usize> = Vec::with_capacity(128); //keys of newstates
     let mut allconflicts = Vec::new();
     for item in &self.States[si].items
     {
       let rule = self.Gmr.Rules.get(item.ri).unwrap();
       if item.pi<rule.rhs.len() { // can goto (dot before end of rule)
          let nextsymi = rule.rhs[item.pi].index;
          if let None = newstates.get(&nextsymi) {
             newstates.insert(nextsymi,MLState::new());
             keyvec.push(nextsymi); // push only if unqiue
          }
          let symstate = newstates.get_mut(&nextsymi).unwrap();
          let newitem = LRitem { // this will be a kernel item in new state
             ri : item.ri,
             pi : item.pi+1,
             la : item.la, 
          };
          //let lhssym = &self.Gmr.Rules[item.ri].lhs.sym;
          let lhssymi = self.Gmr.Rules[item.ri].lhs.index; 
          symstate.insert(newitem,lhssymi);
          // SHIFT/GOTONEXT actions added by addstate function
       }//can goto
       else // . at end of production, this is a reduce situation
       {
          let isaccept = (item.ri == self.Gmr.startrulei && self.Gmr.symref(item.la)=="EOF");
          let mut newconflicts;
          if isaccept {
          
            newconflicts=mladd_action(&mut self.FSM,&self.Gmr,Accept,si,item.la,&mut self.sr_conflicts,true);
          }
          else {
            newconflicts=mladd_action(&mut self.FSM, &self.Gmr,Reduce(item.ri),si,item.la,&mut self.sr_conflicts,true);
          }
          allconflicts.append(&mut newconflicts);
       } // set reduce action
     }// for each item

     for c in allconflicts { self.States[si].conflicts.insert(c); }

     // form closures for all new states and add to self.States list
     for key in keyvec // keyvec must be separate to avoid borrow error
     {
        let mut kernel = newstates.remove(&key).unwrap();
 //       kernel.conflict_close(&self.Gmr,&self.combing); //mlclosure(kernel,&self.Gmr);
        self.mladdstate(kernel,si,key); //only place addstate called
     }
  }//mlmakegotos
*/






//////////////////////////////////////
//////////////////////////////////////
// implemented marked delaying transformations.
impl Grammar
{
  // version that does not change existing rule
  pub fn delay_extend(&mut self,ri:usize,dbegin:usize,dend:usize,combing:&mut Bimap<usize,Vec<usize>>) -> usize
  {
    let mut ntcx = self.ntcxmax+1;
//forget about delay markers.  called with rule number, dbegin and dend
       // check if first symbol at marker is a nonterminal
       let NT1 = &self.Rules[ri].rhs[dbegin];
       //let mut rulei = self.Rules[ri].clone();
       // construct suffix delta to be added to each rule
       let mut delta = Vec::new();
       for i in dbegin+1..dend {
         delta.push(self.Rules[ri].rhs[i].clone());
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

         let mut nttype = String::from("(");
         for i in dbegin .. dend {
           let rsymi = self.Rules[ri].rhs[i].index;
           nttype.push_str(&format!("{},",&self.Symbols[rsymi].rusttype));
         }
         nttype.push(')');
         self.enumhash.insert(nttype.clone(),ntcx); ntcx+=1;
         newnt.rusttype = nttype;
         newnt.index = self.Symbols.len();
         self.Symbols.push(newnt.clone());
         self.Symhash.insert(newntname.clone(),self.Symbols.len()-1);

         // register new combed NT with combing map
         let defvec = vec![self.Rules[ri].rhs[dbegin].index];
         let mut oldvec = combing.get(&self.Rules[ri].rhs[dbegin].index).unwrap_or(&defvec).clone();
         oldvec.push(self.Rules[ri].rhs[dend-1].index);
         combing.insert(newnt.index,oldvec);

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
           let newvar = format!("_delvar_{}_{}_",&newnt.index,dbegin);
// check for return at end of last action.

           let mut actionri = format!(" let {} = {{ {}; ",&newvar,self.Rules[*ntri].action); // retrieves value from original action.
           // need to assign values to new items added to delta
           // they will be popped off of the stack by parser_writer as
           // item2, item1 item0...  because parser writer will write an action
           // for the extended rule. [Mc] --> abc
           
           let mut dtuple = format!("({},",&newvar);
           let mut labi = self.Rules[*ntri].rhs.len(); // original rule rhs len
           for sym in &delta {
             let defaultlabel =format!("_item_del{}_{}_",&labi,ntri);
             let slabel = if sym.label.len()>0 {checkboxlabel(&sym.label)}
               else {
               // set label!
               newrule.rhs[labi].label = defaultlabel.clone();
                &defaultlabel
             };
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

       ////// do not change original rule, form a new rule.
       let mut newrulei = Grule::from_lhs(&self.Rules[ri].lhs); //copy
       let mut newrhs = Vec::with_capacity(self.Rules[ri].rhs.len()-1);
       if dbegin>0 {
         for i in 0..dbegin {newrhs.push(self.Rules[ri].rhs[i].clone());}
       }
       let mut clonenewnt = newnt.clone();
       let ntlabel = format!("_delayeditem{}_",dbegin);
       clonenewnt.label = ntlabel.clone();
       newrhs.push(clonenewnt); // newnt added to rule!
       for i in dend .. self.Rules[ri].rhs.len() {
         newrhs.push(self.Rules[ri].rhs[i].clone());
       }

       /////// change semantic action of original rule.
       let mut newaction = String::from(" ");
       // break up tuple
       //let mut labi = 0;
       for i in dbegin..dend {
          let defaultlab = format!("_item{}_",i);
          let symi = &self.Rules[ri].rhs[i]; // original rule
          let labeli = if symi.label.len()>0 {checkboxlabel(&symi.label)}
            else {&defaultlab};
          newaction.push_str(&format!("let mut {} = {}.{}; ",labeli,&ntlabel,i-dbegin));
          //labi+=1;
       }// break up tuple
       // anything to do with the other values?  they have labels, but indexes
       // may be off - but original actions will refer to them as-is.
       newaction.push_str(&self.Rules[ri].action);
       newrulei.rhs = newrhs; // change rhs of rule
       newrulei.action = newaction;

       // special case: newrule becomes startrule if startrule changed.
       if newrulei.lhs.index == self.startnti {
         self.Rules[self.startrulei] = newrulei;
         return self.startrulei;
       } else {      //register new rule
         self.Rulesfor.get_mut(&newrulei.lhs.index).unwrap().insert(self.Rules.len());
         if self.tracelev>1 {
           print!("TRANSFORMED RULE FOR DELAY: ");
           printrule(&newrulei,ri);
         }
         self.Rules.push(newrulei);
         return self.Rules.len()-1;
       }// new rule added (not start rule, which is replaced).
  }// delay_extend



////////////////// don't touch - in use!
  // this must be called before start symbol, eof and startrule added to grammar!
  pub fn delay_transform(&mut self)
  {
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
           let newvar = format!("_delvar_{}_{}_",&newnt.index,dbegin);
// check for return at end of last action.

           let mut actionri = format!(" let {} = {{ {}; ",&newvar,self.Rules[*ntri].action); // retrieves value from original action.
           // need to assign values to new items added to delta
           // they will be popped off of the stack by parser_writer as
           // item2, item1 item0...  because parser writer will write an action
           // for the extended rule. [Mc] --> abc
           
           let mut dtuple = format!("({},",&newvar);
           let mut labi = self.Rules[*ntri].rhs.len(); // original rule rhs len
           for sym in &delta {
             let defaultlabel =format!("_item_del{}_{}_",&labi,ntri);
             let slabel = if sym.label.len()>0 {checkboxlabel(&sym.label)}
               else {
               // set label!
               newrule.rhs[labi].label = defaultlabel.clone();
                &defaultlabel
             };
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
