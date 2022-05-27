## Chapter 4: Automatically Generating AST

Since version 0.2.7, rustlr is capable of automatically generating the data structures (enums) for the abstract syntax of a language as well as the semantic actions required to create instances of those structures.  For beginners new to writing grammars and parsers, we **do not** recommend starting with an automatically generated AST.  The user must understand clearly the relationship between concrete and abstract syntax and the best way to learn this relationship is by writing ASTs by hand, as demonstrated in the previous two chapters.  Even with Rustlr capable of generating nearly everything one might need from a parser, it is still likely that careful fine tuning may need to be done manually.

We redo the enhanced calculator example from Chapter 2: 

```
# Grammar testing automatic generation of abstract syntax

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

1. There are no semantic actions safe for one of the rules
2. There is no "absyntype" or "valuetype" declaration
3. The non-terminal symbol on the left-hand side of a production rule may also carry a label.  This label is a hint as to how to name the enum variant to be created.

Process the grammar with **`rustlr calcauto.grammar -genabsyn`**.   Two files are created.  Besides **calcautoparser.rs** there will be a **calcauto_ast.rs** with the following (principal) contents:

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
  Expr_8(LBox<Expr<'lt>>),
  Plus(LBox<Expr<'lt>>,LBox<Expr<'lt>>),
  Var(&'lt str),
  Expr_Nothing(&'lt ()),
}
impl<'lt> Default for Expr<'lt> { fn default()->Self { Expr::Expr_Nothing(&()) } }

```

There is an enum created that's named for each non-terminal symbol of the grammar.  There is, essentially, an enum variant for each production rule of the grammar.  The names of the variants are derived from the labels given to the left-hand side nonterminal, or are automatically generated (e.g. `Expr_8` represents the rule-8 variant of type `Expr`). The 'absyntype' of the grammar will be set to `ES`, the symbol declared to be 'topsym'.  Although the generated parser may not be very readable, rustlr also generated semantic actions that created instances of these enum types for the rules.  For example, the rule `Expr:Plus --> Expr + Expr` will have semantic action equivalent to 

```
Expr --> Expr:[a] + Expr:[b] {Plus(a,b)}
```

Recall from [Chapter 2][chap2] that a label of the form `[a]`means that the semantic value associated with the symbol is enclosed in an [LBox][2].  However, there are cases where one might want to override the automatically generated action, as for the rule `Expr --> ( Expr )`.  The parentheses are of no use at the abstract syntax level and the most appropriate action would be to return the same value as the expression on the right-hand side.  The automatically generated action would have created an additional LBox.  It is also possible to override the automatically generation of the type of a grammar symbol.  In case of ES, the labels 'nil' and 'cons' are sufficient for rustlr to create a linked-list data structure.  However, the right-recursive grammar is not optimal for LR parsing.  One might wish to use a left-recursive grammar and a Rust vector to represent a sequence of expressions.  This can be done by making the following changes to the grammar.  First, declare the nonterminal `ES` as follows:

```
nonterminal ES Vec<LBox<Expr<'lt>>>
```

Then replace the two rules for `ES` with the following:

```
ES --> Expr:[e] ; { vec![e] }
ES --> ES:v Expr:[e] ;  { v.push(e); v }

```

Future editions of Rustlr will allow syntax such as Expr+ and Expr*, which will generate such rules automatically.



-------------------

[1]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.StrTokenizer.html
[2]:https://docs.rs/rustlr/latest/rustlr/generic_absyn/struct.LBox.html
[3]:https://docs.rs/rustlr/latest/rustlr/generic_absyn/struct.LRc.html
[4]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.ZCParser.html#method.lbx
[5]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.StackedItem.html#method.lbox
[sitem]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.StackedItem.html
[chap1]:https://cs.hofstra.edu/~cscccl/rustlr_project/chapter1.html
[chap2]:https://cs.hofstra.edu/~cscccl/rustlr_project/chapter2.html
[lexsource]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.LexSource.html
[drs]:https://docs.rs/rustlr/latest/rustlr/index.html
[tktrait]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/trait.Tokenizer.html
[tt]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html
[rtk]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/enum.RawToken.html
[fromraw]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html#method.from_raw
[nextsymfun]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/trait.Tokenizer.html#tymethod.nextsym
[zcp]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.ZCParser.html
[ttnew]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html#method.new
