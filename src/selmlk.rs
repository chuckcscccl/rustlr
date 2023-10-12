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
use std::collections::{HashMap,HashSet,BTreeSet};
use std::cell::{RefCell,Ref,RefMut};
use std::hash::{Hash,Hasher};
use std::io::{self,Read,Write,BufReader,BufRead};
use crate::grammar_processor::*;
use crate::lr_statemachine::*;
use crate::Stateaction;
use crate::Stateaction::*;
use crate::sd_parserwriter::decode_label;

const LTRACE:bool = false; //true;

pub const MAXK:usize = 2; // this is only the default

type AGENDATYPE = Vec<usize>;
type PREAGENDATYPE = HashSet<usize>;
type COMBINGTYPE = HashMap<usize,Vec<usize>>;
type ITEMSETTYPE = HashSet<LRitem>;

// #[derive(Clone,Debug,Default)]
pub struct MLState // emulates LR1/oldlalr engine
{
   index: usize, // index into vector
   items: ITEMSETTYPE,
   lhss: BTreeSet<usize>,  // set of left-side non-terminal indices
   conflicts: ITEMSETTYPE,
   deprecated: HashSet<LRitem>,
   lrkernel : ITEMSETTYPE,  // must be btree, else non-deterministic
}
impl MLState
{
  pub fn new() -> MLState
  {
     MLState {
        index : 0,   // need to change
        items : ITEMSETTYPE::new(), //HashSet::with_capacity(512),
        lhss: BTreeSet::new(), // for quick lookup
        conflicts: ITEMSETTYPE::new(),
        deprecated: HashSet::new(),
        lrkernel : ITEMSETTYPE::new(),
     }
  }

  pub fn insert(&mut self, item:LRitem, lhs:usize) -> bool
  {
     let inserted = self.items.insert(item);
     self.lhss.insert(lhs);
     inserted
  }
  pub fn insert_kernel(&mut self, item:LRitem, lhs:usize) -> bool
  {
     let inserted = self.lrkernel.insert(item);
     self.lhss.insert(lhs);
     inserted
  }  
  fn contains(&self, x:&LRitem) -> bool {self.items.contains(x)}
  
  fn hashval(&self) -> usize  // note: NOT UNIQUE
  {
  /*
    let mut key=self.kernel.len()+ self.lhss.len()*10000;
    let limit = usize::MAX/1000 -1;
    let mut cx = 8;
    for s in &self.lhss {key+=s*1000; cx-=1; if cx==0  || key>=limit {break;}}
    key
  */
    //self.items.len() + self.lhss.len()*10000
    self.lrkernel.len()
  } //


// new closure method, based on kernels, NO SR/RR CONFLICT DETECTION
// returns false on failure. (?)
// if conflicts are detected here, will also have to resolve based on
// precedence, known_conflicts
  fn close_all(&mut self, Gmr:&mut Grammar, combing:&mut COMBINGTYPE, known_conflicts:&mut HashMap<(bool,usize,usize),(bool,usize)>, rhash:&mut HashMap<Vec<usize>,usize>,maxk:usize,mut failed:bool) -> bool
  {  let mut answer = true;
     // start with kernel items.
     if !failed {self.items.clear();}
     //self.items.clear();

/*
     let mut loopcx = 0;
   loop {
     loopcx += 1;
     let ksize = self.lrkernel.len();
     let csize = self.conflicts.len();
     let dsize = self.deprecated.len();
     let isize = self.items.len();
*/
     let mut closure = Vec::new();
     let mut closed = 0;
     let mut onclosure = HashSet::new();
     for item in self.lrkernel.iter() {
       closure.push(*item);
       onclosure.insert(*item);
       //self.items.insert(*item); /////////////////////////
     }
     while closed < closure.len()
     {
        let item = closure[closed]; //copy
        closed+=1;
        if failed && self.deprecated.contains(&item) {continue;} //******
        self.items.insert(item); // maybe duplicate
        let (ri,pi,la) = (&item.ri,&item.pi,&item.la);
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
                 if !self.items.contains(&newitem) && !onclosure.contains(&newitem) /*  && !self.deprecated.contains(&newitem) */ {
                  closure.push(newitem);
                  onclosure.insert(newitem);
                 }
               } //for each possible la
             }//for rulent
        }// add new, non-closureitem (if)

        // detect conflicts -- when should this be done?  on the fly

        // add conflict due to conflict propagation -these can't lead to
        // new closure items because they can't be extended
        let mut newconflicts = HashSet::new();
        let mut defaultclas = HashSet::new(); defaultclas.insert(*la);
        let clas = if pi+1<Gmr.Rules[*ri].rhs.len() {Gmr.Firstseq(&Gmr.Rules[*ri].rhs[pi+1..],*la)} else {defaultclas};

        for citem@LRitem{ri:cri,pi:cpi,la:cla} in self.conflicts.iter() {
            //failure detection
            let rilhs = Gmr.Rules[*ri].lhs.index;
            let defaultcomb = vec![rilhs];            
            //let comb = combing.get(&rilhs).unwrap_or(&defaultcomb);
            //let comb = uncombing(&rilhs,combing);
             /*
             if *cpi==0 { // cannot be extended?
              let comblen = comblength(&rilhs,combing);
              if comblen>maxk {
               if !failed {
                  let comb = uncombing(&rilhs,combing);               
                  Gmr.logprint("FAILURE. MAXIMAL COMBING IN CONFLICT:\n  [[ ");
                  for x in &comb {
                    if Gmr.tracelev>0 { print!("{} ",&Gmr.Symbols[*x].sym);}
                    else {Gmr.genlog.push_str(&format!("{} ",&Gmr.Symbols[*x].sym));}
                  }
                  Gmr.logprint("]]");
                  failed = true;
               }
               //return false;    // report failure
               answer = false;
              }
             }//failure detection
             */
            
            // same conflict item can be used to detect other extension
            // possibilities. can't deprecate til end of for citems loop
            //if self.deprecated.contains(citem) {continue;} //MUST NOT HAVE
            if *cpi==0 && pi+1==Gmr.Rules[*ri].rhs.len() && Gmr.Rules[*cri].lhs.index==Gmr.Rules[*ri].rhs[*pi].index && cla==la{ //conflict propagation


              // !% propagation
              if let Some(cutpi)=Gmr.sdcuts.get(ri) {
                if *cutpi == Gmr.Rules[*ri].rhs.len() {
                   Gmr.sdcuts.insert(*cri,Gmr.Rules[*cri].rhs.len());
                }
              }

              newconflicts.insert(item);
              // PROPAGATION  A --> alpha .B, can't extend further
              // But what if the rule is A -> alpha . B !%?
              // Propagation of conflict should take place, but no extension
              // should take place?
            }
            else if *cpi==0 && pi+1<Gmr.Rules[*ri].rhs.len() && Gmr.Rules[*cri].lhs.index==Gmr.Rules[*ri].rhs[*pi].index && clas.contains(cla) {
              //assert!(!Gmr.Rules[*ri].rhs[*pi].terminal);
              
              // (extension) step
              /*
              // make sure set of la's are exactly the same (according to alg)
              let lascitem = collect_la(&self.conflicts,*cri,*cpi);
              if lascitem != clas {continue;} // onto next item
              */

              let nti = Gmr.Rules[*ri].rhs[*pi].index;
              //let defaultcomb = vec![nti];
              //let comb = combing.get(&nti).unwrap_or(&defaultcomb);

              // new January 2023: must add check against !% marks that
              // forces extension to stop (stateful semantic actions).

               let comblen = comblength(&nti,combing);

               //if Gmr.sdcuts.contains(&(*cri,Gmr.Rules[*cri].rhs.len())) {
               match Gmr.sdcuts.get(cri) {
                 Some(cutpi) if *cutpi==Gmr.Rules[*cri].rhs.len() => {
                   if !failed {
                     Gmr.logprint(&format!("\nSELECTIVE DELAY EXTENSION FAILED due to !% at end of rule {}",cri));
                     printrule2(*cri,Gmr,combing);
                   }
                   answer = false;               
                 },
                 _ => {},
               }//match
               // but what if this is recursive, as in A --> B !%
               // C --> A;  D --> C?  In these cases the as soon as a
               // conflict item with !% at the end is detected, failure is
               // reported.
               if answer {
                match Gmr.sdcuts.get(ri) {
                 Some(cutpi) if *cutpi==pi+1 => {
                   if !failed {
                     Gmr.logprint(&format!("\nSELECTIVE DELAY EXTENSION FAILED due to !% marker at rule {}, position {}, which may have been inherited from an original rule",ri,pi+1));
                     printrule2(*ri,Gmr,combing);
                   }
                   answer = false;
                 },
                 _ => {},
                }//match
               }//if answer
               
               if answer && comblen>maxk {
                 if !failed {
                    let comb = uncombing(&rilhs,combing);               
                    Gmr.logprint0("FAILURE. MAXIMUM COMBING IN CONFLICT:\n  [[ ");
                    for x in &comb {
                      Gmr.logprint0(&format!("{} ",&Gmr.Symbols[*x].sym));
                    }
                    Gmr.logprint("]]");
                    failed = true;
                 }//print
                 answer =  false;
               }//failure detected
             
             else if answer {

              // deprecate "shorter" items
              self.deprecated.insert(item);
              // others of this type?
              //self.deprecated.insert(*citem); /////redundant?
              for dri in Gmr.Rulesfor.get(&nti).unwrap().iter() {
                for dla in clas.iter() {
                  self.deprecated.insert(LRitem{ri:*dri,pi:0,la:*dla});
                }
              }// deprecate closure
              // why not just remove deprecated items on the fly?

              /////// got to create new symbol, rule, then insert into
              /////// items (closed), and deprecate others.
              // maybe dynamically create new symbol, change rule here...***
              // extend one at a time              
              let eri=Gmr.delay_extend(*ri,*pi,pi+2,combing/*,comb*/,rhash);
              // this return index of new rule with longer delay
              let extenditem = LRitem{ri:eri, pi:*pi, la:*la};
              // same rule in different places: deprecation in one place
              // could affect the other.
              if  /* !self.deprecated.contains(&extenditem) && */   !self.items.contains(&extenditem) {
                //self.lrkernel.insert(extenditem);  ////?????
                // if the above line is added, then two kernels should be
                // considered the same if they contain the same non-deprecated
                // items.
                if !onclosure.contains(&extenditem) {
                  closure.push(extenditem);
                  onclosure.insert(extenditem);
//if true || LTRACE {print!("got rule {}, state {}, EXTENSION OF: ",eri,self.index); printitem2(&item,Gmr,combing);}
                }
                //closure.insert(0,extenditem);
                //closed=0;
              } //extenditem already onclosure?
             } // can extend (does not exceed maxk (closes else if answer)
            } // conflict extension
        }//for each conflict item
        let mut added = false;
        for nc in newconflicts {
          if /* !self.deprecated.contains(&nc) && */ self.conflicts.insert(nc) {
             added = true;
//if true ||LTRACE {println!("CONFLICT PROPAGATION in state {}",self.index);         printitem2(&nc,Gmr,combing);}
          }
        }
        // all the items have to now be re-checked against new confs
        if added {
          //closure.clear();
          //self.items.clear();
          //for item in self.lrkernel.iter() {closure.push(*item);} //copy
          closed = 0;
        }
     }// while !closed
/*
     if self.lrkernel.len()==ksize && self.conflicts.len()==csize &&
        self.items.len()==isize && self.deprecated.len()==dsize {break;}
   } // loop
   if loopcx>2 {println!("LOOP RAN {} TIMES",loopcx);}
*/   
     answer
  }//close_all

}// impl MLState


impl PartialEq for MLState
{
/*
testing the entire items closure set - assume deprecated removed.
This didn't work when agend items weren't closed.  but now they are.
STILL goes into loop. keep creating new states, why?  
*/

   fn eq(&self, other:&MLState) -> bool {

      if self.lrkernel.len() != other.lrkernel.len() { return false; } // ***
      for item in self.lrkernel.iter() {
        if !other.lrkernel.contains(item) {return false;}
      }
      true
   }//eq

/*
   // version that compares kernels for non-deprecated items
   fn eq(&self, other:&MLState) -> bool {
     for item in self.lrkernel.iter() {
        if !self.deprecated.contains(item) &&
           !other.deprecated.contains(item) &&
           !other.lrkernel.contains(item) {return false;}
     }
     for item in other.lrkernel.iter() {
        if !other.deprecated.contains(item) &&
           !self.deprecated.contains(item) &&
           !self.lrkernel.contains(item) {return false;}
     }
     true
   }
*/

   fn ne(&self, other:&MLState) ->bool
   {!self.eq(other)}
}
impl Eq for MLState {}

// known-conflict structure:
// (type rr/sr ,rule1, rule2/shift_la)  -> (clearly_resolved, rule# or 0-shift)

pub struct MLStatemachine  // Consumes Grammar
{
   pub Gmr: Grammar,
   pub States: Vec<MLState>, 
   pub Statelookup: HashMap<usize,LookupSet<usize>>,
   pub FSM: Vec<HashMap<usize,Stateaction>>,
   pub Open: Vec<usize>, // for LALR only, vector of unclosed states
   pub known_conflicts:HashMap<(bool,usize,usize),(bool,usize)>,
   pub prev_states : HashMap<usize,HashSet<(usize,usize)>>,
//   pub back_links : HashMap<(usize,usize),HashSet<usize>>,
   pub combing: COMBINGTYPE,
   pub Ruleshash: HashMap<Vec<usize>,usize>,
   preagenda : PREAGENDATYPE,
   deprecated_states: HashSet<usize>,
   maxk : usize,
   pub failed: bool,
   pub regenerate: bool,
}
impl MLStatemachine
{
  pub fn new(gram:Grammar) -> Self
  { 
       MLStatemachine {
          Gmr: gram,
          States: Vec::with_capacity(8*1024), // reserve 8K states
          Statelookup: HashMap::with_capacity(1024),
          FSM: Vec::with_capacity(8*1024),
          Open: Vec::new(), // not used for lr1, set externally if lalr
          known_conflicts:HashMap::new(),
          prev_states:HashMap::new(), //state --> symbol,state parent
//          back_links:HashMap::new(),
          combing: COMBINGTYPE::new(), //stores what each nt really represents
          Ruleshash:HashMap::new(),
          preagenda: PREAGENDATYPE::new(),
          deprecated_states: HashSet::new(),
          maxk:MAXK,
          failed: false,
          regenerate: false,
       }
  }//new

// calle on fully closed state si, produces new state kernels:
// returns false if nothing changed
  fn simplemakegotos(&mut self,si:usize,agenda:&mut AGENDATYPE) -> bool
  {  let mut answer = false;
     // key to following hashmap is the next symbol's index after pi (the dot)
     // the values of the map are the "kernels" of the next state to generate
     let mut newstates:HashMap<usize,MLState> = HashMap::with_capacity(128);
     let mut keyvec:Vec<usize> = Vec::with_capacity(128); //keys of newstates
     let mut newsiconflicts = HashSet::new();
     let mut reagenda = false;

     for item in self.States[si].items.iter()
     {
       if self.States[si].deprecated.contains(item) {continue;}
       let isaccept = (item.ri == self.Gmr.startrulei && item.la==self.Gmr.eoftermi && item.pi>0);
       let rule = self.Gmr.Rules.get(item.ri).unwrap();
       if item.pi<rule.rhs.len() && !isaccept {  // can goto? (dot before end of rule)
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
          symstate.insert_kernel(newitem,lhssymi);
          // SHIFT/GOTONEXT actions to be added by addstate function
       }//can goto
       else // . at end of production, or isaccept - or reduce
       {
          let isconflict;
          if isaccept {
            isconflict =tryadd_action(&mut self.FSM,&mut self.Gmr,Accept,si,item.la,&mut self.known_conflicts,false,self.failed);
          }
          else {
            isconflict=tryadd_action(&mut self.FSM, &mut self.Gmr,Reduce(item.ri),si,item.la,&mut self.known_conflicts,false,self.failed);
          }
          // if tryadd action returned conflict, insert conflict into
          // state's conflict set
          // but this only allows us to insert a reduce-type conflict no
          // info concerning shift conflict.  but that's OK!
          match &isconflict {
            (changed,Some((false,r1,la1))) => { //false=sr conflict
              let add1 = newsiconflicts.insert(LRitem{ri:*r1,pi:self.Gmr.Rules[*r1].rhs.len(),la:*la1});
              if self.Gmr.Rules[*r1].rhs.len()==0 && add1 {reagenda=true;
                //println!("state {} re-agenda because of null rule {}",si,r1);
              }
              answer = *changed || answer;
            },
            (changed,Some((true,r1,r2))) => { //rr conflict
              let add1 = newsiconflicts.insert(LRitem{ri:*r1,pi:self.Gmr.Rules[*r1].rhs.len(),la:item.la});
              let add2 = newsiconflicts.insert(LRitem{ri:*r2,pi:self.Gmr.Rules[*r2].rhs.len(),la:item.la});
              if (self.Gmr.Rules[*r1].rhs.len()==0 && add1)  || (self.Gmr.Rules[*r2].rhs.len()==0 && add2) {reagenda=true;
//println!("state {} added back to agenda because of null-production conflict",si);
              }
              answer = *changed || answer;
            },
            (changed,_) => {answer=*changed||answer;},
          }//match isconflict
          
          // only place addreduce is called
       } // set reduce action
     }// for each item
     let mut inserted = false;
     for ncf in newsiconflicts {
       //if self.States[si].deprecated.contains(&ncf) {continue;}
       inserted = self.States[si].conflicts.insert(ncf) || inserted;
     } // insert new conflicts into CURRENT state
     if inserted /* && !self.failed */ {
if LTRACE {println!("state {} added back to agenda due to new conflicts",si);}
       return self.preagenda_add(si);
     } // re-agenda

     // form closures for all new states and add to self.States list
     for key in keyvec // keyvec must be separate to avoid borrow error
     {
        let mut kernel = newstates.remove(&key).unwrap();
         answer = self.mladdstate(kernel,si,key,agenda) || answer;
     }
     answer
  }//simplemakegotos

// psi is previous state index, nextsym is next symbol, gets kernel,
// if function returns false if nothing changed
  fn mladdstate(&mut self, mut state:MLState, psi:usize, nextsymi:usize, agenda:&mut AGENDATYPE) -> bool
  {  let mut answer = false;
     let nextsym = &self.Gmr.Symbols[nextsymi].sym;
     let newstateindex = self.States.len(); // index of new state
     state.index = newstateindex;
     let lookupkey =state.hashval();
     if let None=self.Statelookup.get(&lookupkey) {
        self.Statelookup.insert(lookupkey,LookupSet::new());
     }
     let indices = self.Statelookup.get_mut(&lookupkey).unwrap();
     let mut toadd = newstateindex; // defaut is add new state (will push)
     for i in indices.iter() //0..self.States.len()
     {
//       if self.preagenda.contains(&i) {continue;}
       if &state==&self.States[*i] {toadd=*i; break; } // state i exists
     }

//     if self.Gmr.tracelev>3 {println!("Transition to state {} from state {}, symbol {}..",toadd,psi,nextsym);}

     // toadd is either a new stateindex or an existing one

     if toadd==newstateindex {  // add new state
if LTRACE {println!("ADDING NEW STATE {}",toadd);}
       indices.insert(newstateindex); // add to StateLookup index hashset
       self.States.push(state);
       self.FSM.push(HashMap::with_capacity(128)); // always add row to fsm at same time
       let mut prev_set = HashSet::new();
       prev_set.insert((nextsymi,psi));
       self.prev_states.insert(newstateindex,prev_set);
       self.preagenda_add(newstateindex);
       answer = true;
if LTRACE {println!("new state {} added to agenda from state {}",newstateindex,psi);}
     }// add new state
     else { // add to prev_states
if LTRACE {println!("FOUND EXISTING STATE {} from state {}",toadd,psi);}
       self.prev_states.get_mut(&toadd).unwrap().insert((nextsymi,psi));
       // propagate conflicts backwards, unless will be done on preagenda
      if !self.preagenda.contains(&toadd) {
       let mut backconfs = HashSet::new();
       for item@LRitem{ri,pi,la} in self.States[toadd].conflicts.iter() {
         //if self.States[toadd].deprecated.contains(item) {continue;} //??
         if *pi>0 && self.Gmr.Rules[*ri].rhs[pi-1].index==nextsymi {
            backconfs.insert(LRitem{ri:*ri,pi:pi-1,la:*la});
         }
       }//for
       let mut bchanged = false;
       for bc in backconfs {
         //if self.States[psi].deprecated.contains(&bc) {continue;}
         if self.States[psi].conflicts.insert(bc) {
            bchanged = true;
            if bc.pi==0 && !self.failed && checkfailure(&bc,&self.Gmr,&self.combing,self.maxk)   {
               let sgri = self.Gmr.Rules[bc.ri].lhs.index;
               reportfailure(&sgri,&mut self.Gmr,&self.combing);
               self.failed=true; break;
            }//failure check
         }//inserted
       }//for bc
       if /* !self.failed &&*/ bchanged && self.preagenda_add(psi) {
if LTRACE {println!("state {} pushed back onto agenda because of backward conflict prop",psi);}
            // since previous state pushed back to agenda, should not
            // form actions from previous state to toadd state
            return true;
       }
       else {  // add back link only if not propagated backwards
         if self.States[toadd].conflicts.len()==0 || bchanged {answer=true;}
//         self.prev_states.get_mut(&toadd).unwrap().insert((nextsymi,psi));
       }       // instead of deprecating conflict alltogether
      }
     }// existing state

     // add to- or change FSM TABLE ...  only Shift or Gotnext added here.
//     let nextsymi = *self.Gmr.Symhash.get(nextsym).expect("GRAMMAR CORRUPTION, UNKOWN SYMBOL");
     let gsymbol = &self.Gmr.Symbols[nextsymi]; //self.Gmr.getsym(nextsym).
     let newaction = if gsymbol.terminal {Stateaction::Shift(toadd)}
        else {Stateaction::Gotonext(toadd)};

     // toadd is index of next state, new or old
     // insert action into FSM

     let isconflict = tryadd_action(&mut self.FSM, &mut self.Gmr, newaction,psi,nextsymi,&mut self.known_conflicts,false,self.failed);
     match &isconflict {
            (changed,Some((false,r1,la1))) => {
              let confitem = LRitem{ri:*r1,pi:self.Gmr.Rules[*r1].rhs.len(),la:*la1};
              if /* !self.failed && !self.States[psi].deprecated.contains(&confitem) && */ self.States[psi].conflicts.insert(confitem) {
                self.preagenda_add(psi);
                answer = true;
if LTRACE {println!("new sr-conflict {:?} detected for state {}, re-agenda",&confitem,psi);}
              }
              answer = *changed || answer;
            },

            (changed,Some((true,r1,r2))) => { // should not be possible
//println!("YIKES!!!!!!");            
              let res1 = self.States[psi].conflicts.insert(LRitem{ri:*r1,pi:self.Gmr.Rules[*r1].rhs.len(),la:nextsymi});
              let res2 = self.States[psi].conflicts.insert(LRitem{ri:*r2,pi:self.Gmr.Rules[*r2].rhs.len(),la:nextsymi});
              if /* !self.failed && */ (res1 || res2) {self.preagenda_add(psi); answer=true;}
              answer = *changed||answer;
            },
            (changed,_) => {answer=*changed||answer},
          }//match isconflict
     answer
  }  //mladdstate

// may not need anymore
// set reduce/accept actions at the end, starting with startrule
  fn mlset_reduce(&mut self)
  {
     let mut interior:HashSet<usize> = HashSet::new();
     let mut frontier = vec![0]; //start with state (not rule) 0
     while frontier.len()>0
     {
       let si = frontier.pop().unwrap();
       if interior.contains(&si) {continue;}  //check during push below

       /*
       // check kernels and items 
         for istate in interior.iter() {
         let same = true;
         if self.States[*istate] == self.States[si] {
           println!("States {} and {} have the same kernel",istate,si);
           println!("Kernel of state {}:",istate);
           for item in self.States[*istate].lrkernel.iter() {
             printitem2(item,&self.Gmr,&self.combing);
           }
         }
       }/////// printing trace
       */
/*
       println!("Kernel of state {}:",si);
       for item in self.States[si].lrkernel.iter() {
         printitem2(item,&self.Gmr,&self.combing);
       }
*/

       interior.insert(si);
       // expand frontier
       for (symi,action) in self.FSM[si].iter() {
         match action {
           Shift(nsi) | Gotonext(nsi) => {
             if !interior.contains(nsi) {
               frontier.push(*nsi);
               //println!("FRONTIER STATE {} ->{}-> {}",si,&self.Gmr.Symbols[*symi].sym,nsi);
             } 
           },
           _ => {},
         }//match
       } // expand frontier
       // process this item - insert actions
       for item in &self.States[si].items
       {
         if self.States[si].deprecated.contains(item) {continue;}
         let (ri,pi,la) = (item.ri,item.pi,item.la);
         let isaccept = (ri== self.Gmr.startrulei && la==self.Gmr.eoftermi && pi>0);
         if isaccept {
               tryadd_action(&mut self.FSM,&mut self.Gmr,Accept,si,la,&mut self.known_conflicts,true,self.failed);  
         }         
         else if pi==self.Gmr.Rules[ri].rhs.len() { //dot at end of rhs
               tryadd_action(&mut self.FSM,&mut self.Gmr,Reduce(ri),si,la,&mut self.known_conflicts,true,self.failed); 
         }//if reduce situation
       } // for each item
     }// while frontier exists
     // eliminate extraneous states
     for si in 0..self.FSM.len() {
       if !interior.contains(&si) { self.FSM[si]=HashMap::new(); }
     }
     self.Gmr.logprint(&format!("LRSD: total reachable states: {}",interior.len()));

  }//mlset_reduce


  fn preagenda_add(&mut self,si:usize) -> bool
  {
  /*
     if !self.onagenda.contains(&si) {
       self.onagenda.insert(si);
       agenda.push(si);
       true
     } else {false}

     let mut answer = self.States[si].close_all(&mut self.Gmr,&mut self.combing,&mut self.known_conflicts,&mut self.Ruleshash,self.maxk,self.failed);
     if !answer {self.failed=true;}
  */     
     self.preagenda.insert(si)
  }


// replaces genfsm procedure
  pub fn selml(&mut self, maxk:usize)// main algorithm (k=max delay)
  {  self.maxk = maxk;
     // modify startrule
     let sri = self.Gmr.startrulei;
     for i in 0..self.maxk+2 {  //give room to detect failure
      self.Gmr.Rules[sri].rhs.push(self.Gmr.Symbols[self.Gmr.eoftermi].clone());
     }
     // agenda is a state index, possibly indicating just a kernel

     // construct initial state for START --> . topsym EOF^k, EOF
     // place in agenda
     let mut startstate = MLState::new();
     startstate.insert_kernel( LRitem {
         ri : self.Gmr.startrulei, //self.Gmr.Rules.len()-1, 
         pi : 0,
         la : self.Gmr.eoftermi,
       },self.Gmr.startnti);
     startstate.index = 0;
     self.States.push(startstate); //index always 0
     self.FSM.push(HashMap::with_capacity(128)); // row for state
     self.prev_states.insert(0,HashSet::new());

     let mut agenda = AGENDATYPE::new();
     let mut onagenda = HashSet::new();
     self.preagenda.clear();
     self.preagenda_add(0);
     let mut failuredetected = false;
     let mut priority_states = HashSet::new();
     
     loop // until agenda empty
     {
       //let mut prcounter = 0;
       priority_states.clear();
       while self.preagenda.len()>0
       {
         if self.failed {break;}
//println!("PREAGENDA SIZE {}, RULES {}",self.preagenda.len(),self.Gmr.Rules.len());

/*
        //////// really crude way to address non-termination (re-shuffle)
        prcounter+=1;
        if prcounter>32 {
println!("PREAGENDA RESUFFLE, preagenda len {}, states {}, rules {}",self.preagenda.len(),self.States.len(),self.Gmr.Rules.len());
if self.preagenda.len()==3 {
  for tsi in self.preagenda.iter() {
    printmlstate(&self.States[*tsi],&self.Gmr,&self.combing);
    println!("{} items, {} kernels",self.States[*tsi].items.len(),self.States[*tsi].lrkernel.len());
  }
  println!();
}
          prcounter = 0;
          let mut newset = HashSet::with_capacity(self.preagenda.len());
          for si in self.preagenda.iter() {newset.insert(*si);}
          self.preagenda = newset;
          priority_states.clear();
        }
        //////// really crude way to address non-termination        
*/

         let mut statei = *self.preagenda.iter().next().unwrap();
      
         while priority_states.len()>0
         {
            let candidatesi = *priority_states.iter().next().unwrap();
            priority_states.remove(&candidatesi);
            if self.preagenda.contains(&candidatesi) {
              statei=candidatesi;
              //println!("PRIORITY STATE {}",statei);
              break;
            }
         } // while there are priority states

         self.preagenda.remove(&statei);
         onagenda.remove(&statei);

         let mut answer = self.States[statei].close_all(&mut self.Gmr,&mut self.combing,&mut self.known_conflicts,&mut self.Ruleshash,self.maxk,self.failed);
         if !answer {self.failed=true; break;}

         // if closure creates new conflicts, they will be propagated.
         let mut new_preagenda = HashSet::new();
         let mut priority = false;
         let mut propagated = false;
         for (symi,psi) in self.prev_states.get(&statei).unwrap().iter() {
           let mut propedpsi=false;
           let mut topropagate = HashSet::new();
           for citem@LRitem{ri,pi,la} in self.States[statei].conflicts.iter() {
             if *pi==0 || *symi!=self.Gmr.Rules[*ri].rhs[pi-1].index  || self.States[statei].deprecated.contains(citem)  {continue;}
             topropagate.insert(LRitem {ri:*ri, pi:pi-1, la:*la});

// should be here according to alg.  All such states are transitory and only
// exists to propagate conflicts backwards.  But what if it also has a
//meaningful conflict of its own?  conflicts can't be deprecated
propagated=true;

             if pi-1>0 { priority=true; }
           }//for each conflict item that must be propagated.
           for bc in topropagate {
             if self.States[*psi].conflicts.insert(bc) {
               if bc.pi==0 && !self.failed && checkfailure(&bc,&self.Gmr,&self.combing,self.maxk) {
                 let sgri = self.Gmr.Rules[bc.ri].lhs.index;
                 reportfailure(&sgri,&mut self.Gmr,&self.combing);
                 self.failed=true; break;
                  }
               propedpsi=true;
             }
           }//for each bc
           if self.failed {break;}
           if propedpsi {new_preagenda.insert(*psi);}
           if propedpsi && priority {priority_states.insert(*psi);}
           propagated = propagated || propedpsi;
         }//for each previous state
//         for dc in deprecated_conflicts {self.States[statei].deprecated.insert(dc); }
         if !propagated && !onagenda.contains(&statei) {
           agenda.push(statei);
           onagenda.insert(statei);
         }
         for psi in new_preagenda {
           self.preagenda_add(psi);
           //onagenda.remove(&psi);  //redundant
         }
         //else {self.deprecated_states.insert(statei);}
         //else { self.prev_states.get_mut(&statei).unwrap().clear(); }
         // if not propagated, can move to agenda, else, invalidate state?
         // not here - prev links could still be useful, if state is
         // rediscovered
         // maybe safest to delete state by putting it on another list.
       }//preagenda loop

       if self.failed {
        if !failuredetected {
          failuredetected = true;
          self.known_conflicts.clear();
          /*
          for i in 0..self.States.len() {
            self.States[i].conflicts.clear();
            //self.States[i].deprecated.clear();
          }

          self.States.truncate(1);
          self.States[0].items.clear();
          self.States[0].conflicts.clear();
          self.States[0].deprecated.clear();          
          self.FSM.clear();
          self.FSM.push(HashMap::with_capacity(128));
          */
          self.prev_states.clear();
          self.preagenda.clear();
          onagenda.clear();
          agenda.clear();
          self.preagenda_add(0);
          self.maxk = 0;
        } // first detecte=ion
        else {
          if agenda.len()==0 {break;}
          let fsi = agenda.pop().unwrap();
          if !onagenda.contains(&fsi) {continue;}
          onagenda.remove(&fsi);          
          self.simplemakegotos(fsi,&mut agenda);
        }
        continue;
       }// failure handling

       // start of main agenda loop, if not failed
       if agenda.len()==0 {break;} //stop outer loop
       let si= agenda.pop().unwrap();
       if !onagenda.contains(&si) {continue;}
       onagenda.remove(&si);
       
/////////////// TRACE
if LTRACE {
  println!("AGENDA len:{}, Popped {},States:{}, kernels:{}, conflicts:{}, deprecated:{}",&agenda.len(),si,self.States.len(), self.States[si].lrkernel.len(), self.States[si].conflicts.len(),self.States[si].deprecated.len());
  if self.States.len()>10000 {break;}
}//trace print
/*
 if self.States.len()>2000 && self.States[si].conflicts.len()<=2 && self.States[si].conflicts.len()>0 {
    println!("REMAINING CONFLICTS:");
    for citem in self.States[si].conflicts.iter() {
      printitem2(citem,&self.Gmr,&self.combing);
    }
    println!("... and REMAINING DEPRECATED in state {} popped from agenda",si);
    for ditem in self.States[si].deprecated.iter() {
      printitem2(ditem,&self.Gmr,&self.combing);      
    }
 }
 */
/////////////// TRACE

          let mut progress=false;
          // all existing FSM transitions for state are now invalid
          // info must be consistent with prev_states
          for (symj, action) in self.FSM[si].iter() {
            let mut nsi = self.FSM.len();
            match action {
              Shift(nnsi) | Gotonext(nnsi) => {nsi=*nnsi;},
              _ => {continue;}
            }//match
            let prevnsi = self.prev_states.get_mut(&nsi);
            if let Some(prevset) = prevnsi {
              prevset.remove(&(*symj,si));
            }
          }// clear prev_states for next states of si
          self.FSM[si].clear();

          // now call makegotos, create kernels of new states, call addstate...
          progress = self.simplemakegotos(si,&mut agenda) || progress;

     }//while agenda exists

     if self.failed {
       self.Gmr.logeprint("\nSELECTIVE DELAY ALGORITHM FAILED; DEFAULTS APPLIED.\n");
     }

     //if LTRACE {println!("CALLING FINAL mlset_reduce..");}
     self.mlset_reduce();

     if self.failed {
       self.Gmr.logeprint("\nConsider the following options:\n  1. extending the maximum length of delays unless you already notice\n     a repeating pattern in the \"maximum combing in conflict.\"\n  2. adding operator precedence and associativity declarations\n  3. rewriting the grammar, perhaps it was ambiguous\n");
     }
//if true || LTRACE {println!("FINAL RULE COUNT: {}",self.Gmr.Rules.len());}

// re-prepare grammar
  if self.regenerate {
    // prepare grammar, recompute reachability and eliminate unused rules
    let mut liverules = BTreeSet::new();
    for state in &self.States {
      if self.FSM[state.index].len()==0 {continue;} //non-reachable state
      /*
      for item in state.lrkernel.iter() {
        if !state.deprecated.contains(item) {
          liverules.insert(item.ri);
        }
      }
      */
      for item in state.items.iter() {
        if !state.deprecated.contains(item) {
          liverules.insert(item.ri);
        }
      }      
    }//for each state, collect liverules

   let mut numrules = self.Gmr.Rules.len();
   while numrules!=liverules.len() {  
    // modify Rulesfor
    for (nt,ntrules) in self.Gmr.Rulesfor.iter_mut() {
      for i in 0..self.Gmr.Rules.len() {
        if !liverules.contains(&i) {ntrules.remove(&i);}
      }
    }//for each rule
    /*
    self.Gmr.Reachable.clear();
    self.Gmr.reachability();
    let reachable = self.Gmr.Reachable.get(&self.Gmr.startnti).unwrap(); //reachable from start
    */
    numrules = liverules.len();
    // modify liverules further
    let mut zombies = HashSet::new();
    for ri in &liverules {
      for sym in self.Gmr.Rules[*ri].rhs.iter() {
      //  if !reachable.contains(&sym.index) {zombies.insert(*ri);}
        if !sym.terminal {
          match self.Gmr.Rulesfor.get(&sym.index) {
            Some(x) if x.len()==0 => {zombies.insert(*ri);},
            _ => {},
          }//match
        }//if nonterminal
      } //for each sym on rhs of rule
    }//second ri loo
    for z in &zombies {liverules.remove(z);}
   } // while numrules!=liverules.len
    if self.Gmr.tracelev>1 {
     println!("ALL RULES OF TRANSFORMED GRAMMAR");
     for ri in &liverules {
       printrule2(*ri,&mut self.Gmr,&self.combing);
     }
    }
  }//if regenerate
     if self.Gmr.tracelev>4 {
      for state in self.States.iter() {
        if self.FSM[state.index].len()!=0 {printmlstate(state,&mut self.Gmr,&self.combing);}}
     }

  }//selml

  pub fn to_statemachine(self) -> Statemachine // transfer before writeparser
  {
     Statemachine {
       Gmr: self.Gmr,
       States: Vec::new(),
       Statelookup:HashMap::new(),
       FSM: self.FSM,
       lalr: false,
       Open:Vec::new(),
       sr_conflicts:HashMap::new(),
     }
  }//to_statemachine

}//impl MLStatemachine

//whenever a state's item set has been expanded, we need to put it back
//on the Open list - LR or LALR - to call closure and makegotos again.
// As soon as step "extension" is applied, adding new NT, new rules and
// new item to a state, we should recompute the state's closure  -- bad idea,
// Better to completely compute the conflicts set and deprecate set first before
// calling it a state!   but later it could change.

// try-add returns option<not-clearly resolved conflict>
pub  fn tryadd_action(FSM: &mut Vec<HashMap<usize,Stateaction>>, Gmr:&mut Grammar, newaction:Stateaction, si:usize, la:usize, known_conflicts:&mut HashMap<(bool,usize,usize),(bool,usize)>, mut printout:bool, failed:bool) -> (bool,Option<(bool,usize,usize)>)
  {  
     let mut answer = None;
     let currentaction = FSM[si].get(&la);
     let mut changefsm = true; // add or keep current
     match (currentaction, &newaction) {
       (None,_) => {},  // most likely: just add
       (Some(Reduce(rsi)), Shift(_)) => {
         if /*true ||*/ Gmr.tracelev>5 {
           println!("Shift-Reduce Conflict between rule {} and lookahead {} in state {}",rsi,Gmr.symref(la),si);
         }
         let (clr,res) =mlsr_resolve(Gmr,rsi,la,si,known_conflicts,printout||failed);
         if res {changefsm = false; }  //***
         if !clr {
            answer = Some((false,*rsi,la));
            //if !failed {changefsm=true;}  // insert shift to detect conflict
         }
       },
       (Some(Reduce(cri)),Reduce(nri)) if cri==nri => { changefsm=false; },
       (Some(Reduce(cri)),Reduce(nri)) if cri!=nri => { // RR conflict
         let winner = if (cri<nri) {cri} else {nri};
         if (printout || failed) && !known_conflicts.contains_key(&(true,*cri,*nri))   {
           Gmr.logprint(&format!("Reduce-Reduce conflict between rules {} and {} resolved by default to {} ",cri,nri,winner));
           printrulela(*cri,Gmr,la);
           printrulela(*nri,Gmr,la);
         }
         if winner==cri {changefsm=false;} //***
         known_conflicts.insert((true,*cri,*nri),(false,*winner));
         known_conflicts.insert((true,*nri,*cri),(false,*winner));
         // false because rr conflicts can never be clearly resolved
         answer = Some((true,*cri,*nri)); //true means rr instead of sr
       },
       (Some(Accept),_) => { changefsm = false; },
       (Some(Shift(_)), Reduce(rsi)) => {
         if Gmr.tracelev>5 {
           println!("Shift-Reduce Conflict between rule {} and lookahead {} in state {}",rsi,Gmr.symref(la),si);
         }
         // look in state to see which item caused the conflict...
         let (clr,res) = mlsr_resolve(Gmr,rsi,la,si,known_conflicts,printout||failed);
         if !res {changefsm = false; }  //*****
         if !clr {
           answer = Some((false,*rsi,la));
           //if !failed {changefsm=false; }
         }
       },
       _ => {}, // default add newstate
     }// match currentaction
     if changefsm /* && (!answer.is_some() || failed)*/ { FSM[si].insert(la,newaction); }
     // if conflict is unresolved, do not overwrite it so that the conflict
     // can be detected again.
     if failed {answer = None; }
     (changefsm,answer)
  }//tryadd_action

  // reslove shift-reduce conflict, returns true if reduce, but defaults
  // to false (shift) so parsing will always continue and terminate.
  // returns (clear,reduce/shift)
fn mlsr_resolve(Gmr:&mut Grammar, ri:&usize, la:usize, si:usize,known_conflicts:&mut HashMap<(bool,usize,usize),(bool,usize)>,printout:bool) -> (bool,bool)
  {
     let mut isknown = true;
     match known_conflicts.get(&(false,*ri,la)) { //false means sr, not rr
       Some((true,r)) => {return (true,*r!=0);}, //true means clearly resolved
       None => { isknown=false; },
       _ => {},
     }//match
     
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
     else if (prec_level(lapred).abs()>prec_level(rulepred).abs() && rulepred!=0) {/*default  println!("HERE! {}",prec_level(rulepred)); */ }
     else if (prec_level(lapred).abs()<prec_level(rulepred).abs() ) {
       resolution = true;
//println!("HERE2! {}, {}, {}, {}",prec_level(lapred),&Gmr.Rules[*ri].lhs.sym,Gmr.Rules[*ri].rhs.len(),&Gmr.Rules[*ri].rhs[0].sym);       
       if lapred==0 {
          clearly_resolved = false;
          if printout && !isknown {
            Gmr.logprint(&format!("Shift-Reduce conflict between lookahead {} and rule {} in state {} non clearly resolved, defaulting to Reduce because the rule has positive precedence.",&Gmr.Symbols[la].sym,ri,si));
            printrulela(*ri,Gmr,la);
          }
       }
     } // reduce
     else {
       clearly_resolved=false;
       // report unclear case
       if printout && !isknown {
         Gmr.logprint(&format!("Shift-Reduce conflict between lookahead {} and rule {} in state {} not clearly resolved, defaulting to Shift",&Gmr.Symbols[la].sym,ri,si));
         printrulela(*ri,Gmr,la);
       }
     }
     known_conflicts.insert((false,*ri,la),(clearly_resolved,if resolution {1} else {0}));
     (clearly_resolved,resolution)
  }//mlsr_resolve
// no printing until end




//////////////////////////////////////
//////////////////////////////////////
// implemented marked delaying transformations.
impl Grammar
{
  // version that does not change existing rule
  pub fn delay_extend(&mut self,ri:usize,dbegin:usize,dend:usize,combing:&mut COMBINGTYPE,/*comb:Vec<usize>,*/ rulehash:&mut HashMap<Vec<usize>,usize>) -> usize
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
/*
       let combget = combing.rget(&comb);
       if let Some(cnti) = combget {
          newnt = self.Symbols[*cnti].clone();
       }
       else
*/
       if let Some(nti) = self.Symhash.get(&newntname) {
          newnt = self.Symbols[*nti].clone();
//println!("REUSING BY-NAME NT {}",&newnt.sym);       //checked         
       } else { // really new

         let mut nttype = String::from("(");
         for i in dbegin .. dend {
           let rsymi = self.Rules[ri].rhs[i].index;
           nttype.push_str(&format!("{},",&self.Symbols[rsymi].rusttype));
         }
         nttype.push(')');
         if !self.enumhash.contains_key(&nttype) {
           self.enumhash.insert(nttype.clone(),ntcx); ntcx+=1;
         }
         newnt.rusttype = nttype;
         newnt.index = self.Symbols.len();
         self.Symbols.push(newnt.clone());
         self.Symhash.insert(newntname.clone(),self.Symbols.len()-1);
         // set First, which is no longer called.
         let mut ntfirsts = self.Firstseq(&self.Rules[ri].rhs[dbegin..dend],self.Symbols.len());
         ntfirsts.remove(&self.Symbols.len());
         self.First.insert(newnt.index,ntfirsts);

         // register new combed NT with combing map
         //let defvec = vec![self.Rules[ri].rhs[dbegin].index];
         //let mut oldvec = combing.get(&self.Rules[ri].rhs[dbegin].index).unwrap_or(&defvec).clone();
         let mut oldvec = uncombing(&self.Rules[ri].rhs[dbegin].index,combing);
         // this assumes that dend = debegin+2
         let mut extension = uncombing(&self.Rules[ri].rhs[dend-1].index,combing);
         oldvec.append(&mut extension);
         //oldvec.push(self.Rules[ri].rhs[dend-1].index);
         combing.insert(newnt.index,oldvec);

         let NTrules:Vec<_> = self.Rulesfor.get(&NT1.index).unwrap().iter().collect();
         let mut rset = HashSet::new(); // rules set for newnt (delayed nt)
         
         for ntri in NTrules {
           // create new rule: A-> a to [Ab] -> ab
           let mut newrule = Grule::from_lhs(&newnt);
           newrule.rhs = self.Rules[*ntri].rhs.clone();
           for d in &delta { newrule.rhs.push(d.clone()); } //extend
           let rhashv = hashrule(&newrule);
           if let Some(rri) = rulehash.get(&rhashv) {  // rule exists
             continue;
           }

           //////// set semantic action for new rule.
           // need to call/refer action for original rule for NT1
           // need to form a tuple.
           // internal variable symbol.
           let newvar = format!("_delvar_{}_{}_",&newnt.index,dbegin);
// check for return at end of last action.


//           let mut actionri = format!(" let {} = {{ {}; ",&newvar,self.Rules[*ntri].action); // retrieves value from original action.
           // change this to a _rrsemaction_ri call on the first args
           let mut aargs=String::from("parser");
           for k in 0..self.Rules[*ntri].rhs.len() { //original len
             let (ltype,label)=decode_label(&self.Rules[*ntri].rhs[k].label,k);
             aargs.push(',');  aargs.push_str(&label);
           }
           let mut actionri= format!(" let {} = _rrsemaction_{}_({}); ",&newvar,ntri,aargs);

           // need to assign values to new items added to delta
           // they will be popped off of the stack by parser_writer as
           // item2, item1 item0...  because parser writer will write an action
           // for the extended rule. [Mc] --> abc

           let newrulenum = self.Rules.len(); //will be index of new rule
           let mut dtuple = format!("({},",&newvar);
           let mut labi = self.Rules[*ntri].rhs.len(); // original rule rhs len
           for sym in &delta {
             let (_,mut slabel) = decode_label(&sym.label,labi);
             if slabel.starts_with("_item") {
               slabel = format!("_item_del{}_{}_{}_",&labi,newrulenum,ntri);
               newrule.rhs[labi].label = slabel.clone();
             }
             dtuple.push_str(&format!("{},",slabel));
             labi+=1;
           }
           actionri.push_str(&format!("{}) }}",&dtuple));  //rparen added here.
           newrule.action = actionri;
           rulehash.insert(rhashv,self.Rules.len());
           self.Rules.push(newrule);
           
//if LTRACE {print!("Added rule "); let pitem = LRitem{ri:self.Rules.len()-1,pi:0,la:self.eoftermi};  printitem2(&pitem,self,combing);}

           rset.insert(self.Rules.len()-1);
         }// for each rule for this NT1 to be delayed, add suffix
         self.Rulesfor.insert(newnt.index,rset);
       } // newnt is actually a new symbol, else it and its rules exist

       ////// do not change original rule, form a new rule.
       ////// HOW TO AVOID DUPLICATE RULES?
       let mut newrulei = Grule::from_lhs(&self.Rules[ri].lhs); //copy
       let mut newrhs = Vec::with_capacity(self.Rules[ri].rhs.len()-1);
       if dbegin>0 {
         for i in 0..dbegin {
           let mut rhsi = self.Rules[ri].rhs[i].clone();
           //if rhsi.label.len()<1 {rhsi.label=format!("_ditem{}_",i/*,self.Rules.len()*/);} //give special labels
           newrhs.push(rhsi);
         }
       }
       let mut clonenewnt = newnt.clone();
       // this is a new grammar sym, can give it any label
       let ntlabel = format!("_delayitem{}_{}_{}",dbegin,ri,ntcx); ntcx+=1;
       clonenewnt.label = ntlabel.clone();
       newrhs.push(clonenewnt); // newnt added to rule!
       for i in dend .. self.Rules[ri].rhs.len() {
         let mut rhsi = self.Rules[ri].rhs[i].clone();
           //if rhsi.label.len()<1 {rhsi.label=format!("_ditem{}_",i/*,self.Rules.len()*/);} //give special labels         
         newrhs.push(rhsi);
       }
       newrulei.rhs = newrhs; // change rhs of rule
       let hashr = hashrule(&newrulei);
       if let Some(rnti) = rulehash.get(&hashr) {
         self.ntcxmax = ntcx;
         return *rnti;
       }

       /////// change semantic action of original rule.
       let mut newaction = String::from(" ");
       let newri = self.Rules.len(); // index of new rule (extended)
       // break up tuple, for arguments to original action
       let mut aargs = String::from("parser");
       for i in 0..dbegin {
          let (_,labeli) = decode_label(&self.Rules[ri].rhs[i].label,i);
          aargs.push(',');  aargs.push_str(&labeli);       
       }
       for i in dbegin..dend {
//          let symi = &self.Rules[ri].rhs[i]; // original rule
//          let (ltype,labeli) = decode_label(&symi.label,i);
//           let defaultlab = format!("_item{}_",i);
//          let labeli = if symi.label.len()>0 {checkboxlabel(&symi.label)}
//            else {&defaultlab};

//          if ltype==2 {newaction.push_str(&format!("let ref mut {} = {}.{}; ",&labeli,&ntlabel,i-dbegin));}
//          else {newaction.push_str(&format!("let {} = {}.{}; ",&labeli,&ntlabel,i-dbegin));}
//          aargs.push(',');  aargs.push_str(&labeli);
            aargs.push_str(&format!(",{}.{}",&ntlabel,i-dbegin));
       }// break up tuple
       // add rest of original arguments
       for i in dend .. self.Rules[ri].rhs.len() {
          let (_,labeli) = decode_label(&self.Rules[ri].rhs[i].label,i-(dend-dbegin-1));
          aargs.push(',');  aargs.push_str(&labeli);
       }
/*       
       // anything to do with the other values?  they have labels, but indexes
       // may be off - but original actions will refer to them as-is.
       // original action will assume labels are _item{}_, unless
       // change them.  replace strings
       let mut originalact = self.Rules[ri].action.clone();  //HACK!
       for i in dend..self.Rules[ri].rhs.len() {
         originalact=originalact.replace(&format!("_item{}_",i),&format!("_rrtempitem{}_",i));
       }
       for i in dend..self.Rules[ri].rhs.len() {
         originalact=originalact.replace(&format!("_rrtempitem{}_",i),&format!("_item{}_",i-1));
       }
       newaction.push_str(&originalact);
*/
       newaction.push_str(&format!("_rrsemaction_{}_({}) }}",ri,aargs));
       newrulei.action = newaction;

////////////////////*************
/////////////////   so startstate needs to be reclosed!
       // special case: newrule becomes startrule if startrule changed.
       if newrulei.lhs.index == self.startnti {
         self.Rules[self.startrulei] = newrulei;
         return self.startrulei;
       } else {      //register new rule
         self.Rulesfor.get_mut(&newrulei.lhs.index).unwrap().insert(self.Rules.len());
         self.Rules.push(newrulei);
         rulehash.insert(hashr,self.Rules.len()-1);
         self.ntcxmax = ntcx;

         if let Some(cutpi) = self.sdcuts.get(&ri) {
           if *cutpi>0 {self.sdcuts.insert(self.Rules.len()-1, cutpi-1);}
         }
         
         return self.Rules.len()-1;
       }// new rule added (not start rule, which is replaced).
  }// delay_extend



////////////////// don't touch - in use!
  // this must be called before start symbol, eof and startrule added to grammar!
  // THIS FUNCTION IS ONLY CALLED STATICALLY ON A GRAMMAR
  pub fn delay_transform(&mut self)
  {
    let mut ntcx = self.ntcxmax+1;
    for (ri, delaymarks) in self.delaymarkers.iter() {
//println!("rule {} has delaymarkers at {:?}",ri,delaymarks);    
     for (dbegin,dend) in delaymarks.iter() {
       // check if first symbol at marker is a nonterminal
       let NT1 = &self.Rules[*ri].rhs[*dbegin];
       if NT1.terminal {
         let msg = format!("WARNING: STARTING DELAY MARKER AT POSITION {} MUST PRECEED NONTERMINAL SYMBOL, PRODUCTION {} IN GRAMMAR.  MARKERS IGNORED\n",dbegin,ri);
         if self.tracelev>0 {
            eprint!("{}",&msg);
            printrule(&self.Rules[*ri],*ri);            
         }
         else {
           self.genlog.push_str(&msg);
           self.genlog.push_str(&printruleb(&self.Rules[*ri],*ri));
         }
         continue;
       }// NT1 is non-terminal
       // construct suffix delta to be added to each rule
       let mut delta = Vec::new();
//println!("!!!!dbegin:{}, dend:{}, ri:{}",dbegin,dend,ri);
//printrule(&self.Rules[*ri],*ri);
       for i in dbegin+1..*dend {
         if i<self.Rules[*ri].rhs.len() {
           delta.push(self.Rules[*ri].rhs[i].clone());
         }
       }
       // construct new nonterminal name ([Mdelta])
       let mut newntname = format!("NEWSDNT_{}",&NT1.sym);
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
              if i < self.Rules[*ri].rhs.len() {
                let rsymi = self.Rules[*ri].rhs[i].index;
                nttype.push_str(&format!("{},",&self.Symbols[rsymi].rusttype));
              }
            }//for
            nttype.push(')');
            self.enumhash.insert(nttype.clone(),ntcx); ntcx+=1;
            newnt.rusttype = nttype;
        }// form type of newnt
//println!("newnttype for {} is {}",&newnt.sym, &newnt.rusttype);         
         newnt.index = self.Symbols.len();
         self.Symbols.push(newnt.clone());
         self.Symhash.insert(newntname.clone(),self.Symbols.len()-1);
         // First set computed after delay_transform called.
/*  NO COMBING AVAILABLE
         // register with combing:
         let defvec = vec![self.Rules[ri].rhs[dbegin].index];
         let mut oldvec = combing.get(&self.Rules[ri].rhs[dbegin].index).unwrap_or(&defvec).clone();
         oldvec.push(self.Rules[ri].rhs[dend-1].index);
         combing.insert(newnt.index,oldvec);
*/
         // change rules by adding suffix
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

           if self.tracelev>2 {
             print!("!COMBINED DELAY RULE: ");
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
        if i < self.Rules[*ri].rhs.len() {
          let defaultlab = format!("_item{}_",i);
          let symi = &self.Rules[*ri].rhs[i]; // original rule
          let labeli = if symi.label.len()>0 {checkboxlabel(&symi.label)}
            else {&defaultlab};
          newaction.push_str(&format!("let mut {} = {}.{}; ",labeli,&ntlabel,i-dbegin));
          //labi+=1;
        }
       }// break up tuple
       // anything to do with the other values?  they have labels, but indexes
       // may be off - but original actions will refer to them as-is.
       let mut originalact = self.Rules[*ri].action.clone();  //HACK!
       for i in *dend..self.Rules[*ri].rhs.len() {
         originalact=originalact.replace(&format!("_item{}_",i),&format!("_rrtempitem{}_",i));
       }
       for i in *dend..self.Rules[*ri].rhs.len() {
         originalact=originalact.replace(&format!("_rrtempitem{}_",i),&format!("_item{}_",i-1));
       }
       newaction.push_str(&originalact);
       self.Rules[*ri].rhs = newrhs; // change rhs of rule
       self.Rules[*ri].action = newaction;
       
       if self.tracelev>2 {
         print!("TRANSFORMED RULE FOR DELAY: ");
         printrule(&self.Rules[*ri],*ri);
       }
       
     } // for each pair of delay marks assume dend>dbegin+1
    }//for each rule
    self.ntcxmax = ntcx;
  }// delay_transform
} // transformation


/*
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
*/
/*
fn printitem(item:&LRitem, Gmr:&Grammar)
{
   let lhs = &Gmr.Rules[item.ri].lhs;
   print!("({}) {} --> ",item.ri,&lhs.sym);
   for i in 0..Gmr.Rules[item.ri].rhs.len() {
      if i==item.pi {print!(" . ");}
      print!("{} ",&Gmr.Rules[item.ri].rhs[i].sym);
   }
   if item.pi==Gmr.Rules[item.ri].rhs.len() {print!(" . ");}
   println!(" LA: {}",&Gmr.Symbols[item.la].sym);
}//printitem
*/

fn printitem2(item:&LRitem, Gmr:&mut Grammar,combing:&COMBINGTYPE)
{
   let lhs = &Gmr.Rules[item.ri].lhs;
      let mut psym = Gmr.Rules[item.ri].lhs.sym.clone();
      if psym.starts_with("NEWDELAYNT") {
        //let defaultcomb = vec![lhs.index];
        //let comb = combing.get(&Gmr.Rules[item.ri].lhs.index).unwrap_or(&defaultcomb);
        let comb = uncombing(&Gmr.Rules[item.ri].lhs.index,combing);
        psym = String::from("[[");
        for c in &comb {
          psym.push_str(&Gmr.Symbols[*c].sym); psym.push(' ');
        }
        psym.push_str("]]");
      }   
   Gmr.logprint0(&format!("({}) {} --> ",item.ri,&psym));
   for i in 0..Gmr.Rules[item.ri].rhs.len() {
      if i==item.pi {Gmr.logprint0(" . ");}
      psym = Gmr.Rules[item.ri].rhs[i].sym.clone();
      if psym.starts_with("NEWDELAYNT") {
        //let defaultcomb = vec![Gmr.Rules[item.ri].rhs[i].index];
        //let comb = combing.get(&Gmr.Rules[item.ri].rhs[i].index).unwrap_or(&defaultcomb);
        let comb = uncombing(&Gmr.Rules[item.ri].rhs[i].index,combing);
        psym = String::new();
        if comb.len()>1 {psym.push_str("[[");}
        for c in &comb {
          psym.push_str(&Gmr.Symbols[*c].sym); psym.push(' ');
        }
        if comb.len()>1 {psym.push_str("]]");}
      }
      Gmr.logprint0(&format!("{} ",&psym));
   }
   if item.pi==Gmr.Rules[item.ri].rhs.len() {Gmr.logprint0(" . ");}
   if item.pi<=Gmr.Rules[item.ri].rhs.len() 
     {Gmr.logprint0(&format!(" LA: {}",&Gmr.Symbols[item.la].sym));}
   Gmr.logprint("");
}//printitem : to avoid printing the dot, give large pi value

fn printrule2(ri:usize, Gmr:&mut Grammar,combing:&COMBINGTYPE) {
   printitem2(&LRitem{ri:ri,pi:usize::MAX,la:0},Gmr,combing);
}


// independent function for tracing
pub fn printmlstate(state:&MLState,Gmr:&mut Grammar,combing:&COMBINGTYPE) 
{
  println!("-----------\nState {}:",state.index);
  for item@LRitem{ri,pi,la} in state.items.iter() {
//  for item@LRitem{ri,pi,la} in state.lrkernel.iter() {
     if state.deprecated.contains(item) {print!("DEPRECATED: ");}
     printitem2(item,Gmr,combing);  
  }//for item
}//printlalrstate

//// compute hash value for a rule
fn hashrule(rule:&Grule) -> Vec<usize>
{
   let mut h = vec![rule.lhs.index];
   for r in &rule.rhs {
     h.push(r.index);
   }
   h
}//hashrule



//// nested combings have to be unraveled (RECURSIVE) -should be ok
//// only call for printing:
fn uncombing(nti:&usize, /*Gmr:&Grammar,*/ combing:&COMBINGTYPE) -> Vec<usize>
{
  let defaultcomb = vec![*nti];
  let combget = combing.get(nti);
  if let None = combget {return defaultcomb;}
  let combref = combget.unwrap();
  let mut comb = Vec::new();
  for refi in combref
    {
      let mut combi = uncombing(refi,/*Gmr,*/combing);
      comb.append(&mut combi);
    }//inner while
  comb
}//uncombing


// recursively calculate true length of combing
fn comblength(nti:&usize, /*Gmr:&Grammar,*/ combing:&COMBINGTYPE) -> usize
{
  let combget = combing.get(nti);
  if let None = combget {return 1;}
  let combref = combget.unwrap();
  let mut comb = 0;
  for refi in combref
  {  comb += comblength(refi,/*Gmr,*/combing); }
  comb
}//uncombing


//true if failed:
fn checkfailure(item:&LRitem,Gmr:&Grammar,combing:&COMBINGTYPE,maxk:usize) -> bool 
{
   if item.pi!=0 {return false;}
   let nti = Gmr.Rules[item.ri].lhs.index;
   comblength(&nti,combing)>maxk
}

fn reportfailure(rilhs:&usize, Gmr:&mut Grammar,combing:&COMBINGTYPE)
{
   let comb = uncombing(rilhs,combing);               
   Gmr.logprint0("FAILURE. MAXIMUM COMBING IN CONFLICT:\n  [[");
   for x in &comb {
      Gmr.logprint0(&format!("{} ",&Gmr.Symbols[*x].sym));
   }
   Gmr.logprint("]]");
}//reportfailure


  // look for all lookaheads of LRitems having ri,pi
  fn collect_la(items:&ITEMSETTYPE,ri:usize, pi:usize) -> HashSet<usize>
  {
    let mut ax = HashSet::new();
    for item in items.iter() {
       if item.ri==ri && item.pi==pi {ax.insert(item.la);}
    }
    ax
  }

// hashsets impls Eq already.
