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


### Major Features and the Versions that Introduced Them

#### Version 0.4.0: AST generation for F\#

#### Version 0.3.95:

Adds the ability to generate Bump-allocated ASTS that
allows nested pattern matching against recursive types.

#### Version 0.3.8:

Adds the ability to generate a  parser for the
F\# language (Microsoft's version of Ocaml).  The system is called "Fussless"
and the F\# end is found at [github.com/chuckcscccl/Fussless](https://github.com/chuckcscccl/Fussless). 


#### Version 0.3.5:

The -lrsd option for *selective Marcus-Leermakers* grammars added as an experimental feature.  See the [Appendix](https://cs.hofstra.edu/~cscccl/rustlr_project/appendix.html) of the tutorial.

#### Version 0.3.0:

Experimental feature: markers that allow delayed-reduction
grammar transformations.  Improved ability to generate reasonable abstract
syntax.


#### Version 0.2.95:

Adds the ability to define custom regular expressions and custom token
types to the built-in lexical analyzer; the lexterminal and
valueterminal directives further simplify the creation of the lexical
analyzer.



#### Version 0.2.9:

Experimental support for a **wildcard token** in writing grammars.  Grammar
production rules can use the now-reserved `_` (underscore) symbol to mean
*unexpected token*.  
```
E -->  a _* b
```
The _ is regarded as a regular terminal symbol during the creation of the
deterministic LR statemachine. But a state table entry for the special wildcard
will apply to any unexpected input symbol.  Please see the tutorial for its
subtleties and usage.

#### Version 0.2.8:

The ability to automatically generate the abstract syntax tree data structures as well as the semantic actions required to create instances of them.  Automatically generated actions can be combined with manually written overrides.

Limited support for *, + and ? expressions introduced.


#### Version 0.2.5:

The ability to write semantic actions returning
values of different types has been added, without the need to use the Any
trait (and can thus accomodate non-static references).  Chapter 3 of
the tutorial was rewritten to reflect this important new option.
Backwards compatibility is retained.

A simplified syntax for forming LBox has been added: Grammar rules can
now contain labeled symbols on the right hand side in the form `E:[x]`, which
means that the semantic value associated with grammar symbol E is automatically placed in an LBox and assigned
to x.

#### Version 0.2.3:

The ability to **automatically generate a lexical
scanner** from a minimal set of grammar declarations has been added, using
the built-in RawToken and StrTokenizer.  This vastly simplifies the process
of producing a working parser.  Other tokenizers can still be used
in the previous way, by adopting them to the Tokenizer trait.


#### Version 0.2.0:

Significant improvements required that several components
are now renamed, while the older ones are retained for compatibility with
parsers already created.  

  -  A new, "zero-copy" lexer interface has been created
  -  A general purpose lexical analyzer is now included, although it is still
     possible to use any lexer due to the use of trait objects.
  -  Improved support for using **`LBox<dyn Any>`** as abstract syntax type by
     automatically generating runtime type casting.  This means that
     semantic actions for grammar productions no longer need to return values of the same
     type. However, this also means that abstract syntax representations
     cannot contain non-static references due to the Rust restriction that
     such types cannot impl Any.  An alternative approach would be to generate
     a enum type that includes all possible return types, but this approach is
     not compatible with allowing the lexical analyzer to be decoupled from
     the parser.

#### Version 0.1.4:

 This version's main enhancements are pattern labels.  In a grammar production,
 the value attached to nonterminal and terminal symbols can be extracted by
 specifying a pattern, which will cause an if-let statement to be automatically
 generated.  For abstract syntax with many layers of enums and structs, but
 which shares a single "absyntype" for the grammar.  For example, if *Exp* and
 *Expl* are variants of a common enum, one can now write rules such as 

 ```
  Exprlist -->  { Expl(Vec::new()) }
  Exprlist --> Exprlist:@Expl(mut ev)@ , Expr:@Exp(e)@  {ev.push(e); Expl(ev)}
 ```
 This capability was used to construct a parser for a scaled-down version of
 Java and is included in the examples directory of the repository.

 Abilities for using LBox were also extended, which allows *`LBox<dyn Any>`* to
 be used as the abstract syntax type, with functions and macros for
 up/downcasting.


#### Version 0.1.2:

  Added the [LBox][2] smartpointer for encapsulating lexical information
  (line and column) into abstract syntax.

  The parse function has been decomposed into a parse_core, which takes a
  functional argument that handles error reporting.  This allows a custom
  parser interface to be created if one does not wish to be restricted to
  the supplied one, which uses stdio.


#### Version 0.1.1:

  The ability to train the parser has been added. The `parse_train`
  function will ask for user input to improve error reporting by augmenting
  the basic generated LR state machine with Error entries.

  Constructing a parser that gives helpful error messages can be tricky,
  especially after a grammar has been modified and the parser is re-generated,
  which changes the state transition table.  Interactive training with
  the parse_train function now produces, in addition to an augmented parser,
  a training-script that records each error encountered along with the line,
  column numbers and the unexpected token.  It's the user's responsibility to
  keep track of the sample input used during interactive training and
  the script that was created from it.  A parser can be retrained from the
  script, given the identical input (and tokenizer).

    --------------------


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
