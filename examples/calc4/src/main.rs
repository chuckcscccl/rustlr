#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
//extern crate rustlr;
//use rustlr::*;
mod exprtrees;
use crate::exprtrees::*;
mod calc4parser;
use calc4parser::*;
//mod calc4lexermodel;
//use calc4lexermodel::*;

mod calcenumparser;

fn main()
{
  let args:Vec<String> = std::env::args().collect(); // command-line args
  let mut input =
"-5-(4-2)*5;
#3(1+2);   # syntax (parsing) error
#5%2;      # syntax error (% is not recognized by grammar)
5-7- -9 ; 
4*3-9; 
2+1/(2-1-1);  # division by 0 (semantic) error
let x = 10 in 2+x;
let x = 1 in (x+ (let x=10 in x+x) + x);
(let x = 2 in x+x) + x;  # unbound variable (semantic) error
(let x = 4 in x/2) + (let x=10 in x*(let y=100 in y/x));
";
  if args.len()>1 {input = args[1].as_str();}
  //let stk2 = StrTokenizer::from_str(input);
  //let src = LexSource::new("input1.txt").unwrap();
  //let stk2 = StrTokenizer::from_source(&src);  
//  let mut scanner2 = Calcscanner::new(stk2);
  let mut scanner2 = calc4lexer::from_str(input);
  let mut parser3 = make_parser();
  let result = parser3.parse_train(&mut scanner2,"calc4parser.rs");
  let bindings = newenv();
   println!("Expression tree from parse: {:?}",result);
   println!("---------------------------------------\n");
   if !parser3.error_occurred() {
     println!("Final result after evaluation: {:?}", eval(&bindings,&result));
   } else {
     println!("Parser error, best effort after recovery: {:?}", eval(&bindings,&result));
   }

   println!("========= ENUM ===========");
   let mut scanner4 = calcenumparser::calcenumlexer::from_str(input);
   let mut parser4 = calcenumparser::make_parser();
   let result4= calcenumparser::parse_with(&mut parser4, &mut scanner4);
   let bindings4 = newenv();
   println!("result after eval: {:?}", eval(&bindings4,&result4));

}//main
