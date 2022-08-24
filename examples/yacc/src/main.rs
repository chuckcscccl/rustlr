#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(dead_code)]
use rustlr::{LexSource,Tokenizer};
mod yacc_ast;
use yacc_ast::*;
mod yaccparser;
use yaccparser::*;
use std::collections::HashMap;

fn main()
{
  let args:Vec<String> = std::env::args().collect();
  let mut srcfile = "test1.y";
  if args.len()>1 {srcfile = &args[1];}
  let sourceopt = LexSource::new(srcfile);
  if sourceopt.is_err() {return;}
  let source = sourceopt.unwrap();

   let mut scanner4 = yaccparser::yacclexer::from_source(&source);
   let mut parser4 = yaccparser::make_parser();
   let tree4= yaccparser::parse_with(&mut parser4, &mut scanner4);
   let result4 = tree4.unwrap_or_else(|x|{println!("Parsing errors encountered; results are partial.."); x});
//   println!("\nABSYN: {:?}\n",&result4);
//   println!("\nSymbol Table: {:?}\n",parser4.shared_state.borrow());
   if parser4.error_occurred() {println!("\n Parser Errors Encountered.. check above");}

   let symboltable = parser4.shared_state.take();
   let rrgmr = build_rr(&result4,&symboltable);
   println!("{}",&rrgmr);
   
}//main

///// building rustlr grammar
use yacc_ast::yacc_decl::*;
use yacc_ast::rhs_symbol::*;
use yacc_ast::label::*;

// ignore all raw C code and all semantic actions as they are meaningless
// in rust anyway.  Only extract and translate the pure grammar.

fn build_rr<'t>(yygmr:&Yacc<'t>, symtab:&symbol_table<'t>) -> String
{
  let mut rrgmr = String::from("# Rustlr grammar converted from Yacc\n\n");
  let Yacc(primary{raw_declarations,yacc_declarations,rules},_) = yygmr;

  // write collected lexterminals from symbol table
  // create reverse hashmap from lexforms to names
  let mut lexhash = HashMap::with_capacity(symtab.lexterminals.len());
  let mut ltcx = 0;
  for lterm in symtab.lexterminals.iter() {
    let tname = format!("TERMINAL{}",ltcx);
    rrgmr.push_str(&format!("lexterminal {} {}\n",&tname,lterm));
    lexhash.insert(lterm,tname);
    ltcx+=1;
  }//for lexterminals in symbol table
  // process yacc_declarations for more terminals,
  for decl in yacc_declarations {  //decl is of type Lbox<yacc_decl>
    match &**decl {
      lexterminal(tn,ts) => {
        rrgmr.push_str(&format!("lexterminal {} {}\n",tn,ts));
      },
      terminals(tlist) => {
        rrgmr.push_str("terminals ");
        for lbxterm in tlist.iter() {
          let lower = (**lbxterm); //.to_owned();
//          lower.make_ascii_lowercase();
          rrgmr.push_str(lower); rrgmr.push(' ');
        }
        rrgmr.push('\n');
      },
      nonterminal(_, nts) => {
        rrgmr.push_str("nonterminals ");
        for lbxnt in nts { rrgmr.push_str(**lbxnt); rrgmr.push(' ');}
        rrgmr.push('\n');  
      },
      // topsym placed in symbol table by metaparser
      _ => {},
    }//match decl
  }//for each yacc_declaration
  // add nonterminals from symbol table, found on the fly by metaparser
  if symtab.nonterminals.len()>0 {
    rrgmr.push_str("nonterminals ");
    for nt in symtab.nonterminals.iter()
      { rrgmr.push_str(*nt); rrgmr.push(' ');}
    rrgmr.push('\n');
  }//symbol table nonterminals

  let mut startsymbol = symtab.topsym;
  if symtab.topsym.len()==0 {
    startsymbol=symtab.nonterminals.iter().next().expect("THIS GRAMMAR DOES NOT HAVE A NON-TERMINAL SYMBOL THAT CAN SERVE AS START SYMBOL");
  }
  rrgmr.push_str(&format!("startsymbol {}\n\n",startsymbol));

  // now for rules:
  for rule in rules {  // rule is LBox<grammar_rules>
    rrgmr.push_str(&format!("{} ==>\n",rule.lhs));
      let mut rhscount = 0;
      for rhside in &rule.rhsides {  //LBox<rhs>
        if rhscount>0 && rhscount<rule.rhsides.len() {
          rrgmr.push_str("        | ");
        } else {rrgmr.push_str("          ");}
        let rhs(rsymunits,_) = &**rhside;
        for rsymu in rsymunits {
          let rhsunit(_,rsym) = &**rsymu;
          match rsym {  //rsym is a rhs_symbol enum
            ID(name,nlabel) => {
              rrgmr.push_str(name);
              nlabel.as_deref().map(|lab|{rrgmr.push_str(&format!(":{}",&getlabel(lab)));});
            },
            LEXCHAR(n) | LEXSTR(n) => {
              let nname = lexhash.get(n).expect("UNEXPECTED ERROR: Grammar's Symbol Table Corrupted");
              rrgmr.push_str(nname);
            },
            _ => {},
          }//match
          rrgmr.push(' ');
        } //for each rsymunit
        rhscount+=1;
        rrgmr.push('\n');
      }//for each rhs of a rule
      rrgmr.push_str("        <==\n");
  }//for each set of rules for a nonterminal
  rrgmr.push_str("\nEOF\n");
  
  rrgmr
}//build_rr from yy


// decipher label
fn getlabel(lab:&label) -> String
{
  match lab {
    simple(n) => String::from(*n),
    boxed(n) => format!("[{}]",n),
    parened(ns) => {
      let mut vs =String::new();
      for nv in ns {
        vs.push_str(&format!("{},",**nv));
      }
      format!("({})",&vs)
    },
    _ => String::new(),
  }
}//getlabel
