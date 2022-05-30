## Chapter 4: Automatically Generating the AST

Since version 0.2.7, rustlr is capable of automatically generating the data structures (enums) for the abstract syntax of a language as well as the semantic actions required to create instances of those structures.  For beginners new to writing grammars and parsers, we **do not** recommend starting with an automatically generated AST.  The user must understand clearly the relationship between concrete and abstract syntax and the best way to learn this relationship is by writing ASTs by hand, as demonstrated in the previous two chapters.  Even with Rustlr capable of generating nearly everything one might need from a parser, it is still likely that careful fine tuning may be required.

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

Expr:Val --> int
Expr:Var --> var
Expr:Letexp --> let var = Expr in Expr
Expr:Plus --> Expr + Expr
Expr:Minus --> Expr - Expr
Expr:Div --> Expr / Expr
Expr:Times --> Expr * Expr
Expr:Neg --> - Expr
# override auto-generated creation of abstract syntax, but type matters
Expr --> ( Expr:e )  { e }
ES:nil -->
ES:cons --> Expr ; ES

lexvalue int Num(n) n
lexvalue var Alphanum(x) x
lexattribute set_line_comment("#")
EOF

```

Note the following differences between this grammar and the one presented in [Chapter 2][chap2]:

1. There are no semantic actions but for one of the rules
2. There is no "absyntype" or "valuetype" declaration
3. Only the types of values carried by certain terminal symbols must be declared (with `typedterminal`).
4. The non-terminal symbol on the left-hand side of a production rule may carry a label.  This label will become the name of the enum variant to be created.

Process the grammar with **`rustlr calcauto.grammar -genabsyn`**.   Two files are created.  Besides **[calcautoparser.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/autocalc/src/calcautoparser.rs)** there will be a **[calcauto_ast.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/autocalc/src/calcauto_ast.rs)** with the following (principal) contents:

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
  Expr_8(LBox<Expr<'lt>>),
  Plus(LBox<Expr<'lt>>,LBox<Expr<'lt>>),
  Var(&'lt str),
  Expr_Nothing(&'lt ()),
}
impl<'lt> Default for Expr<'lt> { fn default()->Self { Expr::Expr_Nothing(&()) } }

```

An enum is created for each non-terminal symbol of the grammar, with the same name as the non-terminal.  There is, essentially, an enum variant for each production rule of the grammar.  The names of the variants are derived from the labels given to the left-hand side nonterminal, or are automatically generated (e.g. `Expr_8` represents the rule-8 variant of type `Expr`).[^footnote 1] The 'absyntype' of the grammar will be set to `ES`, the symbol declared to be 'topsym'.  Although the generated parser may not be very readable, rustlr also generated semantic actions that create instances of these enum types.  For example, the rule `Expr:Plus --> Expr + Expr` will have semantic action equivalent to one created from:

```
Expr --> Expr:[a] + Expr:[b] {Plus(a,b)}
```

Recall from [Chapter 2][chap2] that a label of the form `[a]`means that the semantic value associated with the symbol is enclosed in an [LBox][2].  However, there are cases where one might want to override the automatically generated action, as for the rule `Expr --> ( Expr )`.  The parentheses are of no use at the abstract syntax level and the most appropriate action would be to return the same value as the expression on the right-hand side.  The automatically generated action would have created an additional LBox.  It is also possible to override the automatic generation of the type of a grammar symbol.  In case of ES, the labels 'nil' and 'cons' are sufficient for rustlr to create a linked-list data structure.  However, the right-recursive grammar rule is not optimal for LR parsing.  One might wish to use a left-recursive rule and a Rust vector to represent a sequence of expressions.  This can be done by making the following changes to the grammar.  First, change the declaration of the non-terminal symbol `ES` as follows:

```
nonterminal ES Vec<LBox<Expr<'lt>>>
```

Then replace the two production rules for `ES` with the following:

```rust
ES --> Expr:[e] ; { vec![e] }
ES --> ES:v Expr:[e] ;  { v.push(e); v }

```

Future editions of Rustlr will allow syntax such as Expr+ and Expr*, which will generate such rules automatically.

Since the grammar also contains lexer generation directives, all we need to do is to write the procedures that interpret the AST (see [main](https://cs.hofstra.edu/~cscccl/rustlr_project/autocalc/src/main.rs)).  The procedure to invoke the parser is the same as described in [Chapter 3][chap3], using the `parse_with` or `parse_train_with` functions:

```rust
   let mut scanner = calcautoparser::calcautolexer::from_str("2*3+1;");
   let mut parser = calcautoparser::make_parser();
   let result = calcautoparser::parse_with(&mut parser, &mut scanner);
   let tree = result.unwrap_or_else(|x|{println!("Parsing errors encountered; results are partial.."); x});
   println!("\nAST: {:?}\n",&tree);
   
```

Please note that all generated enums for the grammar will attempt to derive the Debug trait (as well as implement the Default trait).  



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



