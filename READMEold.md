# **[rustlr](https://docs.rs/rustlr/latest/rustlr/index.html)**
**LR(1) and LALR(1) parser generator**

**A [Tutorial](https://cs.hofstra.edu/~cscccl/rustlr_project/) with several examples is available.**

Among the features that Rustlr supports are:

1. The option of automatically creating the abstract syntax data structures and semantic actions from the grammar.
2. operator precedence and associativity declarations allow the use of ambiguous grammars.
3. use of patterns in describing semantic values directly in the grammar (when writing semantic actions manually), e.g.

```rust
    E -->  let E:@Var(x)@ in E:@Expr(e)@  {  Letexp(x,e)  }
```
4. The ability to train the parser, interactively or from script for better error reporting.
5. Semantic actions have access to mutable external state, which (with manually written actions) can recognize some non-context free languages.

#### Version 0.2.8:

The ability to automatically generate the abstract syntax tree data structures as well as the semantic actions required to create instances of them.  Automatically generated actions can be combined with manually written overrides.

Limited support for *, + and ? expressions introduced.

The runtime parser now displays the lines for all parser errors; other internal enhancements.  Ability to parse rules separated by "|" improved.

Bug fixes for versions 0.2.6, 0.2.7

#### Version 0.2.5:

The ability to write semantic actions returning
values of different types has been added, without the need to use the Any
trait (and can thus accomodate non-static references).  Chapter 3 of
the tutorial will be rewritten to reflect this important new option.
Backwards compatibility is retained.

A simplified syntax for forming LBox has been added: Grammar rules can
now contain labeled symbols on the right hand side in the form `E:[x]`, which
means that the semantic value associated with grammar symbol E is automatically placed in an LBox and assigned
to x.


#### Version 0.2.4: mostly internal enhancements
#### Version 0.2.3:

The ability to **automatically generate a lexical
scanner** from a minimal set of grammar declarations has been added, using
the built-in RawToken and StrTokenizer.  This vastly simplifies the process
of producing a working parser.  Other tokenizers can still be used
in the previous way, by adopting them to the Tokenizer trait.

#### Version 0.2.2: internal changes, better reporting of grammar conflicts

#### Version 0.2.1: minor fixes

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

#### Version 0.1.1:

  The ability to train the parser has been added: the [Runtime::parse_train][1]
  function will ask for user input to improve error reporting by augmenting
  the basic generated LR state machine with Error entries.

#### Version 0.1.2:

  Fixed problem with Accept state; added LBox smartpointer for encapsulating
  lexical information into abstract syntax.

  The parse function has been decomposed into a parse_core, which takes a
  functional argument that handles error reporting.  This allows a custom
  parser interface to be created if one does not wish to be restricted to
  the supplied [RuntimeParser::parse][2] function, which uses stdio.

#### Version 0.1.3:

  Training the parser now modifies the same parser file that it reads from.
  The ability to use LBox and LRc for non-intrusively encapsulating lexical
  (line/column/source) information into abstract syntax has been expanded.
  Fixes an error where a non-terminal symbol is declared without any rules
  defined for it.

  parse_core has been retained but a new parse_base function is
  introduced that takes as input the error handler as a trait object.
  This should allow better flexibility in building custom parser
  interfaces while still using the basic state machine generated.

  Constructing a parser that gives helpful error messages can be tricky,
  especially after a grammar has been modified and the parser is re-generated,
  which changes the state transition table.  Interactive training with
  the parse_train function now produces, in addition to an augmented parser,
  a training-script that records each error encountered along with the line,
  column numbers and the unexpected token.  It's the user's responsibility to
  keep track of the sample input used during interactive training and
  the script that was created from it.  A parser can be retrained from the
  script, given the identical input (and tokenizer) using the
  [RuntimeParser::train_from_script][3] function.

  Future releases of rustlr will further enhance the training feature.

  We also hope to identify a robust, generic lexical tokenizer tool
  for Rust so that the parser generator can also automatically
  generate a lexical analyzer from additional specifications in the grammar.
  Another potential feature to be explored is the ability to generate an
  abstract syntax type structure from the grammar itself.

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

    --------------------

This project grew out of the author's compiler construction and
programming languages classes over the years and has been mainly used
for implmentating modestly scaled, experimental programming languages.
But it is becoming sophisticated enough to be more than just a project and
will continue to improve over time.



[1]:https://docs.rs/rustlr/latest/rustlr/runtime_parser/struct.RuntimeParser.html#method.parse_train
[2]:https://docs.rs/rustlr/latest/rustlr/runtime_parser/struct.RuntimeParser.html#method.parse
[3]:https://docs.rs/rustlr/latest/rustlr/runtime_parser/struct.RuntimeParser.html#method.train_from_script
