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
//use std::cell::{RefCell,Ref,RefMut};
use std::io::{self,Read,Write,BufReader,BufRead};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use crate::{Grammar,is_alphanum};

// auto-generate abstract syntax

// prepare Grammar - after parse_grammar first creates grammar
impl Grammar
{
   fn prepare(&mut self) -> String
   {
     //if self.Reachable.len()<1 {self.reachability();}// compute reachability
/*
println!("reachable: {:?}",&self.Reachable);
print!("SYMBOLS:");
for i in 0..self.Symbols.len() {
println!("{}: {}",i,&self.Symbols[i].sym);
}
*/
     let mut ASTS = String::new(); // all asts
     let ltopt = if self.lifetime.len()>0 {format!("<{}>",&self.lifetime)}
          else {String::new()};
     // self.Rulesfor hashmap from nonterminals to set of usize indices

     // setting of type = NT name done by grammar processor, including ltopt

     for (NT,NTrules) in self.Rulesfor.iter()
     {
        let nti = *self.Symhash.get(NT).unwrap();
        let ntsym = &self.Symbols[nti];

        /////// generate ENUM by default
        let mut genstruct = NTrules.len()==1;
        let mut usedlt = false; // did lt appear in type?
	let mut AST = format!("#[derive(Debug)]\npub enum {} {{\n",&ntsym.rusttype);
        if genstruct {AST=format!("#[derive(Default,Debug)]\npub struct {} {{\n",&ntsym.rusttype);}
        
	for ri in NTrules  // for each rule with NT on lhs
	{
          //if self.Rules[*ri].rhs.len()<1 {genstruct=false;}
	  self.Rules[*ri].lhs.rusttype = self.Symbols[nti].rusttype.clone();
	  // look at rhs of rule to form enum variant + action of each rule
          let mut nolhslabel = false;
	  if self.Rules[*ri].lhs.label.len()<1 { // make up lhs label
             nolhslabel = true;
	     let mut lhslab = format!("{}_{}",NT,ri);
	     if self.Rules[*ri].rhs.len()>0 && self.Rules[*ri].rhs[0].terminal {
	       let symname = &self.Rules[*ri].rhs[0].sym;
	       if is_alphanum(symname) { //insert r# into enum variant name
	         lhslab = symname.clone();
		 if self.Rules[*ri].rhs.len()>1 || self.Rules[*ri].rhs[0].rusttype!="()" { lhslab.push_str(&format!("_{}",ri)); }
	       }
	     }
	     self.Rules[*ri].lhs.label = lhslab;
	  } // set lhs label
	  let lhsymtype = self.Rules[*ri].lhs.rusttype.clone();
	  let mut ACTION = format!("{}::{}",NT,&self.Rules[*ri].lhs.label);
	  let mut enumvar = format!("  {}",&self.Rules[*ri].lhs.label);
          if genstruct {
             ACTION=format!("{} {{",NT);
             enumvar = String::new(); // "enumvar" means "struct-fields"
          }
	  else if self.Rules[*ri].rhs.len()>0 {
	    enumvar.push('(');
	    ACTION.push('(');
	  }
	  let mut rhsi = 0; // right-side index
	  let mut passthru:i64 = -1; // index of path-thru NT value
	  for rsym in self.Rules[*ri].rhs.iter_mut()
	  {
	    let rsymi = *self.Symhash.get(&rsym.sym).unwrap(); //symbol index
            let expectedlabel = format!("_item{}_",&rhsi);
	    let itemlabel = if rsym.label.len()>0 && &rsym.label!=&expectedlabel {
            // presence of rhs label also cancels passthru
              passthru=-2; rsym.label.clone()
            } else {expectedlabel}; //{format!("_item{}_",&rhsi)};
            rsym.rusttype = self.Symbols[rsymi].rusttype.clone();
            if rsym.rusttype.contains(&ltopt) || rsym.rusttype.contains(&format!("&{}",&self.lifetime))  {usedlt=true;}
            if self.Symbols[rsymi].terminal && self.Symbols[rsymi].precedence!=0 { passthru = -2; }
            // Lbox or no Lbox:
            if !self.Symbols[rsymi].terminal && &self.Symbols[rsymi].rusttype!="()" && !self.Symbols[rsymi].rusttype.starts_with("Vec") && !self.Symbols[rsymi].rusttype.starts_with("LBox") && !self.Symbols[rsymi].rusttype.starts_with("Option<LBox") {
              if genstruct {
               enumvar.push_str(&format!("  pub {}:LBox<{}>,\n",&itemlabel,&rsym.rusttype));
               ACTION.push_str(&format!("{}:parser.lbx({},{}), ",&itemlabel,&rhsi, &itemlabel));
              }
              else {
               enumvar.push_str(&format!("LBox<{}>,",&rsym.rusttype));
	       ACTION.push_str(&format!("parser.lbx({},{}),",&rhsi, &itemlabel));
             }// not genstruct
             if &rsym.rusttype==&lhsymtype && passthru==-1 {passthru=rhsi;}
             else {passthru = -2;}
	    } // with Lbox
	    else if &self.Symbols[rsymi].rusttype!="()" {  //no Lbox
              if genstruct {
                enumvar.push_str(&format!("  pub {}:{},\n",&itemlabel,&rsym.rusttype));
                ACTION.push_str(&format!("{},",&itemlabel));
              }
	      else {
                enumvar.push_str(&format!("{},",&rsym.rusttype));
	        ACTION.push_str(&format!("{},",&itemlabel));
              }//not genstruct (gen enum)

//if *ri==15 {println!("rule {}, rhs sym {}, type {}, lhs type {}, passthru {}",ri,&rsym.sym,&rsym.rusttype,&lhsymtype,passthru);}
              
              if &rsym.rusttype==&lhsymtype && passthru==-1 {passthru=rhsi;}
              else {passthru = -2;}
	    }// could still be nonterminal but not unit type - no lbox
	    /*
	    check special case: only one NT on rhs that has same type as lhs,
	    and all other symbols have type () AND are marked punctuations.
	    What is a punctuation?  go by precedence level.
            "paththru" indicates rule like E --> ( E ), where semantic
            action passes thru.  In this case pasthru will be 1.
            passthru = -1 means passthru candidate index not yet found,
            -2 means no passthru candidate exists.
	    */
	    rhsi += 1;
	  }// for each symbol on rhs of rule ri
          if genstruct { // this is only rule that forms struct
             if !usedlt && ltopt.len()>0 {
               enumvar.push_str(&format!("  pub phantom:PhantomData<&{} ()>,\n",&self.lifetime));
               ACTION.push_str("phantom:PhantomData, ");
             }
             enumvar.push_str("}\n");
             ACTION.push('}');
          }
          else
          if enumvar.ends_with(',') {
	      enumvar.pop(); 
	      enumvar.push(')');
	      ACTION.pop();
	      ACTION.push(')');
	  } else if enumvar.ends_with('(') {
	    enumvar.pop();
	    ACTION.pop();
	  }
    	  ACTION.push_str(" }");  // action already has last rbrack
	  // determine if action and ast enum should be generated:
//          if self.Rules[*ri].action.len()<=1 && passthru>=0 && nolhslabel { // special case
          let mut actbase = augment_action(&self.Rules[*ri].action);
          if !actbase.ends_with('}') && passthru>=0 && nolhslabel {
            self.Rules[*ri].action = format!("{} _item{}_ }}",&actbase,passthru);
//println!("passthru on rule {}, NT {}",ri,&self.Rules[*ri].lhs.sym);
          }
	  else
          if !actbase.ends_with('}') && ntsym.rusttype.starts_with(NT) {
  	    self.Rules[*ri].action = format!("{} {}",&actbase,&ACTION);
	    AST.push_str(&enumvar); if !genstruct {AST.push_str(",\n");}
	  }
          else if ntsym.rusttype.starts_with(NT) {  // added for 0.2.94
	    AST.push_str(&enumvar); if !genstruct {AST.push_str(",\n");}
          }
//println!("Action for rule {}, NT {}: {}",ri,&self.Rules[*ri].lhs.sym,&self.Rules[*ri].action);
	}// for each rule ri of non-terminal NT

        if !genstruct {
	// coerce Nothing to carry a dummy lifetime if necessary
	let mut defaultvar = format!("{}_Nothing",NT);
	let mut defaultvarinst = format!("{}_Nothing",NT);
	if self.lifetime.len()>0 {
	  defaultvar = format!("{}_Nothing(&{} ())",NT,&self.lifetime);
	  defaultvarinst = format!("{}_Nothing(&())",NT);
	}
	AST.push_str(&format!("  {},\n}}\n",&defaultvar));
	AST.push_str(&format!("impl{} Default for {} {{ fn default()->Self {{ {}::{} }} }}\n\n",&ltopt,&ntsym.rusttype,NT,&defaultvarinst));
        } // !genstruct
        
        // rule only added if there's no override
        if ntsym.rusttype.starts_with(NT) { ASTS.push_str(&AST); }
     }//for each non-terminal and set of rules (NT, NTRules)

     // set Absyntype
//     let topi = self.Symhash.get(&self.topsym).unwrap(); // must exist
//     self.Absyntype = self.Symbols[*topi].rusttype.clone();
//     self.enumhash.insert(self.Absyntype.clone(), 0);
//println!("\n AST generated:\n\n{}",&ASTS);
     self.sametype = false;
     ASTS
   }//prepare_gram


   pub fn writeabsyn(&mut self, filename:&str) ->Result<(),std::io::Error>
   {
     let ASTS = self.prepare();
     //let filename = format!("{}_ast.rs",&self.name);
     let mut fd = File::create(filename)?;
     write!(fd,"//Abstract syntax types generated by rustlr for grammar {}",&self.name)?;
     write!(fd,"\n    
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(dead_code)]
use std::marker::PhantomData;
extern crate rustlr;
use rustlr::LBox;\n")?;
     write!(fd,"{}\n",&self.Extras)?;
     write!(fd,"{}",&ASTS)?;
     println!("Abstract syntax structures created in {}",filename);
     // add the grammar .extras
     self.Extras.push_str("use rustlr::LBox;\n");
     self.Extras.push_str(&format!("use crate::{}_ast;\n",&self.name));
     self.Extras.push_str(&format!("use crate::{}_ast::*;\n",&self.name));     
     Ok(())
   }//writeabsyn

// NOTE including all of Extras (one big string) might cause repeated
// definitions - best to not include as pubs.

/////  Floyd/Warshall reachability - sort of
  pub fn reachability(&mut self)
  {
     let NTs:Vec<_> = self.Rulesfor.keys().collect();
     for NT in NTs
     {
       self.Reachable.insert(*self.Symhash.get(NT).unwrap(), HashSet::new());
     } // create map skeletons

     let mut stillopen = true;
     while stillopen {
       stillopen = false;
       for (NT, NTrules) in self.Rulesfor.iter()
       {
        let iNT = self.Symhash.get(NT).unwrap();
        let mut symset = HashSet::new();
        for ri in NTrules
        {
           for sym in &self.Rules[*ri].rhs
           {
              let symi = *self.Symhash.get(&sym.sym).unwrap();
              symset.insert(symi);
              if !self.Symbols[symi].terminal { // noterminal
                 for nsymi in self.Reachable.get(&symi).unwrap().iter()
                 {
                     symset.insert(*nsymi);
                 }
              }
           } // collect rhs symbols into a set
        }//for ri
        let ireachable = self.Reachable.get_mut(iNT).unwrap(); // refcell
        for sym in symset
        {
          stillopen =  ireachable.insert(sym) || stillopen;
        }
       }//(NT,NTrules)
     }//stillopen
  }// reachability closure

}//impl Grammar


// function to see if given semantic action should be replaced or augmented
// returns String base of action, not closed with } if need auto generation.
fn augment_action(act0:&str) -> String
{
   let act = act0.trim();
   if act.len()<=1 {return String::new();} // completely regenerate
   let rbpo = act.rfind('}');
   if let Some(rbp) = rbpo {
     let ebpo = act[..rbp].rfind("...");
     if let Some(ebp)=ebpo { 
        let mut act2 = String::from(&act[..ebp]) + " ";
        return act2;
     }
   }
   else {return String::new();} // did not end in }
   return String::from(act);
}
