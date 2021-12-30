# **[rustlr](https://docs.rs/rustlr/0.1.3/rustlr/index.html)**
**LR(1) and LALR(1) parser generator by Chuck Liang.**

**A [Tutorial](https://cs.hofstra.edu/~cscccl/rustlr_project/) is being prepared.**

The project grew out of the author's compiler construction and
programming languages classes over the years.  It has been used for
implmentating modestly scaled, experimental programming languages.  It
will be become more robust, with enhanced features, in future
releases.

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


[1]:https://docs.rs/rustlr/0.1.3/rustlr/struct.RuntimeParser.html#method.parse_train
[2]:https://docs.rs/rustlr/0.1.3/rustlr/struct.RuntimeParser.html#method.parse
[3]:https://docs.rs/rustlr/0.1.3/rustlr/struct.RuntimeParser.html#method.train_from_script
