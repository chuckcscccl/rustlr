// Experimental module to implement selML(k,1) parsers introduced roughly by
// Bertsch, Nederhof and Schmitz.

// nonterminals consists of a symbol plus a fixed k-size array of symbols.
// symbol unused represents nothing and allows us to use fixed arrays.

// usize is the type of grammar symbols (as an index)

//pub struct Nonterminal<const k:usize>(usize,[usize;k]);
pub enum GSymbol<const k:usize> {
   Terminal(usize),
   Nonterminal(usize,[usize;k]),
}
// a special usize index, perhaps 0 or usize::MAX, will represent a dummy
// filler so we can have fixed size arrays and const generics.

pub struct Production<const k:usize> {
  pub lhs: GSymbol,
  
}
