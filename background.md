### Background Information

Although writing parsers and creating abstract syntax representations
have been done for decades, no single standard has arisen that is
clearly the best choice.  But we do have some theoretical results:
Donald Knuth proved over half a century ago that *every deterministic
context free language has an LR(1) grammar*.  But such results never
stopped some individuals from claiming that their approach is "more
powerful".  Indeed, these individuals may even have good reasons to
believe so.  Working with strictly unambiguous grammars can be
non-intuitive.  Intuitively we'd like to see something close to the
BNF definition of syntax: `E --> E+E | E*E | E-E | -E`, etc.  Every
expression `E` can be a subexpression of a larger one.  But such
grammars are not LR as they are ambiguous.  The ambiguity in this case
is caused by the unknown precedence relation between operators * and +, and
the unknown associativity of operator -.  These lead to "shift-reduce"
conflicts in a bottom-up parser.  There is also a possible
ambiguity between `E-E` and `-E`: if the grammar is not sensitive enough to
its *left-context*, this becomes a "reduce-reduce" conflict. 


