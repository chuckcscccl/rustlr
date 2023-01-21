extern crate rustlr;
//mod test1parser;
mod calc1parser;
use calc1parser::*;
mod calc1_ast;

mod calc2parser;
mod calc2_ast;

fn main()
{

  let mut input = "5+2*3";
  let args:Vec<String> = std::env::args().collect(); // command-line args
  if args.len()>1 {input = &args[1]; }
/*  
  let mut parser1 = test1parser::make_parser();
  let mut tokenizer1 = test1parser::test1lexer::from_str(input);
  let result = parser1.parse(&mut tokenizer1);
  println!("result after parsing {}: {}",input,result);
*/
  let mut parser1 = make_parser();
  let mut tokenizer1 = calc1lexer::from_str(input);
  let result = parse_with(&mut parser1, &mut tokenizer1)
               .unwrap_or_else(|x|x);
  println!("result after parsing {}: {}",input,result);

  let mut parser2 = calc2parser::make_parser();
  let mut tokenizer2 = calc2parser::calc2lexer::from_str(input);
  let res2 = calc2parser::parse_with(&mut parser2, &mut tokenizer2);
  println!("\nAST: {:?}",res2);

}//main
