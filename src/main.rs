#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_imports)]

#[cfg(feature = "generator")]
fn main()
{
  let args:Vec<String> = std::env::args().collect(); // command-line args
  let res = rustlr::rustle(&args);
  match res {
    Err(s) => { eprintln!("FAILURE: {}",s); },
    Ok(s) => { println!("{}",s);},   // for command-line app only
  }//match
}//main

#[cfg(not(feature = "generator"))]
fn main() {
  println!("the `generator` feature of rustlr is not enabled");
}// alt main
