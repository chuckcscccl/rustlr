#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
#![allow(unused_doc_comments)]
#![allow(unused_imports)]
use std::str::Chars;
use regex::Regex;
use std::collections::{HashSet,BTreeMap,BTreeSet};
use crate::RawToken::*;
use crate::{LBox,LRc,lbup};
use bumpalo::Bump;

use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
pub enum LToken
{
  /// an unsigned integer, though for convenience it is interpreted as
  /// a signed number.  Negative numbers must be recognized by higher-level
  /// parser.  Both decimal and hexadecimal numbers prefixed by 0x are
  /// recognized.
  #[regex(r"\d+")]
  Uint,
  /// floating point number
  #[regex(r"\d*\x2E\d+([eE][+-]?\d+)?")]
  Float,
  #[regex(r"'.'")]
  Char,
  /// String literal, allows for nested quotes.  **String literals always contain the enclosing double quotes**
  #[regex(r"\x22(.*)\x22")]
  Strlit,
  /// Alphanumeric sequence, staring with an alphabetical character or '_',
  /// and followed by arbitrary numbers of alphabetical, numeric or _.
  #[regex(r"[_a-zA-Z][_\da-zA-Z]*")]
  Alphanum,
  /// non-alphanumeric characters, either identified as triples, doubles,
  /// singles, or unrecognized sequences (refering to length of symbol)
  #[regex(r"[!@#$%\^&*\?\-\+\*/\.,<>=~`';:\|\\]+")]
  Symbol,
  /// newline, returned optionally
  #[regex(r"\n")]
  Newline,
  #[regex(r"[ \t\f]+")]
  Whitespace,
  /// tokenizer error
  #[error]
  LexError,
}//RawToken
