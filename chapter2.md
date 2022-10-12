## Chapter 2: Advanced Calculator


In the second chapter of this tutorial, we write a more advanced
version of the calculator example and describe a more complete set of
features of RustLr including:

  * How to write ambiguous grammars with operator precedence and associativity
    declarations.
  * How to parse, create abstract syntax, and report syntactic and semantic errors for more sophisticated kinds of
    expressions that include variables
    and scoping rules, in particular expressions such as `let x=3 in x*x`.
  * How to use a simple error-recovery technique.
  * How to use patterns when defining grammar production rules.
  * How to train the parser interactively for better error reporting.

The [grammar](https://cs.hofstra.edu/~cscccl/rustlr_project/calc4/calc4.grammar)
for the more advanced calculator is as follows:

```ignore
!use crate::exprtrees::*; /* ! lines are injected verbatim into parser */
!use crate::exprtrees::Expr::*;
!use rustlr::{LBox};

lifetime 'src_lt
absyntype Expr<'src_lt>
externtype i64
nonterminals E ES
terminals + - * / ( ) = ;
terminals let in int var
topsym ES
resync ;

left * 500
left / 500
left + 400
left - 400
nonassoc = 200

# for lexical scanner generation:
lexvalue int Num(n) Val(n)
lexvalue var Alphanum(x) Var(x)
lexattribute set_line_comment("#")

E --> int:m { m.value }
E --> var:s { s.value }
E --> E:e1 + E:e2 { Plus(e1.lbox(),parser.lbx(2,e2.value)) }
E --> E:[e1] - E:[e2] { Minus(e1,e2)}
E --> E:[e1] / E:[e2] { Divide(e1,e2) } 
E --> E:[e1] * E:[e2] { Times(e1,e2) }
E(600) --> - E:[e] { Negative(e) }
E --> ( E:e )  { e.value }
E --> let E:@Var(x)@ = E:[e] in E:[b] {Letexp(x,e,b)}
ES --> E:[n] ; { Seq(vec![n]) }
ES ==> ES:@Seq(mut v)@  E:[e] ;  {
   v.push(e);
   Seq(v)
   } <==

# ==> and <== are required for rules spanning multiple lines

EOF
```


### Motivation

  Using an LR parser generator has a non-trivial learning curve.
Working with strictly unambiguous grammars can be non-intuitive.
Intuitively we'd like to see something close to the BNF definition of
syntax: `E --> E+E | E*E | E-E`, etc.  Every expression `E` can
be a subexpression of a larger one.  But such grammars are ambiguous
and thus not LR. The ambiguity comes from the unspecified precedence
of operators + and *, and the unspecified associativity of -.  Another
ambiguity is illustrated by the infamous "dangling else" problem.
In a grammar with

     `E --> if (E) E  |  if (E) E else E`

how should we parse *`if (a) if (b) c else d`*? To associate the
`else` with the inner `if`, we must delay the reduction by the first
rule in favor of the second.  Eliminating such ambiguities by
rewriting the grammar can be non-trivial.  However, these kinds of
ambiguities can also be eliminated by augmenting the ambiguous grammar with
operator precedence and associativity declarations.  These are
evident from declarations such as **`left * 500`**.  In addition to
giving precedence to symbols, however, each production rule of the grammar
must also be assigned a precedence.  This can usually be calculated
automatically by finding the symbol on the right-hand side with the highest
precedence.  However, there are cases when we must assign the precedence
manually, as in the rule `E(600) --> - E`.  The - symbol has higher precedence
when it appears as a unary operator compared to all binary operators.

It is recommended that operator precedence and associativity declarations
are used with moderation.  If abused, situations can arise that would
require further, awkward declarations such as declaring `=` to have higher
precedence than `[`.  It is best to restrict their usage only to the well-known
binary operators.  Chapters [3][chap3] and [4][chap4] contain examples where
moderate uses of these declarations are combined with more "grammatical"
approaches to disambiguate precedence and associativity.

The main purpose of a parser is to transform *concrete syntax*, which
is usually a string, into *abstract syntax*, which is usually a tree.
Parsing is only one of the first stages in a modern interpreter or
compiler. Many important tasks such as type checking can be carried
out in other stages.  Errors can be syntactic - meaning that the
syntax is not accepted by the grammar, or semantic, such as type
incompatibilities.  Usually only syntactic errors are reported by the
parser.  However, all error reports made by the interpreter/compiler
must indicate the location in the orignal text (line and column
numbers) where the error originates.  This implies that the parser
must insert this location information into the abstract syntax tree.
All data structures designed for the abstract syntax must accommodate
this information, which can become rather intrusive when coding all the
match cases for the AST variants.
In an object oriented programming language, this problem is easily
solved by defining an abstract superclass for all AST 
structures.  But Rust has only minimal support for OOP.  It has no
direct support for inheritance.  Instead, rustlr implements a mechanism
called **[LBox][2]**.

Trees are defined recursively - in Rust, this usually means using the
Box smart pointer.
An [LBox][2] encapsulates a Box along with a pair of u32 values
indicating a line and a column number (thus taking up only 64 bits of
extra space).  It implements Deref and DerefMut by redirecting the
dereferences to the encapsulated box.  This means that an LBox can be
used like a Box - except when we need to access the location
information.

It is recommended (but not required) that the enums and
structs making up the abstract syntax use [LBox][2], e.g. 

  `enum Expr { Plus(LBox<Expr>,LBox<Expr>), Times(LBox<Expr>,LBox<expr>), etc.. }`

Rustlr has features that facilitate the use of LBox.  For example, on the
right-hand side of a production rule a labeled symbol in form `E:[a]` means
that the semantic value associated with the symbol is automatically
placed in an LBox that also includes the lexical
location information, and this LBox is assigned to `a`. 
A similar mechanism for Rc, [LRc][3], also exists, but without the same
level of support.  

In the following we further detail the additional features of rustlr
demonstrated by this grammar and associated abstract syntax structures.

### Principal Features


The grammar shown above differs from the [first][chap1] chapter in the following principal ways.

1. The grammar is ambiguous.  There are *shift-reduce* conflicts from
the pure grammar that are resolved using operator precedence and
associativity rules as declared by grammar directives such as **`left * 500`**.
A terminal symbol that's to be used as an operator can be
declared as left, right or non-associative and a positive integer defines
the precedence level.  
If a production rule is not explicitly assigned a precedence,
it is assigned to be the same as that of the right-hand side
symbol with the highest precedence.
The default precedence of all grammar symbols is zero.
*(Internally, the precedence is represented by the absolute value of a signed integer: positive
for left-associative, negative for right-associative.  The second-most
significant bit is used to distinguish these from non-associative values. Zero means unassigned.)*

     Rustlr resolves **shift-reduce** conflicts as follows:

    - A lookahead symbol with strictly higher precedence than the rule results
      in *shift*. A warning is always given if the rule has precedence zero.
    - A lookahead symbol with strictly lower precedence than the rule results
      in *reduce*. A warning is always given if the lookahead has precedence zero (undeclared precedence)  
    - A lookahead symbol with the same precedence and associativity as the rule,
      and which is declared right-associative, will result in *shift*.
    - A lookahead symbol with the same precedence and associativity as the rule,
      and which is declared left-associative, will result in *reduce*.
    - In other situations the conflict is *resolved in favor of shift*, with a
      warning sent to stdout regardless of trace level.  All shift-reduce
      conflicts are warned at trace level 2 or higher.

     Had we left out the last declaration `left = 200`, for example, rustlr
     will give multiple shift-reduce warnings, although the generated parser
     would be the same.

     Using this scheme, the "dangling else" problem can be solved by giving
     "else" a higher precedence than "if".  

     Rustlr also resolves **reduce-reduce**
    conflicts by always favoring the rule that appears first in the
    grammar, although a warning is always sent to stdout regardless of trace
    level.

2. The language that the grammar defines includes expressions of the form
   **`let x = 1 in (let x = 10 in x*x) + x`**, which should evaluate to 101.
   The lexical analyzer and parser must recognize alphanumeric symbols
   such as `x` as variables.  Since version 0.2.0, rustlr no longer requires
   owned strings to represent such constructs: the new [Tokenizer][tktrait] trait
   and [TerminalToken][tt] type allow the construction of zero-copy lexers.
   The **`lifetime`** declaration in the grammar allows the use of constructs
   with non-static references (`'src_lt str`) in abstract syntax representations.  Currently, only a single lifetime declaration is allowed: this is usually
   referring to the lifetime of the input. If it becomes clear that more than
   one lifetime might be needed, rustlr will be updated accordingly.
   Evaluating let-expressions also illustrate the separation of syntactic from semantic
   analysis: checking the scopes of variables introduced by `let` happens after the
   parsing stage.

3. The grammar's abstract syntax is defined in a separate module,
[exprtrees.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/calc4/src/exprtrees.rs).  The abstract syntax tree type ('absyntype') 'Expr' of this module
uses **[LBox][2]**,
as explained above.  LBox implements deref coercion on the boxed value, but also
carries the lexical position information when they're needed.  The
[StackedItem::lbox][5], [ZCParser.lb](https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.ZCParser.html#method.lb) and the [ZCParser.lbx][4] functions can be
invoked from within the semantic actions to automatically transfer the
parser's lexical information while creating an LBox.  However, the easiest way to use LBox, in most situation, is to label a grammar symbol as in **`E:[a]`**: this means that in the semantic action of the rule **`a`** will be bound to an LBox enclosing the semantic value associated with the grammar symbol.  It is
recommended that [LBox][2] (or [LRc][3]) be used instead of Box (Rc) when
defining the recursive enums and structs that typically form the
abstract syntax representation.  This allows accurate error reporting
after the parse tree is built, as in the division-by-zero example
shown below.

4. The language allows a sequence of arithmetic expressions to be evaluated
in turn by separating them with semicolons, such as in `2+3; 4-1;`.
The semicolon also allows us
to define a simple error-recovery point: **`resync ;`** indicates that when a
parser error is encountered, the parser will skip past the next semicolon,
then look down its parse stack for a state with which it can continue parsing.
In otherwords, failure to parse one expression does not mean it will not try to
parse the next ones.  Rustlr does implement other error-recovery techniques, which are explored in a [later chapter](https://cs.hofstra.edu/~cscccl/rustlr_project/cpmz.grammar).

5. The labels attached to grammar symbols on the right-hand side of
   grammar productions can be more than a simple variable or irrefutable pattern
   (as demonstrated in the first calculator). It can also be a pattern
   enclosed in @...@.  Rustlr generates an if-let expression that attempts to
   bind the pattern to what's popped from the parse stack.  The value is
   moved to a mut variable before being deconstructed by the pattern.
   In general, the label associated with a right-hand side grammar symbol can
   be of the following forms (two were used in the first grammar):

   1. **`E:a + E:b`**: this is found in the first grammar, each symbol 'a', 'b'
   is a mutable Rust variable that's assigned to the [StackedItem][sitem]
   popped from the parse stack, which includes .value, .line and .column.

   2. **`E:(a,b)`**: The label can also be a simple, irrefutable pattern
      enclosed in parentheses, which are required even if the pattern is a single
      variable.  Furthermore, (currently) no whitespaces are allowed in the pattern.
      The pattern is bound directly to the .value of the StackedItem popped from
      the stack.  One can recover the line/column information in several
      ways: it is recommended to use [LBox][2] using
      the [ZCParser::lbx][4] or the [StackedItem::lbox][5] functions, or a
      labeled pattern such as `E:[a]`.
      The [StackedItem::lbox][5] function transforms a [StackedItem][sitem]
      into an LBox.
      The [ZCParser::lbx][4] function takes an index and an expression  and produces an LBox.  The index indicates the position, starting
      from zero, of the grammar
      symbol on the right-hand side of the production that the value is
      associated with.  For example, the rule for `E --> E + E` can also be
      written as

        `E --> E:(a) + E:(b) { Plus(parser.lbx(0,a), parser.lbx(2,b)) }`


   3. **`E:[a]`**:  If an alphanumeric label is enclosed in square brackets,
      then the [StackedItem][sitem] is automatically converted into an [LBox][2]
      encapsulating the value and the lexical position.  This form is most
      convenient in the majority of cases if the abstract syntax uses LBox
      in its recursive definitions.  The labels are already LBoxes:
   
        `E --> E:[a] + E:[b] { Plus(a,b) }`   


   4. **`E:@Seq(mut v)@`**: as seen in this grammar.  This pattern is if-let
      bound to the **.value** popped from the stack as a mutable variable (the .value is moved to the pattern).  The
      specified semantic action is injected into the body of if-let.  A parser
      error report is generated if the pattern fails to match, in which
      case the default value of the abstract syntax type is returned.
      To be precise, the semantic action function generated for the last rule of the
      grammar is
      ```
      |parser|{ let mut _item2_ = parser.popstack();
         let mut e = parser.popstack(); let mut _item0_ = parser.popstack(); 
         if let (Seq(mut v),)=(_item0_.value,) { 
           v.push(e.lbox());
           Seq(v)
          }  else {parser.bad_pattern("(Seq(mut v),)")} }
      ```
      Rustlr generates a variable of the form `_item{n}_` to hold the
      value of the [StackedItem][sitem], if no direct label is specified.
      Notice that `_item0_.value` is *moved* into the pattern so generally
      it cannot be referenced again.

   5. **`E:es@Seq(v)@`**  The pattern can be named.  'es' will be a mut variable
      assigned to the StackedItem popped from the stack and an if-let is
      generated that attempts to match the pattern to **`&mut es`**.
      The named label can also be in the form **`[es]`**, which will transform
      the StakedItem into an LBox assigned to es: in this case, the pattern is bound to
      **`&mut *es`**.

   For example, the last production rule of this grammar is equivalent to:
      ```
      ES --> ES:es@Seq(v)@  E:e ;  {
         v.push(parser.lbx(1,e.value));
         es.value
      }   
      ```
      In contrast to a non-named pattern, the value is **not** moved into the
      pattern, which means we can still refer to it as `es.value`.  The call
      to [parser.lbx][4] requires an index, starting from 0, of the grammar symbol
      on the right-hand side of the production along with a value and forms
      an LBox with starting line/column information.  In this case, it is
      equivalent to `v.push(e.lbox())`: the .lbox function converts the
      [StackedItem][sitem] to an [LBox][2].  But calling .lbox is only possible because 
      this form of pattern does not move the .value out of the StackedItem.


#### The Abstract Syntax Type **Expr**

To see how [LBox][2] can be used after the parsing stage, let's take a close look at the definition of the abstract syntax type:
```ignore
pub enum Expr<'t>
{
   Var(&'t str),
   Val(i64),
   Plus(LBox<Expr<'t>>,LBox<Expr<'t>>),  // LBox replaces Box for recursive defs
   Times(LBox<Expr<'t>>,LBox<Expr<'t>>),
   Divide(LBox<Expr<'t>>,LBox<Expr<'t>>),
   Minus(LBox<Expr<'t>>,LBox<Expr<'t>>),
   Negative(LBox<Expr<'t>>),
   Letexp(&'t str,LBox<Expr<'t>>,LBox<Expr<'t>>), // let x=Expr in Expr
   Seq(Vec<LBox<Expr<'t>>>),
   Nothing,
} 
```
The variant `Nothing` allows us to define a default, which is required for
any 'absyntype' of the grammar:
```
impl Default for Expr<'_>  {
  fn default() -> Self { Nothing }
}//impl Default
```

Unlike in the first example, here evaluation is defined after the parsing stage,
when the abstract syntax tree is available as a complete structure.  'Let'-expressions, which introduce variables to the language, can only be
evaluated given a set of bindings for the variables.  This "environment"
structure is defined below:
```
pub enum Env<'t> {
  Nil,
  Cons(&'t str, i64, Rc<Env<'t>>)
}
fn push<'t>(var:&'t str, val:i64, env:&Rc<Env<'t>>) -> Rc<Env<'t>>
{ Rc::new(Cons(var,val,Rc::clone(env))) }
fn lookup<'t>(x:&'t str, env:&Rc<Env<'t>>) -> Option<i64>  {
    let mut current = env;
    while let Cons(y,v,e) = &**current {
      if &x==y {return Some(*v);}
      else {current = e;}
    }
    return None;
}//lookup
```
Since this tutorial is about the parser generation stage and not so much about
later stages of interpretation/compilation, I will not go into too
much detail as to how such a data structure is needed.  It defines a non-mutable
linked list, with a constructive `cons`, that we use to emulate
lexical scoping.  The Env enum also allows lists to share components
(different 'car', same 'cdr').  The `lookup` function looks up the value
bound to a variable in an enviornment.


The evaluation function is given below.  Sequences of expressions
(under the `Seq` variant) are evaluated one after the other with their
results printed, and the value of the last expression of the sequence is
returned.
Note that [LBox][2] is used in the same way as a Box in most of the cases except for 
Division. Here we access the line and column numbers enclosed inside the LBox to print
an error message when division-by-zero is detected.
```
pub fn eval<'t>(env:&Rc<Env<'t>>, exp:&Expr<'t>) -> Option<i64>  {
   match exp {
     Var(x) => {
       if let Some(v) = lookup(x,env) {Some(v)}
       else { eprint!("UNBOUND VARIABLE {} ... ",x);  None}
     },
     Val(x) => Some(*x),
     Plus(x,y) => eval(env,x).map(|a|{eval(env,y).map(|b|{a+b})}).flatten(),
     Times(x,y) => eval(env,x).map(|a|{eval(env,y).map(|b|{a*b})}).flatten(),
     Minus(x,y) => eval(env,x).map(|a|{eval(env,y).map(|b|{a-b})}).flatten(),
     Negative(x) => eval(env,x).map(|a|{-1*a}), //no need for bind here    
     Divide(x,y) => {
       eval(env,y)
       .map(|yval|{if yval==0 {
          eprint!("Division by zero (expression starting at column {}) on line {} of {:?} at column {} ... ",y.column(),y.line(),x,x.column());
	  None
         } else {eval(env,x).map(|xval|{Some(xval/yval)})}
       })
       .flatten().flatten()
     },
     Letexp(x,e,b) => {
       eval(env,e).map(|ve|{
         let newenv = push(x,ve,env);
         eval(&newenv,b) }).flatten()
     }
     Seq(V) => {
       let mut ev = None;
       for x in V
       {
         ev = eval(env,x);
         if let Some(val) = ev {
	   println!("result for line {}: {} ;",x.line(),&val);
         } else { eprintln!("Error evaluating line {};",x.line()); }
       }//for
       ev
     },
     Nothing => None,
   }//match
}//eval
```
For those not familiar with the monadic functors (map and flatten), 
the clause for `Plus`, for example, is equivalent to

`if let Some(a)=eval(env,x) { if let Some(b)= eval(env,y) {Some(a+b)} else {None} } else {None}`.

<br>

#### **Lexical scanner and main**

The directives **lexvalue**, **lexname** and **lexattribute** are used to
configure an automatically generated lexical tokenizer.
'lexvalue' should be used 
for all terminal symbols that carry (non-default) semantic values, such as
numerical constants and string literals.  The two **lexvalue** directives
state that "int" terminal symbols ([TerminalTokens][tt]) are created
from RawToken::Num(n) and carray semantic values Val(n), while
"var" terminals carry values Var(x) and are formed from RawToken::Alphanum(x).
The generated lexer will distinguish alphanumeric tokens that correspond
to other declared terminal symbols of the grammar such as "let".
All other "Alphanums" will be parsed as "var".

The [RawToken][rtk] enum defines n to be an i64 and x to be a &str.
Generally speaking, for each value-carrying terminal symbol of the grammar,
write a similar *lexvalue* declaration by identifying the type of RawToken
that it corresponds to and how to construct the semantic value from the token.

Please note that
whitespaces are only allowed in the specification of the semantic value (in the
future we may rewrite rustlr inside rustlr, but this is not a priority as the
hand-coded grammar parser suffices for most cases if used a little carefully).

Since no reserved symbols such as "|" or "{" are used in this language, the
**lexname** directive is not used.  These reserved symbols cannot be used
as terminal symbols.  One must choose names such as VBAR and write
`lexname VBAR |` to define the correspondence between the grammar symbol
and its textual form.

The **lexattribute** directive can be used to set any attribute on the
lexer to be generated.  Consult the docs for [StrTokenizer][1]. 
The following samples are valid lexattribute declarations

>      lexattribute keep_newline = true
>      lexattribute keep_comment = true
>      lexattribute keep_whitespace = true
>      lexattribute set_multiline_comments("/* */")
>      lexattribute set_line_comment("")

Setting the line_comment or multiline_comments to the empty string will mean
that such comments are not recognized.  The keep_flags are all false be
default.  [StrTokenizer][1] recognizes C-style comments by default.

The presence of these directives automatically enables the -genlex option.
The lexer created is called calc4lexer and is found in with the generated
parser.  Use the **calc4lexer::from_str** and **calc4lexer::from_source**
functions to create instances of this zero-copy lexical scanner (consult 
[main.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/calc4/src/main.rs)
for example).

Generate the parser with

> rustlr calc4.grammar -trace 3 > calculator.states

This creates a file [calc4parser.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/calc4/src/calc4parser.rs), although each time it's generated
the state numbers may be different: the -trace 3 option prints these states
to stdout.
Create a cargo crate with the following dependency in Cargo.toml:
```
rustlr = "0.3"
```
copy the [main.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/calc4/src/main.rs), [exprtrees.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/calc4/src/exprtrees.rs) and the generated [calc4parser.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/calc4/src/calc4parser.rs) files into src/.  The supplied main parses and evaluates the following input:
```
-5-(4-2)*5;
#3(1+2);   # syntax (parsing) error
#5%2;   # syntax error (% is not defined by grammar)
5-7- -9 ; 
4*3-9; 
2+1/(2-1-1);  # division by 0 (semantic) error
let x = 10 in 2+x;
let x = 1 in (x+ (let x=10 in x+x) + x);
(let x = 2 in x+x) + x;  # unbound variable (semantic) error
(let x = 4 in x/2) + (let x=10 in x*(let y=100 in y/x));
```
**cargo run** produces the following output:
```
Expression tree from parse: Seq([Minus(Negative(Val(5)), Times(Minus(Val(4), Val(2)), Val
(5))), Minus(Minus(Val(5), Val(7)), Negative(Val(9))), Minus(Times(Val(4), Val(3)), Val(9
)), Plus(Val(2), Divide(Val(1), Minus(Minus(Val(2), Val(1)), Val(1)))), Letexp("x", Val(1
0), Plus(Val(2), Var("x"))), Letexp("x", Val(1), Plus(Plus(Var("x"), Letexp("x", Val(10),
 Plus(Var("x"), Var("x")))), Var("x"))), Plus(Letexp("x", Val(2), Plus(Var("x"), Var("x")
)), Var("x")), Plus(Letexp("x", Val(4), Divide(Var("x"), Val(2))), Letexp("x", Val(10), T
imes(Var("x"), Letexp("y", Val(100), Divide(Var("y"), Var("x"))))))])
---------------------------------------

result for line 1: -15 ;
result for line 4: 7 ;
result for line 5: 3 ;
Division by zero (expression starting at column 5) on line 6 of Val(1) at column 3 ... Error evaluating line 6;
result for line 7: 12 ;
result for line 8: 22 ;
UNBOUND VARIABLE x ... Error evaluating line 9;
result for line 10: 102 ;
Final result after evaluation: Some(102)
```

This chapter is followed by an **[addendum](https://cs.hofstra.edu/~cscccl/rustlr_project/minijava/mj.grammar)**, which contains a larger example with further
illustration of the techniques explained here.

----------------

### Training The Parser For Better Error Reporting

It is recommended that, when a parser is generated, the -trace 3 option is
given, which will print all the LR states that are created. This may be helpful
when training the parser.  Each time the parser is regenerated the states may
have different numbers identifying them, even if the grammar is unchanged.

With a newly generated parser, when a parser error is encountered, the
line and column numbers are printed and an "unexpected symbol" error
message is given. To print more helpful error messages, the parser can
be trained interactively.  Interactive training also produces a script
for future, automatic retraining when a new parser is generated.

Modify [main.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/calc4/src/main.rs) by uncommenting lines 2 and 3 in the input:
```
3(1+2)   # syntax (parsing) error
5%2;   # syntax error
```
Note that the supplied main already calls `parse_train(&mut scanner2,"calc4parser.rs");`  For input with no errors, this call works the same way as `parse(&mut scanner2);`  The [parse_train](https://docs.rs/rustlr/latest/rustlr/runtime_parser/struct.RuntimeParser.html#method.parse_stdio_train) function takes a path to a copy of the parser being trained (it's not recommended to change the copy that
you're using this way).
Cargo run will lead to the following (possible) training session, depending on
user input:
```
PARSER ERROR: unexpected symbol ( on line 2, column 2 ..

>>>TRAINER: if this message is not adequate (for state 1), enter a replacement (default n
o change): missing an operator symbol such as *
>>>TRAINER: should this message be given for all unexpected symbols in the current state?
 (default yes) no
PARSER ERROR: unexpected symbol % on line 3, column 2 ..

>>>TRAINER: if this message is not adequate (for state 1), enter a replacement (default n
o change): this symbol is not recognized as a valid operator in this language
Expression tree from parse: Seq([Minus(Negative(Val(5)), Times(Minus(Val(4), Val(2)), Val
(5))), Minus(Minus(Val(5), Val(7)), Negative(Val(9))), Minus(Times(Val(4), Val(3)), Val(9
)), Plus(Val(2), Divide(Val(1), Minus(Minus(Val(2), Val(1)), Val(1)))), Letexp("x", Val(1
0), Plus(Val(2), Var("x"))), Letexp("x", Val(1), Plus(Plus(Var("x"), Letexp("x", Val(10),
 Plus(Var("x"), Var("x")))), Var("x"))), Plus(Letexp("x", Val(2), Plus(Var("x"), Var("x")
)), Var("x")), Plus(Letexp("x", Val(4), Divide(Var("x"), Val(2))), Letexp("x", Val(10), T
imes(Var("x"), Letexp("y", Val(100), Divide(Var("y"), Var("x"))))))])
---------------------------------------

result for line 1: -15 ;
result for line 4: 7 ;
result for line 5: 3 ;
Division by zero (expression starting at column 5) on line 6 of Val(1) at column 3 ... Er
ror evaluating line 6;
result for line 7: 12 ;
result for line 8: 22 ;
UNBOUND VARIABLE x ... Error evaluating line 9;
result for line 10: 102 ;
Parser error, best effort after recovery: Some(102)
```
Notice that error recovery was effective and the parser still produced a usable
parse tree: however, the parser's error_occurred flag will be set.  It is
under consideration as to whether future editions of Rustlr will also allow the
error-recovery strategy to be trainable in the same way.  For now, only a fixed
number of strategies are available.  In the opinion of the author, the resync
technique is the simplest and most effective.

If the augmented parser is used on the same input, it will display the trained
message in addition to "unexpected symbol..."

You can see how training augments the LR state transition table by
examining the `load_extras` function at the end of the generated parser:
```
fn load_extras(parser:&mut RuntimeParser<Expr,Expr>)
{
  parser.RSM[1].insert("(",Stateaction::Error("missing an operator symbol such as *"));
  parser.RSM[1].insert("ANY_ERROR",Stateaction::Error("this symbol is not recognized as a
 valid operator in this language"));
}//end of load_extras: don't change this line as it affects augmentation
```
When the "unexpected symbol" is recognized as a declared symbol of the grammar, the trainer will be given the option of entering the error message for either
just that symbol, or all unexpected symbols in the same state.  If the latter is
chosen then an entry is created for the reserved `ANY_ERROR` symbol.  If the
unexpected symbol is not recognized as a terminal symbol of the grammar, an
`ANY_ERROR` entry is always created.  You can see the contents of "state 1"
if you created it with the -trace 3 option. You will of course have to understand the LR parsing algorithm to make use of the information.

When the modified parser runs and encounters another unexpected symbol in the
same state, it will first see if there is an entry for that symbol; if none
exists, it will look for an `ANY_ERROR` entry for a message to display.
Thus the two entries do not conflict with eachother.

The interactive session also generated a script file, which would be called
*"calc4parser.rs_script.txt"*, with the following contents:
```
# Rustlr training script for calc4parser.rs

2       2       ( ::: missing an operator symbol such as *
3       2       ANY_ERROR ::: this symbol is not recognized as an operator in this language
```
This script can be used to retrain a newly genenerated parser (with different state numbers) with the [train_from_script](https://docs.rs/rustlr/latest/rustlr/runtime_parser/struct.RuntimeParser.html#method.train_from_script) function
provided the same input from the original training.  The line and column numbers
of where the errors are expected are recorded in the script.  Please note that
training from script has not yet been tested on a large scale.  

------------


[1]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.StrTokenizer.html
[2]:https://docs.rs/rustlr/latest/rustlr/generic_absyn/struct.LBox.html
[3]:https://docs.rs/rustlr/latest/rustlr/generic_absyn/struct.LRc.html
[4]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.ZCParser.html#method.lbx
[5]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.StackedItem.html#method.lbox
[sitem]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.StackedItem.html
[chap1]:https://cs.hofstra.edu/~cscccl/rustlr_project/chapter1.html
[chap3]:https://cs.hofstra.edu/~cscccl/rustlr_project/chapter3.html
[chap4]:https://cs.hofstra.edu/~cscccl/rustlr_project/chapter4.html
[lexsource]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.LexSource.html
[drs]:https://docs.rs/rustlr/latest/rustlr/index.html
[tktrait]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/trait.Tokenizer.html
[tt]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html
[rtk]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/enum.RawToken.html
[fromraw]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html#method.from_raw
[nextsymfun]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/trait.Tokenizer.html#tymethod.nextsym
[zcp]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.ZCParser.html
[ttnew]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html#method.new
