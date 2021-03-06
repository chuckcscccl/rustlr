## Chapter 4: Automatically Generating the AST and Moving Towards EBNF

One of the advantages of writing ambiguous grammars, e.g., `E-->E+E` instead of `E-->E+T`, is that it becomes easier to generate reasonable abstract syntax representations automatically.  Extra symbols such as `T` that are required for unambiguous grammars generally have no meaning at the abstract syntax level and will only lead to convoluted ASTs.  Rustlr is capable of automatically generating the data structures (enums and structs) for the abstract syntax of a language as well as the semantic actions required to create instances of those structures.  For beginners new to writing grammars and parsers, we **do not** recommend starting with an automatically generated AST.  The user must understand clearly the relationship between concrete and abstract syntax and the best way to learn this relationship is by writing ASTs by hand, as demonstrated in the previous chapters.  Even with Rustlr capable of generating nearly everything one might need from a parser, it is still likely that careful fine tuning will be required.

We redo the enhanced calculator example from [Chapter 2][chap2].  The following grammar is found [here](https://cs.hofstra.edu/~cscccl/rustlr_project/autocalc/calcauto.grammar).

```rust
lifetime 'lt
nonterminals Expr ES
terminals + - * / ( ) = ;
terminals let in
typedterminal int i64
typedterminal var &'lt str
topsym ES
resync ;

left * 500
left / 500
left + 400
left - 400
left = 300

Expr:Val --> int
Expr:Var --> var
Expr:Letexp --> let var = Expr in Expr
Expr:Plus --> Expr + Expr
Expr:Minus --> Expr - Expr
Expr:Div --> Expr / Expr
Expr:Times --> Expr * Expr
# the unary minus has higher precedence (600) than binary operators:
Expr(600):Neg --> - Expr
Expr --> ( Expr:e )

ES:nil -->
ES:cons --> Expr ; ES

lexvalue int Num(n) n
lexvalue var Alphanum(x) x
lexattribute set_line_comment("#")
EOF

```

Note the following differences between this grammar and the one presented in [Chapter 2][chap2]:

1. There are no semantic actions
2. There is no "absyntype" or "valuetype" declaration; any such declaration would be ignored
3. Only the types of values carried by certain terminal symbols must be declared (with `typedterminal`).
4. The non-terminal symbol on the left-hand side of a production rule may carry a label.  These labels will become the names of enum variants to be created.

Process the grammar with **`rustlr calcauto.grammar -genabsyn`** (or **`-auto`**).   Two files are created.  Besides **[calcautoparser.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/autocalc/src/calcautoparser.rs)** there will be, in the
same folder as the parser, a **[calcauto_ast.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/autocalc/src/calcauto_ast.rs)** with the following (principal) contents:

```
#[derive(Debug)]
pub enum ES<'lt> {
  cons(LBox<Expr<'lt>>,LBox<ES<'lt>>),
  nil,
  ES_Nothing(&'lt ()),
}
impl<'lt> Default for ES<'lt> { fn default()->Self { ES::ES_Nothing(&()) } }

#[derive(Debug)]
pub enum Expr<'lt> {
  Neg(LBox<Expr<'lt>>),
  Div(LBox<Expr<'lt>>,LBox<Expr<'lt>>),
  Letexp(&'lt str,LBox<Expr<'lt>>,LBox<Expr<'lt>>),
  Minus(LBox<Expr<'lt>>,LBox<Expr<'lt>>),
  Val(i64),
  Times(LBox<Expr<'lt>>,LBox<Expr<'lt>>),
  Plus(LBox<Expr<'lt>>,LBox<Expr<'lt>>),
  Var(&'lt str),
  Expr_Nothing(&'lt ()),
}
impl<'lt> Default for Expr<'lt> { fn default()->Self { Expr::Expr_Nothing(&()) } }

```

An enum is created for each non-terminal symbol of the grammar that appears on the left-hand side of multiple production rules. The name of the enum is the
same as the name of the non-terminal.
The names of the variants are derived from the labels given to the left-hand side nonterminal, or are automatically generated from the nonterminal name and the rule number (e.g. `Expr_8`).  A special `Nothing` variant is also created to represent a default.[^footnote 1]??The 'absyntype' of the grammar will be set to `ES`, the symbol declared to be 'topsym'.

There is essentially an enum variant for each production rule of this non-terminal.  Each variant is composed of the right-hand side
symbols of the rule that are associated with non-unit types.  Unit typed values
can also become part of the enum if the symbol is given a label.  For example:
  **` E:acase -->  a E `**  where terminal symbol `a` is of unit type, will result in a enum variant
`acase(LBox<E>)`. whereas
  **` E:acase -->  a:m E `**
will result in a variant `acase((),LBox<E>)`


A struct is created for non-terminals symbols that appears on the
left-hand side of exactly one production rule.  The name of the struct
is the same as the non-terminal.  The fields of each struct is named by
the labels given to the right-hand side symbols, or with `_item{i}_` if
no labels are given.  For example, a nonterminal `Ifelse` with a singleton rule
  ```
  Ifelse --> if Expr:condition Expr:truecase else Expr:falsecase
  ```
will result in the generation of:
  ```
  #[derive(Default,Debug)]
  pub struct Ifelse {
    condition: LBox<Expr>,
    truecase: LBox<Expr>,
    falsecase: LBox<Expr>,
  }
  ```
The AST generator always creates a [LBox][2] for each non-terminal field.
Unit typed values are not included in the struct unless given an explicit label.
The struct may be empty if all right-hand-side symbols of the single production
rule are associated with the unit type and do not have labels.

Although the generated parser may not be very readable, rustlr also generated semantic actions that create instances of these AST types.  For example, the rule `Expr:Plus --> Expr + Expr` will have semantic action equivalent to one created from:

```
Expr --> Expr:[a] + Expr:[b] {Plus(a,b)}
```

Recall from [Chapter 2][chap2] that a label of the form `[a]` means that the semantic value associated with the symbol is enclosed in an [LBox][2].

The production rule `Expr --> ( Expr )` is also treated in a special way:
note that there is no variant that correspond to this rule in the generated enum.  Rustlr infers from the fact that
  1. there is no left-hand side label to the nonterminal.
  2. `Expr` is the only grammar symbol on the right-hand side that has a non-unit
     type.
  3. There are no operator precedence/associativity declaration for the
     other symbols.
     
In other words, it infers that the other symbols on the right hand side carry
no meaning at the AST level, and thus generates a semantic action for this rule
that would be equivalent to:
```
  Expr --> ( Expr:e ) { e }
```
thus ignoring the parentheses in the AST.
If the automatically inferred "meaning" of this rule is not what's desired,
it can be altered by using an explicit left-side label: this will generate
a separate enum variant (at the cost of an extra LBox) that distinguishes
the presence of the parentheses.

It is always possible to override the automatically generated type and action.
In case of ES, the labels 'nil' and 'cons' are sufficient for rustlr to create a linked-list data structure.  However, the right-recursive grammar rule is slightly non-optimal for LR parsing (the parse stack grows until the last element of the list before ES-reductions take place).  One might wish to use a left-recursive rule and a Rust vector to represent a sequence of expressions.  This can be done by making the following changes to the grammar.  First, change the declaration of the non-terminal symbol `ES`??as follows:

```
nonterminal ES Vec<LBox<Expr<'lt>>>
```

Then replace the two production rules for `ES` with the following:

```rust
ES --> Expr:[e] ; { vec![e] }
ES --> ES:v Expr:[e] ;  { v.push(e); v }

```
The presence of a non-empty semantic action will override automatic AST generation.
It is also possible to inject custom code into the
automatically generated code:
```
ES --> Expr ; {println!("starting a new ES sequence"); ... }
```
The ellipsis are allowed only before the closing right-brace.  This indicates
that the automatically generated portion of the semantic action should follow.
The ellipsis cannot appear anywhere else.
```

```

   -------------------

### Moving Towards EBNF: Automatically Adding New Rules with *, + and ?

A relatively new feature of rustlr (since verion 0.2.9) allows the use of regular-expression style symbols *, + and ? to automatically generate new production rules.  However, these symbols cannot be used unrestrictedly to form arbitrary
regular expressions (they cannot be nested).  They are given only as a convenience.  They are also guaranteed to only fully work in the -auto mode.

Another way to achieve the same effects as the above (to derive a vector for symbol ES) is to use the following alternative grammar declarations:

```
nonterminal ES Vec<LBox<Expr<'lt>>>
ES --> (Expr ;)*
```

The operator **`*`** means a sequence of zero or more.  This is done by generating several new non-terminal symbols initially.  Essentially, these correspond to

```
ES0 --> Expr:e ; {e}
ES1 --> { Vec::new() }
ES1 --> ES1:v ES0:[e] { v.push(e); v }
ES --> ES1:v {v}
```
These rules replace the original in the grammar.  In the -auto mode,
rustlr also infers that symbols such as ; has no meaning at the
AST level (because it has the unit type and noprecedence/associativity
declaration). It therefore infers that the type of the nonterminal ES0
is the same as Expr, and automatically generates the appropriate semantic action.
If override of this behavior is required, one can manually rewrite the grammar
as
```
ES0:SEMI --> Expr ; 
ES --> ES0*
```
The presence of the left-hand side label will cause the AST generator to 
create an AST representation for the semicolon (assuming that is what's
desired).  Another situation where the user has to write the ES0 rule
manually is if `-auto` (or `-genabsyn`) is not specified, which implies
that a rule with an explicit semantic action is required.  Generally speaking,
the *, + and ? symbols will still work without `-auto` if it follows a
single grammar symbol.

The type rustlr associates with the new non-terminal ES1
will be `Vec<LBox<Expr<'lt>>` and semantic actions are generated to
create the vector for both ES1 rules.  A **`+`** means one or more
`ES1` derivations, producing the same vector type, and a **`?`** will
mean one or zero derivations with type `Option<LBox<Expr<'lt>>>`.

Other alternatives are possible:
```
nonterminal ES
ES:Sequence --> (Expr ;)*
```
This would generate a new struct type for the AST of ES, with a component of
type
`Vec<LBox<Expr<'lt>>>`.  If the type of ES is declared manually
as above, rustlr infers that the appropriate semantic action is equivalent to
`ES --> (Expr ;)*:v {v}`  because there is only one symbol (the internally
generated ES1) on the right-hand side, and it is of the same type.
**The label given for such an expression cannot be a pattern such as `[v]` or something enclosed inside `@...@`.**  These restrictions may eventually be eliminated in future releases.

Another restriction is that the symbols `(`, `)`, `?`, `*` and `+` may not
be separated by white spaces since that would confuse their interpretation
as independent terminal symbols.  For example, `( Expr ; ) *` is not valid.

Yet another alternative is to manually define the type of ES, from which Rustlr will infer that no struct/enum needs to be created for it:
```
nonterminal ES Vec<LBox<Expr<'lt>>>
ES: --> (Expr ;)*
```
This is because rustlr generates an internal non-terminal to represent the right-hand side `*` expression and assigns it type `Vec<LBox<Expr<'lt>>>`.
It then recognizes that this is the only symbol on the
right, which is of the same type as the left-hand side nonterminal `ES`
as declared. This rule will again be given an action equivalent to
`ES: --> (Expr ;)*:v {v}`


The operator-precedence and associativity declarations, the *, + and ?
operators, and the natural ability of LR parsers to handle left-recursive
grammars, including indirect ones, allows a language to be defined
using a grammar that closely resembles EBNF syntax.



### Invoking the Parser

Since the grammar also contains lexer generation directives, all we need to do is to write the procedures that interpret the AST (see [main](https://cs.hofstra.edu/~cscccl/rustlr_project/autocalc/src/main.rs)).  The procedure to invoke the parser is the same as described in [Chapter 3][chap3], using the **`parse_with`** or **`parse_train_with`** functions:

```rust
   let mut scanner = calcautoparser::calcautolexer::from_str("2*3+1;");
   let mut parser = calcautoparser::make_parser();
   let result = calcautoparser::parse_with(&mut parser, &mut scanner);
   let tree = result.unwrap_or_else(|x|{println!("Parsing errors encountered; results are partial.."); x});
   println!("\nAST: {:?}\n",&tree);
   
```
The `parse_with` and `parse_train_with` functions were also backported for
grammars with a single *absyntype.*

Please note that all generated enums for the grammar will attempt to derive the Debug trait (as well as implement the Default trait).

Please also note that using [LBox][2] is already included in all parsers generated with the `-genabsyn` or `-auto` option, so do not use `!use ...` to include
it again.


   ----------------

### Generating a Parser for C

As a larger example, we applied the `-genabsyn` feature of rustlr to the ANSI C Yacc grammar published in 1985 by Jeff Lee, which was converted to rustlr syntax and found [here](https://cs.hofstra.edu/~cscccl/rustlr_project/cparser/cauto.grammar).  The raw grammar could not be used as-is.  The following problems
had to be resolved:

 1.  A shift-reduce conflict caused by the "dangling else" problem.
 2.  The need to distinguish an alphanumeric identifier as a "TYPE_NAME".

The first problem was easily fixed by giving  'else' a higher precedence than 'if'.  The second problem presented a challenge for Rustlr.  The grammar
contained two terminal symbols, "IDENTIFIER" and "TYPE_NAME", each should 
carry as value alphanumeric strings.
In the following C code
```
typedef unsigned int uint;
...
unit x = 1;
```
The first occurrence of "uint", in the typedef line, should be recognized as
an IDENTIFIER while the second, in the declaration of x, should be
recognized as TYPE_NAME.  This suggests that the lexical scanner must be aware of information that exists "several levels of abstraction above".  That
is, the token returned depends on the symbol table, or at least on
previously parsed "typedef" statements.  This sharing of information
between parser and lexer is relative easy in Lex/Yacc given that global,
mutable and shared structures are "simple" to do in C.  In Rust we have
to find another solution.

A Rustlr .grammar file (since versin 0.2.96) can contain a decarative such
as
```
transform |parser,token|{if token.sym=="IDENTIFIER" {let v=extract_value_IDENTIFIER(&token.value); if parser.exstate.contains(v)  {token.sym="TYPE_NAME";}} }
```
The transform directive should be followed by a function of type
```
for <'t> fn(&ZCParser<AT,ET>, &mut TerminalToken<'t,AT>)
```
that allows a lexical token to be modified before being passed to the parser.
The **`transform`** directive also enables a flag that calls this function
each time a new token is returned by the lexical analyzer. The function must
be on a single line and is injected verbatim into the generated parser.
The `transform` directive
also enables the generation of `extract_value_{terminal symbol}`
and `encode_value_{terminal symbol}` function for each terminal symbol.  These
function enables the extraction/encoding of the semantic value attached to
a terminal symbol relative to the internally generated enum that allows
grammar symbols to carry values of different types.

We are also making use of the `external state` that every rustlr parser
carries to maintain a set of strings that should be parsed as TYPE_NAME.


The AST types are found [here](https://cs.hofstra.edu/~cscccl/rustlr_project/cparser/src/cauto_ast.rs)??and the generated parser [here](https://cs.hofstra.edu/~cscccl/rustlr_project/cparser/src/cautoparser.rs).

The most important modificiations to the grammar, in addition to the `transform` directive above, are as follows:
```
lifetime 'lt
externtype HashSet<&'lt str>

!/*the following exposes the names of the generated enum variants*/
!use crate::cauto_ast::declaration_specifiers::*;
!use crate::cauto_ast::storage_class_specifier::*;
!use crate::cauto_ast::init_declarator::*;
!use crate::cauto_ast::init_declarator_list::*;
!use crate::cauto_ast::declarator::*;
!use crate::cauto_ast::declaration::*;
!use crate::cauto_ast::direct_declarator::*;

declaration_specifiers:DSCDS -->  storage_class_specifier declaration_specifiers

type_specifier:Typename --> TYPE_NAME

declaration:DecSpecList ==> declaration_specifiers:ds init_declarator_list:il ;
 { if let (DSCDS(td,_),init_declarator_list_84(x)) = (&ds,&il) {
    if let Typedef = &**td {
      if let init_declarator_86(y) = &**x {
        if let declarator_130(z) = &**y {
          if let IDENTIFIER_131(id)= &**z {
            parser.exstate.insert(id.to_string());
          }}}}} ...
 } <==
 
```
The nested ifs were needed in the rule for `declaration` to look into LBoxes.
Rust does not allow pattern matching inside Box, and so nested pattern matching
with recursively defined trees is difficult.

   ----------------



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

[^footnote 1]: Each enum has a `_Nothing(&'lt ())` variant.  This is used to implement the Default trait.  The lifetime parameter exists so that all enums can be parameterized with a lifetime, if one was declared for the grammar.  Without the dummy reference one would have to compute a closure over the grammar to determine which enums require lifetimes and which do not: something that&#39;s determined to be too expensive relative to its importance.  For structs, rustlr currently uses std::marker::PhantomData to avoid unused lifetimes.

