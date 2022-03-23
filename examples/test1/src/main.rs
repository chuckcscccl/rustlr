//extern crate rustlr;
mod test1parser;

/*
// lexical scanner (new version that allows references)
struct Scanner<'t>(StrTokenizer<'t>);
impl<'t> Tokenizer<'t,i32> for Scanner<'t>
{
   // this function must any kind of token produced by the lexical scanner
   // into TerminalTokens expected by the parser.  The built-in lexer,
   // StrTokenizer, produces RawTokens.
   fn nextsym(&mut self) -> Option<TerminalToken<'t,i32>>
   {
     let tokopt = self.0.next_token();
     if let None = tokopt {return None;}
     let tok = tokopt.unwrap();
     match tok.0 {
       RawToken::Num(n) => Some(TerminalToken::from_raw(tok,"num",n as i32)),
       RawToken::Symbol(s) => Some(TerminalToken::from_raw(tok,s,0)),
       _ => Some(TerminalToken::from_raw(tok,"<<Lexical Error>>",0)),
     }//match
   }
}
*/

fn main()
{
  let mut input = "5+2*3";
  let args:Vec<String> = std::env::args().collect(); // command-line args
  if args.len()>1 {input = &args[1];}
  let mut parser1 = test1parser::make_parser();
  //let mut tokenizer1 =Scanner(StrTokenizer::from_str(input));
  let mut tokenizer1 = test1parser::test1lexer::from_str(input);
  let result = parser1.parse(&mut tokenizer1);
  println!("result after parsing {}: {}",input,result);  
  
}//main
