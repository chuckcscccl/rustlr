//Abstract syntax types generated by rustlr for grammar wc
    
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(dead_code)]
extern crate rustlr;
use rustlr::LBox;

#[derive(Debug)]
pub enum E {
  a_4(i32),
  E_Nothing,
}
impl Default for E { fn default()->Self { E::E_Nothing } }
