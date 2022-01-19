#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_imports)]
extern crate rustlr;
use rustlr::*;
use std::fmt::Display;
use std::any::Any;
mod exprtrees;
use crate::exprtrees::*;

mod lbacalcparser;
use crate::lbacalcparser::*;

fn main()
{
  println!(" testing online calculator with ambiguous grammar ... ");
  let args:Vec<String> = std::env::args().collect(); // command-line args
  let mut input =
"-5-(4-2)*5;
#3-;   # syntax (parsing) error
5-7- -9 ; 
4*3-9; 
2+1/(2-1-1);  # division by 0 (semantic) error
let x = 10 in 2+x;
let x = 1 in (x+ (let x=10 in x+x) + x);
(let x = 2 in x+x) + x;  # unbound variable (semantic) error
(let x = 4 in x/2) + (let x=10 in x*x);
";
  //if args.len()>1 {input = &args[1];}
  println!("------------LBA---------------------------\n");
  let mut stk2 = StrTokenizer::from_str(input);
  stk2.set_line_comment("#");
  let mut scanner2 = Zcscannerlba::new(stk2);
  let mut parser3 = create_parser();
  //let result = parser3.parse(&mut scanner2);  
  let result = parser3.parse_train(&mut scanner2,"src/lbacalcparser.rs");
  let abtree2 = Expr::Seq(lbget!(result,Vec<LBox<Expr>>));
  let bindings = newenv();
   println!("LBA expression tree after parse: {:?}",abtree2);
   if !parser3.error_occurred() {
     println!("Final result after evaluation: {:?}", eval(&bindings,&abtree2));
   } else {
     println!("Parser error, best effort after recovery: {:?}", eval(&bindings,&abtree2));
   }

}//main

fn testing()
{
  let mut stk2 = StrTokenizer::from_str("abc 123");
  let mut scanner2 = Zcscannerlba::new(stk2);

  let aa:&mut (dyn Any+'static) = &mut scanner2;
  let ab = Box::new(aa);
  //print!("{}",ab); type is Any+'static
}