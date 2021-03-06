#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(dead_code)]
mod calcauto_ast;
use calcauto_ast::*;
mod calcautoparser;
use calcautoparser::*;
use rustlr::Tokenizer;
use std::rc::Rc;

fn main()
{
  let args:Vec<String> = std::env::args().collect(); // command-line args
  let mut input =
"-5-(4-2)*5;
#3 hello! ;
3(1+2);   # syntax (parsing) error
5%2;      # syntax error (% is not recognized by grammar)
5-7- -9 ; 
4*3-9; 
2+1/(2-1-1);  # division by 0 (semantic) error
let x = 0x0FFFFFFFFFFFFFFFFFFFB in 2+x;
let x = 1 in (x+ (let x=10 in x+x) + x);
(let x = 2 in x+x) + x;  # unbound variable (semantic) error
(let x = 4 in x/2) + (let x=10 in x*(let y=100 in y/x));
";
  if args.len()>1 {input = args[1].as_str();}
   let mut scanner4 = calcautoparser::calcautolexer::from_str(input);
   let mut parser4 = calcautoparser::make_parser();
   //let tree4= calcautoparser::parse_train_with(&mut parser4, &mut scanner4,"src/calcautoparser.rs");
   let tree4= calcautoparser::parse_with(&mut parser4, &mut scanner4);
   let result4 = tree4.unwrap_or_else(|x|{println!("Parsing errors encountered; results are partial.."); x});

   println!("\nABSYN: {:?}\n",&result4);
   
//   let bindings4 = newenv();
//   println!("\nresult after eval: {:?}", eval_seq(&bindings4,&result4));
   println!("\nline 10: {}",scanner4.get_line(10).unwrap());
}//main

/////////// evaluating generated ast
/*
pub enum Env<'t> {
  Nil,
  Cons(&'t str, i64, Rc<Env<'t>>)
}
use crate::Env::*;
pub fn newenv<'t>() -> Rc<Env<'t>>
{ Rc::new(Nil) }
fn push<'t>(var:&'t str, val:i64, env:&Rc<Env<'t>>) -> Rc<Env<'t>>
{ Rc::new(Cons(var,val,Rc::clone(env))) }
fn pop<'t>(env:Rc<Env<'t>>) ->  Rc<Env<'t>> //not used here, just being complete
{
   match &*env {
      Nil => env,
      Cons(x,v,e) => Rc::clone(e),
   }
}//push
fn lookup<'t>(x:&'t str, env:&Rc<Env<'t>>) -> Option<i64>
{
    let mut current = env;
    while let Cons(y,v,e) = &**current {
      if &x==y {return Some(*v);}
      else {current = e;}
    }
    return None;
}//lookup

use crate::Expr::*;
use crate::ExprList::*;

// evaluation/interpretation
pub fn eval<'t>(env:&Rc<Env<'t>>, exp:&Expr<'t>) -> Option<i64>
{
   match exp {
     Var(x) => {
       if let Some(v) = lookup(x,env) {Some(v)}
       else { eprint!("UNBOUND VARIABLE {} ... ",x);  None}
     },
     Val(x) => Some(*x),
     Plus(x,y) => eval(env,x).map(|a|{eval(env,y).map(|b|{a+b})}).flatten(),
     Times(x,y) => eval(env,x).map(|a|{eval(env,y).map(|b|{a*b})}).flatten(),
     Minus(x,y) => eval(env,x).map(|a|{eval(env,y).map(|b|{a-b})}).flatten(),
     Neg(x) => eval(env,x).map(|a|{-1*a}), //no need for bind here    
     Div(x,y) => {
       eval(env,y)
       .map(|yval|{if yval==0 {
          eprint!("Division by zero (expression starting at column {}) on line {} of expression at column {} ... ",y.column(),y.line(),x.column());
	  None
         } else {eval(env,x).map(|xval|{Some(xval/yval)})}
       })
       .flatten().flatten()
     },
     Letexp(x,e,b) => {
       eval(env,e).map(|ve|{
         let newenv = push(x,ve,env);
         eval(&newenv,b) }).flatten()
     }
     _ => None,
   }//match
}//eval

fn eval_seq<'t>(env:&Rc<Env<'t>>, s:&ExprList) -> Option<i64>
{
  match s {
     nil => None,
     cons(car,cdr) => {
       if let Some(val) = eval(env,&**car) {
	   println!("result for line {}: {} ;",car.line(),&val);
	   if let nil = &**cdr {return Some(val);} // return last value
       } else { println!("Error evaluating line {};",car.line());}
       eval_seq(env,&**cdr)
     },
     _ => None,
  }//match
}//eval_seq

*/
