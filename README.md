"rustlr" LR(1) and LALR(1) parser generator by Chuck Liang.
Tutorial to be made available at https://cs.hofstra.edu/~cscccl/rustlr_project/

This is the first release of the project.  It has only been used for
implmentating modestly scaled, experimental programming languages.  If
there's significant interest, it will be become more robust, with
enhanced features, in future releases.

Version 0.1.1:

  The ability to train the parser has been added: the Runtime::parse_train
  function will ask for user input to improve error reporting by augmenting
  basic generated LR state machine with Error entries.

Future releases of rustlr will also be able to train from scripts:
this means that a new parser can be quickly trained to display
meaningful error messages each time after a grammar is modified (this
feature currently has not been tested extensively).

Future releases of rustlr will also allow the construction of a custom parser
using the generated state machine so that users of rustlr are not limited to
the built-in generic RuntimeParser::parse function.

