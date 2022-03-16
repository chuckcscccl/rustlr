//! Bison-like syntax parsing support module
pub enum Directive
{
  Absyntype(String),
  Topsym(String),
  Nonterminal(String),
  Terminal(String),
  Presassoc(String,String,i32),
}

