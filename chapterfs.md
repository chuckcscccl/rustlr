## Special Chapter: Generating Parsers for F\# using Rustlr and Fussless

Rustlr can also generate parsers for F\#.  With .Net
interoperability, this implies that other .Net languages (C\#) can
also use the generated parsers with a little adaptation.  The .Net side
of this aspect of Rustlr is a system called **[Fussless][fussless]**.
This repository contains the runtime parser written in F\#.  The lexical
analysis aspect of Fussless uses [CsLex][cslex], which is written in C\#.
Fussless can automatically generate a CsLex .lex file from the grammar.
Download Fussless and compile absLexer.cs into a .dll, then, using that
.dll, compile RuntimeParser.fs to a .dll.

At the time of this writing, there are certain limitations to the
Fussless system compared to the native Rust parser generator. There is
no automatic creation of ASTs and the experimental -lrsd option is not
available.  The wildcard token _ also cannot be used in grammars.  The
F\# runtime parser has no significant error-recovery capability.
These limitations will gradually be resolved with future releases.

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
textbooks.  It is an unambiguous grammar.  After you **`cargo install rustlr`**
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
grammar symbols.  Not all symbols need to have values of the same
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
declared using a **valueterminal* line, which has the following format

>      valueterminal terminal_name ~ terminal_type ~ token_name ~ fun:string->terminal_type

The four elements of the declaration must be separated by `~`.  The
terminal_name declares that this is a terminal symbol, with values of
type terminal_type.  The next two fields allow for a lexical scanner
to be generated that recognizes these terminals.  Fussless
automatically creates a .lex file that returns lexical tokens of type
*RawToken* (defined in absLexer.cs).  Each RawToken carries a string
(token_name) that defines the type of the token and a string (token_text)
that defines the text of the token.  The lexer recognizes (unsigned) integers
as type "Num".  Thus the third argument to *valueterminal* is the lexer token
type (not to be confused with the terminal_name, which is what the grammar
will refer to).  The last component of a valueterminal declaration is a
*function* of type *string -> terminal_type*.  The *int* function in F\#
converts strings to integers: *int("32")* returns the integer 32.  You can
also write `(fun x -> int x)`.  This function will be applied to the token
text to produce the value expected by the terminal symbol.

In contrast, terminals such as +, * ( and ) do not carry significant values:
they will always be assigned Unchecked.defaultof<valuetype> just as a filler.
These terminals can be defined in one of two ways.

  1. if the name of the terminal is the same as the text of the terminal,
  they can be defined on a *terminals* line (multiple lines are allowed).
  2. if the name of the terminal is different from the textual form, use
  a *lexterminal* declaration.  These are required for certain symbols that
  are reserved for other uses in Rustlr, including { } | : and a few others.
  The parentheses are also best not used to name terminals by themselfs.
  Thus `lexterminal LPAREN (` means that we will refer to the terminal as
  LPAREN in the grammer and the lexical analyzer will recognize "(" as
  this type of token.

Nonterminal symbols that are to have the same type as the declared valuetype
of the grammar can be defined on one `nonterminals` line.  You should use only
alphanumeric names for non-terminals (Rustlr is also not guaranteed to work
with non-ascii characters).  In this example all nonterminals have int, so
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
symbol followed by `-->`, `::=` , or `==>`.  The symbol `::=` is
interpreted to be the same as `-->`.  `==>` is for rules that span
multiple lines: they must be terminaed with `<==`.  You can specify
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
the body of the semantic action function  This code will have
access to any labels associated with the symbols defined using ":".
In a label such as `E:e`, e is a mutable variable intialized to the value
associated with E.  

**The semantic action of each rule must return a value of the type associated
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

##### Error Reporting

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
  if tf<>0 && (tf/f<>t || tf/t<>f) then
    let (ln,cl) = parser.position(1)
    printf "Warning: arithmetic overflow line %d, column %d" ln cl
  t*f
  } <==
```
The argument (1) passed to parser.position refers to the `*` symbol, that
is the 2nd symbol on the right-hand side.  Index 0 will refer to the T and
index 2 will refer to F.  (0,0) will be returned for an invalid index.

Note also that this rule spans multiple lines and requires ==> and <==. Also,
the injected multi-line F# code should start on a new line and be indented.

The three member functions on parser described above are the only ones that
should be called from semantic actions.  There are other functions that would
corrupt the parser and should never be called.  In general, whatever code
you write inside the braces are entirely your own responsibility.



#### **BUILDING AND INVOKING THE PARSER**

A lexical scanner (aka "tokenizer", "lexer", etc) can either be
created manually by implementing the [Tokenizer][tktrait] trait, or be
generated automatically from a minimal set of declarations using the
built-in [StrTokenizer][1].  This tokenizer makes zero-copy of the
source. It is capable of recognizing multi-line string literals and
comments, alphanumeric and non alpha-numeric symbols, decimal and
hexadecimal constants, floating point constants.
It also has the option of returning newline and
whitespaces (with count) as tokens.  It returns the starting line and
column numbers of each recognized token.  But it has limitations and
may not be the best tokenizer for every scenario.  The process of adopting
another tokenizer for use by a Rustlr parser will be covered in a speparate
chapter.

For this grammar, a lexer is generated from a single declaration

>  lexvalue num Num(n) (n as i32)

This line states that a token of the form [RawToken][rtk]::Num(n)
should be recognized as the terminal grammar symbol "num", carrying
semantic value (n as i32) - because in Num(n), n is of type i64 and
the semantic value attached to each grammar symbol must be of the
declared *absyntype* (valuetype).  The rest of the lexical scanner is
derived from the declarations of terminal symbols in the grammar.

To understand what declarations are needed to generate a lexer in general,
the reader should become familiar with **[RawToken][rtk]**.  This is what
[StrTokenizer][1] returns.
The RawToken enum contains the following principal variants:

 - **Alphanum(&str)**: where the string represents an (ascii) alphanumeric
   symbol that does not start with a digit.  The underscore character is
   also recognized as alphanumeric.
 - **Symbol(&str)**: a string consisting of non alphanumeric characters such as "==",
 - **Num(i64)**: Both decimal and hexidecimals (starting
 with "0x") are recognized as Nums.  However, although the returned value is signed,
 a negative integer such as "-12" is recognized as a Symbol("-") followed by a Num(12),
 and thus must be recognized at the parser level.  Despite this, it is still more convenient
 to return the more generic signed form.  Also, "3u8" would be
 reconized as a Num(3) followed by an Alphanum("u8").
 - **Float(f64)**: like the case of Num, this represents unsigned, decimal floats.
 - **BigNumber(&str)**: Numbers that are too large for i64 or f64 are represented verbatim.
 - **Char(char)**: this represents a character literal in single quotes such as 'c'
 - **Strlit(&str)**: A string literal delineated by double quotes.  These strings can span multiple lines and can contain nested, escaped quotes.  **The
 surrounding double quotes are included in the literal**.
 - **Newline**: optional token indicating a newline character. These tokens
 are **not** returned by the tokenizer by default, but can be returned with
 the directive
   > lexattribute keep_newline = true
 - **Whitespace(usize)**: another optional token that carries the number of
   consecutive whitespaces.  This option is likewise enabled with
   > lexattribute keep_whitespace = true   
 - **Verbatim(&str)**: another optional token carrying verbatim text, usually
   comments.  Enable with
   > lexattribute keep_comment = true
   
   By default, [StrTokenizer][1] recognizes C-style comments, but this can
   be customized with, for example,
   > lexattribute set_line_comment("#")

 - **Custom(&'static str, &str)**: user-defined token type (since Version 0.2.95).  The static
   string defines the token type-key and the other string should point to raw text.
   This token type is intended to be paired with declarations such as
   > lexattribute add_custom("uint32",r"^[0-9]+u32")

   Text matching the given [regex][regex] will be returned as a
   Custom("uint32",_) token.  Please note that custom regular expressions
   should not start with whitespaces and will override all other token types.
   Multiple custom types are matched by the order in which they appear in
   the grammar file.  **Note: this is a change to the original feature
   introduced in version 0.2.95, in which they were 
   matched by the alphabetical ordering of their
   keys.**  An anchor (^) will always
   be added to the start of the regex if none is given.

The most important lexer-generation directive is **lexvalue**.  For
every terminal symbol in the grammar that carries a (non-default)
semantic value, typically numerical and string literals, a
lexvalue directive is needed to identify the corresponding
[RawToken][rtk] that represents the terminal and how to translate the
RawToken's value to the valuetype/absyntype value to be associated
with the terminal symbol.  The lexvalue directive must identify the
name of the terminal symbol, the RawToken form, and the valuetype
form that should be recreated from the RawToken.

Besides **lexvalue**, there are two other lexer-generation directives,
**lexname**, which allows the mapping of a reserved symbol such as `{`
to a terminal symbol (see below), and **lexattribute** which allows the
customization of the scanner. Further usage of these directives can be
found in other chapters and examples.

**Please note that malformed lexattribute declarations will only result
in errors when the generated parser is compiled.**

The generated lexer is a struct called test1lexer alongside the make_parser()
function inside the generated parser file.  One creates a mutable instance
of the lexer using the generated **`test1lexer::from_str`** and **`test1lexer::from_source`** functions.

Here is the [main.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/test1/src/main.rs) associated with this grammar, which forms a simple calculator.  Its
principal contents creates a parser, a lexer, and invokes the parser on
the first command-line argument.
```
mod test1parser;
use test1parser::*;
fn main() {
  let mut input = "5+2*3";
  let args:Vec<String> = std::env::args().collect(); // command-line args
  if args.len()>1 {input = &args[1];}
  let mut parser1 = make_parser(); // calls function in mod test1parser
  let mut tokenizer1 = test1lexer::from_str(input); //creates lexer
  let result = parser1.parse(&mut tokenizer1);
  println!("result after parsing {}: {}",input,result);  
}//main
```
Alternatively, we can choose to create a test1lexer from another source,
such as a file, with:
```
let source = rustlr::LexSource::new("file path").unwrap();
let mut tokenizer1 = test1lexer::from_source(&source);
```

An instance of the runtime parser is created by calling the **`make_parser`**
function.
Once a lexer has also been created, parsing can commence by calling

>      `parser1.parse(&mut tokenizer1)`

This function will return a value of type valuetype.  It will return a valuetype-value
even if parsing failed (but error messages will be printed).  After
.parse returns, you can also check if an error had occurred by calling
`parser1.error_occurred()` before deciding to use the valuetype result
that was returned.

An alternative way to invoke the parser is to call
```
let result = parse_with(&mut parser1, &mut tokenizer1)
.unwrap_or_else(|x|{println!("Parsing errors occurred; results not guaranteed");
 x});
```
The `parse_with` function returns a `Result<T,T>` where `T` is the valuetype/absyntype. 

To run the program, **`cargo new`** a new crate and copy
the contents of [main.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/test1/src/main.rs) and [test1parser.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/test1/src/test1parser.rs) to src/main.rs and src/test1parser.rs respectively.  Add to Cargo.toml
under [dependencies]:
```
rustlr = "0.3"  
```
**`cargo run "2+3*4"`** will print 14 and `cargo run "(2+3)*4"` will print 20.

<br><p>


#### **Reserved Symbols**

The following terminal symbols are reserved and should not be used in a grammar:

>      EOF  ANY_ERROR  _WILDCARD_TOKEN_  :  |  @  {  }  -->  ::=  ==>  <==  _

The following symbols should also NOT be used as non-terminals in your grammar:

>     START valuetype absyntype grammarname resync resynch topsym errsym 
>     nonterminal terminal nonterminals terminals lexvalue lexname typedterminal
>     left right externtype externaltype lifetime lexattribute
>     any symbol starting with `SEQ` or `NEW..NT` may potentially, but unlikely, cause conflict.

For example, if ":" is to be one of the terminal symbols of your
language, then you should call it something like COLON instead in the
grammar. You will then adopt your lexical analyzer so that ":" is
translated to COLON.  This can be accomplished with the directive
(if generating a lexer automatically):

>     lexname COLON :

This directive is equivalent to

>     lexvalue COLON Symbol(":") <valuetype>::default()

where valuetype refers to the declared valuetype.
Underneath, the ":" symbol is translated into a [TerminalToken][tt] with .sym="COLON" before sending the token to the parser. If you
want to treat a whitespace as a token your lexer must similarly
translate whitespaces.  For automatic lexer generation, use
something like the following:

>     lexvalue WHITESPACE Whitespace(n) value

assuming that WHITESPACE is a declared terminal symbol and "value" is
the value you want to be associated with the symbol (usually this is just
the valuetype::default()).  Whitespace(n) is a variant of [RawToken][rtk].

It is possible to combine a lexname declaration with the declaration of a
terminal symbol with

>     lexterminal COLON :

The symbol START and terminal EOF will always be added as additional
symbols to the grammar.  The other symbols that should not be used for
non-terminals are for avoiding clash with grammar directives.

The following identifiers (variable names) are reserved and should
only be used carefully from within the semantic actions of a grammar
production (rust code inside {}s):

-  **`parser`** : the code generated from the semantic actions is of the form
`|parser|{...}`.  The *parser* refers to the instance of the runtime 
parser [ZCParser][zcp].  It is valid to invoke certain functions on this object inside the
semantic actions, including [parser.report](https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.ZCParser.html#method.report) (to report an error message),
[parser.abort](https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.ZCParser.html#method.abort) and most importantly, [parser.lbx](https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.ZCParser.html#method.lbx), which forms an [LBox][2]
smartpointer by inserting into it line/column information that accompanies
an abstract syntax value (see next chapter).  However, there are other functions on parser that are
exported, but should only be called by the automatically generated portion of
the code.  For example, calling parser.popstack() would remove an extra
state/value from the parse stack and corrupt the core parsing algorithm.
-  **`_item0_, item1_, item{n}_`** : these variables may be generated
to hold the values that are popped from the stack.
- **`SYMBOLS, TABLE`**:  these are constant arrays holding essential information
about the LR state machine.
- function names **`make_parser`**, **`load_extras`**, **`_semaction_for_{n}_`**


#### **A self-contained example**

Most rustlr projects will consist of mulitple files: the .grammar file, a module
defining the abstract syntax type, a module defining a lexical analyzer, the
generated parser as another module, and presumably a main to launch the program.
In [this additional example](https://cs.hofstra.edu/~cscccl/rustlr_project/brackets/brackets.grammar),
enough code has been injected into the .grammar so that rustlr can generate a
relatively [self-contained program](https://cs.hofstra.edu/~cscccl/rustlr_project/brackets/src/main.rs), that includes a lexer and a main, and illustrates a
few extra features of Rustlr.

-----------

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
