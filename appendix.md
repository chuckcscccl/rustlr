## Appendix: Experimental Features

### Delayed Reductions

One of the main difficulties faced by writers of context-free grammars is,
borrowing a term from functional programming, the lack of *referential transparency*.  By this we mean the ability to compose grammars from smaller grammars,
to worry about isolated components of the grammar apart from the whole, and
to substitute a part of the grammar with something equivalent.

Take, for example, the following simple grammar:
```
S --> A | B
A --> a b c d x
B --> a b c d y 
```
This obviously unambiguous grammar is LL(5), but it's LR(0).  That's right,
no lookahead is needed even though the ambiguity between A and B is not
resolved until an `x` or a `y` is read at the end.  When an LR parser *shifts*,
it is naturally *delaying* the decision as to which nonterminal symbol to
reduce to.  The LR(0) statemachine will keep both the A and B productions
as candidates for reduction until something distinguishes them.  Even if
we were to replace c with a non-terminal:
```
S --> A | B
C --> c | C c
A --> a b C d x
B --> a b C d y
```
This grammar is still LR(0).  Had we used a right-recursive rule for C,
it would be LR(1).  Unfortunately, this example does not mean that we can substitute one LR grammar into another and still get an LR grammar in general.
The referential transparency breaks easily.  Consider
```
S --> A | B
C --> c | C c
A --> M C d x
B --> a b C d y
M --> a b
```
Surely this is still unambiguous and equivalent to the above grammar,
but it's no longer an LR(k) grammar for any k.  After we have read `a
b` and the next symbol is `c`, we would not know whether to
reduce it to `M`, or to shift `c` with a fixed number of
lookaheads. In such a situation, most LR generators will give a
warning and then resort to a default choice (usually shift over
reduce.)  But that would not help here: reducing would be the wrong
choice if the last input symbol is `y` and shifting would be the wrong
choice if the last symbol is `x`.

It is possible to save the situation, however, by *delaying* the
choice to reduce until more have been read *and* more reductions
applied.  This is done by a "Marcus-Leermakers" transformation, as
named in a [research paper][bns] by Bertsch, Nederhof and Schmitz.
```
S --> A | B
C --> c | C c
A --> MCd x
B --> a b C y
MCd --> a b C d
```
The idea is to create a new non-terminal symbol that associates the
original non-terminal with some amount of its right-context.  The
right context can consist of terminal as well as non-terminal
symbols.  The right-hand side of the rules of the symbol consist of
the right-hand sides of the original, plus the right-context being delayed.
Roughly speaking, it's like extending the number of
lookahead symbols, except the symbols can be *non-terminal*.  The transformation can be done internally: the transformed grammar relates to the original
by an inverse homomorphism.  This just means that the properties and
intent of the original grammar are preserved.  In particular, semantic actions
written for the original grammar (with `M --> a b`) can still be applied
by generating a tuple for the semantic value of the new symbol (`MCd`), then
deconstructing it before applying the action of the original grammar.

The original grammar is not LR(k) but the transformed one is LR(1) (and LALR(1)). 
The Bertsch-Nederhof-Schmitz [paper][bns] shows that such transformations can
be selective, and the amount of right-context to absorb should be flexible
lest further non-determinism may be introduced.
The original grammar is called an "selML(2,1)" grammar because it absorbs
at most 2 symbols of a nonterminal's right-context, and relies on one
lookahead in the traditional sense.  selML(k,m) grammars
contain LR(m) grammars, and are always unambiguous.  They
describe the same class of languages as LR grammars, but more grammars
are selML then are LR.
The paper gives an algorithm that
automatically applies the required transformations to a grammar, up to a fixed
maximum k.  However, only a prototype of the algorithm was ever implemented
and was never applied on a large scale.  We have implemented a
version of this algorithm for Rustlr. Starting with
version 3.4, rustlr accepts the **-lrsd k** option, where k is an (optional) number
indicating the maximum delay length.  This will attempt to construct a
*selML(k,1)* grammar.  The default value for k is 3. -lrsd 0 is equivalent
to LR(1): rustlr always computes exactly one lookahead. The algorithm is fast
enough when it succeeds, but when it should fail, such as for ambiguous
grammars, it may take a long time before failure is detected, especially
for larger values of k.  Still, the option has already proven useful.  We
have used to construct a new grammar for ANSI C (2011 edition). The new
grammar is *selML(3,1)*.  This feature is currently in experimental status,
but Rustlr is the first know parser generator that has seriously attempted
to incorporate this promising extension of LR parsing.  

While we continue to experiment with implementations of this
algorithm, in the meantime
Rustlr also allows a very simple mechanism that costs minimal overhead: the
grammar writer can mark where the transformations need to occur:
```
S --> A | B
C --> c | C c
A --> # M C d # x
B --> a b C d y
M --> a b
```
The hash marks indicate where to apply a transformation (do not
confuse # at the very beginning of a line, which indicates a comment -
the notation may evolve).  The symbol immediately following the first
marker must be non-terminal.  The transformation is applied to the grammar
before an LR(1) or LALR(1) engine is build.
While not nearly as powerful as the generalized algorithm (it cannot
apply transformations to the internally generated productions
themselves), this simple mechanism is still a useful addition.  From a
practical standpoint, it allows us to recover a degree of referential
transparency with minimal effort (both human effort and computational cost).
In the published Yacc grammar for ANSI C (2011 edition), we find the following
productions:
```
declarator -->  pointer  direct_declarator
declarator -->  direct_declarator
```
Both `pointer` and `direct_declarator` are non-terminals.
We would like to combine the two productions into one:
```
declarator -->  pointer?  direct_declarator
```
This is not just easier to write, but when generating the AST for C automatically, we'd prefer to have a simple tuple struct
(` struct declarator(Option<pointer>,direct_declarator)`)  instead of
an enum with two variants.  Rustlr recognizes the ? operator and transforms this
to
```
declarator --> P  direct_declarator
P --> pointer
P --> 
```
The internal introduction of the empty production causes a shift-reduce
conflict (when the lookahead is a left-parentheses).  We still don't know for
sure what's causing the conflict, and we know that adopting a default
shift or reduce strategy might not work, but we solved the problem with
```
declarator --> # pointer?  direct_declarator #
```
This particular transformation attaches `direct_declarator` to the end of the
two productions for `P`, thereby recovering the original LALR grammar. But
the transformation is internal: we get to write a different style of grammar
and generate ASTs, write semantic actions for them as such.

Regular expressions are well-liked by most programmers and many
modern parser generators allow them.  It makes writing grammars easier.
On the surface it may not appear
too difficult to add them to any LR parser generator: just add new productions
rules like A --> A a | null, etc.  But if adding such productions 
lead to further non-determinisms (conflicts) in the grammar, then clearly it
would defeat the purpose of making it easier to write grammars.  We've
already seen that the selective-delay technique helps to alleviate this
problem.


Programmers need to understand that there is a fundamental
distinction between Regex and CFGs. Regex offer
full referential transparency.  You can use `*`, `+` and `?` in any
combination and we can build a deterministic finite state machine for
it in the end.  Two regular expressions juxtaposed, like `A*A*`, is
still a regular expression.  Of course we can translate A* into a
grammar: `P --> P A | null`, but `S --> P P`, that is, `S --> A* A*`
is hopelessly ambiguous: where should the first sequence begin and
the second one end?? Grammars cannot be composed that way without losing
their properties.  Of course, you could invent your own notion of what
a "grammar"
means, and *declare* the grammar to be unambiguous.  In fact, there
are "grammars" that technically can never be ambiguous.
A parser generator for such "grammar" would not be able to even give
a warning that something is wrong because according to definition nothing
can ever be wrong.  It's like writing a program with
a bunch of type errors and still have it compile and run, because the
language considers them not errors but features.
Such an approach is popular, however, because most programmers
are only interested in "getting something to work."  They are not as interested in something that "should not work."
The contrast between
strongly, statically typed programming languages and untyped scripting
languages is a good analogy for the difference between LR-style parser generators
and some other approaches.  The later are easier to learn and use,
especially if you only want to get something to work quickly, while the former
has a steeper learning curve but offer long-term benefits.
If you chose to be a Rust programmer then you should already know which side you're on.
Programmers who write `S --> A* A*` do not deserve a working parser because
they have not thought carefully as to what they really want.

Given an ambiguous grammar, like most LR parser generators, Rustlr will
at least give you a warning before applying any defaults.  Sometimes defaults
work correctly, like for the dangling-else problem.  But as we've shown
above, they do not always work.  This is why we are interested in
more rigorous approaches to extend LR parsing.



    ----------------

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
LR state machine.  The wildcard role of the symbol is only significant
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
[bns]:https://hal.archives-ouvertes.fr/hal-00769668/document
