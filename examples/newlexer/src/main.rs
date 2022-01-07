use regex::Regex;


fn main()
{
  let re = Regex::new(r"\d{4}-\d{2}-\d{2}\sxyz$").unwrap();
  //println!("{}",re.is_match("abc2014-01-01 xyz"));

  // Regex for common token types

  // decimal unsigned int
  let decuint = Regex::new(r"^\d+$").unwrap();
  // hexadecimal number
  let hexnum = Regex::new(r"^0x[\dABCDEFabcdef]+$").unwrap();
  // string literal
  let strlit = Regex::new(r"^\x22(?s)(.*)\x22$").unwrap();

  println!("{}",decuint.is_match("1023"));
  println!("{}",hexnum.is_match("0x22"));
  println!("{}",hexnum.is_match("0x7Ef6"));
  println!("{}",strlit.is_match("\"abc\""));
  println!("{}",strlit.is_match("\"hi \"ok\" th
ere\""));  
}//main
