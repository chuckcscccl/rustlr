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

fn main() {
    /*
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
    */

    /*
    let src = rustlr::LexSource::new("input.txt").expect("input not found");
    let mut scanner4 = calcautoparser::calcautolexer::from_source(&src);
    let mut parser4 = calcautoparser::make_parser();
    parser4.set_err_report(true);
    //let tree4= calcautoparser::parse_train_with(&mut parser4, &mut scanner4,"src/calcautoparser.rs");
    let tree4 = calcautoparser::parse_with(&mut parser4, &mut scanner4);
    let result4 = tree4.unwrap_or_else(|x| {
        println!("ERROR REPORT:\n{}",parser4.get_err_report());
        println!("Parsing errors encountered; results are partial..");
        x
    });
    */

    // testing new base_parser
    let src = rustlr::LexSource::new("input.txt").expect("input not found");
    let mut scanner4 = calcautoparser::calcautolexer::from_source(&src);
    let mut parser4 = calcautoparser::make_parser(scanner4);
    parser4.set_err_report(true);
    let tree4 = calcautoparser::parse_with(&mut parser4);
    let result4 = tree4.unwrap_or_else(|x| {
       //println!("\nParsing errors encountered; results are partial..");
       println!("{}",parser4.get_err_report());       
       x
    });
    println!("\nABSYN: {:?}\n", result4);
    //println!("{}",parser4.get_err_report());       


    //eval_seq(&newenv(), &result4, 1); // evaluate each expression in sequence
    //println!("\nline 10: {}",parser4.get_tokenizer().get_line(10).unwrap());
} //main


/*
/////////// evaluating generated ast

pub enum Env<'t> {
    // enviornment for evaluation
    Empty,
    Binding(&'t str, i64, Rc<Env<'t>>),
}
use crate::Env::*;
pub fn newenv<'t>() -> Rc<Env<'t>> {
    Rc::new(Empty)
}
fn push<'t>(var: &'t str, val: i64, env: &Rc<Env<'t>>) -> Rc<Env<'t>> {
    Rc::new(Binding(var, val, Rc::clone(env)))
}
fn lookup<'t>(x: &'t str, env: &Rc<Env<'t>>) -> Option<i64> {
    let mut current = env;
    while let Binding(y, v, e) = &**current {
        if &x == y {
            return Some(*v);
        } else {
            current = e;
        }
    }
    return None;
} //lookup

use Expr::*;
use ExprList::*;

// evaluation
pub fn eval<'t>(env: &Rc<Env<'t>>, exp: &Expr<'t>) -> Option<i64> {
    match exp {
        Var(x) => {
            if let Some(v) = lookup(x, env) {
                Some(v)
            } else {
                eprint!("UNBOUND VARIABLE {} ... ", x);
                None
            }
        }
        Val(x) => Some(*x),
        Plus(x, y) => eval(env, x).map(|a| eval(env, y).map(|b| a + b)).flatten(),
        Minus(x, y) => eval(env, x).zip(eval(env, y)).map(|(a, b)| a - b), //alternative
        //Times(x,y) => eval(env,x).zip_with(eval(env,y),|a,b|{a*b}), //alternative
        Binop("*", x, y) => eval(env, x).map(|a| eval(env, y).map(|b| a * b)).flatten(),

        //Minus(x,y) => eval(env,x).map(|a|{eval(env,y).map(|b|{a-b})}).flatten(),
        Neg(x) => eval(env, x).map(|a| -1 * a),
        Binop("/", x, y) => eval(env, y)
            .map(|yval| {
                if yval == 0 {
                    eprint!(
                        "Division by zero line {}, column {} ... ",
                        y.line(),
                        y.column()
                    );
                    None
                } else {
                    eval(env, x).map(|xval| xval / yval)
                }
            })
            .flatten(),
        Let {
            let_var: x,
            init_value: e,
            let_body: b,
        } => eval(env, e)
            .map(|ve| {
                let newenv = push(x, ve, env);
                eval(&newenv, b)
            })
            .flatten(),
        _ => None,
    } //match
} //eval

fn eval_seq<'t>(env: &Rc<Env<'t>>, s: &ExprList, line: usize) -> Option<i64> {
    /*
      for expr in seq.0.iter() {
        if let Some(val) = eval(env,expr) {
           println!("result for line {}: {} ;",line,&val);
        } else { println!("Error evaluating line {};",line);}
      }
    */

    match s {
        cons { car, cdr } => {
            if let Some(val) = eval(env, car) {
                println!("result for line {}: {} ;", line, &val);
                if let nil = &**cdr {
                    return Some(val);
                } // return last value
            } else {
                println!("Error evaluating line {};", line);
            }
            eval_seq(env, cdr, cdr.line())
        }
        _ => None,
    } //match
} //eval_seq
*/
