## Appendix: Experimental Features

### The Wildcard Token

Rustlr version 0.2.9 introduced an experimental feature that allows users to write grammar productions that include a "wildcard" using `_` (underscore). 
For example:
```
E --> a _* b
```
The * symbol for zero or more repetitions was introduced in version 0.2.8 (along
with ? and +).  Rustlr processes the above rule by adding a new non-terminal
symbol to represent the sequence:
```
E --> a T b
T -->
T --> _
```

However, the meaning of the `_` symbol is a bit intricate and requires an
understanding of how LR parsing works.  At the heart of an LR parser
is a deterministic state machine (the "viable prefix automaton").  This
automaton *must stay deterministic*.  This means that the correct way to
understand the underscore symbol is not as "any symbol" but as any *unexpected*
symbol.  If a state defines a transition for symbol `b` as well as a transition
for the underscore, then these transitions must not render the machine
non-deterministic.  In other words, the following should **not** cause
a "reduce-reduce" conflict:
```
F --> b | _
```
Rustlr works by treating `_` (represented internally as
`_WILDCARD_TOKEN_`) like any other terminal symbol when generating the
LR state machine.  The wildcard role of the symbol is only signifcant
during parsing when a token is encountered that **does not have a**
regular transition defined for the current state.  Normally, such a
situation results in a parsing error.  However, if the state defines a
transition for `_`, then rustlr will follow the transition.
But the wildcard will never override a regular transition, if there is one.

What this means is that the intended meaning of the expression `a _*
b` is **not** any sequence of symbols bracketed by a's and b's.  The
above grammar (`E --> a _* b`) will fail to parse `"a b b b"` because
it cannot determine that the first two `b`'s are supposed to
recognized as wildcards and that only the last b is a "real b".  That
is, it does not know which to apply to input `b` if the lookahead is also `b`.
It will parse `"a a a b"` because after the initial `a` is read, there are no
further conflicting transitions for `a`.  To parse what we intend to, we have to modify the
grammar as follows:
```
F --> b | _
E --> a F* b
```
This grammar does recognize any sequence of symbols bracketed by a and b.
The wildcard is thus much more subtle to use than one might like, but it
can still be useful, and thus it was decided to include it in Rustlr.




[1]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.StrTokenizer.html
[2]:https://docs.rs/rustlr/latest/rustlr/generic_absyn/struct.LBox.html
[3]:https://docs.rs/rustlr/latest/rustlr/generic_absyn/struct.LRc.html
[4]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.ZCParser.html#method.lbx
[5]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.StackedItem.html#method.lbox
[sitem]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.StackedItem.html
[chap1]:https://cs.hofstra.edu/~cscccl/rustlr_project/chapter1.html
[chap2]:https://cs.hofstra.edu/~cscccl/rustlr_project/chapter2.html
[chap3]:  https://cs.hofstra.edu/~cscccl/rustlr_project/chapter3.html
[lexsource]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.LexSource.html
[drs]:https://docs.rs/rustlr/latest/rustlr/index.html
[tktrait]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/trait.Tokenizer.html
[tt]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html
[rtk]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/enum.RawToken.html
[fromraw]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html#method.from_raw
[nextsymfun]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/trait.Tokenizer.html#tymethod.nextsym
[zcp]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.ZCParser.html
[ttnew]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html#method.new
