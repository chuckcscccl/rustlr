## Chapter 1: Unambiguous LR Grammar for Simple Calculator.

Please note that this tutorial has been rewritten for **[Rustlr version 0.2.x][drs]**,
which contains significant changes over the 0.1.x versions, although it remains
compatible with parsers already created.  The original version of this chapter
is available [here](https://cs.hofstra.edu/~cscccl/rustlr_project/test1grammar0.html).

This tutorial is written for those with sufficient background in computer
science and in Rust programming, with some knowledge of context free grammars
and basic LR parsing concepts.
Those who are already
familiar with similar LR parser generation tools may wish to skip to the
more advanced example in [Chapter 2](https://cs.hofstra.edu/~cscccl/rustlr_project/calculatorgrammar.html).

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

EOF
```

These are the contents of a Rustlr grammar file, called [test1.grammar](https://cs.hofstra.edu/~cscccl/rustlr_project/test1.grammar).
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
  a file called [test1parser.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/test1parser.rs).
- **-trace n**  : where n is a non-negative integer defining the trace level.
  Level 0 prints nothing; level 1, which is the default, prints a little more
  information.  Each greater level will print all information in lower levels.
  -trace 3 will print the states of the LR finite state machine, which could
  be useful for debugging and training the parser for error message output.
- **-nozc** : this produces an older version of the runtime parser that does not use
  the new zero-copy lexical analyzer trait.  This option is only retained
  for backwards compatibility with grammars and lexical scanners written prior
  to rustlr version 0.2.0.

The generated parser will be a program [test1parser.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/test1parser.rs) that contains a **`make_parser`** function.
RustLr will derive the name of the grammar (test1) from the file path, unless
there is a declaration of the form

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
The default valuetype (if none declared) is i64. 

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
by `-->`, `::=` , or  `==>`.  The symbol `::=` is interpreted to be the same
as `-->`.  `==>` is for rules that span multiple lines that you will find used
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
side of the production rule.  This is a piece of Rust code that would
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

Here is the [main.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/test1main.rs) associated with this grammar, which forms a simple calculator.  Its principal
contents define a lexical analyzer that conforms to the [Tokenizer][tktrait] trait.
```
struct Scanner<'t>(StrTokenizer<'t>);
impl<'t> Tokenizer<'t,i32> for Scanner<'t>
{
   // this function must convert any kind of token produced by the lexer
   // into TerminalTokens expected by the parser.  The built-in lexer,
   // StrTokenizer, produces RawTokens along with their line/column numbers.
   fn nextsym(&mut self) -> Option<TerminalToken<'t,i32>>   {
     let tokopt = self.0.next_token();
     if let None = tokopt {return None;}
     let tok = tokopt.unwrap();
     match tok.0 {  // tok.1,tok.2 are line,column numbers
       RawToken::Num(n) => Some(TerminalToken::from_raw(tok,"num",n as i32)),
       RawToken::Symbol(s) => Some(TerminalToken::from_raw(tok,s,0)),
       _ => Some(TerminalToken::from_raw(tok,"<<Lexical Error>>",0)),
     }//match
   }
}
fn main() {
  let mut input = "5+2*3";
  let args:Vec<String> = std::env::args().collect(); // command-line args
  if args.len()>1 {input = &args[1];}
  let mut parser1 = zc1parser::make_parser();
  let mut tokenizer1 =Scanner(StrTokenizer::from_str(input));
  let result = parser1.parse(&mut tokenizer1);
  println!("result after parsing {}: {}",input,result);  
}//main
```

To run the program, **`cargo new`** a new crate and copy
the contents of [main.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/test1main.rs and [test1parser.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/test1parser.rs) to src/main.rs and src/test1parser.rs respectively.  Add to Cargo.toml
under [dependencies]:
```
rustlr = "0.2"  
```
**`cargo run "2+3*4"`** will print 14 and `cargo run "(2+3)*4"` will print
20.

#### RustLr's Lexical Interface

To create a lexical scanner for your grammar, you must become familiar with the [Tokenizer][tktrait] trait and the
[TerminalToken][tt] struct which are defined by rustlr:

```
pub struct TerminalToken<'t,AT:Default>
{
  pub sym: &'t str,
  pub value: AT,
  pub line:usize,
  pub column:usize,
}
pub trait Tokenizer<'t,AT:Default> 
{
  fn nextsym(&mut self) -> Option<TerminalToken<'t,AT>>;
  //... the above is the only required function when impl Tokenizer
}  
```
TerminalTokens are the structures expected by rustlr's built-in runtime parser
(called [ZCParser][zcp], whereas RuntimeParser refers to an older version).  **The
.sym field of each token must correspond to the name of a terminal symbol
of the grammar** being parsed.  The value must be of the valuetype or 'absyntype'
of the grammar.  Each TerminalToken also includes the starting line and column
of where the token begins in the source.

The parser requires a ref mut to a Tokenizer-trait object as an argument.

The nextsym function of the trait object must produce Some(TerminalToken) until
end of input is reached, at which point it should return None.  The
[TerminalToken::new][ttnew] function can be called to create a new token.
Very importantly, the "sym" &str of the TerminalToken must match identically
with the name of a terminal symbol of your grammar (yes that's worth repeating).
The "value" of the token is something of type valuetype/absyntype as defined
by the grammar.  In this case each integer constant must be translated into
a token with .sym=="num" and .value = the value of integer as an i32.

This example uses the built-in [StrTokenizer][1] as lexer.  This tokenizer
suffices for the examples that have been so-far created by rustlr.  It
is capable of recognizing multi-line string literals and comments,
alphanumeric and non alpha-numeric symbols, decimal and hexadecimal
constants (unsigned), floating point constants, character literals
such as `'a'`.  It also has the option of returning newline and
whitespaces (with count) as tokens.  But it does have limitations. It
is not the most efficient (not always one-pass, uses regex).  It
returns all integers as i64 (it would recognize "1u8" as two separate
tokens, a number and an alphanumeric symbol "u8").  Negative integers
must also be recognized at the parser as opposed to lexer level.  The
lexer was not designed to recognize binary input.  But StrTokenizer
does "get the job done" in many cases that are required in compiling and
analyzing source code.

[StrTokenizer][1] produces a structure called [RawToken][rtk].  The
[TerminalToken::from_raw][fromraw] function converts a tuple that consists of
(RawToken,line,column) into a TerminalToken.
[RawToken][rtk] is an enum that includes `Num` that carries an i64 value,
and `Symbol`, which carries a string of non-alphanumeric symbols such as `*`.

Besides the [TerminalToken::from_raw][fromraw] function, there is
no link between the specific tokenizer and the parser.  Any lexer can
be adopted to impl the [Tokenizer][tktrait] trait by converting whatever kind of
tokens they produce into TerminalTokens in the **[nextsym][nextsymfun]**
function required by the trait.

An instance of the runtime parser is created by calling the **`make_parser`**
function, which is the only exported function of the generated parser.
Once a lexer has also been created, parsing can commence by calling

>      `parser1.parse(&mut tokenizer1)`

This function will return a value of type valuetype.  It will return a valuetype-value
even if parsing failed (but error messages will be printed).  After
.parse returns, you can also check if an error had occurred by calling
`parser1.error_occurred()` before deciding to use the valuetype result
that was returned.  



#### Reserved Symbols

The following terminal symbols are reserved and should not be used in a grammar:

>      EOF   ANY_ERROR   :  |  @  {  }  -->  ::=  ==>  <==  

The following symbols should also NOT be used as non-terminals in your grammar:

>     START valuetype absyntype grammarname resync resynch topsym errsym 
>     nonterminal terminal nonterminals terminals flexname lexname typedterminal
>     left right externtype externaltype lifetime

For example, if ":" is to be one of the terminal symbols of your
language, then you should call it something like COLON inside the
grammar.  You will then adopt your lexical analyzer so that ":" is
translated into a [TerminalToken][tt] with .sym="COLON" before sending the token to the parser. If you
want to treat a whitespace as a token your lexer must similarly
translate whitespaces into something like WHITESPACE. Non-terminal
symbol START and terminal EOF will always be added as additional
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
In [this additional example](https://cs.hofstra.edu/~cscccl/rustlr_project/brackets.grammar),
enough code has been injected into the .grammar so that rustlr can generate a
relatively [self-contained program](https://cs.hofstra.edu/~cscccl/rustlr_project/bracketsparser.rs), that includes a lexer and a main, and illustrates a
few extra features of Rustlr.  This example also uses charscanner, which is
another tokenizer that comes with Rustlr, this time designed to parse one
character at a time.

-----------

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
[nextsymfun]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/trait.Tokenizer.html#tymethod.nextsym
[zcp]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.ZCParser.html
[fromraw]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html#method.from_raw
[ttnew]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html#method.new
