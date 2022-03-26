### Chapter 3: Multiple Value Types with **LBox\<dyn Any\>**

Rustlr requires that the value returned by the smantics actions of all grammar rules be of
the same type, declared using the `valuetype` or `absyntype`
directive.  It only allows one other type, the `externtype` which
means that these actions can be stateful.  However, sometimes it may
still be more convenient to allow different rules to return values of
different types.  Theoretically, this can be accomplished by generating
a enum-type internally that would encompass all possible types
returned by the rules.  In other words, instead of leaving the
definition of an enum with a large number of variants to the user, it
is created internally.  The problem with this approach is
that it is not quite compatible with the goal of *decoupling* the
parser from the lexical analyzer.  The tokens returned by a lexical
scanner must also carry values, such as numerical constants and string
literals, and these values must also be of the same "absyntype" as any
that appear on the parse stack.  Integrating such values into a
generated enum will tie the rustlr runtime parsers to one specific type of
lexical token.  Rustlr does include a 
[RawToken][rtk] type, but this type was not created to cover all possible
scenarios when a parser might be needed.  It only exists for the
purpose of including a usable tokenizer, [StrTokenizer][1], with the
rustlr crate.  We wish to allow rustlr to parse any type of input,
including binary formated input, as long as tokenizers can be provided
for them and adopted to the [Tokenizer][tktrait] trait. 

The currently available approach to allowing different grammar rules
to return differently typed values borrows a page from object-oriented
programming and relies on the Any trait, specifically **[LBox][2]\<dyn
Any\>**.  When a terminal or nonterminal symbol of the grammar is
declared, one can optionally specify a type associated with it that's
distinct from the overall absyntype (but which also must impl
Default).  **When the absyntype of the grammar is declared to
be LBox\<dyn Any\>, rustlr will automatically generate code to
downcast the values attached to grammar symbols and upcast the values
returned by the semantic actions to the supertype.** The following is
another grammar for a calculator program, but which demonstrates this
option.

```
!use rustlr::{unbox};
!pub enum Expr
!{
!   Val(i64),
!   Plus(LBox<Expr>,LBox<Expr>), 
!   Times(LBox<Expr>,LBox<Expr>),
!   Divide(LBox<Expr>,LBox<Expr>),
!   Minus(LBox<Expr>,LBox<Expr>),
!   Negative(LBox<Expr>),
!   Nothing,                   
!}
!impl Default for Expr { fn default()->Self {Nothing} }

absyntype LBox<dyn Any>
nonterminal E Expr
nonterminal ES Vec<LBox<Expr>>
terminal + - * / ( ) ;
typedterminal int Expr
topsym ES
resync ;

left * 500
left / 500
left + 400
left - 400

lexvalue int Num(n) Val(n)

E --> int:m { unbox!(m) } 
E --> E:e1 + E:e2 { Plus(e1,e2) }
E --> E:e1 - E:e2 { Minus(e1,e2) }
E --> E:e1 / E:e2 { Divide(e1,e2) }
E --> E:e1 * E:e2 { Times(e1,e2) }
E --> - E:e { Negative(e) }
E --> ( E:e )  { *e.exp }
ES --> E:n ; { vec![n] }
ES ==> ES:v E:e ;  {
   v.push(e);
   unbox!(v)
   } <==

EOF

```
Both the `nonterminal` and the `typedterminal` directives allow a type
to be associated with grammar symbol.  The default type is
the same as the absyntype (LBox\<dyn Any\>) unless so defined.
In this grammar, the type associated with the non-terminal E is
Expr, but that for ES is Vec\<LBox\<Expr\>\>.  This is an alternative
to including a `Seq(Vec<LBox<Expr>>)` variant of Expr, which was
used in the chapter 2 example.

In the  LBox\<dyn Any\> special setting, the labels attached to grammar symbols on
the right-hand side of productions no longer represent value of type
[StackedItem][sitem], but are of type [LBox][2]\<Ty\>, where Ty is the type
associated with that grammar symbol.  That is, in writing the semantic
action for a rule such as
```
E --> E:e1 + E:e2 { Plus(e1,e2) }
```
the labels e1 and e2 are automatically downcast to **LBox\<Expr\>**.  The
semantic action should return a value of type **Expr** because that is
the type associated with the nonterminal **E**.  The returned value will
be placed in a LBox and 
upcast to LBox\<dyn Any\> by rustlr before being pushed onto the parse stack.
Pattern labels can still be used to describe the values.  A pattern
inside @..@ will match the down-casted value inside the LBox.
As a much large example, [this grammar](https://cs.hofstra.edu/~cscccl/rustlr_project/minijava/lbamj.grammar),
which defines a scaled-down version of Java, demonstrates how the LBox\<dyn Any\>
type can still be used alongside patterns.

Macros including [unbox!](https://docs.rs/rustlr/latest/rustlr/macro.unbox.html)
allow the semantic values to be extracted from the LBox.

The final value returned by the parser will also be of type
`LBox<dyn Any>` and must be downcast to a usable value using the provided
[LBox::downcast](https://docs.rs/rustlr/0.2.1/rustlr/generic_absyn/struct.LBox.html#method.downcast) function.


#### **TRADEOFF**

The major downside of using this object-oriented approach is that the
abtract syntax types cannot contain non-static references, because the Any type
does not cover such references.  Even
though rustlr allows lexical scanners to be zero-copy, the `lifetime`
directive is meaningless in this mode.  Basically, this means that
instead of `&'t str` one may have to use owned strings in the abstract
syntax representation.  Thus there is a non-zero runtime overhead to
this approach.  In addition to being slower, using the Any trait with
downcasting also sacrafices a degree of static type safety. Thus, the
tradeoff here reflects the fundamental tradeoff between Rust and
object-oriented languages in general.  Those choosing the LBox\<dyn Any\>
option for its convenience should understand and accept this tradeoff.


[1]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.StrTokenizer.html
[2]:https://docs.rs/rustlr/latest/rustlr/generic_absyn/struct.LBox.html
[3]:https://docs.rs/rustlr/latest/rustlr/generic_absyn/struct.LRc.html
[4]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.ZCParser.html#method.lbx
[5]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.StackedItem.html#method.lbox
[sitem]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.StackedItem.html
[chap1]:https://cs.hofstra.edu/~cscccl/rustlr_project/test1grammar.html
[lexsource]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.LexSource.html
[drs]:https://docs.rs/rustlr/latest/rustlr/index.html
[tktrait]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/trait.Tokenizer.html
[tt]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html
[rtk]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/enum.RawToken.html
[fromraw]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html#method.from_raw
[nextsymfun]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/trait.Tokenizer.html#tymethod.nextsym
[zcp]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.ZCParser.html
[ttnew]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html#method.new
