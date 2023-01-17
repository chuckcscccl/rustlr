# **[rustlr](https://docs.rs/rustlr/latest/rustlr/index.html)**
**LR-Style Parser Generator**

**A [Tutorial](https://cs.hofstra.edu/~cscccl/rustlr_project/) with several examples is available.**

Besides traditional LR and LALR parser generation, Rustlr supports the following
options

1. An experimental feature that generates parsers for *Selective Marcus-Leermakers* grammars.  This is a larger class of unambiguous grammars than traditional LR and helps to allow new productions to be added to a grammar without
creating conflicts (see the [Appendix](https://cs.hofstra.edu/~cscccl/rustlr_project/appendix.html) of the tutorial).
2. The option of creating the abstract syntax data types and semantic actions from the grammar. Rustlr grammars contain a sub-language that defines how ASTs are
to be generated.  For example, in a grammar with `E --> E + T` a dependency
between `T` and `E` can be declared so that only one AST type is generated for both.
3. Support for choosing [bumpalo](https://docs.rs/bumpalo/latest/bumpalo/index.html) to create recursive ASTs that use references instead of smart pointers: this
enables *deep pattern matching* on recursive structures.
4. Recognizes regex-style operators `*`, `+` and `?`, which simplify
the writing of grammars and allow better ASTs to be created.
5. Generates a lexical scanner automatically from the grammar.
6. Operator precedence and associativity declarations further allow grammars
to be written that's closer to EBNF syntax.
7. The ability to train the parser, interactively or from script, for better error reporting.
8. Generates parsers for Rust [and for F\#](https://github.com/chuckcscccl/Fussless).  Rustlr is designed to promote typed functional programming languages in the creation of compilers and
language-analysis tools.  Parser generation for other such languages will
gradually become available.

<p>


### Quick Example: JSON Parser

The following is a Rustlr grammar:
```
# Rustlr Grammar for JSON
auto
lifetime 'lt
lexterminal LBRACE {
lexterminal RBRACE }
lexterminal LBRACK [
lexterminal RBRACK ]
lexterminal LPAREN (
lexterminal RPAREN )
lexterminal COLON :
lexterminal COMMA ,
lexterminal NULL null
lexterminal MINUS -
valueterminal TRUE~ bool~ Alphanum("true")~ true
valueterminal FALSE~ bool~ Alphanum("false")~ false
valueterminal STRING~ &'lt str~ Strlit(n)~ &n[1..n.len()-1]
valueterminal NUM~ i64~ Num(n)~ n
valueterminal FLOAT~ f64~ Float(n)~ n
valueterminal BIGNUM~ &'lt str~ BigNumber(n)~ n
nonterminal Integer i64
nonterminal Floatpt f64
nonterminal Boolean bool
nonterminals Value KeyValuePair Number
nonterminal List : Value
nonterminal Object HashMap<&'lt str, LBox<@Value>>

startsymbol Value
resync COMMA RBRACK RBRACE

Integer --> MINUS?:m NUM:n {if m.is_some() {n*-1} else {n}}
Floatpt --> MINUS?:m FLOAT:n {if m.is_some() {-1.0*n} else {n}}
Number:Bignum --> MINUS?:m BIGNUM
Number:Int --> Integer
Number:Float --> Floatpt
Boolean --> TRUE | FALSE
Value:Number --> Number
Value:Boolean --> Boolean
Value:Str --> STRING
Value:Objectmap --> Object
Value --> List
Value --> NULL
Value --> LPAREN Value RPAREN
KeyValuePair --> STRING COLON Value
List:List --> LBRACK Value<COMMA*> RBRACK
Object ==> LBRACE KeyValuePair<COMMA*>:entries RBRACE {
  let mut kvmap = HashMap::new();
  for (mut lbx) in entries {
    if let KeyValuePair(k,v) = lbx.take() { kvmap.insert(k,v); }
  }
  kvmap
} <==

# The following line is injected into json_ast.rs
$use std::collections::HashMap;

# The following lines are injected into the parser
!mod json_ast;
!fn main()  {
!  let srcfile = std::env::args().nth(1).unwrap();
!  let source = LexSource::new(&srcfile).unwrap();
!  let mut scanner1 = jsonlexer::from_source(&source);
!  let mut parser1 = make_parser();
!  let parseresult = parse_with(&mut parser1, &mut scanner1);
!  let ast = parseresult.unwrap_or_else(|x|{println!("Parsing errors encountered; results not guaranteed.."); x});
!  println!("\nAST: {:?}\n",&ast);
!}//main
```
In addition to a parser, the grammar generates a lexical scanner from the `lexterminal` and `valueterminal` declarations.  It also created most of the
abstract syntax types and semantic actions required by the parser, alongside
some manual overrides. As this is a quick example, we've also injected a main
function, which expects the name of a json source as argument, directly into
the parser.  To run rustlr on this example,

  1. Install rustlr as a command-line application: **`cargo install rustlr`**
  2. Create a Cargo crate with **`rustlr = "0.4"`** in its dependencies
  3. save the grammar in the crate as `json.grammar` (must have `.grammar`
     suffix).
  4. Run rustlr with **`rustlr json.grammar -o src/main.rs`**
  5. call `cargo run [some json file]`

Note that `<COMMA*>` specifies a comma-separated list and normally generates
semantic actions to create
a vector.  However, for JSON "objects" we chose to create a hashmap
by manually writing the semantic action.  Normally, Rustlr in `auto` mode
generates an enum for each non-terminal symbol that's on the left-hand side
of multiple productions, and a struct for non-terminals with a single
production.  However, declarations such as `nonterminal List : Value` allow
a type to be absorbed into another: there is no separate type for
`List`.  There are other ways that a grammar can specify how ASTs are to be
created, distinguishing the *abstract syntax tree* from the *parse tree*.
Please consult the [tutorial](https://cs.hofstra.edu/~cscccl/rustlr_project/)
for further documentation.



[1]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.StrTokenizer.html
[2]:https://docs.rs/rustlr/latest/rustlr/generic_absyn/struct.LBox.html
[3]:https://docs.rs/rustlr/latest/rustlr/generic_absyn/struct.LRc.html
[4]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.ZCParser.html#method.lbx
[5]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.StackedItem.html#method.lbox
[sitem]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.StackedItem.html
[chap1]:https://cs.hofstra.edu/~cscccl/rustlr_project/chapter1.html
[lexsource]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.LexSource.html
[drs]:https://docs.rs/rustlr/latest/rustlr/index.html
[tktrait]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/trait.Tokenizer.html
[tt]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html
[rtk]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/enum.RawToken.html
[fromraw]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html#method.from_raw
[nextsymfun]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/trait.Tokenizer.html#tymethod.nextsym
[zcp]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.ZCParser.html
[ttnew]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html#method.new
