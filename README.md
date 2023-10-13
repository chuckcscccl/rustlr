# **[rustlr](https://docs.rs/rustlr/latest/rustlr/index.html)**
**LR-Style Parser Generator**

**A [Tutorial](https://chuckcscccl.github.io/rustlr_project/) with several examples is available.**

Besides traditional LR and LALR parser generation, Rustlr supports the following
options

1. An experimental feature that generates parsers for *Selective Marcus-Leermakers* grammars.  This is a larger class of unambiguous grammars than traditional LR and helps to allow new productions to be added to a grammar without
creating conflicts (see the [Appendix](https://chuckcscccl.github.io/rustlr_project/appendix.html) of the tutorial).
2. The option of creating the abstract syntax data types and semantic actions from the grammar. Rustlr grammars contain a sub-language that controls how ASTs are to be generated. 
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

Rustlr aims to simplify the creation of precise and efficient parsers and
will continue to evolve and incorporate new features, though backwards
compatibility will be maintained as much as possible.

<p>


### Quick Example: Arithmetic Expressions and Their Abstract Syntax

The following are the contents of a Rustlr grammar, [`simplecalc.grammar`](https://github.com/chuckcscccl/rustlr/blob/main/examples/simplecalc/simplecalc.grammar):
```
auto
terminals + * - / ; ( )   # verbatim terminal symbols
valterminal Int i32       # terminal symbol with value
nonterminal E
nonterminal T : E  # specifies that AST for T should merge into E
nonterminal F : E
nonterminal ExpList
startsymbol ExpList
variant-group-for E BinaryOp + - * /  # group operators in AST generation

# production rules:
E --> E + T  | E - T | T
T --> T * F | T / F | F
F:Neg --> - F                   # 'Neg' names enum variant in AST
F --> Int | ( E )
ExpList --> E<;+> ;?    # ;-separated list with optional trailing ;


!mod simplecalc_ast; // !-lines are injected verbatim into the parser
!fn main()  {
!  let mut scanner1 = simplecalclexer::from_str("10+-2*4; 9-(4-1)");
!  let mut parser1 = make_parser();
!  let parseresult = parse_with(&mut parser1, &mut scanner1);
!  let ast =
!    parseresult.
!    unwrap_or_else(|x| {
!       println!("Parsing errors encountered; results not guaranteed..");
!       x
!    });
!  println!("\nAST: {:?}\n",&ast);
!}//main
```
The grammar recognizes one or more arithmetic expressions separated by
semicolons.  In addition to a parser, the grammar generates a lexical
scanner from the declarations of terminal symbols.  It also created
the following abstract syntax types and the semantic actions that
produce instances of the types.
```
#[derive(Debug)]
pub enum E {
  BinaryOp(&'static str,LBox<E>,LBox<E>),
  Int(i32),
  Neg(LBox<E>),
  E_Nothing,
}
impl Default for E { fn default()->Self { E::E_Nothing } }

#[derive(Default,Debug)]
pub struct ExpList(pub Vec<LC<E>>,);
```
[LBox](https://docs.rs/rustlr/latest/rustlr/generic_absyn/struct.LBox.html)
and
[LC](https://docs.rs/rustlr/latest/rustlr/generic_absyn/struct.LC.html)
are structures that contain the line and column positions of the start
of the AST constructs in the original source.  This information is
automatically inserted into the structures by the parser.  LBox
encapsulates a Box and serves as a custom smart pointer while LC
contains the extra information in an exposed tuple.  Both `LBox<T>`
and `LC<T>` implement `Deref<T>` and `DerefMut<T>`, thus carrying the
extra information non-intrusively.

Rustlr generates AST types based on the grammar but special
declarations can control the precise structure of these types.  A
struct is normally generated for nonterminal symbols with a single
production while an enum is generated for nonterminals with multiple
productions, with a variant for each production.  However, the enum
variants generated from the productions for `T` and `F` are merged
into the type for `E` by the declarations `nonterminal T : E` and
`nonterminal F : E`.  The `variant-group-for` declaration combined what
would-have-been four variants into one.  The `Neg` label on the unary
minus rule separates that case from the "BinaryOp" variant group.

Rustlr AST types implement the Default trait so that a partial result is
always returned even when parse errors are encountered.

Automatically generated AST types and semantic actions can always be
manually overridden.

Specifying operator precedence and associativity instead of using the
`T` and `F` categories is also supported.

The generated parser and lexer normally form a separate module.  However,
as this is a quick example, we've injected a `main` directly into the parser
to demonstrate how to invoke the parser.
To run this example,

  1. Install rustlr as a command-line application: **`cargo install rustlr`**
  
  2. Create a Cargo crate and add
  ```
    [dependencies]
    rustlr = {version="0.5", default-features=false}
  ```
  to its Cargo.toml.  Turning off default features will include
  only the runtime parsing routines of rustlr as part of the crate.
  
  3. save [the grammar](https://github.com/chuckcscccl/rustlr/blob/main/examples/simplecalc/simplecalc.grammar) in the crate as **`simplecalc.grammar`**.
  The filename determines the names of the modules created, and must 
  have a `.grammar` suffix.
  
  4. Run the rustlr application in the crate with
  >  **`rustlr simplecalc.grammar -o src/main.rs`**
  
  5. **`cargo run`**

The expected output is
```
AST: ExpList([BinaryOp("+", Int(10), BinaryOp("*", Neg(Int(2)), Int(4))), BinaryOp("-", Int(9), BinaryOp("-", Int(4), Int(1)))])
```

Rustlr can also be invoked from within Rust by calling the [rustlr::generate](https://docs.rs/rustlr/latest/rustlr/fn.generate.html) function.

<br>

#### New in Version 0.5.0

The option to install only the runtime parser, without parser generation routines

#### New in Versions 0.4.13

Boxed labels such as `[x]` are now represented by LC instead of LBox during
auto-generation.  

#### New in Versions 0.4.11 and 0.4.12

The wildcard `_` token now carries the original text of the token as
its semantic value by default.  The `variant-group` directive is now
deprecated (though still available) by `variant-group-for`.

#### New in Version 0.4.10:

When called from the [rustlr::generate](https://docs.rs/rustlr/latest/rustlr/fn.generate.html) function, rustlr can be made completely silent if given the
`-trace 0` option.  All reports are logged and returned by the function.


#### New in Version 0.4.9: Error logging option

Given a parser instance `parser`, it's now possible to call
`parser1.set_err_report(true)`, which will log parse errors internally
instead of printing them to stderr.  The error report can be retrieved
by calling `parser1.get_err_report()`.

#### New in Version 0.4.8: Conversion From Yacc/Bison Grammar.

If the rustlr executable is given a file path that ends in ".y", it will
attempt to convert a yacc/bison style grammar into rustlr's own grammar
syntax, stripping away all semantic actions and other language-specific
content.  All other command-line options are ignored.



<br>

Please consult the [tutorial](https://chuckcscccl.github.io/rustlr_project/)
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
