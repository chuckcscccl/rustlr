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
use std::io::{self,Read,Write,BufReader,BufRead};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use crate::{Grammar,is_alphanum,checkboxlabel};

// FSHARP AST Writer mirroring bumpast_writer.
// Should we have active patterns whenever there are LBoxes?

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
   fn fs_prepare(&mut self) -> String
   {
     // reachability already called by grammar parser, call reachability_types:
     // at this point, self.Reachable can be cloned if needs to be preserved
     self.reachability_types();

     //let ltref = if self.lifetime.len()>0 {format!("&{} ",&self.lifetime)}
     //     else {String::new()};
     //let ltref = String::new(); // just in case
     
     // assign types to all non-terminal symbols
     // first pass: assign types to "" types, skip all others
     let mut ntcx = self.ntcxmax+1;
     for nt in self.Rulesfor.keys() { // for each nonterminal index
       if self.Symbols[*nt].rusttype.len()==0 { // type "" means generate type
         let reach = self.Reachable.get(nt).unwrap();
/////
//for r in reach.iter() {println!("{} reaches {}",&self.Symbols[*nt].sym,&self.Symbols[*r].sym);}
/////

         self.Symbols[*nt].rusttype = self.Symbols[*nt].sym.clone();
         self.enumhash.insert(self.Symbols[*nt].rusttype.clone(),ntcx);
         ntcx+=1;
       }//need type assignment during first pass
       else if &self.Symbols[*nt].rusttype=="()" { // for safety
         self.Symbols[*nt].rusttype="unit".to_owned();
       }
//println!("TYPE OF {} is {}",&self.Symbols[*nt].sym,&self.Symbols[*nt].rusttype);
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
       let simplestruct = false;
       /*
       for rs in &self.Rules[sri].rhs {
         if rs.label.len()>0 && !rs.label.starts_with("_item") 
           { simplestruct = false; break; }
       } //determine if simple struct
       */ // do not use simple records: all must have names
       let ntsym = &self.Symbols[*nt];
       let mut vfields = Vec::new(); // metaast vector representing fields
       let mut rhsi = 0; // right-side index
       let mut passthru:i32 = -1; // index of path-thru NT value
       for rsym in self.Rules[sri].rhs.iter_mut() {
         let expectedlabel = format!("_item{}_",&rhsi);
         let alreadyislc = rsym.label.len()>1 && rsym.label.starts_with('[') && rsym.label.ends_with(']');
         let mut itemlabel = if rsym.label.len()>0 && &rsym.label!=&expectedlabel && !rsym.label.starts_with('@') {
            // presence of rhs label also cancels passthru
            passthru=-2; checkboxlabel(&rsym.label).to_owned()
            } else {expectedlabel};
         if rsym.terminal && rsym.precedence!=0 { passthru = -2; }
         let mut rsymtype = &self.Symbols[rsym.index].rusttype[..];
	 if rsymtype=="()" {rsymtype="unit";}
         // check if rsym is non-terminal and reaches lsym

         if alreadyislc {
               if rsymtype==&lhsymtype && passthru==-1 {passthru=rhsi as i32;}
               else {passthru = -2;}
               vfields.push((rhsi,itemlabel.clone(),alreadyislc,format!("LBox<{}>",rsymtype)));
         }// lbox

         else if rsymtype!="unit" || (rsym.label.len()>0 && !rsym.label.starts_with("_item")) {  //no Lbox, and not unit type without label
           vfields.push((rhsi,itemlabel.clone(),alreadyislc,rsymtype.to_owned()));
           if rsymtype==&lhsymtype && passthru==-1 {passthru=rhsi as i32;}
              else {passthru = -2;}
         } //no Lbox, and not unit type without label
         rhsi+=1;
       } //for each symbol on right in a iter_mut()
       structasts.insert(*nt,(simplestruct,canflatten,passthru,String::new(),vfields));
     }// structs generation loop 1

// got to allow mutual recursive refs with **and**

     // REAL struct generation loop: APPLY FLATTEN, create and set actions
     // -only 1 levels of indirection allowed?
     let mut newsa = HashMap::with_capacity(structasts.len());
     let mut firsttype = false;
     for (nt,(simplestruct,canflatten,passthru,_,vecfields)) in structasts.iter() {
       let sri = *self.Rulesfor.get(nt).unwrap().iter().next().unwrap();
       let NT = &self.Symbols[*nt].sym;
       let lhsymtype = self.Symbols[*nt].rusttype.clone();
       let ntsym = &self.Symbols[*nt];
       // actual string for struct type to be generated:
       let mut SAST;
       if firsttype {
         firsttype=false;
         SAST = format!("//non-simplestruct\ntype {} =\n  {{\n",&ntsym.rusttype);
       }
       else { // not first type - need to add "and"
         SAST = format!("\nand {} =\n  {{\n",&ntsym.rusttype);       
       }

       let mut fields = String::new();  // like "enumvar in previous version"
       let mut vfields = Vec::new(); // (rhsi,label,type)
       // actual semantic action code to be generated
       let mut SACTION = format!(" {{{}.",NT); // { structtype. .. for F#
       let mut viadjust:i32 = 0; //not used (not inc'ed)

       let mut totalitems = 0;
       for (rhsi,itemlabel,alreadylbx,rsymtype) in vecfields {
         let rhssymi = self.Rules[sri].rhs[*rhsi].index;
         if rhssymi==*nt {
            eprintln!("WARNING: TYPE {} CANNOT FLATTEN INTO ITSELF\n",&self.Rules[sri].rhs[*rhsi].sym);
         }

         let lhsreachable = match self.Reachable.get(&rhssymi) {
               None => false,
               Some(rset) => rset.contains(nt),
              };
         let needref = false; //lhsreachable && !nonlctype(&rsymtype) && !self.basictypes.contains(&rsymtype[..]) &&ltref.len()>0;
         let mut flattened = false;
         if rhssymi!=*nt && flattentypes.contains(&rhssymi) {
           match structasts.get(&rhssymi) {
             Some((simp,true,pthr,_,flatfields)) => {  //flatten in
               if *pthr<0 /* && flatfields.len()>0 */ && (!simplestruct||*simp) && !self.Rules[sri].rhs[*rhsi].label.starts_with('[') {
                 flattened=true;
                 let mut fi = 0;
                 for (frhsi,flab,albx,ftype) in flatfields {
                   let newlab = format!("{}_{}",itemlabel,flab);
                   let newactionlab = if *simp {format!("{}.{}",itemlabel,fi)}
                       else {format!("{}.{}",itemlabel,flab)};
                   let newindex = rhsi+(viadjust as usize)+fi;
                   let fltref = ""; //if nonlctype(ftype) || self.basictypes.contains(&ftype[..])  || ltref.len()==0 {""} else {&ltref};
                   fields.push_str(&format!("    mutable {}:{}{};\n",&newlab,fltref,ftype));  // non-simpletype
                   totalitems +=1;
                   let islctype = ftype.starts_with("LBox<") || ftype.starts_with("LC<");
                    SACTION.push_str(&format!("{}={}; ",&newlab,&newactionlab));
                   vfields.push((newindex,newlab,*albx,ftype.to_owned()));
                   fi+=1;
                 }//for each field in flatten source
                 //viadjust += (flatfields.len() as i32)-1;
               }//if can flatten
             },
             aaa => { // println!("def {:?}",aaa); 
             }, //no flattening
           }//match
         }//if in flattentypes list (flatten this rhs symbol)
         if !flattened {
           let islctype = rsymtype.starts_with("LBox<") || rsymtype.starts_with("LC<");
           let withref = ""; //if  needref  ||  islctype {&ltref} else {""}; 
           // not simpletype
           totalitems += 1;
           fields.push_str(&format!("    mutable {}:{};\n",itemlabel,rsymtype));
//           if !islctype || *alreadylbx {
             SACTION.push_str(&format!("{}={}; ",itemlabel,itemlabel));      
//           }
           vfields.push((rhsi+(viadjust as usize),itemlabel.to_owned(),*alreadylbx,rsymtype.to_owned()));
         }// !flatten
       }//for each original field
       // post actions
       fields.push_str("  }\n");   SACTION.push_str("}}");
       let mut actbase = augment_action(&self.Rules[sri].action);
       if !actbase.ends_with("}") && *passthru>=0 /* && nolhslabel*/  {
            self.Rules[sri].action = format!("{} _item{}_ ",&actbase,passthru)
//println!("passthru on rule {}, NT {}",nri,&self.Rules[nri].lhs.sym);
       } else if !actbase.ends_with("}") {
  	    self.Rules[sri].action = format!("{}{}",&actbase,&SACTION);
	    SAST.push_str(&fields);
       }
       else  {SAST.push_str(&fields);}
       // no empty records allowed in F#
       if totalitems==0 {
         if let Some(pos) = SAST.rfind("\n  {\n  }") {
           SAST.replace_range(pos..," unit  //empty record\n");
           SACTION=String::from(" ()");
           if !actbase.ends_with("}") {
             self.Rules[sri].action = format!("{}{}",&actbase,&SACTION);
           }
         }
       }//totalitems=0 (empty record to unit)
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
     //main loop: for each nt and its rules
     for (nt,NTrules) in self.Rulesfor.iter() // for each nt and its rules
     {
        if structasts.contains_key(nt) {continue;}
        let nti = *nt; 
        let mut ntsym = &self.Symbols[nti];
        let willextend = toextend.contains_key(nt);
        // default for new enum
	let mut AST = if willextend {String::new()}
          else if firsttype {
	    firsttype=false;
	    format!("//enum\ntype {} =\n",&ntsym.rusttype)
	  }
	  else {
	    format!("//enum\nand {} =\n",&ntsym.rusttype)	  
	  };
        let NT = &self.Symbols[nti].sym;
	let mut targetnt = nti;
 	if let Some(ntd) = toextend.get(nt) { targetnt = *ntd;}
        let groupenums = groupvariants.entry(targetnt).or_default();
        // group enums are only generated for tuple variants, the presence
        // of any left or right-side label will cancel its generation.
	for ri in NTrules  // for each rule with NT on lhs
	{
          let mut nolhslabel=false;
          let mut groupoper = ""; // variant-group operator, default none
          // groupoper cancelled if there is a lhs label
          if self.Rules[*ri].lhs.label.len()==0 { // make up lhs label
            nolhslabel = true;
            let mut lhslab = format!("{}_{}",NT,ri); //default

            // search for variant-group operator (only if no lhs label)
            if self.vargroupnames.len()>0 {
	     let enti = *toextend.get(&nti).unwrap_or(&nti);
             for rsym in self.Rules[*ri].rhs.iter() {
              if let Some(gnamei) = self.vargroups.get(&(enti,rsym.index)) {
                if groupoper.len()==0 { // not yet set 
                  lhslab = self.vargroupnames[*gnamei].clone();
                  groupoper = &self.Symbols[rsym.index].sym;
                }
              }// found  variant-group operator for this lhs nt
              else if let Some(gnamei) = self.vargroups.get(&(usize::MAX,rsym.index)) {
                if groupoper.len()==0 { // not yet set 
                  lhslab = self.vargroupnames[*gnamei].clone();
                  groupoper = &self.Symbols[rsym.index].sym;
                }
              }// found generic variant-group operator (first one taken)
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
	        if self.Rules[*ri].rhs.len()>1 { lhslab.push_str(&format!("_{}",ri)); }
	      }//is_alphanum
  	    }  // determine enum variant name based on 1st rhs symbol
	    self.Rules[*ri].lhs.label = lhslab;
          } //nolhslabel
	  firstcap(&mut self.Rules[*ri].lhs.label);
          let lhsi = self.Rules[*ri].lhs.index; //copy before mut borrow
	  let lhsymtype = self.Symbols[lhsi].rusttype.clone();
          let enumname = &self.Symbols[*toextend.get(nt).unwrap_or(nt)].sym;
	  let mut ACTION = format!("{}.{}",enumname,&self.Rules[*ri].lhs.label);
          // enumvariant
	  let mut enumvar = format!("  | {}",&self.Rules[*ri].lhs.label);

          // determine if tuple variant or struct/named variant
	  /*
          let mut tuplevariant = true; // stay true
          for rs in &self.Rules[*ri].rhs {
            if rs.label.len()>0 && !rs.label.starts_with("_item") 
              { tuplevariant = false; break; }
          } //determine if tuplevariant
	  */
          let mut nullenum = false; // enum variant already exists
          // form start of enumvariant and action...
	  if self.Rules[*ri].rhs.len()>0 { // rhs exists
	     enumvar.push_str(" of"); ACTION.push('(');
             if groupoper.len()>0 {
                if groupenums.contains(&self.Rules[*ri].lhs.label) {
                  nullenum = true;
                } else {
                  enumvar.push_str(" string *");
                  groupenums.insert(self.Rules[*ri].lhs.label.clone());
                }
                ACTION.push_str(&format!("\"{}\",",groupoper));
             } // group oper exists
	  }//rhsexists
	  let mut rhsi = 0; // right-side index
          let mut viadjust = 0;
	  let mut passthru:i32 = -1; // index of path-thru NT value
	  for rsym in self.Rules[*ri].rhs.iter_mut()
	  {
            let expectedlabel = format!("_item{}_",&rhsi);
            // check if item has a label of the form [x], which forces an
            // lbox
            let alreadyislc =
              rsym.label.len()>1 && rsym.label.starts_with('[') && rsym.label.ends_with(']');
            let nonnamedfield = rsym.label.starts_with("_item") || rsym.label.len()<1;
	    let mut itemlabel = if rsym.label.len()>0 && &rsym.label!=&expectedlabel && !rsym.label.starts_with('@') {
            // presence of rhs label also cancels passthru
              passthru=-2; checkboxlabel(&rsym.label).to_owned()
            } else {expectedlabel};
            
            if rsym.terminal && rsym.precedence!=0 { passthru = -2; }
            // Lbox or no Lbox:  ***************
            let mut rsymtype = &self.Symbols[rsym.index].rusttype[..];
	    if rsymtype=="()" {rsymtype="unit";}
            
            let mut flattened = false;
            if !rsym.terminal && flattentypes.contains(&rsym.index) {
              match structasts.get(&rsym.index) {
               Some((simp,true,pthr,_,flatfields)) => {  //flatten in
                if *pthr<0 /* && flatfields.len()>0 */ && !rsym.label.starts_with('['){
                 flattened=true;
                 let mut fi = 0;
                 for (frhsi,flab,albx,ftype) in flatfields {
                   let newlab = format!("{}_{}",itemlabel,flab);
                   let newactionlab = if *simp {format!("{}.{}",itemlabel,fi)}
                       else {format!("{}.{}",itemlabel,flab)};
                   let newindex = rhsi+viadjust+fi;

                   if nonnamedfield {
		     enumvar.push_str(&format!(" {} *",ftype));
                     ACTION.push_str(&newactionlab); ACTION.push(',');
                   } else {
                     enumvar.push_str(&format!(" {}:{} *",&newlab,ftype));
                     //ACTION.push_str(&format!("{}={},",&newlab,&newactionlab));
		     ACTION.push_str(&format!("{},",&newactionlab));

                   }// named/non-named field
                   
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
            let needref = false; //lhsreachable && !nonlctype(rsymtype);
            let localref = ""; //if needref {&ltref} else {""};
            if alreadyislc /* || (lhsreachable && !nonlctype(rsymtype))*/ {
              let semact;
              if nonnamedfield {
                enumvar.push_str(&format!(" LBox<{}> *",rsymtype));
                semact = format!("{},",&itemlabel);
              } else {
                enumvar.push_str(&format!(" {}:LBox<{}> *",itemlabel,rsymtype));
                //semact = format!("{}={},",&itemlabel,&itemlabel);
		semact = format!("{},",&itemlabel);
              } // non-tuple variant
              ACTION.push_str(&semact);
              
               if rsymtype==&lhsymtype && passthru==-1 {passthru=rhsi as i32;}
               else {passthru = -2;}
	    } // with LBox
	    else if rsymtype!="unit" || (rsym.label.len()>0 && !rsym.label.starts_with("_item")) {  //no Lbox
              if nonnamedfield {
                enumvar.push_str(&format!(" {} *",rsymtype));
                ACTION.push_str(&format!("{},",&itemlabel));
              } else {
                enumvar.push_str(&format!(" {}:{} *",&itemlabel,rsymtype));
                ACTION.push_str(&format!("{},",&itemlabel));          
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
          if enumvar.ends_with('*') {
	      enumvar.pop(); 
	      ACTION.pop();
	      ACTION.push(')');
	  } else if enumvar.ends_with('(') {
	    enumvar.pop();
	    ACTION.pop();
	  }
	  if enumvar.ends_with("of") {
	    enumvar.pop(); enumvar.pop();
	  }
	  if ACTION.ends_with('(')||ACTION.ends_with(',') {ACTION.pop();}
	  if ACTION.ends_with('\"') { // for Binaryop("/" ..
	    ACTION.push(')');
	  }
    	  //ACTION.push_str(" }");  // action already has last rbrack
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
	    if !nullenum {AST.push_str(&enumvar); AST.push_str("\n");}
	  }
          else if shouldpush {  // added for 0.2.94
	    if !nullenum {AST.push_str(&enumvar); AST.push_str("\n");}
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

     }//for each non-terminal and set of rules (NT, NTRules)

     // Now close all unclosed enums
     for (nt,ntast) in enumasts.iter() {
       if !self.Symbols[*nt].rusttype.starts_with(&self.Symbols[*nt].sym) {continue;}
       if ntast.starts_with("//enum") { // enum
 	let defaultvar = format!("  | {}_Nothing",&self.Symbols[*nt].sym);
        let mut ast = format!("{}{}\n",ntast,&defaultvar);
        ASTS.push_str(&ast);
       } // !genstruct - is enum
       else { ASTS.push_str(ntast); }
     }// closing all enums and add to ASTS (for loop)

     // set Absyntype
     self.Absyntype = self.Symbols[self.topsym].rusttype.clone();
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


   pub fn write_fsast(&mut self, filename:&str) ->Result<(),std::io::Error>
   {
     let ASTS = self.fs_prepare();
     let mut firstchar = self.name.chars().next().unwrap();
     firstchar.make_ascii_uppercase();
     let mut fd = File::create(filename)?;
     write!(fd,"//FSharp AST types generated by rustlr for grammar {}",&self.name)?;
     write!(fd,"\nmodule {}{}.AST\n",firstchar,&self.name[1..])?;
     write!(fd,"  
open System;
open Fussless;
open Fussless.RuntimeParser;\n")?;
     if self.ASTExtras.len()>0 {write!(fd,"\n{}\n",&self.ASTExtras)?;}
     write!(fd,"\ntype LC<'T> = LBox<'T> // dummy\n")?;
     write!(fd,"{}",&ASTS)?;
     println!("F# AST types created in {}",filename);
     // add the grammar .extras - these will only be placed in parser file
     self.Extras.push_str(&format!("open {}{}.AST\n",firstchar,&self.name[1..]));
     Ok(())
   }//write_fsast

// NOTE including all of Extras (one big string) might cause repeated
// definitions - best to not include as pubs.

}//impl Grammar


// function to see if given semantic action should be replaced or augmented
// returns String base of action, not closed with } if need auto generation.
// strategy for F#: replace '}' with "(* end *)"
fn augment_action(act0:&str) -> String
{
   let act = act0.trim();
   if act.len()<=1 {return String::new();} // completely regenerate
   if let Some(ebp) = act.rfind("...") {
      let mut act2 = String::from(&act[..ebp]) + "; ";
      return act2;
   }
   return String::from(act); // + " (*end*)"; // means no auto generation
}

  // non-LC types
pub fn nonlctype(ty:&str) -> bool
  {
     ty=="string" || ty.starts_with("Vec<") || ty.starts_with("LBox") || ty.starts_with("option<LBox") || ty.starts_with("LC<") || ty.starts_with("option<LC<")
  //  true
  }//nonlbxtype

fn firstcap(s:&mut String) {
  let mut fc = s.chars().next().unwrap();
  fc.make_ascii_uppercase();
  s.replace_range(0..1,&fc.to_string());
}
