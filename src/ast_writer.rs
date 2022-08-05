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
use crate::{Grammar,is_alphanum,checkboxlabel};
//use crate::parser_writer::checkboxlabel;

// auto-generate abstract syntax

// prepare Grammar - after parse_grammar first creates grammar
impl Grammar
{
   fn prepare(&mut self) -> String
   {
     // reachability already called by grammar parser
     
     // assign types to all non-terminal symbols
     // first pass: assign types to "" types, skip all others
     let mut ntcx = self.ntcxmax+1;
     for nt in self.Rulesfor.keys() { // for each nonterminal index
       if self.Symbols[*nt].rusttype.len()==0 { // type "" means generate type
         // determine if lifetime needed.
         let reach = self.Reachable.get(nt).unwrap();
         let mut needlt = false;
         if self.lifetime.len()>0 {
           for ti in self.haslt_base.iter() {
             if reach.contains(ti) {needlt = true; break;}
           }
         }//if lifetime check needed
         if needlt {
           self.Symbols[*nt].rusttype = format!("{}<{}>",&self.Symbols[*nt].sym,&self.lifetime);
         } else {
           self.Symbols[*nt].rusttype = self.Symbols[*nt].sym.clone();
         }//don't need lt
         self.enumhash.insert(self.Symbols[*nt].rusttype.clone(),ntcx);
         ntcx+=1;
       }//need type assignment during first pass
     }// first pass

     // Set of nti that will extend other types
     let mut toextend = HashMap::new();  // usize->usize nti's

     //// second pass: change @EXPR to actual type, change :Expr to direct
     for nt in self.Rulesfor.keys() {
       // two possibilities : @expr, or <@expr> or :Expr
       // assume only one.
       let addtoextend = self.Symbols[*nt].rusttype.starts_with(':');
       let mut addtosymhash = false; // because already added above
       let mut limit = self.Symbols.len()+1;
       let mut indirect = true;
       while (indirect || self.Symbols[*nt].rusttype.contains('@')) && limit>0
       {
         indirect = false;
         addtosymhash = true;
         let stype = &self.Symbols[*nt].rusttype; //reborrow
         let mut symtocopy = ""; // symbol to copy type from
         let (mut start,mut end) = (0,stype.len());
         if stype.starts_with(':') || stype.starts_with('@') {
           symtocopy = stype[1..].trim();
         } else if let Some(pos1)=stype.find("<@") {
           if let Some(pos2)=stype[pos1+2..].find('>') {
              symtocopy = &stype[pos1+2..pos1+2+pos2];
              start = pos1+1; end = pos1+2+pos2;
           }
         } else if let Some(pos1)=stype.find("<:") {
           if let Some(pos2)=stype[pos1+2..].find('>') {
              symtocopy = &stype[pos1+2..pos1+2+pos2];
              start = pos1+1; end = pos1+2+pos2;
              indirect = true; // make sure              
           }
         }         
         if symtocopy.len()>0 {
           let symi = *self.Symhash.get(symtocopy).unwrap();
           let mut replacetype = self.Symbols[symi].rusttype.clone();
           if replacetype.starts_with(':') {indirect = true;}
           else if addtoextend {
              toextend.insert(*nt,symi);
//println!("{} will extend {}",&self.Symbols[*nt].sym,&self.Symbols[symi].sym);              
           }

           // change type to actual type.
           
           let mut newtype = stype.clone();
           newtype.replace_range(start..end,&replacetype);
           self.Symbols[*nt].rusttype = newtype;
         } // if symtocopy.len>0
         limit -= 1;
       }//while still contains @ - keep doing it
       if addtosymhash && limit>0 {self.enumhash.insert(self.Symbols[*nt].rusttype.clone(),ntcx); ntcx+=1;}
       else if limit==0 {
          eprintln!("CIRCULARITY DETECTED IN PROCESSING TYPE DEPENDENCIES (type {} for nonterminal {}). THIS TYPE WILL BE RESET AND REGENERATED",&self.Symbols[*nt].rusttype,&self.Symbols[*nt].sym);
          self.Symbols[*nt].rusttype = String::new();
       }
     }//second pass
     
     // third pass on all instances of symbols:
     // don't need to reclone types ! - will never look at instance type
     // final pass sets enumhash
     self.ntcxmax = ntcx;
     // grammar_processor also needs to set enumhash if not -auto

     // setup hashmap from nt numbers to ASTS
     let mut ASTmap:HashMap<usize,String> = HashMap::new();
     let mut ASTS = String::from("\n"); // all asts

     let ltopt = if self.lifetime.len()>0 {format!("<{}>",&self.lifetime)}
          else {String::new()};
          
     //main loop: for each nt and its rules
     for (nt,NTrules) in self.Rulesfor.iter() // for each nt and its rules
     {
        let nti = *nt; //*self.Symhash.get(NT).unwrap();
        let mut ntsym = &self.Symbols[nti];
        let willextend = toextend.contains_key(nt);
//println!("NT {}, type {}, willextend {}",&ntsym.sym,&ntsym.rusttype,willextend);

/* BUT SEMANTIC ACTIONS ARE ALSO SKIPPED ?
        if !ntsym.rusttype.starts_with(&ntsym.sym) && !willextend  {
println!("SKIPPING nt {}, type {}",&ntsym.sym, &ntsym.rusttype);        
continue;}
*/

        // default for new enum
	let mut AST = format!("#[derive(Debug)]\npub enum {} {{\n",&ntsym.rusttype);

/*
// check that the type should append an existing enum, continue...
//        if ntsym.rusttype.starts_with(':') {
        if willextend {
          let copysym = ntsym.rusttype.clone();
println!("setting type of {} to {}, was {}",&ntsym.sym,&copysym,&self.Symbols[nti].rusttype);          
          self.Symbols[nti].rusttype = copysym;
          AST = String::new();
        }// append existing enum - which enum???
*/
//        ntsym = &self.Symbols[nti];   // re-borrow
        if willextend {AST=String::new();}
        let NT = &self.Symbols[nti].sym;

        /////// generate new ENUM by default
        let mut genstruct = NTrules.len()==1 && !willextend;  //check lhs label
        let mut simplestruct = false;

	for ri in NTrules  // for each rule with NT on lhs
	{
          //if self.Rules[*ri].rhs.len()<1 {genstruct=false;}
	  //self.Rules[*ri].lhs.rusttype = self.Symbols[nti].rusttype.clone();
	  // look at rhs of rule to form enum variant + action of each rule
          let mut nolhslabel = false;
	  if self.Rules[*ri].lhs.label.len()>0 { genstruct = false; }
	  else { // make up lhs label
             nolhslabel = true;
	     let mut lhslab = format!("{}_{}",NT,ri);
	     if self.Rules[*ri].rhs.len()>0 && self.Rules[*ri].rhs[0].terminal {
	       let symname = &self.Rules[*ri].rhs[0].sym;
	       if is_alphanum(symname) { //insert r# into enum variant name
	         lhslab = symname.clone();
		 if self.Rules[*ri].rhs.len()>1 /*|| self.Rules[*ri].rhs[0].gettype()!="()"*/ { lhslab.push_str(&format!("_{}",ri)); }
	       }
	     }  // determine enum variant name based on 1st rhs symbol
	     self.Rules[*ri].lhs.label = lhslab;
	  } // set lhs label

          // determine if simplestruct can be used
//          genstruct = genstruct && self.Rules[*ri].lhs.label.len()==0;
//println!("***genstruct for lhs {}, label {}: {}",&self.Rules[*ri].lhs.sym,&self.Rules[*ri].lhs.label,genstruct);
          if genstruct {
            simplestruct = true;
             for rs in &self.Rules[*ri].rhs {
               if rs.label.len()>0 && !rs.label.starts_with("_item") {
                 simplestruct = false; break;
               }
              //if rs.rusttype.contains(&ltopt) || rs.rusttype.contains(&format!("&{}",&self.lifetime))  {usedlt=true;}
             }
            if simplestruct {AST = format!("#[derive(Default,Debug)]\npub struct {}(",&ntsym.rusttype);}
            else  {AST=format!("#[derive(Default,Debug)]\npub struct {} {{\n",&ntsym.rusttype);}
          } //if genstruct, determine if it's a simple struct, change AST
          
          let lhsi = self.Rules[*ri].lhs.index; //copy before mut borrow
	  let lhsymtype = self.Symbols[lhsi].rusttype.clone();
//	  let mut ACTION = format!("{}::{}",NT,&self.Rules[*ri].lhs.label);
          let enumname = &self.Symbols[*toextend.get(nt).unwrap_or(nt)].sym;
	  let mut ACTION = format!("{}::{}",enumname,&self.Rules[*ri].lhs.label);
	  let mut enumvar = format!("  {}",&self.Rules[*ri].lhs.label);
          if genstruct { // name can only be same as non-terminal
             if simplestruct {ACTION=format!("{}(",NT);}
             else {ACTION=format!("{} {{",NT);}
             enumvar = String::new(); // "enumvar" means "struct-fields"
          }//genstruct
	  else if self.Rules[*ri].rhs.len()>0 {
	    enumvar.push('(');
	    ACTION.push('(');
	  }//enum
	  let mut rhsi = 0; // right-side index
	  let mut passthru:i64 = -1; // index of path-thru NT value
	  for rsym in self.Rules[*ri].rhs.iter_mut()
	  {
	    //let rsymi = rsym.index; //*self.Symhash.get(&rsym.sym).unwrap();
            let expectedlabel = format!("_item{}_",&rhsi);

            // check if item has a symbol of the form [x], which forces an
            // lbox
            let alreadyislbx =
              rsym.label.len()>1 && rsym.label.starts_with('[') && rsym.label.ends_with(']');
	    let itemlabel = if rsym.label.len()>0 && &rsym.label!=&expectedlabel {
            // presence of rhs label also cancels passthru
              passthru=-2; checkboxlabel(&rsym.label).to_owned()
            } else {expectedlabel}; //{format!("_item{}_",&rhsi)};
            if rsym.terminal && rsym.precedence!=0 { passthru = -2; }
            // Lbox or no Lbox:  ***************
            let rsymtype = &self.Symbols[rsym.index].rusttype;
            // check if rsym is non-terminal and reaches lsym
            let lhsreachable = match self.Reachable.get(&rsym.index) {
               None => false,
               Some(rset) => rset.contains(&lhsi),
              };
            if alreadyislbx || (lhsreachable && !nonlbxtype(rsymtype) /* && !self.basictypes.contains(&rsymtype[..]) */) {
              if genstruct {
               if simplestruct {  // action formed here
                enumvar.push_str(&format!("pub LBox<{}>,",rsymtype));
                ACTION.push_str(&format!("parser.lbx({},{}), ",&rhsi, &itemlabel));
               } else {
                 enumvar.push_str(&format!("  pub {}:LBox<{}>,\n",&itemlabel,rsymtype));
                 let semact = if alreadyislbx {format!("{}:{}, ",&itemlabel,&itemlabel)} else {format!("{}:parser.lbx({},{}), ",&itemlabel,&rhsi, &itemlabel)};
                ACTION.push_str(&semact); 
               }//not simplestruct
              } //genstruct
              else { //enum
               enumvar.push_str(&format!("LBox<{}>,",rsymtype));
               let semact = if alreadyislbx {format!("{}, ",&itemlabel)} else {format!("parser.lbx({},{}),",&rhsi, &itemlabel)};
	       ACTION.push_str(&semact);
             }// not genstruct
             if rsymtype==&lhsymtype && passthru==-1 {passthru=rhsi;}
             else {passthru = -2;}
	    } // with Lbox
	    else if rsymtype!="()" || (rsym.label.len()>0 && !rsym.label.starts_with("_item")) {  //no Lbox
//println!("looking at symbol {}, rusttype {}, label {}",&rsym.sym, &rsym.rusttype, &rsym.label);
              if genstruct {
                if simplestruct {
                  enumvar.push_str("pub ");
                  enumvar.push_str(rsymtype);
                  enumvar.push(',');
                } else {
                  enumvar.push_str(&format!("  pub {}:{},\n",&itemlabel,rsymtype));
                }
                ACTION.push_str(&format!("{},",&itemlabel)); //simplestruct too
              }//genstruct
	      else {
                enumvar.push_str(&format!("{},",rsymtype));
	        ACTION.push_str(&format!("{},",&itemlabel));
              }//not genstruct (gen enum)

//if *ri==15 {println!("rule {}, rhs sym {}, type {}, lhs type {}, passthru {}",ri,&rsym.sym,&rsym.rusttype,&lhsymtype,passthru);}
              
              if rsymtype==&lhsymtype && passthru==-1 {passthru=rhsi;}
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
             if simplestruct {
               enumvar.push_str(");\n\n");
               ACTION.push(')');
             } else {
               enumvar.push_str("}\n\n");
               ACTION.push('}');
             }
          }//genstruct
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
          let shouldpush = ntsym.rusttype.starts_with(NT) || toextend.contains_key(nt);
          let mut actbase = augment_action(&self.Rules[*ri].action);
          if !actbase.ends_with('}') && passthru>=0 && nolhslabel {
            self.Rules[*ri].action = format!("{} _item{}_ }}",&actbase,passthru);
//println!("passthru on rule {}, NT {}",ri,&self.Rules[*ri].lhs.sym);
          }
	  else
          if !actbase.ends_with('}') && shouldpush {
  	    self.Rules[*ri].action = format!("{} {}",&actbase,&ACTION);
	    AST.push_str(&enumvar); if !genstruct {AST.push_str(",\n");}
	  }
          else if shouldpush {  // added for 0.2.94
	    AST.push_str(&enumvar); if !genstruct {AST.push_str(",\n");}
          }
//println!("Action for rule {}, NT {}: {}",ri,&self.Rules[*ri].lhs.sym,&self.Rules[*ri].action);
	}// for each rule ri of non-terminal NT

        ////////////////// KEEP ENUM OPEN, INSERT INTO HASHMAP
        
        

        let mut storedAST;
        if willextend {
            let targetnti = toextend.get(&nti).unwrap();
            storedAST = ASTmap.remove(targetnti).unwrap_or(String::new());
            storedAST.push_str(&AST);
            ASTmap.insert(*targetnti,storedAST);            
        }
        else {  // check if something already exist, if so add before it
          storedAST = ASTmap.remove(&nti).unwrap_or(String::new());
          storedAST = format!("{}{}",&AST,&storedAST);
          ASTmap.insert(nti,storedAST);
        }
        

/*
        if !genstruct {  // CLOSE THE ENUM  -DO THIS AT END!
	// coerce Nothing to carry a dummy lifetime if necessary
        
        // rule only added if there's no override
        if ntsym.rusttype.starts_with(NT) { ASTS.push_str(&AST); }
*/
     }//for each non-terminal and set of rules (NT, NTRules)

     // Now close all unclosed enums
     for (nt,ntast) in ASTmap.iter() {
       if !self.Symbols[*nt].rusttype.starts_with(&self.Symbols[*nt].sym) {continue;}
       if ntast.starts_with("#[derive(Debug)]") { // enum
 	let defaultvar = format!("{}_Nothing",&self.Symbols[*nt].sym);
        let mut ast = format!("{}  {},\n}}\n",ntast,&defaultvar);
        
        let uselt = if self.lifetime.len()>0 && self.Symbols[*nt].rusttype.contains(&self.lifetime) {&ltopt} else {""};
	ast.push_str(&format!("impl{} Default for {} {{ fn default()->Self {{ {}::{} }} }}\n\n",uselt,&self.Symbols[*nt].rusttype,&self.Symbols[*nt].sym,&defaultvar));
        ASTS.push_str(&ast);
       } // !genstruct - is enum
       else { ASTS.push_str(ntast); }
     }// closing all enums and add to ASTS (for loop)

     // set Absyntype
     let topi = self.Symhash.get(&self.topsym).unwrap(); // must exist
     self.Absyntype = self.Symbols[*topi].rusttype.clone();
     self.enumhash.insert(self.Absyntype.clone(), 0);
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
extern crate rustlr;
use rustlr::LBox;\n")?;
     if self.Extras.len()>0 {write!(fd,"{}\n",&self.Extras)?;}
     if self.ASTExtras.len()>0 {write!(fd,"\n{}\n",&self.ASTExtras)?;}
     write!(fd,"{}",&ASTS)?;
     println!("Abstract syntax structures created in {}",filename);
     // add the grammar .extras - these will only be placed in parser file
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
     for NT in self.Rulesfor.keys()
     {
       self.Reachable.insert(*NT, HashSet::new());
     } // create map skeletons

     let mut stillopen = true;
     while stillopen {
       stillopen = false;
       for (NT, NTrules) in self.Rulesfor.iter()
       {
        //let iNT = *NT; //self.Symhash.get(NT).unwrap();
        let mut symset = HashSet::new(); // symbols to be added to NT's reach
        for ri in NTrules
        {
           for sym in &self.Rules[*ri].rhs
           {
              let symi = sym.index; //*self.Symhash.get(&sym.sym).unwrap();
              symset.insert(symi);
              if !sym.terminal { // noterminal
                 for nsymi in self.Reachable.get(&symi).unwrap().iter()
                 {
                     symset.insert(*nsymi);
                 }
              }
           } // collect rhs symbols into a set
        }//for ri
        let ireachable = self.Reachable.get_mut(NT).unwrap();
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

  // non-LBox types
fn nonlbxtype(ty:&str) -> bool
  {
     ty=="String" || (ty.starts_with('&') && !ty.contains("mut")) || ty.starts_with("Vec<LBox") || ty.starts_with("LBox") || ty.starts_with("Option<LBox")
  }//nonlbxtype

