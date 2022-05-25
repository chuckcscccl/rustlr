#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
#![allow(unused_doc_comments)]
#![allow(unused_imports)]
use std::io::{self,Read,Write,BufReader,BufRead};
use std::collections::HashMap;
use crate::Grammar;

// auto-generate abstract syntax

// prepare Grammar - after parse_grammar first creates grammar
impl Grammar
{
   pub fn prepare(&mut self) -> String
   {
     let mut ASTS = String::new();
     let ltopt = if self.lifetime.len()>0 {format!("<{}>",&self.lifetime)}
          else {String::new()};
     // self.Rulesfor hashmap from nonterminals to set of usize indices

     // construct type of ast for each non-terminal NT
     let mut ntcx=0;
     for NT in self.Rulesfor.keys()  // for each non-terminal
     {
        let nti = *self.Symhash.get(NT).unwrap();
	if self.Symbols[nti].rusttype.len()<3 {
	  self.Symbols[nti].rusttype = format!("{}{}",NT,&ltopt);
	}
        self.enumhash.insert(self.Symbols[nti].rusttype.clone(), ntcx);
        ntcx += 1;
     }
     for (NT,NTrules) in self.Rulesfor.iter()
     {
        let nti = *self.Symhash.get(NT).unwrap();
        let ntsym = &self.Symbols[nti];
	ASTS.push_str(&format!("pub enum {} {{\n",&ntsym.rusttype));
	for ri in NTrules  // for each rule with NU on lhs
	{
	  self.Rules[*ri].lhs.rusttype = self.Symbols[nti].rusttype.clone();
	  // look at rhs of rule to form enum variant + action of each rule
	  if self.Rules[*ri].lhs.label.len()<1 {
	     self.Rules[*ri].lhs.label = format!("{}_{}",NT,ri);
	  }
	  let mut ACTION = format!("{}::{}",NT,&self.Rules[*ri].lhs.label);
	  let mut enumvar = self.Rules[*ri].lhs.label.clone();
	  if self.Rules[*ri].rhs.len()>0 {
	    enumvar.push('(');
	    ACTION.push('(');
	  }
	  let mut rhsi = 0; // right-side index
	  for rsym in self.Rules[*ri].rhs.iter_mut()
	  {
	    let rsymi = *self.Symhash.get(&rsym.sym).unwrap(); //symbol index
	    if !self.Symbols[rsymi].terminal {
	       rsym.rusttype = self.Symbols[rsymi].rusttype.clone();
	       enumvar.push_str(&format!("Box<{}>,",&rsym.rusttype));
	       ACTION.push_str(&format!("parser.lbx({},_item{}_),",&rhsi, &rhsi));
	    }
	    else if &self.Symbols[rsymi].rusttype!="()" {
	      //rsym.rusttype = self.Symbols[rsymi].rusttype.clone();
	      enumvar.push_str(&format!("{},",&rsym.rusttype));
	      ACTION.push_str(&format!("_item{}_,",&rhsi));
	    }
	    rhsi += 1;
	  }// for each symbol on rhs of rule ri
          if enumvar.ends_with(',') {
	      enumvar.pop(); 
	      enumvar.push(')');
	      ACTION.pop();
	      ACTION.push(')');
	  } else if enumvar.ends_with('(') {
	    enumvar.pop();
	    ACTION.pop();
	  }
	  ASTS.push_str(&enumvar); ASTS.push_str(",\n");
	  ACTION.push_str(" }");
	  if self.Rules[*ri].action.len()<=1 {
  	    self.Rules[*ri].action = ACTION;
	  }
println!("Action for rule {}: {}",ri,&self.Rules[*ri].action);
	}// for each rule ri of non-terminal NT
	ASTS.push_str(&format!("{}_Nothing,\n}}\n",NT));
	ASTS.push_str(&format!("impl{} Default for {} {{ fn default()->Self {{ {}_Nothing }} }}\n\n",&ltopt,&ntsym.rusttype,NT));
     }//for each non-terminal and set of rules

     // set Absyntype
     let topi = self.Symhash.get(&self.topsym).unwrap();
     self.Absyntype = self.Symbols[*topi].rusttype.clone();
println!("\n AST generated:\n\n{}",&ASTS);     
     ASTS
   }//prepare_gram
}//impl Grammar