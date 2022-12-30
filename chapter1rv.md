## Chapter 1: Unambiguous LR Grammar for Simple Calculator.

Please note that this tutorial has been rewritten for **[Rustlr version 0.3][drs]**,
Parsers created since version 0.1.3 remain compatible.  The original version of this chapter
is available [here](https://cs.hofstra.edu/~cscccl/rustlr_project/test1grammar0.html).

This tutorial is written for those with sufficient background in computer
science and in Rust programming, with some knowledge of context free grammars
and basic bottom-up parsing concepts.
Those who are already
familiar with similar LR parser generation tools may wish to skip to the
more advanced example in [Chapter 2](https://cs.hofstra.edu/~cscccl/rustlr_project/chapter2.html) or [Chapter 4][chap4].


The tutorial will start with a sample grammar.
```ignore
valuetype i32
nonterminals E T F
terminals + * ( ) num
topsym E

E --> E:e + T:t { e.value + t.value }
E --> T:t { t.value }
T --> T:(t) * F:(f) { t*f }
T --> F:(f) { f }
F --> ( E:e )  { e.value }
F --> num:n { n.value }

lexvalue num Num(n) (n as i32)

EOF
```

These are the contents of a Rustlr grammar file, called [test1.grammar](https://cs.hofstra.edu/~cscccl/rustlr_project/test1/test1.grammar).
This classic example of LR parsing is found in virtually all compiler
textbooks.  It is an unambiguous grammar.  After you **`cargo install rustlr`**
you can produce a LALR parser from this grammar file with:

>  rustlr test1.grammar

The first and the only required argument to the executable is the path of the
grammar file.  Optional arguments (after the grammar path) that can be
given to the executable are:

- **-lr1** : this will create a full LR(1) parser if LALR does not suffice.
  The default is LALR, which works for most examples.  A sample grammar
  requiring full LR(1) can be found **[here](https://cs.hofstra.edu/~cscccl/rustlr_project/nonlalr.grammar).**
  Rustlr will always try to resolve shift-reduce conflicts by precedence and associativity
  declarations (see later examples) and reduce-reduce conflicts by rule order.
  So it will generate some kind of parser in any case.  The next chapter will
  explain in detail how conflicts are resolved.
- **-o filepath** : changes the default destination of the generated parser, which is
  a file called [test1parser.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/test1/src/test1parser.rs).
- **-genlex** : automatically generates a lexical scanner using the built-in
[StrTokenizer][1].  Manually constructing a scanner is also
possible and will be the subject of a future chapter.  The genlex option is
also automatically enabled by the presence of certain declarations in the
grammar file, such as **`lexvalue`**.
- **-auto** or **-genabsyn** : automatically generates abstract syntax data types and required semantic actions.  See [Chapter 4](https://cs.hofstra.edu/~cscccl/rustlr_project/chapter4.html).  This feature is not recommended for beginners.
- **-trace n**  : where n is a non-negative integer defining the trace level.
  Level 0 prints nothing; level 1, which is the default, prints a little more
  information.  Each greater level will print all information in lower levels.
  -trace 3 will print the states of the LR finite state machine, which could
  be useful for debugging and training the parser for error message output.
- **-nozc** : this produces an older version of the runtime parser that does not use
  the new zero-copy lexical analyzer trait.  This option is only retained
  for backwards compatibility with grammars and lexical scanners written prior
  to rustlr version 0.2.0.  This option is not capable of generating a lexical
  scanner.

The generated parser will be a program
[test1parser.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/test1/src/test1parser.rs)
that contains a **`make_parser`** function.  If the `-genlex` option
is used, it will also contain a struct `test1lexer` that implements
the [Tokenizer][tktrait].  RustLr will derive the name of the grammar
(test1) from the file path, unless there is a declaration of the form

>  grammarname somename

in the grammar spec, in which case the parser generated will be called
"somenameparser.rs". The parser must import some elements of rustlr so it
should be used in a crate.  We will come back to how to use the
generated parser later.

####  GRAMMAR FORMAT

The first line in the grammar specification:
 
>  valuetype i32  

(alternatively `absyntype i32`) defines the type of value returned by
the parser.  In most cases that would be some enum that defines an
abstract syntax tree, but here we will just calculate an i32 value.
The default valuetype (if none declared) is `()`, unit. 

**The valuetype you choose must implement the Default trait.**

RustLr requires that all grammar symbols be defined before any production
rules using multiple "nonterminals" or "terminals" directives.


####  Top Nonterminal
>  topsym E

You should designate one particular non-terminal symbol as the top symbol:
The parser generator will always create an extra production rule of the
form   `START -->  topsym EOF`

####  Grammar Production Rules

You will get an error message if the grammar symbols are not defined before
the grammar rules.  Each rule is indicated by a non-terminal symbol followed
by `-->`, or  `==>`.  The symbol `==>` is for rules that span multiple
lines that you will find used
in other grammars (later chapters).  You can specify multiple production
rules with the same left-hand side nonterminal using |  which you will
also find used in other grammars.

The right hand side of each rule must separate each symbol with
whitespaces.  For each grammar symbol such as E, you can optionally
bind a "label" such as `E:a`, `E:(a)`, `E:@pattern@` or
`E:v@pattern@`.  Each type of binding carries a different meaning and
affects how they will be used in the semantic action part of the rule. The
grammar used in this Chapter will only use the first two forms: `a` and `(a)`.

The right-hand side of a rule may be empty, which will make the
non-terminal on the left side of `-->` "nullable".
           
####  SEMANTIC ACTIONS

Each rule can optionally end with a semantic action inside { and },
which can only follow all grammar symbols making up the right-hand
side of the production rule.  This is a piece of Rust code that will
be injected *verbatim* into the generated parser.  This code will have
access to any labels associated with the symbols defined using ":".
In a label such as `E:e`, e is of type [StackedItem][sitem], which includes the
following fields:
   -  **.value** : `e.value` refers to the semantic value associated with this
     symbol, which in this case is of type i32 but in general will be of the
     type defined by the "valuetype" or "absyntype" directive.
   -  **.line** : the line number in the original source where this syntactic
     construct begins.  Lines start at 1.
   -  **.column** : the column number (character position on the line) where
     this syntactic construct begins.  Columns start at 1.

However, if we are only interested in the .value of the label, we can
also capture the value directly using the form demonstrated by
`T:(t)`: in this case `t` refers *only* to the .value of the popped
StackedItem.  In case the valuetype can be described by an irrefutable
pattern, such as `(i32,i32)`, a label such as `E:(a,b)` can also used
to directly capture the value.  The other kinds of labels (with the
`@` symbol) will be described in the next chapter.

**The semantic action code must return a value of type
valuetype** (in this case i32).  If no semantic action is given, then a
default one is created that just returns valuetype::default(), which is
why the valuetype must implement the Default trait.  Here's an example,
taken from the generated parser, of how the code is injected:

```
rule.Ruleaction = |parser|{ let mut t = parser.popstack(); let mut _item1_ = parser.popstack(); let mut e = parser.popstack();  e.value + t.value };
```
This is the semantic action generated from the rule

>      E --> E:e + T:t { e.value + t.value }

Notice that if a symbol carries no label, then rustlr generates a name
`_item{n}_` for it.  The parser generator is not responsible if you
write an invalid semantic action that's rejected by the Rust compiler.
Within the { } block, you may also call other actions on the parser,
including reporting error messages and telling the parser to abort.
However, you should not try to "pop the stack"
or change the parser state in other ways: leave that to the generated
code.



#### **CREATING A LEXER AND INVOKING THE PARSER**

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
