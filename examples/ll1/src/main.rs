#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(dead_code)]
use rustlr::Tokenizer;
use std::io;

mod ll1calcast;
mod calcparser;
mod calc_ast;

fn main() {
  //  let args:Vec<String> = std::env::args().collect(); // command-line args
    let stdin = io::stdin();
    let mut strbuf = String::new();
    let lines = stdin.lines();
    for ln in lines {let res = ln.map(|x|strbuf.push_str(&x));}
    let mut scanner4 = calcparser::calclexer::from_str(&strbuf);
    let mut parser4 = calcparser::make_parser();
    let tree4 = calcparser::parse_with(&mut parser4, &mut scanner4);
    let result4 = tree4.unwrap_or_else(|x| {
        println!("Parsing errors encountered; results are partial..");
        x
    });

    println!("\nABSYN: {:?}\n", &result4);
} //main

