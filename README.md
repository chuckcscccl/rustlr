# **[rustlr](https://docs.rs/rustlr/0.1.1/rustlr/index.html)**
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

Future releases of rustlr will also be able to train from scripts:
this means that a new parser can be quickly trained to display
meaningful error messages each time after a grammar is modified (this
feature currently has not been tested extensively).

Future releases of rustlr will also allow the construction of a custom parser
using the generated state machine so that users of rustlr are not limited to
the built-in generic [RuntimeParser::parse][2] function.

[1]:https://docs.rs/rustlr/0.1.1/rustlr/struct.RuntimeParser.html#method.parse_train
[2]:https://docs.rs/rustlr/0.1.1/rustlr/struct.RuntimeParser.html#method.parse
