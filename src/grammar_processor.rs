//! Grammar processing module.  The exported elements of this module are
//! only intended for re-implementing rustlr within rustlr.

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
#![allow(unused_doc_comments)]
#![allow(unused_imports)]
//use std::fmt::Display;
//use std::default::Default;
use std::collections::{HashMap,HashSet,BTreeSet};
use std::cell::{RefCell,Ref,RefMut};
use std::hash::{Hash,Hasher};
use std::io::{self,Read,Write,BufReader,BufRead};
use std::fs::File;
use std::io::prelude::*;

pub const DEFAULTPRECEDENCE:i32 = 0;   // 20
pub const TRACE:usize = 0;

#[derive(Clone)]
pub struct Gsym // struct for a grammar symbol
{
  pub sym : String,
  pub rusttype : String, // used to derive private enum
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
      rusttype : String::new(),
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
   println!("{{ {}, precedence {}",rule.action.trim(),rule.precedence);  // {{ is \{
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
  pub Resynch : HashSet<String>, // resynchronization terminal symbols, ordered
  pub Errsym : String,        // error recovery terminal symbol
  pub Lexnames : Vec<(String,String)>, // print names of grammar symbols
  pub Extras : String,        // indicated by {% .. %}, mostly  use ...
  pub sametype: bool,  // determine if absyntype is only valuetype
  pub lifetime: String,
  pub tracelev:usize,
  pub Lexvals: Vec<(String,String,String)>,  //"int" -> ("Num(n)","Val(n)")
  pub Haslexval : HashSet<String>,
  pub Lexextras: Vec<String>,
  pub genlex: bool,
}

impl Default for Grammar {
  fn default() -> Self { Grammar::new() }
}

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
//       Recover : HashSet::new(),
       Resynch : HashSet::new(),
       Errsym : String::new(),
       Lexnames : Vec::new(),
       Extras: String::new(),
       sametype:true,
       lifetime:String::new(), // empty means inferred
       tracelev:1,
       Lexvals:Vec::new(),
       Haslexval:HashSet::new(),
       Lexextras:Vec::new(),
       genlex: false,
     }
  }//new grammar

  pub fn getsym(&self,s:&str) -> Option<&Gsym>
  {
     match self.Symhash.get(s) {
       Some(symi) => Some(&self.Symbols[*symi]),
       _ => None,
     }//match
  }

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

  fn using_generic(&self) -> bool
  { self.Absyntype=="GenAbsyn" || self.Absyntype=="ABox"  }

////// meta (grammar) parser
  pub fn parse_grammar(&mut self, filename:&str)
  {
     let mut reader =  match File::open(filename) {
       Ok(f) => { Some(BufReader::new(f)) },
       _ => { eprintln!("cannot open file, reading from stdin..."); None},
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
       }
       else if linelen>1 && &line[0..1]!="#" {
         let toksplit = line.split_whitespace();
         let stokens:Vec<&str> = toksplit.collect();
         if stokens.len()<1 {continue;}
         match stokens[0] {
         /*  deprecated by !
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
         */            
            "!" => {
               let pbi = line.find('!').unwrap();
               self.Extras.push_str(&line[pbi+1..]);
               self.Extras.push_str("\n");                             
            },
            "grammarname" => {
               self.name = String::from(stokens[1]);
            },
            "EOF" => {atEOF=true},
            ("terminal" | "terminals") if stage==0 => {
               for i in 1..stokens.len() {
	          let newterm = Gsym::new(stokens[i],true);
                  self.Symhash.insert(stokens[i].to_owned(),self.Symbols.len());
                  self.Symbols.push(newterm);
                  //self.Symbols.insert(stokens[i].to_owned(),newterm);
		  //if self.tracelev>3 {println!("terminal {}",stokens[i]);}
               }
            }, //terminals
	    "typedterminal" if stage==0 => {
	       let mut newterm = Gsym::new(stokens[1],true);
               let mut tokentype = String::new();
               for i in 2..stokens.len() {
                  tokentype.push_str(&stokens[i][..]);
                  tokentype.push(' ');
               }
               newterm.settype(tokentype.trim());
               self.Symhash.insert(stokens[1].to_owned(),self.Symbols.len());
               self.Symbols.push(newterm);           
	    }, //typed terminals
	    "nonterminal" if stage==0 => {   // with type
	       let mut newterm = Gsym::new(stokens[1],false);
               let mut tokentype = String::new();
               for i in 2..stokens.len() {
                  tokentype.push_str(&stokens[i][..]);
                  tokentype.push(' ');
               }
//	       if stokens.len()>2 && stokens[2]!="mut" {
//	         let rtype = format!("PE_Variant_{}({})",self.Symbols.len(),tokentype.trim());
//		 newterm.settype(&rtype);
               newterm.settype(tokentype.trim());
//               }
//               else {newterm.settype(&self.Absyntype);}
               self.Symhash.insert(stokens[1].to_owned(),self.Symbols.len());
               self.Symbols.push(newterm);
               self.Rulesfor.insert(stokens[1].to_owned(),HashSet::new());
	    }, //nonterminals
            "nonterminals" if stage==0 => {
               for i in 1..stokens.len() {
	          let newterm = Gsym::new(stokens[i],false);
                  self.Symhash.insert(stokens[i].to_owned(),self.Symbols.len());
                  self.Symbols.push(newterm);
                  self.Rulesfor.insert(stokens[i].to_owned(),HashSet::new());
		  //if TRACE>2 {println!("nonterminal {}",stokens[i]);}
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
            "errsym" | "errorsymbol" => {
               if stage>1 {
                 panic!("!!! Error recover symbol must be declared before production rules, line {}",linenum);
               }
               if stage==0 {stage=1;}
               if !self.terminal(stokens[1]) {
                 panic!("!!!Error recover symbol {} is not a terminal, line {} ",stokens[1],linenum);
               }
               self.Errsym = stokens[1].to_owned();
            },
            /*
            "recover"  => {
               if stage==0 {stage=1;}
               for i in 1..stokens.len()
               {
                  if !self.nonterminal(stokens[i]) {
                     panic!("!!!Error recovery symbol {} is not a declared non-terminal, line {}",stokens[i],linenum);
                  }
                  self.Recover.insert(stokens[i].to_owned());
               } // for each subsequent token
            },
            */
            "resynch" | "resync"  => {
               if stage==0 {stage=1;}
               for i in 1..stokens.len()
               {
                  if !self.terminal(stokens[i]) {
                     panic!("!!!Error recovery re-synchronization symbol {} is not a declared terminal, line {}",stokens[i],linenum);
                  }
                  self.Resynch.insert(stokens[i].trim().to_owned());
               } // for each subsequent token
            },
            "lifetime" if stokens.len()==2 && stokens[1].len()>0 => {
               
               self.lifetime = if &stokens[1][0..1]=="'" && stokens[1].len()>1 
                 {String::from(stokens[1])} else {format!("'{}",stokens[1])};
            },
            "absyntype" | "valuetype" if stage==0 => {
               let pos = line.find(stokens[0]).unwrap() + stokens[0].len();
               self.Absyntype = String::from(line[pos..].trim());
	       if TRACE>2 {println!("abstract syntax type is {}",&self.Absyntype);}
            },
            "externtype" | "externaltype" if stage==0 => {
               let pos = line.find(stokens[0]).unwrap() + stokens[0].len();
               self.Externtype = String::from(line[pos..].trim());            
	       if TRACE>2 {println!("external structure type is {}",&self.Externtype);}
            },            
	    "left" | "right" if stage<2 => {
               if stage==0 {stage=1;}
               if stokens.len()<3 {continue;}
	       let mut preclevel:i32 = DEFAULTPRECEDENCE;
	       if let Ok(n)=stokens[2].parse::<i32>() {preclevel = n;}
               else {panic!("did not read precedence level on line {}",linenum);}
	       if stokens[0]=="right" && preclevel>0 {preclevel = -1 * preclevel;}
               if let Some(index) = self.Symhash.get(stokens[1]) {
                 if preclevel.abs()<=DEFAULTPRECEDENCE {
                   println!("WARNING: precedence of {} is non-positive",stokens[1]);
                 }
                 self.Symbols[*index].precedence = preclevel;
               }
	    }, // precedence and associativity
	    "lexname"  => {
               if stokens.len()<3 {continue;}  // "||" -> "OROR"
               self.Lexnames.push((stokens[2].to_string(),stokens[1].to_string()));
	       self.Haslexval.insert(stokens[1].to_string());
	       self.genlex = true;
            },
	    "lexvalue" => {
	       if stokens.len()<4 {continue;}  // "int" -> ("Num(n)","Val(n)")
	       let mut valform = String::new();
	       for i in 3 .. stokens.len()
	       {
	         valform.push_str(stokens[i]);
		 if (i<stokens.len()-1) {valform.push(' ');}
	       }
	       self.Lexvals.push((stokens[1].to_string(),stokens[2].to_string(),valform));
	       // record that this terminal always carries a value
	       self.Haslexval.insert(stokens[1].to_string());
	       self.genlex = true;
	    },
	    "lexset" | "lexattribute" => {
	       let mut prop = String::new();
	       for i in 1 .. stokens.len()
	       {
	          prop.push_str(stokens[i]); prop.push(' ');
	       }
	       self.Lexextras.push(prop);
	       self.genlex = true;
	    },
//////////// case for grammar production:            
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
              let lhsym = &self.Symbols[*symindex]; //.clone();

              // split by | into separate rules

              let pos0 = line.find(stokens[1]).unwrap() + stokens[1].len();
              let mut linec = line[pos0..].to_string();
              let barsplit:Vec<_> = linec.split('|').collect();
              
              for rul in &barsplit 
              { //if rul.trim().len()>0 {  // must include empty productions!
              //println!("see rule seg ({})",rul);
              let bstokens:Vec<_> = rul.trim().split_whitespace().collect();
              let mut rhsyms:Vec<Gsym> = Vec::new();
              let mut semaction = "}";
	      let mut i:usize = 0;
              let mut maxprec:i32 = 0;
              let mut seenerrsym = false;
              while i<bstokens.len() {
	        let strtok = bstokens[i];
		i+=1;
                if strtok.len()>0 && &strtok[0..1]=="{" {
                   let position = rul.find('{').unwrap();
                   semaction = rul.split_at(position+1).1;
		   break;
                }
		let toks:Vec<&str> = strtok.split(':').collect();
//if TRACE>2&&toks.len()>1 {println!("see labeled token {}",strtok);}		
		match self.Symhash.get(toks[0]) {
		   None => {panic!("unrecognized grammar symbol {}, line {}",toks[0],linenum); },
		   Some(symi) => {
                     let sym = &self.Symbols[*symi];
                     if self.Errsym.len()>0 && &sym.sym == &self.Errsym {
                       if !seenerrsym { seenerrsym = true; }
                       else { panic!("Error symbol {} can only appear once in a production, line {}",&self.Errsym,linenum); }
                     }
                     if !sym.terminal && seenerrsym {
                       panic!("Only terminal symbols may follow the error recovery symbol {}, line {}",&self.Errsym, linenum);
                     }
		     let mut newsym = sym.clone();
		     
		     if toks.len()>1 && toks[1].trim().len()>0 { //label exists
		       let mut label = String::new();
		       
		       if let Some(atindex) = toks[1].find('@') { //if-let pattern
			 label.push_str(toks[1]);
			 while !label.ends_with('@') && i<bstokens.len()
			 { // i indexes all tokens split by whitespaces
			    label.push(' '); label.push_str(bstokens[i]); i+=1;
			 }
			 if !label.ends_with('@') { panic!("pattern labels must be closed with @, line {}",linenum);}			 
		       } // if-let pattern
/*		       
                       if &toks[1][0..1]=="'" { // if-let pattern
		         label.push_str(&toks[1][..]); // include leading ' in string
			 while !label.ends_with("'") && i<bstokens.len()
			 { // i indexes all tokens split by whitespaces
			    label.push(' '); label.push_str(bstokens[i]); i+=1;
			 }
			 if !label.ends_with("'") { panic!("pattern labels must be closed with  a ', line {}",linenum);}
		       }// ' pattern '
*/		       
                       else { label = toks[1].trim().to_string(); }
		       newsym.setlabel(label.trim_end_matches('@'));
		       //newsym.setlabel(label.trim_end_matches("'"));		       
		       //newsym.setlabel(toks[1].trim()); 
	             }//label exists
			
                     if maxprec.abs() < newsym.precedence.abs()  {
                        maxprec=newsym.precedence;
                     }
		     rhsyms.push(newsym);
                   }
                }//match
	      } // while there are tokens on rhs
	      // form rule
	      let rule = Grule {
	        lhs : lhsym.clone(),
		rhs : rhsyms,
		action: semaction.to_owned(),
		precedence : maxprec,
	      };
	      if self.tracelev>3 {printrule(&rule);}
	      self.Rules.push(rule);
              // Add rules to Rulesfor map
              if let None = self.Rulesfor.get(LHS) {
                 self.Rulesfor.insert(String::from(LHS),HashSet::new());
              }
              let rulesforset = self.Rulesfor.get_mut(LHS).unwrap();
              rulesforset.insert(self.Rules.len()-1);
            //} 
            } // for rul
            }, 
            _ => {panic!("error parsing grammar on line {}, grammar stage {}",linenum,stage);},  
         }//match first word
       }// not an empty or comment line
     } // while !atEOF
     if self.Symhash.contains_key("START") || self.Symhash.contains_key("EOF") || self.Symhash.contains_key("ANY_ERROR")
     {
        panic!("Error in grammar: START and EOF are reserved symbols");
     }
     // add start,eof and starting rule:
     let startnt = Gsym::new("START",false);
     let eofterm =  Gsym::new("EOF",true);
//     let anyerr = Gsym::new("ANY_ERROR",true);
     self.Symhash.insert(String::from("START"),self.Symbols.len());
     self.Symhash.insert(String::from("EOF"),self.Symbols.len()+1);
//   self.Symhash.insert(String::from("ANY_ERROR"),self.Symbols.len()+2);     
     self.Symbols.push(startnt.clone());
     self.Symbols.push(eofterm.clone());
//     self.Symbols.push(anyerr.clone());     
     let topgsym = &self.Symbols[*self.Symhash.get(&self.topsym).unwrap()];
     let startrule = Grule {  // START-->topsym EOF
        lhs:startnt,
        rhs:vec![topgsym.clone()], //,eofterm],  //eofterm is lookahead
        action: String::default(),
        precedence : 0,
     };
     self.Rules.push(startrule);  // last rule is start rule
     if self.tracelev>0 {println!("{} rules in grammar",self.Rules.len());}
     if self.Externtype.len()<1 {self.Externtype = self.Absyntype.clone();}
     // compute sametype value (default true)
     for ri in 0..self.Symbols.len()
     {
        if self.Symbols[ri].rusttype.len()<2 {
          self.Symbols[ri].settype(&self.Absyntype);
        }
        else if &self.Symbols[ri].rusttype!=&self.Absyntype {
          self.sametype = false;
        }
     }//compute sametype
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
  }//FirstSeqb


// procedure to generate lexical scanner from lexname, lexval and lexattribute
// declarations in the grammar file.  Added for Version 0.2.3.  This procedure
// is only used by other modules internally
pub fn genlexer(&self,fd:&mut File) -> Result<(),std::io::Error>
{
    ////// WRITE LEXER
      let ref absyn = self.Absyntype;
      let lifetime = if (self.lifetime.len()>0) {&self.lifetime} else {"'t"};
      write!(fd,"\n// Lexical Scanner using RawToken and StrTokenizer\n")?;
      let lexername = format!("{}lexer",&self.name);
      let mut keywords:Vec<&str> = Vec::new();
      let mut singles:Vec<char> = Vec::new();
      let mut doubles:Vec<&str> = Vec::new();
      // collect symbols from grammar
      for symbol in &self.Symbols
      {
        if !symbol.terminal {continue;}
        if is_alphanum(&symbol.sym) && &symbol.sym!="EOF" && &symbol.sym!="ANY_ERROR" && !self.Haslexval.contains(&symbol.sym) {
	   keywords.push(&symbol.sym);
	}
	else if symbol.sym.len()==1 && !is_alphanum(&symbol.sym) {
	   singles.push(symbol.sym.chars().next().unwrap());
	}
	else if symbol.sym.len()==2 && !is_alphanum(&symbol.sym) {
	   doubles.push(&symbol.sym);
	}
      }//for each symbol
      for (sym,_) in &self.Lexnames
      {
        if is_alphanum(sym) {continue;}
	if sym.len()==1 {
	   singles.push(sym.chars().next().unwrap());
	}
	else if sym.len()==2 {
	   doubles.push(&sym);
	}      
      }// for symbols in lexnames such as "||" --> OROR
      write!(fd,"pub struct {0}<'t> {{
   stk: StrTokenizer<'t>,
   keywords: HashSet<&'static str>,
}}
impl<'t> {0}<'t> 
{{
  pub fn from_str(s:&'t str) -> {0}<'t>  {{
    Self::new(StrTokenizer::from_str(s))
  }}
  pub fn from_source(s:&'t LexSource<'t>) -> {0}<'t>  {{
    Self::new(StrTokenizer::from_source(s))
  }}
  pub fn new(mut stk:StrTokenizer<'t>) -> {}<'t> {{
    let mut keywords = HashSet::with_capacity(16);
    for kw in [",&lexername)?; // end of write

      for kw in &keywords {write!(fd,"\"{}\",",kw)?;}
      write!(fd,"] {{keywords.insert(kw);}}
    for c in [")?;
      for c in singles {write!(fd,"'{}',",c)?;}
      write!(fd,"] {{stk.add_single(c);}}
    for d in [")?;
      for d in doubles {write!(fd,"\"{}\",",d)?;}
      write!(fd,"] {{stk.add_double(d);}}\n")?;
    for attr in &self.Lexextras {write!(fd,"    stk.{};\n",attr.trim())?;}
      write!(fd,"    {} {{stk,keywords}}\n  }}\n}}\n",&lexername)?;
      // end of impl lexername
      write!(fd,"impl<{0}> Tokenizer<{0},{1}> for {2}<{0}>
{{
   fn nextsym(&mut self) -> Option<TerminalToken<{0},{1}>> {{
",lifetime,absyn,&lexername)?;
      write!(fd,"    let tokopt = self.stk.next_token();
    if let None = tokopt {{return None;}}
    let token = tokopt.unwrap();
    match token.0 {{
")?;
    if keywords.len()>0 {
      write!(fd,"      RawToken::Alphanum(sym) if self.keywords.contains(sym) => Some(TerminalToken::from_raw(token,sym,<{}>::default())),\n",absyn)?;
    }
      // write special alphanums first - others might be "var" form
      // next - write the Lexvals hexmap int -> (Num(n),Val(n))
      for (tname,raw,val) in &self.Lexvals
      {  
        write!(fd,"      RawToken::{} => Some(TerminalToken::from_raw(token,\"{}\",{})),\n",raw,tname,val)?;
      }
      for (lform,tname) in &self.Lexnames
      {
        write!(fd,"      RawToken::Symbol(r\"{}\") => Some(TerminalToken::from_raw(token,\"{}\",<{}>::default())),\n",lform,tname,absyn)?;
      }
      write!(fd,"      RawToken::Symbol(s) => Some(TerminalToken::from_raw(token,s,<{}>::default())),\n",absyn)?;
      write!(fd,"      RawToken::Alphanum(s) => Some(TerminalToken::from_raw(token,s,<{}>::default())),\n",absyn)?;      
      write!(fd,"      _ => Some(TerminalToken::from_raw(token,\"<LexicalError>\",<{}>::default())),\n    }}\n  }}",absyn)?;
      write!(fd,"
   fn linenum(&self) -> usize {{self.stk.line()}}
   fn column(&self) -> usize {{self.stk.column()}}
   fn position(&self) -> usize {{self.stk.current_position()}}
}}//impl Tokenizer
\n")?;
      Ok(())
}//genlexer


}//impl Grammar continued


// used by genlexer routines
fn is_alphanum(x:&str) -> bool
{
/*
  let alphan = Regex::new(r"^[_a-zA-Z][_\da-zA-Z]*$").unwrap();
  alphan.is_match(x)
*/
  if x.len()<1 {return false};
  let mut chars = x.chars();
  let first = chars.next().unwrap();
  if !(first=='_' || first.is_alphabetic()) {return false;}
  for c in chars
  {
    if !(c=='_' || c.is_alphanumeric()) {return false;}
  }
  true
}//is_alphanum
