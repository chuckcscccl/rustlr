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
T --> T _
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
it cannot determine that the first two `b`'s are supposed to be
recognized as wildcards and that only the last b is a "real b".  That
is, it does not know which rule to apply to input `b` if the lookahead is also `b`.
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

####  The Semantic Value of Wildcards

When a symbol is matched to wildcard, a unique token is created that
carries a semantic value.  In case there is only a single type, the
declared absyntype/valuetype, then the wildcard token will have the
default value of the absyntype as its semantic value.  However, when there
are multiple types (forcing the generation of an internal enum -see Chapter 3),
or when the -auto/-genabsyn option is given (which automatically generates
the AST types - see Chapter 4), then the **the semantic value of the wildcard
is `(usize,usize)`, which indicates the starting and ending positions of the
token in the original text.**  The actual text can be accessed with the
the [Tokenizer::get_slice][getslice] function.  For example, if we modified the
grammar into:
```
terminals c a b
nonterminal T usize
nonterminal E
topsym E

T --> b {parser.current_position()}
T --> _:@(x,y)@ {x}
E --> a T*:positions b
```
and used the -auto (or -genabsyn) option when generating the parser, rustlr
will generate a struct:
```
pub struct E {
  pub positions:Vec<LBox<usize>>,
}
```
It will not generate a type for `T` since its type was overridden with usize.
The generated parser will record in a `Vec<LBox<usize>>` the starting positions
of each wildcard or `b` token.  The following main can then be used to
extract the actual text from the semantic information returned by the parser:
```
mod wc_ast;
use wc_ast::*;
mod wcparser;
use rustlr::Tokenizer;  // needed to make the get_slice function visible

fn main()
{
  let mut input = "a c d e f b b";
  let mut scanner4 = wcparser::wclexer::from_str(input);
  let mut parser4 = wcparser::make_parser();
  let tree4= wcparser::parse_with(&mut parser4, &mut scanner4);
  let result4 = tree4.unwrap_or_else(|x|{println!("Parsing errors encountered; results not guaranteed.."); x});
  println!("\nABSYN: {:?}\n",&result4);
  let E {positions:v} = result4;
  if v.len()==0 {return;}
  let start = *v[0];
  let end = *v[v.len()-1];
  let text = scanner4.get_slice(start,end);
  println!("text of slice: {}",text);
}//main
```
This code will produce the output
```
ABSYN: E { positions: [1, 3, 5, 7, 11] }

text of slice:  c d e f b
```


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
[getslice]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/trait.Tokenizer.html#tymethod.get_slice
