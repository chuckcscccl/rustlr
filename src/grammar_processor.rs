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

pub const DEFAULTPRECEDENCE:i32 = 0;
pub const NONASSOCBIT:i32 = -1 - 0x40000000; // less than this means nonassoc
// if lev<NONASSOCIBT, true precedence level = (lev-NONASSOCIBIT)*-1
pub const TRACE:usize = 0; //deprecated

#[derive(Clone)]
pub struct Gsym // struct for a grammar symbol
{
  pub sym : String,
  pub rusttype : String, // used to derive private enum
  pub terminal : bool,
  pub label : String,  // object-level variable holding value
  pub precedence : i32,   // negatives indicate right associativity
  pub index : usize,   // index into Grammar.Symbols list, Grammar.Symhash
//  pub canextend:boold, // for selML(k,1) algorithm (not static)
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
      index:0,
//      canextend: true,
    }
  }
  pub fn setlabel(&mut self, la:&str)
  { self.label = String::from(la); }
  pub fn settype(&mut self, rt:&str)
  { self.rusttype = String::from(rt); }
  pub fn setprecedence(&mut self, p:i32)
  { self.precedence = p; }
  pub fn gettype<'t>(&self,Gmr:&'t Grammar) -> &'t str
  { &Gmr.Symbols[self.index].rusttype }
  
}// impl for Gsym


//Grammar Rule structure
// This will be used only statically: the action is a string.
// The Gsym structures are repeated on the right-hand side because each
// one can have a different label
#[derive(Clone)]
pub struct Grule  // struct for a grammar rule
{
  pub lhs : Gsym,  // left-hand side of rule
  pub rhs : Vec<Gsym>, // right-hand side symbols (cloned from Symbols)
  pub action : String, //string representation of Ruleaction
  pub precedence : i32, // set to rhs symbol with highest |precedence|
//  pub iprecedence : i8, // for internally generated rules to support REs
}
impl Grule
{
  pub fn new_skeleton(lh:&str) -> Grule
  {
     Grule {
       lhs : Gsym::new(lh,false),
       rhs : Vec::new(),
       action : String::default(),
       precedence : DEFAULTPRECEDENCE,
//       iprecedence : 0,
     }
  }
  pub fn from_lhs(nt:&Gsym) -> Grule
  {
     Grule {
       lhs : nt.clone(),
       rhs : Vec::new(),
       action : String::default(),
       precedence : DEFAULTPRECEDENCE,
//       iprecedence : 0,
     }     
  }
}//impl Grule

pub fn printrule(rule:&Grule,ri:usize)  //independent function
{
   print!("PRODUCTION_{}: {} --> ",ri,rule.lhs.sym);
   for s in &rule.rhs {
      print!("{}",s.sym);
      if s.label.len()>0 {print!(":{}",s.label);}
      print!(" ");
   }
   println!(" action{{ {}, precedence {}",rule.action.trim(),rule.precedence);  // {{ is \{
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
  pub First : HashMap<usize,HashSet<usize>>,
  pub Rulesfor: HashMap<usize,HashSet<usize>>,  //rules for a non-terminal
  pub Absyntype : String,     // string name of abstract syntax type
  pub Externtype : String,    // type of external structure
  pub Resynch : HashSet<String>, // resynchronization terminal symbols, ordered
  pub Errsym : String,        // error recovery terminal symbol
  pub Lexnames : HashMap<String,String>, // print names of grammar symbols
  pub Extras : String,        // indicated by {% .. %}, mostly  use ...
  pub sametype: bool,  // determine if absyntype is only valuetype
  pub lifetime: String,
  pub tracelev:usize,
  pub Lexvals: Vec<(String,String,String)>,  //"int" -> ("Num(n)","Val(n)")
  pub Lexconditionals: Vec<(String,String)>,  
  pub Haslexval : HashSet<String>,
  pub Lexextras: Vec<String>,
  pub enumhash:HashMap<String,usize>, //enum index of each type
  pub genlex: bool,
  pub genabsyn: bool,
  pub Reachable:HashMap<usize,HashSet<usize>>, //usize indexes self.Symbols
//  pub transform_function: String, // for 0.2.96
  pub basictypes : HashSet<&'static str>,
  pub ASTExtras : String,
  pub haslt_base: HashSet<usize>,
  pub delaymarkers: HashMap<usize,BTreeSet<(usize,usize)>>,
  pub flattentypes: HashSet<usize>, //for AST generation
  pub ntcxmax : usize,
  pub startnti: usize,
  pub eoftermi: usize,
  pub startrulei: usize,
  pub mode: i32, // generic mode information
}

impl Default for Grammar {
  fn default() -> Self { Grammar::new() }
}

impl Grammar
{
  pub fn new() -> Grammar
  {
     let mut btypes = HashSet::with_capacity(14);
     for t in ["()","bool","i64","u64","usize","f64","i32","u32","u8","u16","i8","i16","f32","char","(usize,usize)"] { btypes.insert(t);}
     Grammar {
       name : String::from(""),       // name of grammar
       Symbols: Vec::new(),           // grammar symbols
       Symhash: HashMap::new(),       
       Rules: Vec::new(),                 // production rules
       topsym : String::default(),        // top symbol
       Nullable : HashSet::new(),
       First : HashMap::new(),
       Rulesfor: HashMap::new(),
       Absyntype:String::from("()"), //changed for 0.2.7
       Externtype:String::from("()"),    // changed to () for 0.2.9
//       Recover : HashSet::new(),
       Resynch : HashSet::new(),
       Errsym : String::new(),
       Lexnames : HashMap::new(),
       Extras: String::new(),
       sametype:true,
       lifetime:String::new(), // empty means inferred
       tracelev:1,
       Lexvals:Vec::new(),
       Haslexval:HashSet::new(),
       Lexextras:Vec::new(),
       Lexconditionals:Vec::new(),
       genlex: false,
       genabsyn: false,
       enumhash:HashMap::new(),
       Reachable:HashMap::new(),
//       transform_function: String::new(),
       basictypes : btypes,
       ASTExtras: String::new(),
       haslt_base: HashSet::new(), //terminals that contains lifetime
       delaymarkers:HashMap::new(), // delayed LR markers for transformation
       flattentypes:HashSet::new(),
       ntcxmax : 0,
       startnti : 0,
       eoftermi : 0,
       startrulei : 0,
       mode : 0,
     }
  }//new grammar

  pub fn basictype(&self,ty0:&str) -> bool
  {
   let ty=ty0.trim();
   if self.basictypes.contains(ty) {return true;}
   if ty.starts_with('&') && !ty.contains("mut") {return true;}
   false
  }

  pub fn getsym(&self,s:&str) -> Option<&Gsym>
  {
     match self.Symhash.get(s) {
       Some(symi) => Some(&self.Symbols[*symi]),
       _ => None,
     }//match
  }
  pub fn symref(&self,i:usize) -> &str
  {
    &self.Symbols[i].sym
  }
  pub fn Symref(&self,i:usize) -> &Gsym
  {
    &self.Symbols[i]
  }
  pub fn nonterminal(&self,s:&str) -> bool
  {
     match self.Symhash.get(s) {
        Some(symi) => !self.Symbols[*symi].terminal,
	_ => false,
     }
  }
  pub fn nonterminali(&self,s:usize) -> bool
  {
       match self.Symbols.get(s) {
         Some(sym) => !sym.terminal,
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
  pub fn terminali(&self,s:usize) -> bool
  {
       match self.Symbols.get(s) {
         Some(sym) => sym.terminal,
         _ => false,
       }
  }
  pub fn lookuptype(&self,t:&str) -> &str
  {
     if let Some(ti) = self.Symhash.get(t) {&self.Symbols[*ti].rusttype}
     else {""}
  }

////// meta (grammar) parser
  pub fn parse_grammar(&mut self, filename:&str) -> bool // true on success
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
     // records internally generated nt's and symbol they're associated with
     let mut NEWNTs:HashMap<String,usize> = HashMap::new(); 
     let mut enumindex = 0;  // 0 won't be used:inc'ed before first use
     let mut ltopt = String::new();
     let mut ntcx = 2;  // used by -genabsyn option
     self.enumhash.insert("()".to_owned(), 1); //for untyped terminals at least
     let mut wildcard = Gsym::new("_WILDCARD_TOKEN_",true); // special terminal
     wildcard.rusttype="(usize,usize)".to_owned();
     self.enumhash.insert("(usize,usize)".to_owned(),ntcx); ntcx+=1;
     wildcard.index = self.Symbols.len();
     self.Symhash.insert(String::from("_WILDCARD_TOKEN_"),self.Symbols.len());
     self.Symbols.push(wildcard); // wildcard is first symbol.
     let mut markersexist = false; //delay markers exist
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
            eprintln!("MULTI-LINE GRAMMAR PRODUCTION DID NOT END WITH <==, line {}",linenum);  return false;
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
           if line[1..].trim().starts_with("pub ") {
             eprintln!("WARNING: this public declaration may result in redundancy and conflicts, line {}",linenum);
           }
       }
       else if linelen>1 && &line[0..1]=="$" {
           self.ASTExtras.push_str(&line[1..]);
       }       
       else if linelen>1 && &line[0..1]!="#" {
         let toksplit = line.split_whitespace();
         let mut stokens:Vec<&str> = toksplit.collect();
         if stokens.len()<1 {continue;}
                                    
         match stokens[0] {
            "!" => {  // place only in parser file
               let pbi = line.find('!').unwrap();
               self.Extras.push_str(&line[pbi+1..]);
               self.Extras.push_str("\n");                             
            },
            "$" => {  // Place only in AST
               let pbi = line.find('$').unwrap();
               self.ASTExtras.push_str(&line[pbi+1..]);
               self.ASTExtras.push_str("\n");
            },
            "grammarname" => {
               self.name = String::from(stokens[1]);
            },
            "auto" | "genabsyn" => {
               if stage==0 {self.genabsyn=true;} else if !self.genabsyn {
                 eprintln!("ERROR: Place 'auto' at beginning of the grammar or run with -auto option, directive may not be effective.");
               }
            },
            "EOF" => {atEOF=true},
            ("terminal" | "terminals") if stage==0 => {
               for i in 1..stokens.len() {
                 if self.Symhash.contains_key(stokens[i]) {
  	           eprintln!("WARNING: REDEFINITION OF SYMBOL {} SKIPPED, line {} of grammar",stokens[i],linenum);
                   continue;
                 }               
	          let mut newterm = Gsym::new(stokens[i],true);
		  if self.genabsyn {
  		    newterm.rusttype = "()".to_owned();
		  }
		  else {
		    newterm.rusttype = self.Absyntype.clone();
		  }
		  newterm.index = self.Symbols.len();
                  self.Symhash.insert(stokens[i].to_owned(),self.Symbols.len());
                  self.Symbols.push(newterm);
               }
            }, //terminals
	    "typedterminal" if stage==0 && stokens.len()>2 => {
                 if self.Symhash.contains_key(stokens[1]) {
  	           eprintln!("WARNING: REDEFINITION OF SYMBOL {} SKIPPED, line {} of grammar",stokens[1],linenum);
                   continue;
                 }            
	       let mut newterm = Gsym::new(stokens[1],true);
               let mut tokentype = String::new();
               for i in 2..stokens.len() {
                  tokentype.push_str(&stokens[i][..]);
                  tokentype.push(' ');
               }
               let mut nttype = tokentype.trim();
               if nttype.len()<1 {nttype = &self.Absyntype}
               else if nttype!=&self.Absyntype {self.sametype=false;}
               newterm.settype(nttype);
	       self.enumhash.insert(nttype.to_owned(), ntcx);  ntcx+=1;
               newterm.index = self.Symbols.len();
               self.Symhash.insert(stokens[1].to_owned(),self.Symbols.len());
               if self.lifetime.len()>0 && nttype.contains(&self.lifetime) {
                 self.haslt_base.insert(newterm.index);
               }
               self.Symbols.push(newterm);
	    }, //typed terminals
	    "nonterminal" | "typednonterminal" if stage==0 && stokens.len()>1 => {   // with type
	       if self.Symhash.get(stokens[1]).is_some() {
	         eprintln!("WARNING: REDEFINITION OF SYMBOL {} SKIPPED, line {} of grammar",stokens[1],linenum);
		 continue;
	       }
	       let mut newterm = Gsym::new(stokens[1],false);
               if !self.genabsyn {newterm.rusttype = self.Absyntype.clone();}
              if stokens.len()>2 { // type specified, else stays ""
               let mut tokentype = String::new();
               for i in 2..stokens.len() {
                  tokentype.push_str(&stokens[i][..]);
                  tokentype.push(' ');
               }
               // set rusttype
               let mut nttype = tokentype.trim().to_owned();
               // check non-transitive extension:
               if nttype.starts_with(':') {
                let mut limit = self.Symbols.len();
                loop {
                 let copynt = nttype[1..].trim();
                 let copynti = *self.Symhash.get(copynt).expect(&format!("ERROR: EXTENSION TYPE {} NOT DEFINED YET, LINE {}\n\n",copynt,linenum));
                 if self.Symbols[copynti].rusttype.starts_with(':') {
//                   eprintln!("WARNING: TYPE EXTENSIONS ARE NOT TRANSITIVE: THIS MAY NOT WORK, LINE {}\n\n",linenum);
                   nttype = self.Symbols[copynti].rusttype.clone();
                 } else {break;}
                 limit -=1;
                 if limit==0 {
                   eprintln!("WARNING: CIRCULARITY DETECTED IN TYPE DEPENDENCIES; TYPE RESET, LINE {}",linenum);
                   nttype = String::new();
                   break;
                 }
                }//loop
               }//check extension type integrity
               
	       if nttype.contains('@') {// copy type from other NT
                let mut limit =self.Symbols.len()+1;
                loop {
                 let mut copynt="";
                 let (mut start,mut end) = (0,0);
                 if nttype.starts_with('@') {
                    copynt = nttype[1..].trim();
                    start = 0; end = nttype.len();
                 }
                 if let Some(pos1)=nttype.find("<@") {
                   if let Some(pos2)=nttype[pos1+2..].find('>') {
                       copynt = &nttype[pos1+2..pos1+2+pos2];
                       start = pos1+1; end = pos1+2+pos2;
                   }
                 }
                 if copynt.len()>0 {
  	          let onti = *self.Symhash.get(copynt).expect(&format!("UNRECOGNIZED NON-TERMINAL SYMBOL {} TO COPY TYPE FROM (ORDER OF DECLARATION MATTERS), line {} of grammar",copynt,linenum));
		  if !self.genabsyn {
                    nttype.replace_range(start..end,&self.Symbols[onti].rusttype);
                  }//if !auto, need to set type now
                 }
                 limit -= 1;
                 if !nttype.contains('@') || limit==0 {break;}
                }//loop
	       } // *NT copy type from other NT
               if nttype.len()<1 && !self.genabsyn {nttype = self.Absyntype.clone()};
	       if !nttype.contains('@') && !nttype.starts_with(':') {self.enumhash.insert(nttype.clone(), ntcx); ntcx+=1;}
               if &nttype!=&self.Absyntype {self.sametype=false;}
	       newterm.rusttype = nttype;
              } // type specified
               newterm.index = self.Symbols.len();
               self.Symhash.insert(stokens[1].to_owned(),self.Symbols.len());
               self.Symbols.push(newterm);
               self.Rulesfor.insert(self.Symbols.len()-1,HashSet::new());
	    }, //nonterminal
            "nonterminals" if stage==0 => {
               for i in 1..stokens.len() {
                 if self.Symhash.contains_key(stokens[i]) {
  	           eprintln!("WARNING: REDEFINITION OF SYMBOL {} SKIPPED, line {} of grammar",stokens[i],linenum);
                   continue;
                 }
                 
	          let mut newterm = Gsym::new(stokens[i],false);
                  newterm.index = self.Symbols.len();                  
                  self.Symhash.insert(stokens[i].to_owned(),self.Symbols.len());
		  if !self.genabsyn {newterm.rusttype = self.Absyntype.clone();}
		  ntcx+=1; 
                  self.Symbols.push(newterm);
                  self.Rulesfor.insert(self.Symbols.len()-1,HashSet::new());
               }
            },
	    "topsym" | "startsymbol" /*if stage==0*/ => {
               if stage>1 {eprintln!("Grammar start symbol must be defined before production rules, line {}",linenum); return false;}  else {stage=1;}
               match self.Symhash.get(stokens[1]) {
                 Some(tsi) if *tsi<self.Symbols.len() && !self.Symbols[*tsi].terminal => {
              	    self.topsym = String::from(stokens[1]);
                    let toptype = &self.Symbols[*tsi].rusttype;
                    if toptype != &self.Absyntype && !self.genabsyn && toptype.len()>0 {
                       eprintln!("WARNING: Type of Grammar start symbol {} set to {}; you should declare the valuetype unless using -auto mode.",stokens[1],&self.Absyntype);
                       if !self.genabsyn {self.Symbols[*tsi].rusttype = self.Absyntype.clone();}
                    }
                 },
                 _ => { eprintln!("top symbol {} not found in declared non-terminals; check ordering of declarations, line {}",stokens[1],linenum);
                        return false;
                 },
               }//match
	       //if TRACE>4 {println!("top symbol is {}",stokens[1]);}
	    }, //topsym
            "flatten" if stokens.len()>=2 => {
              for tok in stokens[1..].iter() {
                let fnti = *self.Symhash.get(&tok[..]).expect(&format!("UNDEFINED GRAMMAR SYMBOL {}, LINE {}\n",tok,linenum));
                if self.Symbols[fnti].terminal {
                  eprintln!("WARNING: ONLY NON-TERMINALS CAN HAVE THEIR ASTS FLATTENED ({}), LINE {}\n",tok,linenum);
                }
                else {self.flattentypes.insert(fnti);}
              }//for each tok
            },
            "errsym" | "errorsymbol" => {
               if stage>1 {
                 eprintln!("!!! Error recover symbol must be declared before production rules, line {}",linenum);
                 return false;
               }
               if stage==0 {stage=1;}
               if !self.terminal(stokens[1]) {
                 eprintln!("!!!Error recover symbol {} is not a terminal, line {} ",stokens[1],linenum);
                 return false;
               }
               self.Errsym = stokens[1].to_owned();
            },
            /*
            "recover"  => {
               if stage==0 {stage=1;}
               for i in 1..stokens.len()
               {
                  if !self.nonterminal(stokens[i]) {
                     eprintln!("!!!Error recovery symbol {} is not a declared non-terminal, line {}",stokens[i],linenum);  return false;
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
                     eprintln!("!!!Error recovery re-synchronization symbol {} is not a declared terminal, line {}",stokens[i],linenum); return false;
                  }
                  self.Resynch.insert(stokens[i].trim().to_owned());
               } // for each subsequent token
            },
            "lifetime" if stokens.len()==2 && stokens[1].len()>0  && stage==0 => {
               self.lifetime = if &stokens[1][0..1]=="'" && stokens[1].len()>1 
                 {String::from(stokens[1])} else {format!("'{}",stokens[1])};
	       ltopt = format!("<{}>",&self.lifetime);
            },
            "absyntype" | "valuetype" /*if stage==0*/ => {
               if stage>0 {eprintln!("The grammar's abstract syntax type must be declared before production rules, line {}",linenum); return false;}
               if self.genabsyn {
                 eprintln!("WARNING: absyntype/valuetype declaration ignored in -auto (genabsyn) mode, line {}", linenum);
                 continue;
               }
               let pos = line.find(stokens[0]).unwrap() + stokens[0].len();
               self.Absyntype = String::from(line[pos..].trim());
            },
            "externtype" | "externaltype" if stage==0 => {
               let pos = line.find(stokens[0]).unwrap() + stokens[0].len();
               self.Externtype = String::from(line[pos..].trim());            
            },            
	    "left" | "right" | "nonassoc" if stage<2 && stokens.len()>2 => {
               if stage==0 {stage=1;}
               if stokens.len()<3 {
	         eprintln!("MALFORMED ASSOCIATIVITY/PRECEDENCE DECLARATION SKIPPED ON LINE {}",linenum);
	         continue;
	       }
	       let mut preclevel:i32 = DEFAULTPRECEDENCE;
	       if let Ok(n)=stokens[2].parse::<i32>() {
                 if n>0 && n<=0x40000000 {preclevel = n;}
                 else {eprintln!("ERROR: PRECEDENCE VALUE MUST BE BETWEEN 1 AND {}, LINE {}\n",0x40000000,linenum); return false;}
               }
               else {eprintln!("ERROR: Did not read precedence level on line {}\n",linenum); return false;}
               if stokens[0]=="nonassoc" && preclevel>0 { preclevel = NONASSOCBIT-preclevel;}
	       else if stokens[0]=="right" && preclevel>0 {preclevel = -1 * preclevel;}
               let mut targetsym = stokens[1];
               if targetsym=="_" {targetsym = "_WILDCARD_TOKEN_";}
               if let Some(index) = self.Symhash.get(targetsym) {
               /*
                 if preclevel.abs()<=DEFAULTPRECEDENCE {
                   eprintln!("WARNING: precedence of {} is non-positive",stokens[1]);
                 }
               */
                 self.Symbols[*index].precedence = preclevel;
               } else {eprintln!("UNDEFINED GRAMMAR SYMBOL {}, LINE {}\n",targetsym,linenum); return false;}
	    }, // precedence and associativity, left or right or nonassoc
	    "lexname"  => {
               if stokens.len()<3 {
	         eprintln!("MALFORMED lexname declaration line {} skipped",linenum);
	         continue;
	       }
               self.Lexnames.insert(stokens[2].to_string(),stokens[1].to_string());
	       self.Haslexval.insert(stokens[1].to_string());
	       self.genlex = true;
            },
	    "lexvalue" => {
               let pos = line.find("lexvalue").unwrap()+9;
               let declaration = &line[pos..];
               let dtokens:Vec<_>=declaration.split_whitespace().collect();
	       if dtokens.len()<3 {
	         eprintln!("MALFORMED lexvalue declaration skipped, line {}",linenum);
	         continue;
	       }  // "int" -> ("Num(n)","Val(n)")
	       let mut valform = String::new();
	       for i in 2 .. dtokens.len()
	       {
	         valform.push_str(dtokens[i]);
		 if (i<dtokens.len()-1) {valform.push(' ');}
	       }
               let tokform = dtokens[1].to_owned();
	       self.Lexvals.push((dtokens[0].to_string(),tokform,valform));
	       // record that this terminal always carries a value
	       self.Haslexval.insert(dtokens[0].to_string());
	       self.genlex = true;
	    },
            "valueterminal" => {
               let pos = line.find("valueterminal").unwrap()+14;
               let declaration = &line[pos..];
               let mut usingcolon = true;
               let mut dtokens:Vec<_> = declaration.split('~').collect();
               if dtokens.len()>1 && dtokens.len()<4 {
                 eprintln!("ERROR ON LINE {}. MISSING ~",linenum);
                 return false;
               }
               if dtokens.len()<4 {dtokens=declaration.split_whitespace().collect(); usingcolon=false;}
	       if dtokens.len()<4 {
	         eprintln!("MALFORMED valueterminal declaration skipped, line {}",linenum);
	         continue;
	       }  // valueterminal ID: String: Alphanum(n) if ... : n.to_owned()
               let termname = dtokens[0].trim();
                 if self.Symhash.contains_key(termname) {
  	           eprintln!("WARNING: REDEFINITION OF SYMBOL {} SKIPPED, line {} of grammar",termname,linenum);
                   continue;
                 }               
               let mut newterm = Gsym::new(termname,true);
               let termtype = dtokens[1].trim();
               if termtype.len()<1 {newterm.settype(&self.Absyntype);}
               else {newterm.settype(termtype);}
               if &newterm.rusttype!=&self.Absyntype {self.sametype=false;}
               self.enumhash.insert(newterm.rusttype.clone(),ntcx); ntcx+=1;
               newterm.index = self.Symbols.len();
               self.Symhash.insert(termname.to_owned(),self.Symbols.len());
               if self.lifetime.len()>0 && newterm.rusttype.contains(&self.lifetime) {
                 self.haslt_base.insert(newterm.index);
               }
               self.Symbols.push(newterm);
	       let mut valform = String::new(); // equiv to lexvalue...
	       for i in 3 .. dtokens.len()
	       {
	         valform.push_str(dtokens[i]);
		 if (i<dtokens.len()-1 && !usingcolon) {valform.push(' ');}
                 else if (i<dtokens.len()-1) {valform.push('~');}
	       }
               let tokform = dtokens[2].to_owned();
	       self.Lexvals.push((termname.to_string(),tokform,valform));
	       // record that this terminal always carries a value
	       self.Haslexval.insert(termname.to_string());
	       self.genlex = true;
            }, //valueterminal
            "lexterminal" => {
               if stokens.len()!=3 {
               eprintln!("MALFORMED lexterminal declaration line {}: a terminal name and a lexical form are required",linenum); return false;
	         //continue;
               }
               let termname = stokens[1].trim();
                 if self.Symhash.contains_key(termname) {
  	           eprintln!("WARNING: REDEFINITION OF SYMBOL {} SKIPPED, line {} of grammar",termname,linenum);
                   continue;
                 }                              
               let mut newterm = Gsym::new(termname,true);
               if self.genabsyn { newterm.settype("()"); }
               else {newterm.settype(&self.Absyntype);}
               newterm.index = self.Symbols.len();
               self.Symhash.insert(termname.to_owned(),self.Symbols.len());
               self.Symbols.push(newterm);
               self.Lexnames.insert(stokens[2].to_string(),termname.to_string());
	       self.Haslexval.insert(termname.to_string());
	       self.genlex = true;
            }, //lexterminal
	    "lexattribute" => {
	       let mut prop = String::new();
	       for i in 1 .. stokens.len()
	       {
	          prop.push_str(stokens[i]); prop.push(' ');
	       }
	       self.Lexextras.push(prop);
	       self.genlex = true;
	    },

            "lexconditional" if stokens.len() > 2 => {
               let pos = line.find("lexconditional").unwrap()+15;
               let mut dtokens:Vec<_> = line[pos..].split('~').collect();
               self.Lexconditionals.push((dtokens[0].trim().to_owned(),dtokens[1].trim().to_owned()));
            },

            "transform" => {   // new for 0.2.96, transform_token added
              /*
               let pos = line.find("transform").unwrap()+10;
               self.transform_function = line[pos..].trim().to_owned();
              */
              eprintln!("WARNING: DECLARATION IGNORED, Line {}. The transform directive was only used in Rustlr version 0.2.96 and no longer supported.  Use the shared_state variable for a more general solution.",linenum);
            },
//////////// case for grammar production:

	    LHS0 if stokens.len()>1 => {
              let mut separator = "-->";
              let sepposition;
              if let Some(spos) = line.find("-->") {
                sepposition = spos;
              }
              else if let Some(mpos) = line.find("==>") { // multiline mode
                sepposition = mpos;
                separator = "==>";
              }
              else {
                eprintln!("ERROR PARSING GRAMMAR LINE {}, unexpected declaration at grammar stage {}",linenum,stage);
                return false;
              }
              if !foundeol && separator=="==>" {multiline=true; continue;}
              else if foundeol {foundeol=false;}
              // println!("RULE {}",&line);

              if sepposition < stokens[0].len() {
                stokens[0] = &stokens[0][..sepposition];
              }

              if stage<2 {stage=2;}
              
	    // construct lhs symbol
	      let findcsplit:Vec<_> = stokens[0].split(':').collect();
	      let mut LHS = findcsplit[0];
	      //findcsplit[1] will be used to auto-gen AST type below
              //let mut lhsym = &self.Symbols[*symindex]; //not .clone();

            // parse default rule precedence (for all bar-splits!)
            let mut manual_precedence = 0;
            let (lb,rb)=findmatch(LHS0,'(',')');
            if rb!=0 && lb+1<rb {
              let parseopt = LHS0[lb+1..rb].parse::<i32>();
              if let Ok(lev)=parseopt {manual_precedence=lev;}
              else {eprintln!("ERROR: Precedence Level ({}) must be numeric, line {}\n",&LHS[lb+1..rb],linenum); return false;}
              LHS = &stokens[0][..lb];  // change LHS from above
            }
            else if (lb,rb)!=(0,0) {
               eprintln!("MALFORMED LEFT HAND SIDE LINE {}\n",linenum);
               return false;
            }// parse default precedence
            let symindex = match self.Symhash.get(LHS) {
               Some(smi) if *smi<self.Symbols.len() && !self.Symbols[*smi].terminal => smi,
               _ => {eprintln!("unrecognized non-terminal symbol {}, line {}",LHS,linenum); return false;},
             };
            let symind2 = *symindex;
             let mut ntcnt = 0; // for generating new nonterminal names

              // split by | into separate rules

              let pos0 = sepposition + 3; // position after --> or ==>
              let mut linec = &line[pos0..]; //.to_string();
              //let barsplit:Vec<_> = linec.split('|').collect();
 	      // this can't handle the | symbol that's inside the semantic
	      // action block - 0.2.6 fix NOT COMPLETE:  print("|x|")
	      // use split_once + loop
	      let mut barsplit = Vec::new();
	      let mut linecs = linec;
	      while let Some(barpos) = findskip(linecs,'|') //findskip at end
              {
		 let (scar,scdr) = linecs.split_at(barpos);
		 barsplit.push(scar.trim());
		 linecs = &scdr[1..];
	      }//barsplit loop
	      barsplit.push(linecs.trim()); // at least one

              if barsplit.len()>1 && findcsplit.len()>1 {
	        eprintln!("The '|' symbol is not accepted in rules that has an labeled non-terminal on the left-hand side ({}) as it becomes ambiguous as to how to autmatically generate abstract syntax, line {}",findcsplit[1],linenum);
                return false;
	      }
              
              for rul in &barsplit 
              { //if rul.trim().len()>0 {  // must include empty productions!
              //println!("see rule seg ({})",rul);
              let bstokens:Vec<_> = rul.trim().split_whitespace().collect();
              let mut rhsyms:Vec<Gsym> = Vec::new();
              let mut semaction = "}";
	      let mut i:usize = 0;   // bstokens index on one barsplit 
              let mut maxprec:i32 = 0;
              let mut seenerrsym = false;
              let mut iadjust = 0;
              let mut markers = Vec::new(); // record delay markers
              while i<bstokens.len() {
	        let mut strtok = bstokens[i];
		i+=1;
                if strtok.len()>0 && &strtok[0..1]=="{" {
                   let position = rul.find('{').unwrap();
                   semaction = rul.split_at(position+1).1;
                   if self.genabsyn && semaction.contains("return ") {
                     eprintln!("WARNING: USING \"return\" INSIDE SEMANTIC ACTIONS COULD CAUSE CONFLICTS WITH AUTOMATIC CODE GENERATION, LINE {}\n",linenum);
                   }
		   break;
                }
// look for delay marker and record
                if strtok=="#" {markers.push(i-1-iadjust); iadjust+=1; markersexist=true; continue; }

/*
Strategfy for parsing EBNF syntax:
a. transform (E ;)* to E1*, E1 --> E ;
b. transform E1* to E2,  E2 --> | E2 E1

strtok is bstokens[i], but will change
*/

                // add code to recognize (E ;)*, etc.
                // (E ;)* and (E ,)* are to have different meaning, then dont
                // use this notation.  Only use in -auto mode as it will
                // generate ast, semaction for the new nonterminal.
// NEWNTs table not used - so duplicates may result
                let newtok2;
		if strtok.len()>1 && strtok.starts_with('(') {
                  let mut ntname2 = format!("NEWSEQNT_{}_{}",self.Rules.len(),ntcnt);
                  ntcnt+=1;
                  let mut newnt2 = Gsym::new(&ntname2,false);
                  let mut newrule2 = Grule::new_skeleton(&ntname2);
	          let mut defaultrelab2 = String::new(); //format!("_item{}_",i-1-iadjust);
                  let mut retoki = &strtok[1..]; // without (
                  let mut passthru:i64 = -1;
                  let mut jk = 0;  //local index of rhs
                  let mut suffix="";
                  let mut precd = 0; // set precedence
                  while i<=bstokens.len() // advance i until see )*, or )+, )?
                  {
                     // get the part before :label
                     let retokisplit:Vec<&str> = retoki.split(':').collect();
                     let mut breakpoint = false;
                     if retokisplit[0].ends_with('>') {
                        if let Some(rpp) = retokisplit[0].rfind(')') {
                           breakpoint = true;
                           retoki = &retokisplit[0][..rpp];
                           if (retoki.len()<1) {eprintln!("INVALID EXPRESSION IN GRAMMAR LINE {}: DO NOT SEPARATE TOKEN FROM `)`\n",linenum); return false;}
                           if retokisplit.len()>1 {
                             defaultrelab2=retokisplit[1].to_owned();
                             if !is_alphanum(checkboxlabel(&defaultrelab2)) {
                               eprintln!("ERROR: LABELS FOR RE EXPRESSIONS CANNOT BE PATTERNS, LINE {}\n",linenum); return false;
                             }
                           }
                        }
                        else {eprintln!("INVALID EXPRESSION IN GRAMMAR LINE {}: DO NOT SEPARATE TOKEN FROM `)`\n",linenum); return false;}
                     }
                     else
                     if retokisplit[0].ends_with(")*") || retokisplit[0].ends_with(")+") || retokisplit[0].ends_with(")?") {
                       breakpoint=true;
                       retoki =  &retokisplit[0][..retokisplit[0].len()-2];
                       if (retoki.len()<1) {eprintln!("INVALID EXPRESSION IN GRAMMAR LINE {}: DO NOT SEPARATE TOKEN FROM `)`\n",linenum); return false;}
                       suffix = &retokisplit[0][retokisplit[0].len()-1..];
                       if retokisplit.len()>1 {defaultrelab2=retokisplit[1].to_owned();}
                     } // if retokisplit[0].ends_with(")*")...
                     else if retokisplit.len()>1 {
                       eprintln!("LABELS (:{}) ARE NOT ALLOWED INSIDE (..) GROUPINGS, LINE {}",retokisplit[1],linenum); return false;
                     }
                     // retoki should not end with )?, etc...
                     if retoki.ends_with("*") || retoki.ends_with("+") || retoki.ends_with("?") || retoki.ends_with(">") {
                         eprintln!("NESTED *, +, ? and <> EXPRESSIONS ARE NOT ALLOWED, LINE {}\n",linenum); return false;
                       }
                     
                     let errmsg = format!("unrecognized grammar symbol '{}', line {}",retoki,linenum);
		     let gsymi = *self.Symhash.get(retoki).expect(&errmsg);
                     let igsym = &self.Symbols[gsymi];
                     if prec_level(igsym.precedence).abs()>prec_level(precd).abs() {precd =igsym.precedence;}
                     if passthru==-1 && (!igsym.terminal || igsym.rusttype!="()") {
                       passthru=jk;
                       //newnt2.rusttype = igsym.rusttype.clone();
                       newnt2.rusttype = format!("@{}",&igsym.sym);
                       // or put * before it, fill-in later
                     }
                     else if passthru>=0 && (!igsym.terminal || igsym.rusttype!="()" || igsym.precedence!=0)
                     {passthru=-2; newnt2.rusttype=String::new();}
                     newrule2.rhs.push(self.Symbols[gsymi].clone());
                     //if retokisplit[0].ends_with(")*") || retokisplit[0].ends_with(")+") {break;}
                     if breakpoint {break;}
                     else if bstokens[i-1].starts_with('{') {i=bstokens.len()+1; break;}
                     jk += 1; //local, for passthru
                     i+=1; // indexes bstokens
                     retoki = bstokens[i-1];
                  }// while i<=bstokens.len()
                  if i>bstokens.len() {eprintln!("INVALID EXPRESSION IN GRAMMER, line {}",linenum); return false;}
                  iadjust += jk as usize;
                  if passthru>=0 { // set action of new rule to be passthru
                    newrule2.action = format!(" _item{}_ }}",passthru);
//   println!("passthru found on {}, type is {}",&newnt2.sym,&newnt2.rusttype);
                  }
                  // register new symbol
                  //// form hashkey from rhs of newrule2
                  let mut hashkey = String::from("(");
                  for s in &newrule2.rhs {
                    hashkey.push_str(&s.sym); hashkey.push(' ');
                  }
                  hashkey.push(')');
                  if let Some(snti) = NEWNTs.get(&hashkey) { //reuse
                    ntname2 = self.Symbols[*snti].sym.clone();
                  }// reuse nt
                  else { // create new nt, rule
                  newrule2.precedence = precd;
                  newnt2.index = self.Symbols.len();
                  newrule2.lhs.index = newnt2.index;
                  self.Symhash.insert(ntname2.clone(),self.Symbols.len());
                  self.Symbols.push(newnt2);
                  // register new rule
                   if self.tracelev>3 {
                     printrule(&newrule2,self.Rules.len());
                   }
                  self.Rules.push(newrule2);
                  let mut rulesforset = HashSet::new();
                  rulesforset.insert(self.Rules.len()-1);
                  self.Rulesfor.insert(self.Symbols.len()-1,rulesforset);
                  // i-1 is now at token with )* or )+
                  if defaultrelab2.len()<1 && !markersexist {defaultrelab2=format!("_item{}_",i-1-iadjust);}
                  else if defaultrelab2.len()<1 {defaultrelab2=format!("_itemre{}_{}",i-1-iadjust,ntcx); ntcx+=1;}
                  /*
                  labels must be different to avoid clashes if static delay
                  markers exist.  But they should be the standard _item_ to
                  recognize passthru on user defined types during AST gen.
                  */
                  NEWNTs.insert(hashkey,self.Symbols.len()-1); //record
                  }// create new nt       
                  newtok2 = format!("{}{}:{}",&ntname2,suffix,&defaultrelab2);
                  strtok = &newtok2;
                } // starts with (
//println!("i at {}, iadjust {},  line {}",i,iadjust,linenum);



		// add code to recognize E*, E+ and E?, aftert ()'s removed -
                // Assuming *,+,? preceeded by a single grammar symbol
                let newtok; // will be new strtok
		let retoks:Vec<&str> = strtok.split(':').collect();
		if retoks.len()>0 && retoks[0].len()>1 && (retoks[0].ends_with('*') || retoks[0].ends_with('+') || retoks[0].ends_with('?')) {
		   strtok = retoks[0]; // to be changed back to normal a:b
                   let defaultrelab;
                   if !markersexist {
		     defaultrelab = format!("_item{}_",i-1-iadjust);
                   } else {
                     defaultrelab = format!("_itemre{}_{}",i-1-iadjust,ntcx);
                     ntcx+=1;
                   }
		   let relabel = if retoks.len()>1 && retoks[1].len()>0
                     {
                         if !is_alphanum(checkboxlabel(retoks[1])) {
                            eprintln!("ERROR: LABELS FOR RE EXPRESSIONS CANNOT BE PATTERNS, LINE {}\n",linenum); return false;
                         }                     
                       retoks[1]
                     }
                     else {&defaultrelab};
		   let mut gsympart = strtok[0..strtok.len()-1].trim(); //no *
                   if gsympart=="_" {gsympart="_WILDCARD_TOKEN_";}
		   let errmsg = format!("unrecognized grammar symbol '{}', line {}",gsympart,linenum);
		   let gsymi = *self.Symhash.get(gsympart).expect(&errmsg);

                 //// generate new nt or reuse one that already exists
                 if let Some(enti) = NEWNTs.get(retoks[0]) {
                   newtok = format!("{}:{}",&self.Symbols[*enti].sym,relabel);
                   strtok = &newtok;
                 }
                 else { //** generate new set of terms and rules

		   let newntname = format!("NEWRENT_{}_{}",self.Rules.len(),ntcnt); ntcnt+=1;
		   let mut newnt = Gsym::new(&newntname,false);
                   newnt.rusttype = "()".to_owned();
                   // following means symbols such as -? will not be
                   // part of ast type unless there is a given label: -?:m
                   if &self.Symbols[gsymi].rusttype!="()" || (retoks.len()>1 && retoks[1].len()>0) {
		     newnt.rusttype = if strtok.ends_with('?') {
                       if self.basictypes.contains(&self.Symbols[gsymi].rusttype[..]) || self.Symbols[gsymi].rusttype.starts_with("Vec<") || self.Symbols[gsymi].rusttype.starts_with("LBox") {
                        if self.genabsyn {format!("Option<@{}>",&self.Symbols[gsymi].sym)} else {format!("Option<{}>",&self.Symbols[gsymi].rusttype)} }
                       else {
                        if self.genabsyn {format!("Option<LBox<@{}>>",&self.Symbols[gsymi].sym)} else {format!("Option<LBox<{}>>",&self.Symbols[gsymi].rusttype)} }
                     }
                     else {
                       if self.genabsyn {format!("Vec<LBox<@{}>>",&self.Symbols[gsymi].sym)} else {format!("Vec<LBox<{}>>",&self.Symbols[gsymi].rusttype)} };
                   }
                   // else type stays () (,*)
                   /*  later
		   if !self.enumhash.contains_key(&newnt.rusttype) {
 		     self.enumhash.insert(newnt.rusttype.clone(),ntcx);
		     ntcx+=1;
		   }
                   */
                   newnt.index = self.Symbols.len();
		   self.Symhash.insert(newntname.clone(),self.Symbols.len());
		   self.Symbols.push(newnt.clone());
		   // add new rules
		   let mut newrule1 = Grule::new_skeleton(&newntname);
		   //newrule1.lhs.rusttype = newnt.rusttype.clone();
                   newrule1.lhs.index = newnt.index;
                   let nr1type = &self.Symbols[newnt.index].rusttype;
                   newrule1.precedence = self.Symbols[gsymi].precedence;
		   if strtok.ends_with('?') {
		     newrule1.rhs.push(self.Symbols[gsymi].clone());
                     if nr1type.starts_with("Option<LBox<") {
		       newrule1.action=String::from(" Some(parser.lbx(0,_item0_)) }"); } else if nr1type.starts_with("Option<") {newrule1.action = String::from(" Some(_item0_) }"); } // else nothing
		   }// end with ?
		   else { // * or +
  		     newrule1.rhs.push(newnt.clone());
		     newrule1.rhs.push(self.Symbols[gsymi].clone());
                     if nr1type!="()" {
		       newrule1.action = String::from(" _item0_.push(parser.lbx(1,_item1_)); _item0_ }");
                     }
		   } // * or +
		   let mut newrule0 = Grule::new_skeleton(&newntname);
		   //newrule0.lhs.rusttype = newnt.rusttype.clone();
                   let nr0type = &self.Symbols[newnt.index].rusttype;
                   newrule0.lhs.index = newnt.index;
		   if strtok.ends_with('+') {
		     newrule0.rhs.push(self.Symbols[gsymi].clone());
                     if nr0type!="()" {
		       newrule0.action=String::from(" vec![parser.lbx(0,_item0_)] }");
                     }
		   }// ends with +
		   else if strtok.ends_with('*') && nr0type!="()" {
		     newrule0.action = String::from(" Vec::new() }");
		   }
		   else if strtok.ends_with('?') && nr0type!="()" {
		     newrule0.action = String::from(" None }");
		   }
                   if self.tracelev>3 {
                     printrule(&newrule0,self.Rules.len());
                     printrule(&newrule1,self.Rules.len()+1);   
                   }
		   self.Rules.push(newrule0);
		   self.Rules.push(newrule1);
		   let mut rulesforset = HashSet::with_capacity(2);
		   rulesforset.insert(self.Rules.len()-2);
		   rulesforset.insert(self.Rules.len()-1);
		   newtok = format!("{}:{}",&newntname,relabel);
		   self.Rulesfor.insert(self.Symbols.len()-1,rulesforset);
                   NEWNTs.insert(retoks[0].to_owned(),newnt.index);
		   // change strtok to new form
		   strtok = &newtok;
                  } //** generate new set of rules and nt's
//println!("2 strtok now {}",strtok);                   
		}// processes RE directive - add new productions


                ///// process E<COMMA*>  or    E<SEMICOLON+>
                ///// vector of E-values separated by the indicated
                ///// terminal - must be terminal symbol of type ()
                let mut newtok3; // will be new strtok
		let septoks:Vec<&str> = strtok.split(':').collect();
		if septoks.len()>0 && septoks[0].len()>2 && (septoks[0].ends_with("*>") || septoks[0].ends_with("+>")) {                
                  let (lb,rb) = findmatch(strtok,'<','>');
                  let termi;
                  if lb!=0 && lb+2<rb  {
                    // determine if what's inside <> is valid
                    let termsym = &strtok[lb+1..rb-1]; // like COMMA
                    let termiopt = self.Symhash.get(termsym);
                    if !self.terminal(termsym) {
                      eprintln!("ERROR ON LINE {}, {} is not a terminal symbol of this grammar\n",linenum,termsym); return false;
                    }
                    termi = *termiopt.unwrap();
                  } else {eprintln!("MALFORMED EXPRESSION LINE {}\n",linenum); return false;}
                  strtok = septoks[0]; // to the left of :, E<,*>
                  let defaultrelab3;
                  if !markersexist {
   	            defaultrelab3 = format!("_item{}_",i-1-iadjust);
                  } else {
                    defaultrelab3 = format!("_itemre{}_{}",i-1-iadjust,ntcx);
                    ntcx+=1;
                  }
		  let relabel3 = if septoks.len()>1 && septoks[1].len()>0 {
                     if !is_alphanum(checkboxlabel(septoks[1])) {
                       eprintln!("ERROR: LABELS FOR RE EXPRESSIONS CANNOT BE PATTERNS, LINE {}\n",linenum); return false;
                     }
                     septoks[1]
                  } else {&defaultrelab3};
    	          let mut gsympart3 = strtok[0..lb].trim(); //before <,*>
                  if gsympart3=="_" {gsympart3="_WILDCARD_TOKEN_";}
   	          let errmsg = format!("UNRECOGNIZED GRAMMAR SYMBOL '{}', LINE {}\n",gsympart3,linenum);
	          let gsymi = *self.Symhash.get(gsympart3).expect(&errmsg);

                 //// generate new nt or reuse one that already exists
                 let hashkey = format!("{}{}",gsympart3,&strtok[lb..rb+1]);
                 if let Some(enti) = NEWNTs.get(&hashkey) {
                   newtok3 = format!("{}:{}",&self.Symbols[*enti].sym,relabel3);
                   strtok = &newtok3;
                 }
                 else { // need to generate new set of terms and rules ***
		  let newntname3 = format!("NEWSEPNT_{}_{}",self.Rules.len(),ntcnt); ntcnt+=1;
	          let mut newnt3 = Gsym::new(&newntname3,false);
                  newnt3.rusttype = "()".to_owned();
                  if &self.Symbols[gsymi].rusttype!="()" || (septoks.len()>1 && septoks[1].len()>0) {
		     newnt3.rusttype = format!("Vec<LBox<@{}>>",&self.Symbols[gsymi].sym);
                  } // else rusttype stays ()
                  // note: if types are not being generated, what should this
                  // type be? can note that it should follow from
                  // type of a symbol, in special format like Expr* or
                  // can create map inside Grammar with this info.
                  /*
	          if !self.enumhash.contains_key(&newnt3.rusttype) {
 		     self.enumhash.insert(newnt3.rusttype.clone(),ntcx);
		     ntcx+=1;
		  }
                  */
                  newnt3.index = self.Symbols.len();
		  self.Symhash.insert(newntname3.clone(),self.Symbols.len());
		  self.Symbols.push(newnt3.clone()); // register new nt
		   // add new rules
		  let mut newrule3 = Grule::new_skeleton(&newntname3);
		  let mut newrule4 = Grule::new_skeleton(&newntname3);
  		  //newrule3.lhs.rusttype = newnt3.rusttype.clone();
  		  //newrule4.lhs.rusttype = newnt3.rusttype.clone();
                  newrule3.lhs.index = newnt3.index;
                  newrule4.lhs.index = newnt3.index;
                  newrule3.precedence = self.Symbols[termi].precedence;
                  //PRECEDENCE SET TO SEPARATOR SYMBOL
                  newrule4.precedence = self.Symbols[termi].precedence;
                  // GENERATE AS FOR <COMMA+>
                  newrule3.rhs.push(self.Symbols[gsymi].clone()); //N-->E
                  newrule4.rhs.push(newnt3.clone());
                  newrule4.rhs.push(self.Symbols[termi].clone());
                  newrule4.rhs.push(self.Symbols[gsymi].clone());//N-->N,E
                  if newnt3.rusttype.starts_with("Vec") {
                    newrule3.action=String::from(" vec![parser.lbx(0,_item0_)] }");                  
                    newrule4.action=String::from(" _item0_.push(parser.lbx(2,_item2_)); _item0_ }");
                  } // else leave at default for ()
                  if self.tracelev>3 {
                    printrule(&newrule3,self.Rules.len());
                    printrule(&newrule4,self.Rules.len()+1);
                  }
		  self.Rules.push(newrule3);
   	          self.Rules.push(newrule4);
		  let mut rulesforset3 = HashSet::with_capacity(2);
		  rulesforset3.insert(self.Rules.len()-2);
		  rulesforset3.insert(self.Rules.len()-1);
                  newtok3 = format!("{}:{}",&newntname3,relabel3);
                  self.Rulesfor.insert(newnt3.index,rulesforset3);
                  // ANOTHER RULE IS NEEDED IF strtok ends in *>
                  if !strtok.ends_with("*>") {
                    NEWNTs.insert(hashkey,newnt3.index);
                  } else { // M --> null | N for *>
                    //still enter newnt3 into NEWNTs map as it could be useful
                    let hashkey2 = format!("{}+>",&hashkey[..hashkey.len()-2]);
                    NEWNTs.insert(hashkey2,newnt3.index);                    
                    let newntname5 = format!("NEWSEPNT2_{}_{}",self.Rules.len(),ntcnt); ntcnt+=1;
                    let mut newnt5 = Gsym::new(&newntname5,false);
                    newnt5.rusttype = newnt3.rusttype.clone();
                    newnt5.index = self.Symbols.len();
		    self.Symhash.insert(newntname5.clone(),self.Symbols.len());
                    self.Symbols.push(newnt5.clone()); // register new nt
                    let mut newrule5 = Grule::new_skeleton(&newntname5);
                    let mut newrule6 = Grule::new_skeleton(&newntname5);
  		    //newrule5.lhs.rusttype = newnt5.rusttype.clone();
  		    //newrule6.lhs.rusttype = newnt5.rusttype.clone();
                    newrule5.lhs.index = newnt5.index; // simplify
                    newrule6.lhs.index = newnt5.index;
                    // 0 precedence for rule, newrule5 has empty rhs
                    newrule6.rhs.push(newnt3.clone());
                    if newnt5.rusttype.starts_with("Vec") {
                       newrule5.action = String::from(" vec![] }");
                       newrule6.action = String::from("_item0_ }");
                    }
                    if self.tracelev>3 {
                      printrule(&newrule5,self.Rules.len());
                      printrule(&newrule6,self.Rules.len()+1);
                    }                    
  		    self.Rules.push(newrule5);
   	            self.Rules.push(newrule6);
  		    let mut rulesforset5 = HashSet::with_capacity(2);
		    rulesforset5.insert(self.Rules.len()-2);
		    rulesforset5.insert(self.Rules.len()-1);
                    newtok3 = format!("{}:{}",&newntname5,relabel3);
                    self.Rulesfor.insert(newnt5.index,rulesforset5);
                    // insert into NEWNTs map
                    NEWNTs.insert(hashkey,newnt5.index);
                  } // *> processed
                  strtok = &newtok3;
                 } // *** needed to generate new nt, rules
                } // if ends with *> or +>


                ////////////////////////// BACK TO ORIGINAL
		//////////// separte gsym from label:
		let mut toks:Vec<&str> = strtok.split(':').collect();
                if toks[0]=="_" {toks[0] = "_WILDCARD_TOKEN_";}
		match self.Symhash.get(toks[0]) {
		   None => {eprintln!("Unrecognized grammar symbol '{}', line {} of grammar",toks[0],linenum); return false; },
		   Some(symi) => {
                     let sym = &self.Symbols[*symi];
                     if self.Errsym.len()>0 && &sym.sym == &self.Errsym {
                       if !seenerrsym { seenerrsym = true; }
                       else { eprintln!("Error symbol {} can only appear once in a production, line {}",&self.Errsym,linenum); return false; }
                     }
                     if !sym.terminal && seenerrsym {
                       eprintln!("Only terminal symbols may follow the error recovery symbol {}, line {}",&self.Errsym, linenum); return false;
                     }
		     let mut newsym = sym.clone();
                     if newsym.rusttype.len()<1 && !self.genabsyn {newsym.rusttype = self.Absyntype.clone();}

                     if toks.len()>1 && toks[1].trim().len()==0 {
                       eprintln!("WARNING: EMPTY LABEL FOR {}, LINE {}; remove whitespaces between ':' and the label\n",toks[0],linenum);
                     }
                     else
		     if toks.len()>1 && toks[1].trim().len()>0 { //label exists
		       let mut label = String::new();
		       
		       if let Some(atindex) = toks[1].find('@') { //if-let pattern
			 label.push_str(toks[1]);
			 while !label.ends_with('@') && i<bstokens.len()
			 { // i indexes all tokens split by whitespaces
			    label.push(' '); label.push_str(bstokens[i]); i+=1;
			 }
			 if !label.ends_with('@') { eprintln!("pattern labels must be closed with @, line {}",linenum); return false;}			 
		       } // if-let pattern
                       else { label = toks[1].trim().to_string(); }
		       newsym.setlabel(label.trim_end_matches('@'));
	             }//label exists
			
                     if prec_level(maxprec).abs() < prec_level(newsym.precedence).abs()  { maxprec=newsym.precedence; }
		     rhsyms.push(newsym);
                   },
                }//match
	      } // while there are tokens on rhs

              ///// at this point, we can transform grammar to apply delays
              let ruleindex = self.Rules.len();
              if markers.len()%2==1 {eprintln!("ERROR: DELAY MARKERS MUST COME IN PAIRS, LINE {}\n",linenum); return false;}
              else if markers.len()>=2 {
                self.delaymarkers.insert(ruleindex,BTreeSet::new());
              }
              let mut i = 0;
              while i+1<markers.len()
              {
                 let dbegin = markers[i];
                 let dend = markers[i+1];
                 i += 2;
                 if dend>dbegin+1 {
                   self.delaymarkers.get_mut(&ruleindex).unwrap().insert((dbegin,dend));
                 }
              }// while there are delay transformations to record
              ///// delays

	      // form rule
	      //let symind2 = *self.Symhash.get(LHS).unwrap(); //reborrowed
              let mut newlhs = self.Symbols[symind2].clone(); //lhsym.clone();
	      if findcsplit.len()>1 {newlhs.label = findcsplit[1].to_owned();}
              //if newlhs.rusttype.len()<1 && !self.genabsyn {newlhs.rusttype = self.Absyntype.clone();}
              if manual_precedence!=0 {maxprec=manual_precedence;} //0.2.97
	      let rule = Grule {
	        lhs : newlhs,
		rhs : rhsyms,
		action: semaction.to_owned(),
		precedence : maxprec,
	      };
	      if self.tracelev>3 {printrule(&rule,self.Rules.len());}
	      self.Rules.push(rule);
              // Add rules to Rulesfor map
              if let None = self.Rulesfor.get(&symind2) { //symind2 is LHS
                 self.Rulesfor.insert(symind2,HashSet::new());
              }
              let rulesforset = self.Rulesfor.get_mut(&symind2).unwrap();
              rulesforset.insert(self.Rules.len()-1);
            //} 
            } // for rul
            }, 
            _ => {eprintln!("ERROR parsing grammar on line {}, unexpected declaration at grammar stage {}",linenum,stage); return false;},  
         }//match first word
       }// not an empty or comment line
     } // while !atEOF

     self.ntcxmax = ntcx;

     // at the very end, add start, eof symbols, startrule
     if self.Symhash.contains_key("START") || self.Symhash.contains_key("EOF") || self.Symhash.contains_key("ANY_ERROR")
     {
        eprintln!("Error in grammar: START and EOF are reserved symbols");
        return false;
     }
     // add start,eof and starting rule:
     let mut startnt = Gsym::new("START",false);
     let mut eofterm = Gsym::new("EOF",true);
     if self.genabsyn || !self.sametype {
       startnt.rusttype="()".to_owned(); // this doesn't matter
     }
     else {startnt.rusttype = self.Absyntype.clone();}
     if self.genabsyn || !self.sametype {eofterm.rusttype = "()".to_owned();}
     else {eofterm.rusttype = self.Absyntype.clone();}
     let mut wildcard = Gsym::new("_WILDCARD_TOKEN_",true);
//     let anyerr = Gsym::new("ANY_ERROR",true);
     startnt.index = self.Symbols.len();
     eofterm.index = self.Symbols.len()+1;
     self.startnti = startnt.index;
     self.eoftermi = eofterm.index;
     self.Symhash.insert(String::from("START"),self.startnti);
     self.Symhash.insert(String::from("EOF"),self.eoftermi);
//   self.Symhash.insert(String::from("ANY_ERROR"),self.Symbols.len()+3);
     self.Symbols.push(startnt.clone());
     self.Symbols.push(eofterm.clone());
//     self.Symbols.push(anyerr.clone());     
     let topgsym = &self.Symbols[*self.Symhash.get(&self.topsym).expect("GRAMMAR START SYMBOL (topsym) NOT DECLARED")];
     let startrule = Grule {  // START-->topsym EOF
        lhs:startnt,
        rhs:vec![topgsym.clone()], //,eofterm],  //eofterm is lookahead
        action: String::default(),
        precedence : DEFAULTPRECEDENCE,
//        iprecedence : 0,
     };
     self.Rules.push(startrule);  // last rule is start rule
     self.startrulei = self.Rules.len()-1;
     let mut startrfset = HashSet::new();
     startrfset.insert(self.Rules.len()-1); // last rule is start rule
     self.Rulesfor.insert(self.startnti,startrfset); //for START
//     if self.tracelev>0 {println!("{} rules in grammar",self.Rules.len());}
     if self.Externtype.len()<1 {self.Externtype = self.Absyntype.clone();}
     // compute sametype value (default true)
     if &topgsym.rusttype!=&self.Absyntype && topgsym.rusttype.len()>0 {
        eprintln!("\nWARNING: THE TYPE FOR THE START SYMBOL ({}) IS NOT THE SAME AS THE VALUETYPE ({}",&topgsym.rusttype,&self.Absyntype);
        self.Absyntype = topgsym.rusttype.clone();
     }
     /*
     for ri in 1..self.Symbols.len() // exclude Symbols[0] for wildcard
     {
        if &self.Symbols[ri].rusttype!=&self.Absyntype {
//	  println!("NOT SAME TYPE: {} for {} and {}",&self.Symbols[ri].rusttype,&self.Symbols[ri].sym,&self.Absyntype);	
          self.sametype = false;
        }
     }//compute sametype
     */
     // reset wildcard type if sametype on all other symbols
     if self.sametype && !self.genabsyn {self.Symbols[0].rusttype = self.Absyntype.clone();}
     if !self.genabsyn {self.enumhash.insert(self.Absyntype.clone(),0);} // 0 reserved

     // compute reachability relation.
     self.reachability(); // in ast_writer module
     let startreach = self.Reachable.get(&(self.Symbols.len()-2)).unwrap();
     // final integrity checks
     for sym in &self.Symbols {
        // skip wildcard, START and EOF
        if sym.index>0 && sym.index<self.Symbols.len()-2 && !startreach.contains(&sym.index) {
          eprintln!("WARNING: The symbol {} is not reachable from the grammar's start symbol.\n",&sym.sym);
        }
        if !sym.terminal {
          if let Some(rset) = self.Rulesfor.get(&sym.index) {
            if rset.len()<1 {
             eprintln!("WARNING: The symbol {}, which was declared non-terminal, does not occur on the left-hand side of any production rule.\n",&sym.sym);
            }
          } else {
            self.Rulesfor.insert(sym.index,HashSet::new()); 
          }
        }   // nonterminal actually used on left side
     }
     //integrity checks
     if self.tracelev>0 {println!("{} rules in grammar",self.Rules.len());}
     true
  }//parse_grammar
}// impl Grammar
// last rule is always start rule and first state is start state


//////////////////////  Nullable

//// also sets the RulesFor map for easy lookup of all the rules for
//// a non-terminal  **no-longer done!
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
             if gs.terminal || !self.Nullable.contains(&gs.sym)
             {addornot=false; break;}
          } // for each rhs symbol
	  if (addornot) {
             changed = self.Nullable.insert(rule.lhs.sym.clone()) || changed;
             //if TRACE>3 {println!("{} added to Nullable",rule.lhs.sym);}
          }
/*
          // add rule index to Rulesfor map:
          if let None = self.Rulesfor.get(&rule.lhs.sym) {
             self.Rulesfor.insert(rule.lhs.sym.clone(),HashSet::new());
          }
          let ruleset = self.Rulesfor.get_mut(&rule.lhs.sym).unwrap();
          ruleset.insert(rulei);
*/
          rulei += 1;
       } // for each rule
     } //while changed
  }//nullable

  // calculate the First set of each non-terminal
// with interior mutability, no need to clone HashSets. // USE THIS ONE!
  pub fn compute_FirstIM(&mut self)
  {
     let mut FIRST:HashMap<usize,RefCell<HashSet<usize>>> = HashMap::new();
     let mut changed = true;
     while changed 
     {
       changed = false;
       for rule in &self.Rules
       {
         let nti = rule.lhs.index; // left symbol of rule is non-terminal
	 if !FIRST.contains_key(&nti) {
            changed = true;
	    FIRST.insert(nti,RefCell::new(HashSet::new()));
         } // make sure set exists for this non-term
	 let mut Firstnt = FIRST.get(&nti).unwrap().borrow_mut();
	 // now look at rhs
	 let mut i = 0;
	 let mut isnullable = true;
 	 while i< rule.rhs.len() && isnullable
         {
            let gs = &rule.rhs[i]; // rhs grammar symbol
	    if gs.terminal {
	      changed=Firstnt.insert(gs.index) || changed;
              isnullable = false;
            }
            else if gs.index!=nti {   // non-terminal
              if let Some(firstgs) = FIRST.get(&gs.index) {
                  let firstgsb = firstgs.borrow();
                  for symi in firstgsb.iter() {
                    changed=Firstnt.insert(*symi) || changed;
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
          self.First.insert(*nt,rcell.take());
        }
     }
  }//compute_FirstIM


  // First set of a sequence of symbols
  pub fn Firstseq(&self, Gs:&[Gsym], la:usize) -> HashSet<usize>
  {
     let mut Fseq = HashSet::new();
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
     if nullable {Fseq.insert(la);}
     Fseq
  }//FirstSeqb

// procedure to generate lexical scanner from lexname, lexval and lexattribute
// declarations in the grammar file.  Added for Version 0.2.3.  This procedure
// is only used by other modules internally
pub fn genlexer(&self,fd:&mut File, fraw:&str) -> Result<(),std::io::Error>
{
    ////// WRITE LEXER
      let ref absyn = self.Absyntype;
      let ref extype = self.Externtype;
      let ltopt = if self.lifetime.len()>0 {format!("<{}>",&self.lifetime)}
          else {String::new()};
      let retenum = format!("RetTypeEnum{}",&ltopt);
      let retype = if self.sametype {absyn} else {&retenum};
      let lifetime = if (self.lifetime.len()>0) {&self.lifetime} else {"'t"};
      write!(fd,"\n// Lexical Scanner using RawToken and StrTokenizer\n")?;
      let lexername = format!("{}lexer",&self.name);
      let mut keywords:HashSet<&str> = HashSet::new();
      let mut singles:Vec<char> = Vec::new();
      let mut doubles:Vec<&str> = Vec::new();
      let mut triples:Vec<&str> = Vec::new();
      // collect symbols from grammar
      for symbol in &self.Symbols
      {
        if !symbol.terminal {continue;}
        if is_alphanum(&symbol.sym) && &symbol.sym!="EOF" && &symbol.sym!="ANY_ERROR" && !self.Haslexval.contains(&symbol.sym) {
	   keywords.insert(&symbol.sym);
	}
	else if symbol.sym.len()==1 && !is_alphanum(&symbol.sym) {
	   singles.push(symbol.sym.chars().next().unwrap());
	}
	else if symbol.sym.len()==2 && !is_alphanum(&symbol.sym) {
	   doubles.push(&symbol.sym);
	}
	else if symbol.sym.len()==3 && !is_alphanum(&symbol.sym) {
	   triples.push(&symbol.sym);
	}	
      }//for each symbol
      for (sym,symmap) in self.Lexnames.iter()
      {
        if is_alphanum(sym) {
	  keywords.remove(&symmap[..]);
	  keywords.insert(sym);
	  continue;
	}
	if sym.len()==1 {
	   singles.push(sym.chars().next().unwrap());
	}
	else if sym.len()==2 {
	   doubles.push(&sym);
	}
	else if sym.len()==3 {
	   triples.push(&sym);
	}      	
      }// for symbols in lexnames such as "||" --> OROR

      write!(fd,"pub struct {0}<{2}> {{
   stk: StrTokenizer<{2}>,
   keywords: HashSet<&'static str>,
   lexnames: HashMap<&'static str,&'static str>,
   shared_state: Rc<RefCell<{1}>>,
}}
impl<{2}> {0}<{2}> 
{{
  pub fn from_str(s:&{2} str) -> {0}<{2}>  {{
    Self::new(StrTokenizer::from_str(s))
  }}
  pub fn from_source(s:&{2} LexSource<{2}>) -> {0}<{2}>  {{
    Self::new(StrTokenizer::from_source(s))
  }}
  pub fn new(mut stk:StrTokenizer<{2}>) -> {0}<{2}> {{
    let mut lexnames = HashMap::with_capacity(64);
    let mut keywords = HashSet::with_capacity(64);
    let shared_state = Rc::new(RefCell::new(<{1}>::default()));
    for kw in [",&lexername,extype,lifetime)?; // end of write

      for kw in &keywords {write!(fd,"\"{}\",",kw)?;}
      write!(fd,"] {{keywords.insert(kw);}}
    for c in [")?;
      for c in singles {write!(fd,"'{}',",c)?;}
      write!(fd,"] {{stk.add_single(c);}}
    for d in [")?;
      for d in doubles {write!(fd,"\"{}\",",d)?;}
      write!(fd,"] {{stk.add_double(d);}}
    for d in [")?;
      for d in triples {write!(fd,"\"{}\",",d)?;}
      write!(fd,"] {{stk.add_triple(d);}}
    for (k,v) in [")?;
      for (kl,vl) in &self.Lexnames {write!(fd,"(r\"{}\",\"{}\"),",kl,vl)?;}
      write!(fd,"] {{lexnames.insert(k,v);}}\n")?;
    for attr in &self.Lexextras {write!(fd,"    stk.{};\n",attr.trim())?;}
      write!(fd,"    {} {{stk,keywords,lexnames,shared_state}}\n  }}\n}}\n",&lexername)?;
      // end of impl lexername
      write!(fd,"impl<{0}> Tokenizer<{0},{1}> for {2}<{0}>
{{
   fn nextsym(&mut self) -> Option<TerminalToken<{0},{1}>> {{
",lifetime,retype,&lexername)?;

      for (condition,action) in self.Lexconditionals.iter() {
        write!(fd,"    if (self.{}) {{ self.stk.{} }}\n",condition,action)?;
      }

      write!(fd,"    let tokopt = self.stk.next_token();
    if let None = tokopt {{return None;}}
    let token = tokopt.unwrap();
    match token.0 {{
")?;

// write skip_trigger cases
    
// change sym to r#sym
    if keywords.len()>0 {
      write!(fd,"      RawToken::Alphanum(sym) if self.keywords.contains(sym) => {{
        let truesym = self.lexnames.get(sym).unwrap_or(&sym);
        Some(TerminalToken::{}(token,truesym,<{}>::default()))
      }},\n",fraw,retype)?;
    }//if keywords.len()>0
      // write special alphanums first - others might be "var" form
      // next - write the Lexvals hexmap int -> (Num(n),Val(n))
      for (tname,raw,val) in &self.Lexvals //tname is terminal name
      {
        let mut Finalval = val.clone();
        if !self.sametype /*&& fraw=="from_raw"*/ {
          let emsg = format!("FATAL ERROR: '{}' IS NOT A SYMBOL IN THIS GRAMMAR",tname);
          let symi = *self.Symhash.get(tname).expect(&emsg);
          let ttype = &self.Symbols[symi].rusttype;
          let ei = self.enumhash.get(ttype).expect("FATAL ERROR: GRAMMAR CORRUPTED");
          Finalval = format!("RetTypeEnum::Enumvariant_{}({})",ei,val);
        }
        write!(fd,"      RawToken::{} => Some(TerminalToken::{}(token,\"{}\",{})),\n",raw,fraw,tname,&Finalval)?;
      }

      write!(fd,"      RawToken::Symbol(s) if self.lexnames.contains_key(s) => {{
        let tname = self.lexnames.get(s).unwrap();
        Some(TerminalToken::{}(token,tname,<{}>::default()))
      }},\n",fraw,retype)?;
      
      write!(fd,"      RawToken::Symbol(s) => Some(TerminalToken::{}(token,s,<{}>::default())),\n",fraw,retype)?;
      write!(fd,"      RawToken::Alphanum(s) => Some(TerminalToken::{}(token,s,<{}>::default())),\n",fraw,retype)?;      
      write!(fd,"      _ => Some(TerminalToken::{}(token,\"<LexicalError>\",<{}>::default())),\n    }}\n  }}",fraw,retype)?;
      write!(fd,"
   fn linenum(&self) -> usize {{self.stk.line()}}
   fn column(&self) -> usize {{self.stk.column()}}
   fn position(&self) -> usize {{self.stk.current_position()}}
   fn current_line(&self) -> &str {{self.stk.current_line()}}
   fn get_line(&self,i:usize) -> Option<&str> {{self.stk.get_line(i)}}
   fn get_slice(&self,s:usize,l:usize) -> &str {{self.stk.get_slice(s,l)}}")?;
   if (!self.sametype) || self.genabsyn {
//      let ttlt = if self.lifetime.len()>0 {&self.lifetime} else {"'wclt"};
//      let ltparam = if self.lifetime.len()>0 {""} else {"<'wclt>"};
      write!(fd,"
   fn transform_wildcard(&self,t:TerminalToken<{},{}>) -> TerminalToken<{},{}> {{ TerminalToken::new(t.sym,RetTypeEnum::Enumvariant_2((self.stk.previous_position(),self.stk.current_position())),t.line,t.column) }}",lifetime,retype,lifetime,retype)?;
   }
   write!(fd,"
}}//impl Tokenizer
\n")?;
      Ok(())
}//genlexer


// generates the enum type unifying absyntype. - if !self.sametype
pub fn gen_enum(&self,fd:&mut File) -> Result<(),std::io::Error>
{
    let ref absyn = self.Absyntype;
//println!("enumhash for absyn {} is {:?}",absyn,self.enumhash.get(absyn));
    let ref extype = self.Externtype;
    let ref lifetime = self.lifetime;
    let has_lt = lifetime.len()>0 && (absyn.contains(lifetime) || extype.contains(lifetime) || absyn=="LBox<dyn Any>");
    let ltopt = if has_lt {format!("<{}>",lifetime)} else {String::from("")};
    //enum name is Retenumgrammarname, variant is _grammarname_enum_{n}
    let enumname = format!("RetTypeEnum{}",&ltopt);  // will be pub
    let symlen = self.Symbols.len();
    write!(fd,"\n//Enum for return values \npub enum {} {{\n",&enumname)?;

    for (typesym,eindex) in self.enumhash.iter()
    {
       write!(fd,"  Enumvariant_{}({}),\n",eindex,typesym)?;
       //println!("  Enumvariant_{}({}),\n",eindex,typesym);
    }
    write!(fd,"}}\n")?;
    write!(fd,"impl{} Default for {} {{ fn default()->Self {{RetTypeEnum::Enumvariant_0(<{}>::default())}} }}\n\n",&ltopt,&enumname,&self.Absyntype)?;
    Ok(())
}// generate enum from rusttype defs RetTypeEnum::Enumvariant_0 is absyntype

}//impl Grammar continued


pub fn checkboxlabel(s:&str) -> &str
{
    if s.starts_with('[') && s.ends_with(']') {s[1..s.len()-1].trim()} else {s}
}// check if label is of form [x], returns x, or s if not of this form.

// used by genlexer routines
pub fn is_alphanum(x:&str) -> bool
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


// find | symbol, ignore enclosing {}'s
fn findskip(s:&str, key:char) -> Option<usize>
{
   let mut i = 0;
   let mut cx:i32 = 0;
   for c in s.chars()
   {
      match c {
        x if x==key && cx==0 => {return Some(i); },
	'{' => {cx+=1;},
	'}' => {cx-=1;},
	_ => {},
      }//match
      i += 1;
   }//for
   return None;
}//findskip

// find matching right to left, with initial counter cx, returns indices or
// (0,0)
fn findmatch(s:&str, left:char, right:char) -> (usize,usize)
{
   let mut ax = (0,0);
   let mut index:usize = 0;
   let mut foundstart=false;
   let mut cx = 0;
   for c in s.chars()
   {
      if c==left {
        cx+=1;
        if !foundstart { ax=(index,0); foundstart=true; }
      }
      else if c==right {cx-=1;}
      if cx==0 && foundstart {
         ax=(ax.0,index);
         return ax;
      }
      index+=1;
   }
   ax
}//findmatch

// calculate real precedence level as a signed value: always use this
// function to extract true precedence level. call .abs() to get nonneg val
pub fn nonassoc(lev:i32)-> bool { lev<NONASSOCBIT }
pub fn leftassoc(lev:i32)->bool { lev>0 }
pub fn rightassoc(lev:i32) -> bool {lev<0 && lev>NONASSOCBIT }
pub fn make_nonassoc(lev:i32) -> i32 { NONASSOCBIT-lev}
pub fn prec_level(lev:i32) -> i32
{
   if lev<NONASSOCBIT {-1*(lev-NONASSOCBIT)} else {lev}
}//prec_level

