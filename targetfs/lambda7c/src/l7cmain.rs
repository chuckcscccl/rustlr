#![allow(unused_imports)]
#![allow(dead_code)]
extern crate rustlr;
use rustlr::{StrTokenizer,LexSource};

mod l7c_ast;
mod l7cparser;
use l7cparser::*;

mod typing;
mod l7cc_ansic;


fn main()
{
  let args:Vec<String> = std::env::args().collect(); // command-line args
  let mut srcfile = "";
  if args.len()>1 {srcfile = &args[1];}
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

}//main


