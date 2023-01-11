## Special Chapter: Generating Parsers for F\# using Rustlr and Fussless

Rustlr can generate parsers for F\#.  With .Net interoperability,
other languages (C\#) can also use the generated parsers with a little
adaptation, though some knowledge of F\# is required.  The .Net side
of this aspect of Rustlr is a system called **[Fussless][fussless]**.
This repository contains the runtime parser written in F\#.  The lexical
analysis aspect of Fussless uses [CsLex][cslex], which is written in C\#.
Fussless can automatically generate a CsLex .lex file from the grammar.
Download Fussless and follow instructions in the [Fussless README][fussless]
to install the system.  If you're not using the latest Mono, you may have
to re-compile absLexer.cs into a .dll, and then, using that
.dll, compile RuntimeParser.fs to a .dll.

At the time of this writing, there are still some features missing from
Fussless compared to the native Rust parser generator. There is
only one, simple error-recovery mechanism (`resynch`). The experimental -lrsd
option and the wildcard symbol are not currently supported 
and the interactive training feature is also not available.
These limitations will gradually be resolved with future releases.

As of Rustlr release 4.0, the Fussless system can now automatically
generate the abstract syntax types and semantic actions from a grammar with
the `auto` option.  However, the first part of this chapter will show how
to write grammars with manually defined types and actions.

### Invoking the Parser Generator

To create a parser, you will first need a .grammar file.  Rustlr/Fussless
has its own format for specifying grammars:
```ignore
# Unambiguous LR grammar for simple calculator.

valuetype int
nonterminals E T F
terminals + *
valueterminal number ~ int ~ Num ~ int
lexterminal LPAREN (
lexterminal RPAREN )
topsym E

E --> E:e + T:t { e + t }
E --> T:t { t }
T --> T:t * F:f { t*f }
T --> F:f { f }
F --> LPAREN E:e RPAREN  { e }
F --> number:n { n }

EOF
```
These are the contents of a Fussless grammar file, called [test1.grammar](https://cs.hofstra.edu/~cscccl/rustlr_project/fstarget/test1.grammar).
This classic example of LR parsing is found in virtually all compiler
textbooks.  After you **`cargo install rustlr`**
you can produce a LALR(1) parser from this grammar file with:

>  rustlr test1.grammar -fsharp

The first and the only required argument to the executable is the path of the
grammar file.  However, without the -fsharp option it will try to create a
parser for Rust.  Other optional arguments (after the grammar path) that can be
given to the executable are:

- **-lr1** : this will create a full LR(1) parser if LALR does not suffice.
  The default is LALR(1), which works for most examples.  A sample grammar
  requiring full LR(1) can be found **[here](https://cs.hofstra.edu/~cscccl/rustlr_project/nonlalr.grammar).**
  Rustlr will always try to resolve shift-reduce conflicts by precedence and associativity
  declarations (see later examples) and reduce-reduce conflicts by rule order.
  So it will generate some kind of parser in any case.  However, unless it's
  understood clearly what's causing the conflict, default resolutions should
  not be accepted: rewrite the grammar instead.
- **-o filepath** : changes the default destination of the generated parser, which is
  a F\# program file called test1parser.fs.  This program must be compiled
  with RuntimeParser.dll.
- **-genlex** : automatically generates a lexical scanner in the form of
a CsLex .lex file.  The genlex option is
also automatically enabled by the presence of certain declarations in the
grammar file, such as **`lexterminal`** or **`valueterminal`**.  A file
called [test1.lex](https://cs.hofstra.edu/~cscccl/rustlr_project/fstarget/test1.lex) will be created.  This file must be processed by *lex.exe*, which is the
CsLex executable: this in turn generates test1_lex.cs, which must be compiled
with absLexer.dll.
- **-trace n**  : where n is a non-negative integer defining the trace level.
  Level 0 prints nothing; level 1, which is the default, prints a little more
  information.  Each greater level will print all information in lower levels.
  -trace 3 will print the states of the LR finite state machine, which could
  be useful for debugging and training the parser for error message output.

The generated parser will be a program
[test1parser.fs](https://cs.hofstra.edu/~cscccl/rustlr_project/fstarget/test1parser.fs)
that contains a **`parse_with`** function.  RustLr will derive the
name of the grammar (test1) from the file path, unless there is a
declaration of the form

>  grammarname somename

in the grammar spec, in which case the parser generated will be called
"somenameparser.fs".

###  GRAMMAR FORMAT


#### Valuetype

A context free grammar is not very useful unless we associate *values*
with grammar symbols.  Without these values all a parser can do is
tell us if something parsed or not.  The first line in the grammar
specification should define the default type of value carried by each
grammar symbol.  Not all symbols need to have values of the same
type.  However, the "start symbol" or "topsym" of the grammar must
have this type, and this is the type of value that's ultimately
returned by the parser.
 
>  valuetype int  

(alternatively `absyntype int`).
In most cases the type would be some type that defines an abstract syntax
tree, but here we will just calculate an int.  

#### Declaring Terminal and Nonterminal symbols.

Rustlr requires that all terminal and non-terminal symbols be declared
before writing any grammar rules.  Terminal symbols can be divided
into two categories: those that carry important values, and those that
do not.  In this example, the only terminal symbol with imporant values
is *number*, which carries values of type *int*.  Such terminals should be
declared using a **valueterminal** line, which has the following format

>      valueterminal terminal_name ~ terminal_type ~ token_name ~ fun:string->terminal_type

The four elements of the declaration must be separated by `~`.  The
terminal_name declares that this is a terminal symbol, with values of
type terminal_type.  The next two fields allow for a lexical scanner
to be generated that recognizes these terminals.  Fussless
automatically creates a .lex file that returns lexical tokens of type
*RawToken* (defined in absLexer.cs).  Each RawToken carries a string
(token_name) that defines the type of the token and a string (token_text)
that defines the text of the token.  The lexer pre-defines a category for
(unsigned) integers as token_type "Num".  Thus the third argument to
*valueterminal* is the lexer token
type (not to be confused with the terminal_name, which is what the grammar
will refer to).  The last component of a valueterminal declaration is a
*function* of type *string -> terminal_type*.  The *int* function in F\#
converts strings to integers: *int("32")* returns the integer 32.  You can
also write `(fun x -> int x)`.  This function will be applied to the token
text to produce the value expected by the terminal symbol.

The lexical scanner generated by Rustlr recognizes other token types
including "Float", "Alphanum" and "StrLit", which will be discussed in
a later section.

In contrast, terminals such as +, * ( and ) do not carry significant values:
they will always be assigned Unchecked.defaultof\<valuetype\> just as a filler.
These terminals can be defined in one of two ways.

  1. if the name of the terminal is the same as the text of the terminal,
  they can be defined on a *terminals* line (multiple lines are allowed).
  2. if the name of the terminal is different from the textual form, use
  a *lexterminal* declaration.  These are required for certain symbols that
  are reserved for other uses in Rustlr, including { } | : and a few others.
  The parentheses are also best not used to name terminals by themselfs.
  Thus `lexterminal LPAREN (` means that we will refer to the terminal as
  LPAREN in the grammar and the lexical analyzer will recognize "(" as
  this type of token.

Nonterminal symbols that are to have the same type as the declared valuetype
of the grammar can be defined on one `nonterminals` line.  You should use only
alphanumeric names for non-terminals (Rustlr is also not guaranteed to work
with non-ascii characters).  In this example all nonterminals have type int, so
one such line suffices.  Otherwise declare differently typed non-terminals
using lines such as `nonterminal S string`.


####  Top Nonterminal
>  topsym E

(alternatively startsymbol E). You should designate one particular
nonterminal symbol as the top symbol.  This symbol must have the same
type (for its value ) as the declared 'valuetype' so you should not
try to assign it a different type.


####  Grammar Production Rules

You will get an error message if the grammar symbols are not defined
before the grammar rules.  Each rule is indicated by a non-terminal
symbol followed by `-->`  or `==>`. The symbol `==>` is for rules that 
span multiple lines: they must be terminated with `<==`.  You can specify
multiple production rules with the same left-hand side nonterminal
using | but Rustlr discourages their use.

The right-hand side of each rule must separate symbols with
whitespaces.  For each grammar symbol such as E, you can optionally
bind a "label" such as `E:a`,   The label 'a' refers to the
value associated with this occurrence of E.

The right-hand side of a rule may be empty, which will make the
non-terminal on the left side of `-->` "nullable".
           
####  SEMANTIC ACTIONS

Values for non-terminal symbols are returned by functions commonly referred
to as *semantic actions*. 
Each rule can optionally end with a semantic action inside \{ and \},
which can only follow all grammar symbols making up the right-hand
side of the production rule.  This is a piece of F\# code that will form
the body of the semantic action function.  This code will have
access to any labels associated with the symbols defined using ":".
In a label such as `E:e`, e is a mutable variable intialized to the value
associated with E.  

The semantic action of each rule must return a value of the type associated
with the left-hand side symbol of that rule.  Generally speaking,
the semantic action of a rule `A --> B:b C:c D:d` is a function that
`f` that takes as arguments value of the types for `B`, `C` and `D`
and `f(b,c,d)` will be the value associated with `A`.

It is recommended that if you use multiple lines that you start the
semantic action on a new line after the openning `{`.  Be reminded
that F\# doesn't use braces to group code (they're used to form
records).  The braces are just Rustlr syntax to separate the semantic
action from the rest of the grammar rule.

The semantic action code is injected verbatim into the generated parser,
thus any errors in the code will not show up until you try to compile
the parser.  For security reasons it's generally not a good idea to run 
programs like parser generators with systems privileges.

If no semantic action is given, a default one is created that just returns
a default value.

#### Error Reporting

Semantic actions always have access a parameter named 'parser'.  The
functions that can be called on parser are **report_error** and
**abort**.  For example, `parser.abort("failure")` or
`parser.report_error("problem encountered",true)`.  The `report_error`
function takes a boolean argument that determines if line/column
numbers should be displayed.  The `abort` function terminates parsing.

Another important function that can be called on `parser` is parser.position,
which returns a pair (line,column) that's associated with one of the symbols
on the right-hand side of a rule: the exact symbol is indicated as a 0-based
integer that's passed to parser.position.  An example will make this clear:
the rule for multiplication can be replaced with
```
T ==> T:t * F:f {
  let tf = t*f
  if f<>0 && (tf/f <> t) then
    let (ln,cl) = parser.position(1)
    printfn "Warning: arithmetic overflow line %d, column %d" ln cl
  t*f
  } <==
```
The argument (1) passed to parser.position refers to the `*` symbol, that
is the 2nd symbol on the right-hand side.  Index 0 will refer to the T and
index 2 will refer to F.  (0,0) will be returned for an invalid index.

Note also that this rule spans multiple lines and requires ==> and <==. Also,
the injected multi-line F\# code should start on a new line and be indented.

The three member functions on parser described above are the only ones that
should be called from semantic actions.  There are other functions that would
corrupt the parser and should never be called.  In general, whatever code
you write inside the braces are entirely your own responsibility.

#### LBox

  Not all errors are parsing errors.  After the AST is successfully
built, other phases usually follow that perform semantic analysis such as
type checking.  Errors detected in later stages must also be reported
with line/column numbers indicating their origin.  The AST therefore must
carry this information. Fussless defines a structure *LBox* that encapsulates
a value along with line and column information:
```
type LBox<'AT> =
  {
    value: 'AT;
    line : int;
    column: int;
  }
let lbox<'AT> (v:'AT,ln:int,cn:int) = { LBox.value =v; line=ln; column=cn; }
let (|Lbox|) (b:LBox<'AT>) = Lbox(b.value)
```
The structure comes with two other definitions: *lbox* is an ordinary
constructor and *Lbox* is an *active pattern*.  The active pattern
allows the lexical information to be hidden: exposing
only the value within the box. ASTs can be defined using LBox as
demonstrated below:
```
type expr = Val of LBox<int> | Plus of LBox<expr>*LBox<expr> | Times of LBox<expr>*LBox<expr> | Divide of LBox<expr>*LBox<expr> 
```
The active pattern form *Lbox* allows pattern matching on these
structures without the intrusive line/column information, *except*
when we actually need them
```
let rec eval = function
  | Val(Lbox(x)) -> x
  | Plus(Lbox(a),Lbox(b)) -> (eval a) + (eval b)
  | Times(Lbox(a),Lbox(b)) -> (eval a) * (eval b)
  | Divide(Lbox(a),(Lbox(b) as n)) ->
    let bv = (eval b)
    if bv=0 then
       raise(Exception(sprintf "division by zero column %d\n" n.column))
    (eval a) / bv
```
Fussless has built-in support for creating LBoxes.  In a grammar production,
symbols on the right-hand side can be given "boxed labels".  For example:
```
E --> E:[e1] + T:[e2] { Plus(e1,e2) }
```
A boxed label such as `[e1]` instructs the parser to place the value
associated with the grammar symbol inside an LBox and to bind the variable
`e1` to it.

The LBox is named for its counterpart in Rust parsers created by Rustlr,
although it is not a "smart pointer".


#### **BUILDING AND INVOKING THE PARSER**

The steps for creating and calling a parser is best illustrated by the
following example ([test1main.fs](https://cs.hofstra.edu/~cscccl/rustlr_project/fstarget.test1main.fs)).  
```
module Test1
open System
open Fussless
open Test1

let parser1 = make_parser(); // create parser
Console.Write("Enter Expression: ");  
let lexer1 = test1lexer<unit>(Console.ReadLine());  // create lexer

let result = parse_with(parser1,lexer1); // invokes parser printfn
printfn "Result = %A" result;;
```
The lexical analyzer 'test1.lex' that's
generated automatically defines the C\# class 'test1lexer\<E\>'.  The
generic type argument E defines an "shared state' between the parser
and lexer.  By default, this type is unit.  The class comes with two
constructors: one taking a string, as used in the above program, and
one taking a System.IO.FileStream.

Assuming that [Fussless][fussless] has been downloaded and that 'absLexer.dll'
and 'RuntimeParser.dll' are available, compile the test1parser.fs file with
RuntimeParser.dll and the test1_lex.cs file (produced by lex.exe) with
absLexer.dll.  Note that both the generated parser and lexer are defined within
the 'Fussless' namespace and the parser is defined under the Test1 module, which
is why 'open Test1' is used: you would have to call Test1.make_parser and
Test1.parse_with otherwise.

Now compile the above test1main.fs program with the .dlls of the parser and
lexer.  Under mono this is done with

>       fsharpc test1main.fs -r test1parser.dll -r test1_lex.dll

which produces an executable.  Alternatively, there is a [Makefile](https://github.com/chuckcscccl/Fussless/blob/main/Makefile) included inside the Fussless
repository.  Consult the [Fussless Readme](https://github.com/chuckcscccl/Fussless/blob/main/README.md) for instructions.

The `parse_with` function must be passed instances of a parser and a lexer.
It returns an **option type** value of type **valuetype option**.


#### Injection of Top Level Code

A line that begin with '!' will be injected verbatim into
the generated parser.  Such lines will always be injected towards the beginning
of the code regardless of where they appear in the grammar.  Typically, these
lines will specify additional modules to open, such as
```
!open System.Collections.Generic;
```


#### Lexical Analyzer Directives

In order for Rustlr-Fussless to generate a .lex file, there must be at least
one 'lexterminal' or valueterminal' declaration in the grammar; otherwise 
rustlr must be invoked with the -genlex option.  

The generated .lex will recognize the following token types, including "Num"
that appeared in the 'test1' example

  - Alphanum: alphanumeric sequences starting with an alphabetical letter or
_ (underscore), and followed by zero or more alphabetical or numeric characters
or _.

  - Num: unsigned base-10 integers.  It is better to process negative 
    integers at the grammar level, lest "3-2" be recognized as two tokens 
    instead of three.

  - Hexnum: hexadecimal sequences starting with 0x

  - Float: unsigned floating point sequences

  - StrLit:  string literals

Note that the lexer will not check the returned tokens for overflow: that must
be done with the the function that you specify as the last argument to
'valueterminal'.

Besides the common types of tokens above, you can also define new token types
and their associated regular expressions:

>      lexattribute custom ULong [0-9]+UL

This defines a new token type that will be returned along with the text that
matched the given regex.  Such user-defined custom categories **will override
the other categories**.  This means that "205UL" will now be returned as a
single RawToken with token type "ULong" instead of two tokens, a "Num" and
an "Alphanum".  Multiple custom token types will be prioritized in the order
in which they appear inside the grammar.  

Once a custom token type is defined, **a valueterminal declaration is still
required** to translate such tokens into terminal symbols of the grammar, such
as
```
!let conv64 (x:string) :uint64 = System.UInt64.Parse(x.Substring(0,x.Length-2))
valueterminal U64 ~ uint64 ~ ULong ~ conv64
```

#### Other lexattribute directives

As of this writing, the only other lexattribute directive available is
`line_comment`.  By default, the generated lexer recognizes (and ignores)
C-style comments.  The line_comment directive can be
used to change the symbol for single-line comments, such as
```
lexattribute line_comment #
```
The symbol selected should be non-alphanumeric.  `lexattribute line_comment disable` will disable the recognition of single-line comments.


#### Operator precedence and associativity declarations.

Rustlr allows **left**, **right** and **nonassoc** declarations for terminal
symbols.  Each such declaration must specify a positive integer defining the
precedence levels.  These declarations are used to break shift-reduce conflicts
and allows the writing of some ambiguous grammars (`E --> E+E`) instead of 
(`E --> E+T`).   The default precedence is zero, which means no precedence has
been defined.  However, these kinds of declarations should not be overused
(see below).

#### Error Recovery

An LR parser is defined by a state action (transition) table and a stack of states.  The 
top of the stack is the current state. A parsing error occurs when
the current state has no entry defined for the next input.
Currently, only one method of error recovery has been implemented for F\# parsers. 
A declaration such as 

 ```
 resync SEMICOLON COMMA
 ```

designates one or more terminal symbols as *resynchronization points*.  When
an error occurs, the parser will skip input tokens until it finds one of
these points.  It then looks down its stack of states to find one that
has an entry for the *next* input symbol after the resynch
point, and continues parsing.  If no resync point is declared, the parser
will just skip input until it finds one that has an entry defined with 
respect to the current state.  A natural resync point is the semicolon
that separates statements in many languages.  If an error occurs, the
parser will skip past the semicolon and parse the next line.  

There are more sophisticated error recovery techniques that could be
implemented so this is currently a minimal feature.

Up on the detection of any error, the **parser.err_occurred** flag will
be set and it's up to the user to examine this flag before deciding what
to do with the result.  



#### Using Regular Expression-Like Operators +, \*, ?, etc

Rustlr allows grammar rules to be written in the following way:
```
E --> A* B+ C? D<,+> E<;*>
```
These regular-expression like operators serve to translate the grammar into the
following:
```
E --> As Bp Cq Dp Es
As -->  | As A
Bp --> B | Bp B
Cq -->  | C
Dp --> D | Dp , D
Es -->  | Ep
Ep --> E | Ep ; E
```
Furthermore, the semantic values associated with A*, B+ D<,+> and E<;*>
are always of type Vec<\_> (ResizeArray<\_>) and type for C? is
option<\_>, where _ represents the types of the respective
non-terminals.  The \*, + and ? operators have the same meaning as in 
regular expressions.  In <sym+> and <sym\*>, sym must be a terminal symbol.
These operations represent sequences separated by the terminal, but not
ending in the terminal.  For example:
```
function_call --> functional_name ( expression<,*> )
```
defines function calls with zero or more comma-separated arguments.

These operators are available as a convenience, but they come at a price.
The introduction of new production rules to a grammar increases the chance
of non-determinism even if the grammar remains unambiguous.  Rustlr does not
allow the regex-like operators to be *nested*: such expressions easily become
ambiguous.  Consider `(a?)+`:  a single `a` will have an infinite
number of parse trees because any number of `a?` can be empty.


#### A More Advanced Example

We consolidate the features of Fussless with a more advanced version
of an online calculator.  In addition to several new terminal token
types, this grammar approaches the sophistication of a programming
language with let-expression, as in `let x=1 in x+(let x=3 in x+x)+x`
(which should evaluate to 8).  Checking for the proper scoping of
variables, however, is typically not done at the parsing stage.
The language also allows a sequence of expressions separated by semicolons.
The semicolon (`;`) is also declared as the error-recovery resynch point.

This time, the parser will build abstract syntax trees, and we've injected
the AST discriminated union type directly into the parser, though usually
this is done separately in another module.  Discriminated unions and
pattern matching definitely give F\# and similar languages an advantage over
conventional languages when processing ASTs.  Even Rust cannot compete as
recursive types require smart pointers (Box) that prevent deep pattern matching.

```
!type expr = Val of int | Var of string | Float of float | Plus of expr*expr | Times of expr*expr | Minus of expr*expr | Divide of expr*expr | Negative of expr | Letexp of string*expr*expr | Equals of (expr*expr) | Uint of uint64;;
!
!let conv64 (x:string) :uint64 = System.UInt64.Parse(x.Substring(0,x.Length-2))

valuetype Vec<expr>
# Vec is defined in the Fussless namespace as an alias for ResizeArray

nonterminal E expr
nonterminal ES
terminals + - * / ( ) == = ;
terminals let in
valueterminal Val ~ int ~ Num ~ int
valueterminal Var ~ string ~ Alphanum ~ (fun x -> x)
valueterminal Float ~ float ~ Float ~ float
lexattribute custom U64 [0-9]+UL
valueterminal Ulong ~ uint64 ~ U64 ~ conv64

lexattribute line_comment #

topsym ES
resync ;

left * 500
left / 500
left + 400
left - 400
nonassoc = 200
right == 300

E --> Val:m { Val(m) }
E --> Var:s { Var(s) }
E --> Float:f { Float(f) }
E --> Ulong:u { Uint(u) }
E --> let Var:x = E:e in E:b {Letexp(x,e,b)}
E --> E:e1 + E:e2 { Plus(e1,e2) }
E --> E:e1 - E:e2 { Minus(e1,e2) }
E --> E:e1 * E:e2 { Times(e1,e2) }
E ==> E:e1 / E:e2 {
  if e2=Val(0) then
     let (ln,cl) = parser.position(2)
     printfn "Warning:obvious division by 0, line %d column %d" ln cl
  Divide(e1,e2)
  } <==

E --> E:e1 == E:e2 { Equals(e1,e2) }
E(600) --> - E:e { Negative(e) }
E --> ( E:e )  { e }
ES --> E<;+>:v ;? { v }

EOF
```
Note that the - (minus) symbol serves as both a unary and a binary operator.
As a unary operator, it should have precedence over \*.
This means that using operator precedence/associativity declarations for
the symbol is not enough.  "-3*5" should be parsed as (-3)\*5 and not as
-(3\*5): never mind that they both evaluate to -15: the point is that the
parse trees are different.  Thus, a precedence level can also be assigned
to a *rule*, which is done for the rule for unary minus.  Without a 
particular precedence assignment, a rule is assigned the precedence of the
highest precedence symbol it finds on the right-hand side.  Precedence declarations are definitely a *hack*, almost as bad as some parser generators that
claim to "work with any grammar".
They should not be overly relied on.  One place where it
is useful is in disambiguating the *dangling else* problem: assign "else"
a higher precedence than "if".  This will force a *shift* when an "else" is
encountered, which means that it will be associated with the nearest "if".


#### Using C\#

Using C\# is possible by virtue of .Net interoperability.  The abstract syntax
structures can be defined in C\# and the semantic actions to construct such
structures should generally not be difficult to call from F\#.  The 
integration of the .dlls from the different languages may face some challenges
depending on your development platform. On Mono there where some problems
importing a .dll compiled with F\# into a C\# project.  But these problems
can be mostly avoided by writing some minimal components of the parser in 
F#.  

<br>

### **Automatically Generating the Abstract Syntax**

With Rustlr 0.4 and the latest Fussless the grammar can now generate the
abstract syntax types and semantic actions automatically.  Manual overrides
are also possible.  Essentially, non-terminal symbols that are on the
left-hand side of multiple productions generate discriminated unions while
those with a single production generate records.  However, the AST types do 
not necessarily just mirror the grammar. For
example, non-terminal symbols such as E, T and F (of the calculator grammar)
can be specified to define a single union type as opposed to individual
types.  Records can be absorbed or "flattened" into other types.
Rustlr/Fussless grammars contain a sub-language that defines 
how ASTs are to be created that can also be stable across small changes to 
the grammar. The system has the same capabilities as described for
Rust parsers in **[Chapter 4][chap4]**.  In fact it is simpler since there
is no need for lifetimes and smart pointers.  Fussless *LBox* structures
are created in the same way as their counterparts in Rust, without the
pointer aspect.

Two sample grammars are available that demonstrate these abilities:

  1. [calcautofs.grammar](https://github.com/chuckcscccl/Fussless/blob/main/calcautofs.grammar).  This grammar describes another version of the calculator
grammar with `auto` types and actions and some overrides.  It also demonstrates the injection of code that precedes the automatically generated actions, and
the 'flattening' feature for structs.

  2. [fs7c.grammar](https://github.com/chuckcscccl/Fussless/blob/main/fs7c.grammar).  This grammar defines a simplified, typed functional
programming language that was used in a compilers class taught at Hofstra
University. 

To invoke the feature, replace the "valuetype" declaration with "auto"
at the top of the grammar.  In addition to the parser file, an `_ast.fs`
file will be created.  Code can be injected into the AST file with lines
beginning with `$`.


--------------

[1]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.StrTokenizer.html
[2]:https://docs.rs/rustlr/latest/rustlr/generic_absyn/struct.LBox.html
[3]:https://docs.rs/rustlr/latest/rustlr/generic_absyn/struct.LRc.html
[4]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.ZCParser.html#method.lbx
[5]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.StackedItem.html#method.lbox
[sitem]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.StackedItem.html
[chap1]:https://cs.hofstra.edu/~cscccl/rustlr_project/test1grammar.html
[chap4]:https://cs.hofstra.edu/~cscccl/rustlr_project/chapter4.html
[lexsource]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.LexSource.html
[drs]:https://docs.rs/rustlr/latest/rustlr/index.html
[tktrait]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/trait.Tokenizer.html
[tt]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html
[rtk]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/enum.RawToken.html
[nextsymfun]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/trait.Tokenizer.html#tymethod.nextsym
[zcp]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.ZCParser.html
[fromraw]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html#method.from_raw
[ttnew]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html#method.new
[regex]:https://docs.rs/regex/latest/regex/
[fussless]:https://github.com/chuckcscccl/Fussless
[cslex]:https://github.com/zbrad/CsLex
