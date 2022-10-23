#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
//extern crate rustlr;
//use rustlr::*;
mod bxprtrees;
use crate::bxprtrees::*;
mod bcalcparser;
use rustlr::{LexSource};

mod bautocalc_ast;
mod bautocalcparser;

fn main()
{
  let args:Vec<_> = std::env::args().collect(); // command-line args
  let mut srcfile = "testinput.txt";
  if args.len()>1 {srcfile = &args[1];}
  let source = LexSource::new(srcfile).expect("cannot open source");
   let mut scanner4 = bcalcparser::bcalclexer::from_source(&source);
   let bump = bumpalo::Bump::new();
   let mut parser4 = bcalcparser::make_parser();
   parser4.exstate.set(&bump);
   let tree4= bcalcparser::parse_with(&mut parser4, &mut scanner4);
   let result4 = tree4.unwrap_or_else(|x|{println!("Parsing errors encountered; results are partial.."); x});
   println!("ast: {:?}",&result4);
   
   let bindings4 = newenv();
   println!("result after eval: {:?}", eval(&bindings4,&result4));

   //let lexer:& dyn Tokenizer<'_,_> = &scanner4;
   //println!("\nline 10: {}",lexer.get_line(10).unwrap());
   //   println!("\nline 10: {}",scanner4.get_line(10).unwrap());
   // interesting: only need to use Tokenizer for it to recognize function,
   // don't need to typecast

   ////////////////////////////////////////////////////////////////
   println!("====== auto-bump ======\n");
   let source2=LexSource::with_bump(srcfile).unwrap();
   let mut scanner5 = bautocalcparser::bautocalclexer::from_source(&source2);   
   let mut parser5 = bautocalcparser::make_parser();
   //bump = bumpalo::Bump::new(); // drops old bump
   //parser5.exstate.set(&bump);
   let tree5= bautocalcparser::parse_with(&mut parser5, &mut scanner5);
   let result5 = tree5.unwrap_or_else(|x|{println!("Parsing errors encountered; results are partial.."); x});
   println!("ast: {:?}",&result5);

   test();
}//main



enum Exp<'t> {
  Var(&'t str),
  Negative(&'t Exp<'t>),
  Plus(&'t Exp<'t>, &'t Exp<'t>),
  Minus(&'t Exp<'t>, &'t Exp<'t>),
}

fn pprint<'t>(expr:&'t Exp<'t>,) {
  use crate::Exp::*;
  match expr {
    Negative(Negative(n)) => pprint(n),
    Negative(Plus(a,b)) => pprint(&Minus(&Negative(a),b)),
    Negative(Minus(a,b)) => pprint(&Plus(&Negative(a),b)),
    Negative(n) => {print!("-"); pprint(n)},
    Plus(a,Negative(b)) => pprint(&Minus(a,b)),
    Minus(a,Negative(b)) => pprint(&Plus(a,b)),
    Minus(a,p@Minus(_,_)) => { pprint(a); print!("-("); pprint(p); print!(")")},
    Minus(a,p@Plus(_,_)) => { pprint(a); print!("+("); pprint(p); print!(")")},
    Plus(a,b) => {pprint(a); print!("+"); pprint(b);},
    Minus(a,b) => {pprint(a); print!("-"); pprint(b)},    
    Var(x) => print!("{}",x),
  }//match expr
}//pprint

fn test() {
  use crate::Exp::*;
  let x = Var("x");
  let y = Var("y");
  let n = Negative(&x);
  let nn = Negative(&n);
  let m = Negative(&y);
  let p = Plus(&nn,&m);
  pprint(&p); // prints (x-y)
  let q2 = Minus(&Var("a"),&Minus(&Var("b"),&Var("c")));
  let q3 = Minus(&Negative(&Var("y")), &Negative(&Negative(&Var("z"))));


let q = Plus(&Negative(&Negative(&Var("x"))),&Negative(&Var("y"))); //x-y


  let bump = bumpalo::Bump::new();
  let q4 = bump.alloc(Negative(bump.alloc(Negative(bump.alloc(Var("z"))))));
  pprint(&q3);
  pprint(&q);

  print!("\n");
  pprint(&q2);

//  let a = Negative(&Negative(&x));
//  let b = Negative(&Var("y"));

//  let p = Plus(&a,&b);
//  pprint(&p);
}
//creates a temporary which is freed while still in use

/*
fn pprint<'t>(expr:&'t Exp<'t>,) {
  use crate::Exp::*;
  match expr {
    Negative(Negative(n)) => pprint(n),
    Plus(a,Negative(b)) => pprint(&Minus(a,b)),
    Minus(a,Negative(b)) => pprint(&Plus(a,b)), 
    Plus(a,b) => {pprint(a); print!(" + "); pprint(b);},
    Minus(a,b) => {pprint(a); print!(" - "); pprint(b)},    
    Negative(n) => {print!("-"); pprint(n)},
    Var(x) => print!("{}",x),
  }//match expr
}//pprint
*/
