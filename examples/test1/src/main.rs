extern crate rustlr;
mod test1parser;

fn main()
{
  let mut input = "5+2*3";
  let args:Vec<String> = std::env::args().collect(); // command-line args
  if args.len()>1 {input = &args[1];}
  let mut parser1 = test1parser::make_parser();
  let mut tokenizer1 = test1parser::test1lexer::from_str(input);
  let result = parser1.parse(&mut tokenizer1);
  println!("result after parsing {}: {}",input,result);  
}//main
