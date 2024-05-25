#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(dead_code)]
mod json_ast;
use json_ast::*;
mod jsonparser;
use jsonparser::*;
use rustlr::Tokenizer;

fn main() {
    let src = rustlr::LexSource::new("person.json").expect("input not found");
    let mut scanner4 = jsonparser::jsonlexer::from_source(&src);
    let mut parser4 = jsonparser::make_parser(scanner4);
    parser4.set_err_report(true);
    //let tree4= jsonparser::parse_train_with(&mut parser4,"src/jsonparser.rs");
    let tree4 = jsonparser::parse_with(&mut parser4);
    let result4 = tree4.unwrap_or_else(|x| {
        println!("ERROR REPORT:\n{}",parser4.get_err_report());
        println!("Parsing errors encountered; results are partial..");
        x
    });

    println!("\nABSYN: {:?}\n", &result4);
} //main

