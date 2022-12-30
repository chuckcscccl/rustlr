#![allow(unused_imports)]
#![allow(dead_code)]
extern crate rustlr;
use rustlr::{StrTokenizer,LexSource};

/*
mod l7c_ast;
mod l7cparser;
use l7cparser::*;
mod typing;
mod l7cc_ansic;
*/

mod bump7c_ast;
//mod bump7c_ast_nonrc; // required by all but l7cc_rc
mod bump7cparser;
use bump7cparser::*;

mod btyping;
mod btypingso;

mod llvmir;

mod l7cc_llvm; 
mod l7cc_ssa;
mod l7cc_so;
//mod l7cc_rc;

//use l7cc_llvm::*;
//use l7cc_ssa::*;
use l7cc_so::*;
//use l7cc_rc::*;


mod prefix_printer;
use prefix_printer::prefixseq;

fn main()
{

  let args:Vec<String> = std::env::args().collect(); // command-line args
  let mut srcfile = "";
  if args.len()>1 {srcfile = &args[1];}
  let source = LexSource::with_bump(srcfile).unwrap();
  let mut scanner4 = bump7clexer::from_source(&source);
  let mut parser4 = make_parser();
  let result4 = parse_with(&mut parser4, &mut scanner4);
  let absyntree4 = result4.unwrap_or_else(|x|{println!("Parsing Errors Encountered"); x});

  if args.len()>2 && &args[2]=="prefix" {
    let outs = prefixseq(&absyntree4);
    println!("{}",outs);
    return;
  }//prefix print

  println!(";  /* ===== type checking ===== */");

  //let mut symbol_table = SymbolTable::new();
  //symbol_table.check_sequence(&absyntree4);
  
  let mut llvmcompiler = LLVMCompiler::new_skeleton(srcfile);
  //llvmcompiler.symbol_table = symbol_table;
  llvmcompiler.bumpopt = source.get_bump();
  let output = llvmcompiler.compile_program(&absyntree4);

  println!("{}",&output);
}//main



/*  //non-bump
  let source = LexSource::new(srcfile).unwrap();
  let mut scanner3 = l7clexer::from_source(&source);
  let mut parser3 = make_parser();
  let result3 = parse_with(&mut parser3, &mut scanner3);
  let absyntree3 = result3.unwrap_or_else(|x|{println!("Parsing Errors Encountered"); x});
  println!("/*abstract syntax tree after parse: {:?}\n",absyntree3);

  println!("===== type unification test =====");
  typing::unifytest();

  println!("===== type checking =====");

  let mut compiler = l7cc_ansic::CCompiler::new();
  compiler.symbol_table.check_sequence(&absyntree3);

  println!("======== code generation (compile to ANSI C) ========\n*/");
  let program = compiler.compile_program(&absyntree3);
  println!("{}",&program);
*/

