## Chapter 4: Automatically Generating the AST

One of the advantages of writing ambiguous grammars, e.g., `E-->E+E` instead of `E-->E+T`, is that it becomes easier to generate reasonable abstract syntax representations automatically.  Extra symbols such as `T` that are required for unambiguous grammars generally have no meaning at the abstract syntax level and will only lead to convoluted ASTs.  Since version 0.2.8, rustlr is capable of automatically generating the data structures (enums) for the abstract syntax of a language as well as the semantic actions required to create instances of those structures.  For beginners new to writing grammars and parsers, we **do not** recommend starting with an automatically generated AST.  The user must understand clearly the relationship between concrete and abstract syntax and the best way to learn this relationship is by writing ASTs by hand, as demonstrated in the previous two chapters.  Even with Rustlr capable of generating nearly everything one might need from a parser, it is still likely that careful fine tuning will be required.

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
Expr:Neg --> - Expr
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
2. There is no "absyntype" or "valuetype" declaration
3. Only the types of values carried by certain terminal symbols must be declared (with `typedterminal`).
4. The non-terminal symbol on the left-hand side of a production rule may carry a label.  This label will become the name of the enum variant to be created.

Process the grammar with **`rustlr calcauto.grammar -genabsyn`** (or **`-auto`**).   Two files are created.  Besides **[calcautoparser.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/autocalc/src/calcautoparser.rs)** there will be a **[calcauto_ast.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/autocalc/src/calcauto_ast.rs)** with the following (principal) contents:

```rust
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

An enum is created for each non-terminal symbol of the grammar, with the same name as the non-terminal.  There is, essentially, an enum variant for each production rule of the grammar.  The names of the variants are derived from the labels given to the left-hand side nonterminal, or are automatically generated from the nonterminal name and the rule number (e.g. `Expr_8`).[^footnote 1] The 'absyntype' of the grammar will be set to `ES`, the symbol declared to be 'topsym'.  Although the generated parser may not be very readable, rustlr also generated semantic actions that create instances of these enum types.  For example, the rule `Expr:Plus --> Expr + Expr` will have semantic action equivalent to one created from:

```
Expr --> Expr:[a] + Expr:[b] {Plus(a,b)}
```

Recall from [Chapter 2][chap2] that a label of the form `[a]` means that the semantic value associated with the symbol is enclosed in an [LBox][2].

The production rule `Expr --> ( Expr )` is also treated in a special way:
note that there is no variant that correspond to this rule in the enum.  Rustlr
infers from the fact that
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
In case of ES, the labels 'nil' and 'cons' are sufficient for rustlr to create a linked-list data structure.  However, the right-recursive grammar rule is slightly non-optimal for LR parsing (the parse stack grows until the last element of the list before ES-reductions take place).  One might wish to use a left-recursive rule and a Rust vector to represent a sequence of expressions.  This can be done by making the following changes to the grammar.  First, change the declaration of the non-terminal symbol `ES` as follows:

```
nonterminal ES Vec<LBox<Expr<'lt>>>
```

Then replace the two production rules for `ES` with the following:

```rust
ES --> Expr:[e] ; { vec![e] }
ES --> ES:v Expr:[e] ;  { v.push(e); v }

```
The presence of a non-empty semantic action or a user-defined nonterminal
type will always cancel their automatic generation.

   -------------------

### Adding New Rules with *, + and ?

A relatively new feature of rustlr (since verion 0.2.9) allows the use of regular-expression style symbols *, + and ? to automatically generate new production rules.  However, these symbols cannot be used unrestrictedly to form arbitrary
regular expressions (they cannot be nested).  They are given only as a convenience.  They are also guaranteed to only fully work in the -auto mode.

Another way to achieve the same effects as the above is to use the following alternative grammar declarations:

```
nonterminal ES Vec<LBox<Expr<'lt>>>
ES --> (Expr ;)*
```

The operator **`+`** means a sequence of at least one.  This is done by generating several new non-terminal symbols initially.  Essentially, these correspond to

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
This would generate a new enum type for the AST of ES, with a variant
`Sequence(Vec<LBox<Expr<'lt>>>)`.  If the type of ES is declared manually
as above, rustlr infers that the appropriate semantic action is equivalent to
`ES --> (Expr ;)*:v {v}`  because there is only one symbol (the internally
generated ES1) on the right-hand side, and it is of the same type.
The label given for such an expression cannot be a pattern such as `[v]` or something enclosed inside `@...@`.  These restrictions may eventually be eliminated in future releases.

Another restrictions is that the symbols `(`, `)`, `?`, `*` and `+` may not
be separated by white spaces since that would confuse their interpretation
as independent terminal symbols.  For example, `( Expr ; ) *` is not valid.


##### Invoking the Parser

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

#### Generating a Parser for C

As a larger example, we applied the `-genabsyn` feature of rustlr to the ANSI C Yacc grammar published in 1985 by Jeff Lee, which was converted to rustlr syntax and found [here](https://cs.hofstra.edu/~cscccl/rustlr_project/cparser/cauto.grammar).  The raw grammar produced a shift-reduce conflict caused by the *dangling else* problem, which we fixed by giving  'else' a higher precedence than 'if'.  The raw grammar contained a few other issues we have not addressed, most notably when an identifier should be considered as `TYPE_NAME`. Manual fine tuning will definitely be required, as one should expect.  However, the generated AST is at least a good starting point.  In forming the enum types, for production rules without a left-hand side label, rustlr will also sometimes use the name of an alpha-numeric terminal symbol to create the name of the enum variant (if it is the first symbol on the right-hand side).

The AST enums are found [here](https://cs.hofstra.edu/~cscccl/rustlr_project/cparser/src/cauto_ast.rs) and the generated parser [here](https://cs.hofstra.edu/~cscccl/rustlr_project/cparser/src/cautoparser.rs).



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

[^footnote 1]: Each enum has a `_Nothing(&'lt ())` variant.  This is used to implement the Default trait.  The lifetime parameter exists so that all enums can be parameterized with a lifetime, if one was declared for the grammar.  Without the dummy reference one would have to compute a closure over the grammar to determine which enums require lifetimes and which do not: something that&#39;s determined to be too expensive relative to its importance.



