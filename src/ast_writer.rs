#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
#![allow(unused_doc_comments)]
#![allow(unused_imports)]
use std::collections::{HashMap,HashSet};
//use std::cell::{RefCell,Ref,RefMut};
use std::io::{self,Read,Write,BufReader,BufRead};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use crate::{Grammar,is_alphanum,emptybox,checkboxexp};

// metaast for asts
// grammar_processor needs to keep set of nti's to have types flattened.
// keep a hashmap from nt names to structs
// structasts:HashMap<usize,(simpletypebool,Vec<(labelString,typename)>)>
// generate all struct types first and store in table,
// then generate enums.   complements toextend.
// How can structs flatten into structs?  By changing the definition
// into structasts.  How to prevent circular flattening? make sure flatten
// target is not reachable from itself using the reachability_type relation
// Howabout need for lbox because of reachability? more lboxes
// ok, not less....

// first bool is simpletype, second bool is flatten-able, i32 is passthru
// String is type rep, fields are rhs index, label, alreadylbox, type
type METAASTTYPE = HashMap<usize,(bool,bool,i32,String,Vec<(usize,String,bool,String)>)>;

// auto-generate abstract syntax
// prepare Grammar - after parse_grammar first creates grammar
impl Grammar
{
   fn prepare(&mut self) -> String
   {
     // reachability already called by grammar parser, call reachability_types:
     // at this point, self.Reachable can be cloned if needs to be preserved
     self.reachability_types();
     
     // assign types to all non-terminal symbols
     // first pass: assign types to "" types, skip all others
     let mut ntcx = self.ntcxmax+1;
     for nt in self.Rulesfor.keys() { // for each nonterminal index
//println!("TYPE FOR {}: {}",&self.Symbols[*nt].sym,&self.Symbols[*nt].rusttype);       
       if self.Symbols[*nt].rusttype.len()==0 { // type "" means generate type
         // determine if lifetime needed.
         let reach = self.Reachable.get(nt).unwrap();
/////
//for r in reach.iter() {println!("{} reaches {}",&self.Symbols[*nt].sym,&self.Symbols[*r].sym);}
/////
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
     let mut extendtargets = HashSet::new();
     
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
              symtocopy = stype[pos1+2..pos1+2+pos2].trim();
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
              extendtargets.insert(symi);
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
     
     // final pass sets enumhash
     self.ntcxmax = ntcx;
     // grammar_processor also needs to set enumhash if not -auto

     ////////////////////////////// struct generation stage
     // third pass: generate structtypes first so they can be flattened,
     // store generated types in metaast map:

     // two mutually recursive types cannot flatten into each other
     let mut flattentypes = self.flattentypes.clone();
     for a in self.flattentypes.iter() {
       let mut acanflatten = true;
       if !flattentypes.contains(a) {continue;}
       for b in self.flattentypes.iter() {
         if a!=b && flattentypes.contains(b) {
            let areach = self.Reachable.get(a).unwrap();
            let breach = self.Reachable.get(b).unwrap();
            if areach.contains(b) && breach.contains(a) {
               flattentypes.remove(a); flattentypes.remove(b);
               eprintln!("WARNING: MUTUALLY RECURSIVE TYPES {} AND {} CANNOT FLATTEN INTO EACHOTHER\n",&self.Symbols[*a].sym,&self.Symbols[*b].sym);
            }
         }
       }
     }// discover mutually recursive flatten types

     let mut structasts = METAASTTYPE::new(); 
     for (nt,NTrules) in self.Rulesfor.iter() {  //first loop
       if NTrules.len()!=1 || extendtargets.contains(nt) || toextend.contains_key(nt) { /*print warning*/ continue;}
       let sri = *NTrules.iter().next().unwrap();
       if self.Rules[sri].lhs.label.len()>0 {continue;}
       let NT = &self.Symbols[*nt].sym;
       let lhsymtype = self.Symbols[*nt].rusttype.clone();         
       if !lhsymtype.starts_with(NT) {continue;}
       let mut canflatten = true;
       /*
       if self.Reachable.get(nt).unwrap().contains(nt) {
         canflatten=false;
         if self.flattentypes.contains(nt) {eprintln!("WARNING: Recursive non-terminals cannot have their ASTs flattened ({})\n",NT);}
       }
       */
       let mut simplestruct = true;
       for rs in &self.Rules[sri].rhs {
         if rs.label.len()>0 && !rs.label.starts_with("_item") && !emptybox(&rs.label)
           { simplestruct = false; break; }
       } //determine if simple struct
       let ntsym = &self.Symbols[*nt];
       let mut vfields = Vec::new(); // metaast vector representing fields
       let mut rhsi = 0; // right-side index
       let mut passthru:i32 = -1; // index of path-thru NT value
       for rsym in self.Rules[sri].rhs.iter_mut() {
         let expectedlabel = format!("_item{}_",&rhsi);
         let alreadyislbx = rsym.label.len()>1 && rsym.label.starts_with('[') && rsym.label.ends_with(']');
         let itemlabel = if rsym.label.len()>0 && &rsym.label!=&expectedlabel && !rsym.label.starts_with('@') {
            // presence of rhs label also cancels passthru
            passthru=-2; checkboxexp(&rsym.label,&expectedlabel).to_owned()
            } else {expectedlabel};
         if rsym.terminal && rsym.precedence!=0 { passthru = -2; }
         let rsymtype = &self.Symbols[rsym.index].rusttype;
         // check if rsym is non-terminal and reaches lsym
         let lhsreachable = match self.Reachable.get(&rsym.index) {
               None => false,
               Some(rset) => rset.contains(nt),
              };
         if alreadyislbx || (lhsreachable && !nonlbxtype(rsymtype) /* && !self.basictypes.contains(&rsymtype[..])*/) {
               if rsymtype==&lhsymtype && passthru==-1 {passthru=rhsi as i32;}
               else {passthru = -2;}
               vfields.push((rhsi,itemlabel.clone(),alreadyislbx,format!("LBox<{}>",rsymtype)));
         }// lbox
         else if rsymtype!="()" || (rsym.label.len()>0 && !rsym.label.starts_with("_item")) {  //no Lbox, and not unit type without label
           vfields.push((rhsi,itemlabel.clone(),alreadyislbx,rsymtype.to_owned()));
           if rsymtype==&lhsymtype && passthru==-1 {passthru=rhsi as i32;}
              else {passthru = -2;}
         } //no Lbox, and not unit type without label
         rhsi+=1;
       } //for each symbol on right in a iter_mut()
       structasts.insert(*nt,(simplestruct,canflatten,passthru,String::new(),vfields));
     }// structs generation loop 1

     // REAL struct generation loop: APPLY FLATTEN, create and set actions
     // -only 1 levels of indirection allowed?
     let mut newsa = HashMap::with_capacity(structasts.len());
     for (nt,(simplestruct,canflatten,passthru,_,vecfields)) in structasts.iter() {
       let sri = *self.Rulesfor.get(nt).unwrap().iter().next().unwrap();  // !
       let NT = &self.Symbols[*nt].sym;
       let lhsymtype = self.Symbols[*nt].rusttype.clone();
       let ntsym = &self.Symbols[*nt];
       let mut SAST = if !simplestruct {format!("#[derive(Default,Debug)]\npub struct {} {{\n",&ntsym.rusttype)}
         else {format!("#[derive(Default,Debug)]\npub struct {}(",&ntsym.rusttype)};  // sets struct header
       let mut fields = String::new();  // like "enumvar in previous version"
       let mut vfields = Vec::new(); // (rhsi,label,type)

       let mut SACTION = if *simplestruct {format!("{}(",NT)}
               else {format!("{} {{",NT)};
               
       let mut viadjust:i32 = 0; //not used (not inc'ed)
       for (rhsi,itemlabel,alreadylbx,rsymtype) in vecfields { //original field
         let rhssymi = self.Rules[sri].rhs[*rhsi].index;
         if rhssymi==*nt {
            eprintln!("WARNING: TYPE {} CANNOT FLATTEN INTO ITSELF\n",&self.Rules[sri].rhs[*rhsi].sym);
         }
         let mut flattened = false;
         if rhssymi!=*nt && flattentypes.contains(&rhssymi) { // maybe able to flatten in
           match structasts.get(&rhssymi) {
             Some((simp,true,pthr,_,flatfields)) => {  //flatten in
               if *pthr<0 && /* flatfields.len()>0 && */ (!simplestruct||*simp) && !self.Rules[sri].rhs[*rhsi].label.starts_with('[') {
                 flattened=true;
                 let mut fi = 0;
                 for (frhsi,flab,albx,ftype) in flatfields {
                   let newlab = format!("{}_{}",itemlabel,flab);
                   let newactionlab = if *simp {format!("{}.{}",itemlabel,fi)}
                       else {format!("{}.{}",itemlabel,flab)};
                   let newindex = rhsi+(viadjust as usize)+fi;
                   if *simplestruct {
                     fields.push_str("pub ");
                     fields.push_str(ftype); fields.push(',');
                   } else {
                     fields.push_str(&format!("  pub {}:{},\n",&newlab,ftype));
                   }
                   let islbxtype = ftype.starts_with("LBox<");
                   if *simplestruct /*&& !islbxtype */{
                     SACTION.push_str(&newactionlab); SACTION.push(',');
                   }
                   /*
                   else if *simplestruct {
                     SACTION.push_str(&format!("parser.lbx({},{}),",newindex,&newactionlab));
                   }
                   */
                   else /* if !simplestruct  && (!islbxtype || *albx) */ {
                    SACTION.push_str(&format!("{}:{}, ",&newlab,&newactionlab));
                     
                   }
                   /*
                   else { // !simplestruct and need to form parser.lbx
                     SACTION.push_str(&format!("{}:parser.lbx({},{}), ",&newlab,newindex,&newactionlab));
                   }
                   */
                   vfields.push((newindex,newlab,*albx,ftype.to_owned()));
                   fi+=1;
                 }//for each field in flatten source
                 //viadjust += (flatfields.len() as i32)-1;
               }//if can flatten
             },
             aaa => { 
               //println!("NOT FLATTENING {}",&self.Symbols[rhssymi].sym);
             }, //no flattening
           }//match
         }//if in flattentypes list
         if !flattened {
           let islbxtype = rsymtype.starts_with("LBox<");
           if  *simplestruct {
             fields.push_str("pub ");
             fields.push_str(rsymtype); fields.push(',');
             if islbxtype {
               SACTION.push_str(&format!("parser.lbx({},{}),",rhsi+(viadjust as usize),itemlabel));
//println!("HEER3 lbx({},{})",(viadjust as usize),itemlabel);               
             } else { SACTION.push_str(itemlabel); SACTION.push(','); }
           } else { // not simpletype
             fields.push_str(&format!("  pub {}:{},\n",itemlabel,rsymtype));
             if !islbxtype || *alreadylbx {
               SACTION.push_str(&format!("{}:{}, ",itemlabel,itemlabel));
             } else {
               SACTION.push_str(&format!("{}:parser.lbx({},{}), ",itemlabel,rhsi+(viadjust as usize),itemlabel));
             }
           }//not simpletype
           vfields.push((rhsi+(viadjust as usize),itemlabel.to_owned(),*alreadylbx,rsymtype.to_owned()));
         }// !flatten
       }//for each original field
       // post actions
       if *simplestruct {
          fields.push_str(");\n\n");  SACTION.push(')');
       } else {
          fields.push_str("}\n\n");   SACTION.push('}');
       }
       SACTION.push_str(" }");
       let mut actbase = augment_action(&self.Rules[sri].action);
       if !actbase.ends_with('}') && *passthru>=0 /* && nolhslabel*/  {
            self.Rules[sri].action = format!("{} _item{}_ }}",&actbase,passthru);
//println!("passthru on rule {}, NT {}",nri,&self.Rules[nri].lhs.sym);
       } else if !actbase.ends_with('}') {
  	    self.Rules[sri].action = format!("{} {}",&actbase,&SACTION);
	    SAST.push_str(&fields);
       }
       else  {SAST.push_str(&fields);}            
       newsa.insert(*nt,(*simplestruct,*canflatten,*passthru,SAST,vfields));
     }// REAL struct generation loop: apply flatten
     structasts = newsa;
     

/////////////////////////////////////// enums generation stage

     // setup hashmap from nt numbers to ASTS
     let mut enumasts:HashMap<usize,String> = HashMap::new();
     let mut ASTS = String::from("\n"); // all asts to be generated

     let ltopt = if self.lifetime.len()>0 {format!("<{}>",&self.lifetime)}
                 else {String::new()};
     let mut groupvariants:HashMap<usize,HashSet<String>> = HashMap::new();
     /*
     for (_,ntd) in toextend.iter() {
       groupvariants.insert(*ntd,HashSet::new());
     }
     */
     //main loop: for each nt and its rules
     for (nt,NTrules) in self.Rulesfor.iter() // for each nt and its rules
     {
        if structasts.contains_key(nt) {continue;}
        let nti = *nt; 
        let mut ntsym = &self.Symbols[nti];
        let willextend = toextend.contains_key(nt);
        // default for new enum
	let mut AST = if willextend {String::new()}
          else {format!("#[derive(Debug)]\npub enum {} {{\n",&ntsym.rusttype)};
        let NT = &self.Symbols[nti].sym;
	let mut targetnt = nti;
	if let Some(ntd) = toextend.get(nt) { targetnt = *ntd;}
	if !groupvariants.contains_key(&targetnt) {
	  groupvariants.insert(targetnt,HashSet::new());
	}
	let groupenums = groupvariants.get_mut(&targetnt).unwrap();
        // group enums are only generated for tuple variants, the presence
        // of any left or right-side label will cancel its generation.
	for ri in NTrules  // for each rule with NT on lhs
	{
          let mut nolhslabel=false;
          let mut groupoper = ""; // variant-group operator, default none
          // groupoper cancelled if there is a lhs label
          if self.Rules[*ri].lhs.label.len()==0 { // make up lhs label
            nolhslabel = true;
            let mut lhslab = format!("{}_{}",NT,ri); // default

            // search for variant-group operator (only if no lhs label)
            if self.vargroupnames.len()>0 {
             for rsym in self.Rules[*ri].rhs.iter() {
              if let Some(gnamei) = self.vargroups.get(&rsym.index) {
                if groupoper.len()==0 { // not yet set 
                  lhslab = self.vargroupnames[*gnamei].clone();
                  groupoper = &self.Symbols[rsym.index].sym;
                }
              }// found variant-group operator (first one taken)
              if rsym.label.len()>0 && !rsym.label.starts_with("_item") {
                groupoper = "";
                lhslab = format!("{}_{}",NT,ri); // default
                break;
              }// group variant canceled
             }// search for variant-group operator
            } // if there are variant groups

            if groupoper.len()==0 && self.Rules[*ri].rhs.len()>0 && self.Rules[*ri].rhs[0].terminal {
	      let symname = &self.Rules[*ri].rhs[0].sym;
	      if is_alphanum(symname) { //insert r# into enum variant name
	        lhslab = symname.clone();
	        if self.Rules[*ri].rhs.len()>1 /*|| self.Rules[*ri].rhs[0].gettype()!="()"*/ { lhslab.push_str(&format!("_{}",ri)); }
	        }
  	    }  // determine enum variant name based on 1st rhs symbol
	    self.Rules[*ri].lhs.label = lhslab;
          } //nolhslabel
          let lhsi = self.Rules[*ri].lhs.index; //copy before mut borrow
	  let lhsymtype = self.Symbols[lhsi].rusttype.clone();
          let enumname = &self.Symbols[*toextend.get(nt).unwrap_or(nt)].sym;

          // enum action prefix
	  let mut ACTION =format!("{}::{}",enumname,&self.Rules[*ri].lhs.label);
          // enumvariant name
	  let mut enumvar = format!("  {}",&self.Rules[*ri].lhs.label);
          

          // determine if tuple variant or struct/named variant
          let mut tuplevariant = true;
          for rs in &self.Rules[*ri].rhs {
            if rs.label.len()>0 && !rs.label.starts_with("_item") && !emptybox(&rs.label)
              { tuplevariant = false; break; }
          } //determine if tuplevariant

          let mut nullenum = false; // enum variant already exists
          
          // form start of enumvariant and action...
	  if self.Rules[*ri].rhs.len()>0 { // rhs exists
            if tuplevariant {
	      enumvar.push('('); ACTION.push('(');
              if groupoper.len()>0 {
                if groupenums.contains(&self.Rules[*ri].lhs.label) {
                  nullenum = true;
                } else {
                  enumvar.push_str("&'static str,");
		  let toinsert = self.Rules[*ri].lhs.label.clone();
println!("inserting {} to groupenums, had len {}",&toinsert, groupenums.len());
                  groupenums.insert(toinsert);
                }
                ACTION.push_str(&format!("\"{}\",",groupoper));
              }
            } else {
              enumvar.push('{'); ACTION.push('{');
            }  // struct variant
	  }//rhsexists
	  let mut rhsi = 0; // right-side index
          let mut viadjust = 0;
	  let mut passthru:i32 = -1; // index of path-thru NT value
	  for rsym in self.Rules[*ri].rhs.iter_mut()
	  {
            let expectedlabel = format!("_item{}_",&rhsi);
            // check if item has a symbol of the form [x], which forces an
            // lbox
            let alreadyislbx =
              rsym.label.len()>1 && rsym.label.starts_with('[') && rsym.label.ends_with(']');
	    let itemlabel = if rsym.label.len()>0 && &rsym.label!=&expectedlabel && !rsym.label.starts_with('@') {
            // presence of rhs label also cancels passthru
              passthru=-2; checkboxexp(&rsym.label,&expectedlabel).to_owned()
            } else {expectedlabel};
            
            if rsym.terminal && rsym.precedence!=0 { passthru = -2; }
            // Lbox or no Lbox:  ***************
            let rsymtype = &self.Symbols[rsym.index].rusttype;
            
            let mut flattened = false;
            if !rsym.terminal && flattentypes.contains(&rsym.index) {
              match structasts.get(&rsym.index) {
               Some((simp,true,pthr,_,flatfields)) => {  //flatten in
                if *pthr<0 && /* flatfields.len()>0 && */ !rsym.label.starts_with('['){
                 flattened=true;
                 let mut fi = 0;
                 for (frhsi,flab,albx,ftype) in flatfields {
                   let newlab = format!("{}_{}",itemlabel,flab);
                   let newactionlab = if *simp {format!("{}.{}",itemlabel,fi)}
                       else {format!("{}.{}",itemlabel,flab)};
                   let newindex = rhsi+viadjust+fi;

                   if tuplevariant {
                     enumvar.push_str(ftype); enumvar.push(',');
                     ACTION.push_str(&newactionlab); ACTION.push(',');
                   } else {
                     enumvar.push_str(&format!("{}:{},",&newlab,ftype));
                     ACTION.push_str(&format!("{}:{},",&newlab,&newactionlab));
                   }//non-tuplevariant
                   
                   fi+=1;
                 }//for each field in flatten source
                 //viadjust += flatfields.len() -1;
                }//if can flatten
               },
               _ => {},
              }//match
              if flattened {rhsi+=1; continue;}
            }// possible to flatten
            // not possible to flatten:

            // check if rsym is non-terminal and reaches lsym
            let lhsreachable = match self.Reachable.get(&rsym.index) {
               None => false,
               Some(rset) => rset.contains(&lhsi),
              };
            if alreadyislbx || (lhsreachable && !nonlbxtype(rsymtype) /* && !self.basictypes.contains(&rsymtype[..]) */) {

              let semact;
              if tuplevariant {
                enumvar.push_str(&format!("LBox<{}>,",rsymtype));
                semact = if alreadyislbx {format!("{},",&itemlabel)} else {format!("parser.lbx({},{}),",&rhsi, &itemlabel)};
              } else {
                enumvar.push_str(&format!("{}:LBox<{}>,",itemlabel,rsymtype));
                semact = if alreadyislbx {format!("{0}:{0},",&itemlabel)} else {format!("{}:parser.lbx({},{}),",&itemlabel,&rhsi, &itemlabel)};                
              } // non-tuple variant
              ACTION.push_str(&semact);
              
               if rsymtype==&lhsymtype && passthru==-1 {passthru=rhsi as i32;}
               else {passthru = -2;}
	    } // with Lbox
	    else if rsymtype!="()" || (rsym.label.len()>0 && !rsym.label.starts_with("_item")) {  //no Lbox
//println!("looking at symbol {}, rusttype {}, label {}",&rsym.sym, &rsym.rusttype, &rsym.label);
              if tuplevariant {
                enumvar.push_str(&format!("{},",rsymtype));
                ACTION.push_str(&format!("{},",&itemlabel));
              } else {
                enumvar.push_str(&format!("{}:{},",&itemlabel,rsymtype));
                ACTION.push_str(&format!("{0}:{0},",&itemlabel));              
              }// non-tuple variant
              
              if rsymtype==&lhsymtype && passthru==-1 {passthru=rhsi as i32;}
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
          if enumvar.ends_with(',') {
	      enumvar.pop(); 
	      if tuplevariant {enumvar.push(')');}
              else {enumvar.push('}');}
	      ACTION.pop();
	      if tuplevariant {ACTION.push(')');}
              else {ACTION.push('}');}
	  } else if enumvar.ends_with('(') || enumvar.ends_with('{') {
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
	    if !nullenum {AST.push_str(&enumvar); AST.push_str(",\n");}
	  }
          else if shouldpush {  // added for 0.2.94
	    if !nullenum {AST.push_str(&enumvar); AST.push_str(",\n");}
          }
//println!("Action for rule {}, NT {}: {}",ri,&self.Rules[*ri].lhs.sym,&self.Rules[*ri].action);
	}// for each rule ri of non-terminal NT

        ////////////////// KEEP ENUM OPEN, INSERT INTO HASHMAP
        
        let mut storedAST;
        if willextend {
            let targetnti = toextend.get(&nti).unwrap();
            storedAST = enumasts.remove(targetnti).unwrap_or(String::new());
            storedAST.push_str(&AST);
            enumasts.insert(*targetnti,storedAST);            
        }
        else {  // check if something already exist, if so add before it
          storedAST = enumasts.remove(&nti).unwrap_or(String::new());
          storedAST = format!("{}{}",&AST,&storedAST);
          enumasts.insert(nti,storedAST);
        }

/*
        if !genstruct {  // CLOSE THE ENUM  -DO THIS AT END!
	// coerce Nothing to carry a dummy lifetime if necessary
        
        // rule only added if there's no override
        if ntsym.rusttype.starts_with(NT) { ASTS.push_str(&AST); }
*/
     }//for each non-terminal and set of rules (NT, NTRules)

     // Now close all unclosed enums
     for (nt,ntast) in enumasts.iter() {
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

     // now add all the generated struct asts
     for (_,(_,_,_,Sast,_)) in structasts.iter() {
       ASTS.push_str(Sast);
     }

     self.sametype = false;
     self.ntcxmax = ntcx;
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
pub use rustlr::LC;
use rustlr::LBox;\n")?;
//     if self.Extras.len()>0 {write!(fd,"{}\n",&self.Extras)?;}
     if self.ASTExtras.len()>0 {write!(fd,"\n{}\n",&self.ASTExtras)?;}
     write!(fd,"{}",&ASTS)?;
     println!("Abstract syntax structures created in {}",filename);
     // add the grammar .extras - these will only be placed in parser file
     self.Extras.push_str("use rustlr::LBox;\n");
     //self.Extras.push_str(&format!("use crate::{}_ast;\n",&self.name));
     self.Extras.push_str(&format!("use crate::{}_ast::*;\n",&self.name));     
     Ok(())
   }//writeabsyn

// NOTE including all of Extras (one big string) might cause repeated
// definitions - best to not include as pubs.

/////  Floyd/Warshall reachability - sort of // new for 0.3.1
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
  }// reachability closure - part 1

  // extend the reachability relation to include type dependencies
  // assumes that reachability has already been called.
  pub fn reachability_types(&mut self)
  {
     let mut needtoclose = false;
     for (NT,NTrules) in self.Rulesfor.iter()
     {
       let mut ntreach = self.Reachable.get_mut(NT).unwrap();
       // seed reachable sets with type dependencies like Term : Expr
       let nttype = &self.Symbols[*NT].rusttype;
       if nttype.starts_with(':') {
         if let Some(othernti)=self.Symhash.get(nttype[1..].trim()) {
	     if ntreach.insert(*othernti) && !needtoclose {needtoclose=true;}
	     let otherreach=self.Reachable.get_mut(othernti).unwrap();
	     if otherreach.insert(*NT) && !needtoclose {needtoclose=true;}
	     // w/r to reachability for ast gen purposes.
	     // nt reaches othernt because because if bnt-->nt bnt must know
	     // that nt can reach othernt to calculate lifetime,etc.
	     // othernt reach nt because, since the cases of nt are included
	     // as cases under type of othernt, it's as if othernt had more
	     // productions.
	 }
       } // if : starts type
     } // create map skeletons (for loop)
     // create closure
     while needtoclose {
       needtoclose = false;
       for NT in self.Rulesfor.keys()
       {
        let ireachable1 = self.Reachable.get(NT).unwrap();	
        let mut symset = HashSet::new(); // symbols to be added to NT's reach
        for ni in ireachable1.iter() { // for next nt that can be reached
          if !self.Symbols[*ni].terminal {
	     let nireachable = self.Reachable.get(ni).unwrap();
	     for nsymi in nireachable.iter() { symset.insert(*nsymi); }
	  }
        }// for each intermediate symbol
        let ireachable = self.Reachable.get_mut(NT).unwrap(); //re-borrow
        for sym in symset
        {
	  if ireachable.insert(sym) && !needtoclose {needtoclose=true;}
        }
       }//(NT,NTrules)
     }//stillopen, needtoclose
  }// reachability closure

/*  COMBINED VERSION
  pub fn reachability(&mut self)
  {
     for NT in self.Rulesfor.keys() {
       self.Reachable.insert(*NT,HashSet::new());
     }
     for (NT,NTrules) in self.Rulesfor.iter()
     {
       let mut ntreach = self.Reachable.get_mut(NT).unwrap();
       // seed reachable sets with type dependencies like Term : Expr
       let nttype = &self.Symbols[*NT].rusttype;
       if nttype.starts_with(':') {
         if let Some(othernti)=self.Symhash.get(nttype[1..].trim()) {
	     ntreach.insert(*othernti);
	     let otherreach=self.Reachable.get_mut(othernti).unwrap();
	     otherreach.insert(*NT);  // nt, othernt should be considered same
	     // w/r to reachability for ast gen purposes.
	     // nt reaches othernt because because if bnt-->nt bnt must know
	     // that nt can reach othernt to calculate lifetime,etc.
	     // othernt reach nt because, since the cases of nt are included
	     // as cases under type of othernt, it's as if othernt had more
	     // productions.
	 }
       }
       ntreach = self.Reachable.get_mut(NT).unwrap();       //re-borrow
       for ri in NTrules // seed based on rhs of rules (just one level)
        {
           for sym in &self.Rules[*ri].rhs
           {
	      ntreach.insert(sym.index);
           } // collect rhs symbols into 1st level reachable set
       }//for ri       
//       self.Reachable.insert(*NT, ntreach);
     } // create map skeletons
     // create closure
     let mut stillopen = true;
     while stillopen {
       stillopen = false;
       for NT in self.Rulesfor.keys()
       {
        let ireachable1 = self.Reachable.get(NT).unwrap();	
        let mut symset = HashSet::new(); // symbols to be added to NT's reach
        for ni in ireachable1.iter() { // for next nt that can be reached
          if !self.Symbols[*ni].terminal {
	     let nireachable = self.Reachable.get(ni).unwrap();
	     for nsymi in nireachable.iter() { symset.insert(*nsymi); }
	  }
        }// for each intermediate symbol
        let ireachable = self.Reachable.get_mut(NT).unwrap(); //re-borrow
        for sym in symset
        {
	  if ireachable.insert(sym) && !stillopen {stillopen=true;}
          //stillopen =  ireachable.insert(sym) || stillopen;
        }
       }//(NT,NTrules)
     }//stillopen
  }// reachability closure  - combined version
*/
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

