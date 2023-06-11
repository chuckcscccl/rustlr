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
terminals + * - / ( )   # verbatim terminal symbols
valterminal Int i32     # terminal symbol with value
nonterminal E
nonterminal T : E  # specifies that AST for T should merge into E
nonterminal F : E
startsymbol E
variant-group BinaryOp + - * /   # simplifies AST enum by combining variants

# production rules:
E --> E + T  | E - T | T
T --> T * F | T / F | F
F:Neg --> - F                    # 'Neg' names enum variant in AST
F --> Int | ( E )

!mod simplecalc_ast; // !-lines are injected verbatim into the parser
!fn main()  {
!  let mut scanner1 = simplecalclexer::from_str("10+-2*4");
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

In addition to a parser, the grammar generates a lexical scanner from
the declarations of terminal symbols.  It also created the following
abstract syntax type and the semantic actions that produce instances of
the type.
```
#[derive(Debug)]
pub enum E {
  BinaryOp(&'static str,LBox<E>,LBox<E>),
  Int(i32),
  Neg(LBox<E>),
  E_Nothing,
}
impl Default for E { fn default()->Self { E::E_Nothing } }
```
The form of the AST type(s) was determined by additional declarations
within the grammar.  An enum is normally generated for each
non-terminal with multiple productions, with a variant for each
production.  However, the enum variants generated from the productions
for `T` and `F` are merged into the type for `E` by the declarations
`nonterminal T : E` and `nonterminal F : E`.  The `variant-group`
declaration combined what would-have-been four variants into one.  The
`Neg` label on the unary minus rule separates that case from the
"BinaryOp" variant group.

[LBox](https://docs.rs/rustlr/latest/rustlr/generic_absyn/struct.LBox.html)
is a *custom smart pointer*
that automatically contains the line and column position of the start
of the AST construct in the original source.  This information is
usually required beyond the parsing stage.

Rustlr AST types implement the Default trait so that a partial result is
always returned even when parse errors are encountered.

Automatically generated AST types and semantic actions can always be
manually overridden.

Specifying operator precedence and associativity instead of using the
`T` and `F` categories is also supported.

The generated parser and lexer normally form a separate module.  However,
as this is a quick example, we've injected a `main` directly into the parser
file to demonstrate how to invoke the parser.
To run this example,

  1. Install rustlr as a command-line application: **`cargo install rustlr`**
  2. Create a Cargo crate and **`cargo add rustlr`** inside the crate
  3. save [the grammar](https://github.com/chuckcscccl/rustlr/blob/main/examples/simplecalc/simplecalc.grammar) in the crate as **`simplecalc.grammar`**.
  The filename determines the names of the modules created, and must 
  have a `.grammar` suffix.
  4. Run rustlr in the crate with
  >  **`rustlr simplecalc.grammar -o src/main.rs`**
  5. **`cargo run`**

The expected output is
```
AST: BinaryOp("+", Int(10), BinaryOp("*", Neg(Int(2)), Int(4)))
```

<br>

### New in Version 0.4.8: Conversion From Yacc/Bison Grammar.

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
