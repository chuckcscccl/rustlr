# **[rustlr](https://docs.rs/rustlr/0.1.2/rustlr/index.html)**
**LR(1) and LALR(1) parser generator by Chuck Liang.**

**A [Tutorial](https://cs.hofstra.edu/~cscccl/rustlr_project/) is being prepared.**

The project grew out of the author's compiler construction and
programming languages classes over the year.  It has been used for
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


Future releases of rustlr will be able to train from scripts:
this means that a new parser can be quickly trained to display
meaningful error messages each time after a grammar is modified (this
feature currently has not been tested extensively).

[1]:https://docs.rs/rustlr/0.1.1/rustlr/struct.RuntimeParser.html#method.parse_train
[2]:https://docs.rs/rustlr/0.1.1/rustlr/struct.RuntimeParser.html#method.parse
