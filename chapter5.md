## Chapter 5: Using Regular Expressions in Production Rules

This chapter closely follows [Chapter 4][chap4] as it is closely related
to how Rustlr generates ASTs.

### Automatically Adding New Rules with *, + and ?

Rustlr allows the use of regular-expression style symbols *, + and ?,
which lead to the generate of new production rules and semantic actions
that create vectors and options (for ?).  However, these
symbols cannot be used unrestrictedly to form arbitrary regular
expressions. They cannot be nested (see reason below).
They are also guaranteed to only fully work in the -auto mode.

Referring to the auto-ast generation example in [Chapter 4][chap4], 
another way to define the nonterminal `ExprList` as a semicolon-separated
sequence of `LetExpr` is to replace the two productions for `ExprList` with
the following rule
```
ExprList --> (LetExpr ;)*
```
This would lead to the generation of a tuple struct for type ExprList:
```
#[derive(Default,Debug)]
pub struct ExprList<'lt>(pub Vec<LBox<Expr<'lt>>>,);
```
The operator **`*`** means a sequence of zero or more.  This is done by generating several new non-terminal symbols internally.
Essentially, these correspond to
```
ES0 --> Expr:e ; {e}
ES1 --> { Vec::new() }
ES1 --> ES1:v ES0:[e] { v.push(e); v }
ExprList --> ES1:v {v}
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
ExprList --> ES0*
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

Another way to generate the AST for `ExprList` is to manually define the type
of `ExprList`, from which Rustlr will infer that it is a `pass-thru` case.
No type will be created for `ExprList` as it would inherit the type of
the right-hand side of its lone production rule.
```
nonterminal ExprList Vec<LBox<Expr<'lt>>>
ExprList --> (LetExpr ;)*
```
This is because rustlr generates an internal non-terminal to represent the right-hand side `*` expression and assigns it type `Vec<LBox<Expr<'lt>>>`.
It then recognizes that this is the only symbol on the
right, which is of the same type as the left-hand side nonterminal `ExprList`
as declared. This rule will be given an action equivalent to
`ExprList --> (Expr ;)*:v {v}`

**The label given for a regex-like expression cannot be a pattern that
includes `@...@`.** It can only be a simple alphanumeric label or a
boxed label (`[x]`).

Another restriction is that the symbols `(`, `)`, `?`, `*` and `+` may not
be separated by white spaces since that would confuse their interpretation
as independent terminal symbols.  For example, `( Expr ; ) *` is not valid.

#### Special Operators

In addition to the `*`, `+` and `?` suffixes, rustlr also recognizes (non-nested)
suffixes such as **`<Comma*>`** or **`<;+>`**.  Assuming that
`Comma` is a declared terminal symbol of the grammer, the expression
`Expr<Comma+>` represents a sequence of one or more Expr separated by Comma,
but not ending in Comma, e.g *a,b,c* but not *a,b,c,*.  In contrast,
`(Expr Comma)+` means that the expression must end in a Comma.  <Comma*>
allows the sequence to be empty. The AST generator will also create vectors
as the semantic values of such expressions.  Please avoid whitespaces in
these expressions: `<Comma *>` is not recognized.

#### Nesting Restriction

The regex operators **cannot be nested.** It is unlikely that
Rustlr will ever allow their nesting for such combinations can easily 
be ambiguous.  For example, with **`(a?)+`**, even a single `a`
will have an infinite number of derivations: as one `a?` or as three,
for example.

In general, be warned that overusing these regex-like operators,
especially in the same production, can easily lead to new
non-determinisms in the grammar.  The new productions generated for
these operators could lead to additional Shift-Reduce and even
Reduce-Reduce conflicts.  For example, a production with right-hand
side **`Expr<Comma*> Comma?`** will lead to a shift-reduce conflict.
However, **`Expr<Comma+> Comma?`** is fine and represents a
comma-separated sequence with an optional trailing comma.

Rustlr does its best to try to prevent the regex operators from causing
ambiguity.  For example, it caches the uses of the operators to avoid
duplicate rules that are almost certain to cause conflicts.
However, mixing regular expressions and context-free grammars will still
have unexpected consequences for the user.  We never have to worry about
a regex being "ambiguous" because they all reduce to a normal form (a
deterministic finite automaton with minimal states).  The same is not
true for grammars.  Grammars cannot be *combined* like regex can: for
example, a rule like `A --> B* B*` is hopelessly ambiguous as
there is no way to determine where the first sequence of B's should stop 
and the second one begins.

The motivation for adding regex-like operators to a parser generator
are both usability and improved ability in creating abstract syntax
automatically.  But much work needs to be done before we can parse
arbitrary EBNF syntax.  We are currently exploring extensions of LR
parsing including *delayed reductions*, which can potentially allow
non-ambiguous grammars to be more easily composed without running into
new conflicts: see the [Appendix][apnd] of this tutorial for experimental
features.



   ----------------

### JSON Parser

To give another, complete example of the features described in Chapters
4 and 5, we build a parser for JSON.  The grammar is as follows
```
# Rustlr Grammar for JSON
auto
lifetime 'lt
lexterminal LBRACE {
lexterminal RBRACE }
lexterminal LBRACK [
lexterminal RBRACK ]
lexterminal LPAREN (
lexterminal RPAREN )
lexterminal COLON :
lexterminal COMMA ,
lexterminal NULL null
lexterminal MINUS -
valueterminal TRUE~ bool~ Alphanum("true")~ true
valueterminal FALSE~ bool~ Alphanum("false")~ false
valueterminal STRING~ &'lt str~ Strlit(n)~ &n[1..n.len()-1]
valueterminal NUM~ i64~ Num(n)~ n
valueterminal FLOAT~ f64~ Float(n)~ n
valueterminal BIGNUM~ &'lt str~ BigNumber(n)~ n

nonterminal Integer i64
nonterminal Floatpt f64
nonterminal Boolean bool
nonterminals Value KeyValuePair Number
nonterminal Object : Value
nonterminal List : Value
nonterminal Boolean : Value
topsym Value
resync COMMA RBRACK RBRACE

Integer --> MINUS?:m NUM:n {if m.is_some() {n*-1} else {n}}
Floatpt --> MINUS?:m FLOAT:n {if m.is_some() {-1.0*n} else {n}} 
Number:Bignum --> MINUS?:m BIGNUM
Number:Int --> Integer
Number:Float --> Floatpt
Boolean --> TRUE | FALSE

Value:Number --> Number
Value:Boolean --> Boolean
Value:Str --> STRING
Value --> Object
Value --> List
Value --> NULL
Value --> LPAREN Value RPAREN
KeyValuePair --> STRING COLON Value
List:List --> LBRACK Value<COMMA*> RBRACK
Object:Object --> LBRACE KeyValuePair<COMMA*> RBRACE
```

This grammar uses the latest variations on Rustlr syntax that were introduced
in more recent versions.
Note that the **`valueterminal`** lines, which combine `typedterminal` with
`lexvalue` declarations, requires that their components be separated by
a **`~`**.  The line for terminal STRING strips the string literal returned
by the tokenizer of this enclosing double quotes.

The grammar is a hybrid with some manually defined types and actions
along with automatically generated ones.
The AST types that are created by this grammar are
```
#[derive(Debug)]
pub enum Number<'lt> {
  Int(i64),
  Bignum(Option<()>,&'lt str),
  Float(f64),
  Number_Nothing,
}
impl<'lt> Default for Number<'lt> { fn default()->Self { Number::Number_Nothing } }

#[derive(Debug)]
pub enum Value<'lt> {
  Str(&'lt str),
  Number(Number<'lt>),
  NULL,
  Boolean(bool),
  Object(Vec<LBox<KeyValuePair<'lt>>>),
  List(Vec<LBox<Value<'lt>>>),
  Value_Nothing,
}
impl<'lt> Default for Value<'lt> { fn default()->Self { Value::Value_Nothing } }

#[derive(Default,Debug)]
pub struct KeyValuePair<'lt>(pub &'lt str,pub LBox<Value<'lt>>,);
```

The following is the Debug-output of a sample AST produced by the parser,
from which anyone familiar with JSON can surely discern the original source:
```
Object([KeyValuePair("firstName", Str("John")), KeyValuePair("lastName", Str("Smith")), KeyValuePair("isAlive", Boolean(true)), KeyValuePair("age", Number(Int(27))), KeyValuePair("address", Object([KeyValuePair("streetAddress", Str("21 2nd Street")), KeyValuePair("city", Str("New York")), KeyValuePair("state", Str("NY")), KeyValuePair("postalCode", Str("10021-3100"))])), KeyValuePair("phoneNumbers", List([Object([KeyValuePair("type", Str("home")), KeyValuePair("number", Str("212 555-1234"))]), Object([KeyValuePair("type", Str("office")), KeyValuePair("number", Str("646 555-4567"))])])), KeyValuePair("children", List([Str("Catherine"), Str("Thomas"), Str("Trevor")])), KeyValuePair("spouse", NULL)])
```

Finally, here is an alternative way to parse JSON objects into
Rust HashMaps.  We can override not only the type to be generated but the
semantic action as well.  Modify the declarations and rules for Object as
follows:
```
$use std::collections::HashMap;
nonterminal Object HashMap<&'lt str, LBox<@Value>>

Object ==> LBRACE KeyValPair<COMMA*>:entries RBRACE {
  let mut kvmap = HashMap::new();
  for (mut lbx) in entries {
    if let KeyValPair(k,v) = lbx.take() { kvmap.insert(k,v); }
  }
  kvmap
  } <==
```
The `$` directive is similar to `!`, except that it adds the verbatim line
only to the generated AST file as opposed to parser file. The syntax
**`@Value`** refers to the type of the `Value` nonterminal (or you can say
`Value<'lt>`, but you will in general know what type would be generated
by the system for the nonterminal: the `@` symbol offers a convenience.)

Each key-value pair inside the vector created by `<COMMA*>` is wrapped
inside an LBox.  We can take the value from the box with
[LBox::take][take] (which leaves a default value inside the box).

The debug output of the AST from the same JSON source would now be:
```
Object({"spouse": NULL, "age": Number(Int(27)), "phoneNumbers": List([Object({"type": Str("home"), "number": Str("212 555-1234")}), Object({"number": Str("646 555-4567"), "type": Str("office")})]), "children": List([Str("Catherine"), Str("Thomas"), Str("Trevor")]), "lastName": Str("Smith"), "address": Object({"streetAddress": Str("21 2nd Street"), "city": Str("New York"), "state": Str("NY"), "postalCode": Str("10021-3100")}), "isAlive": Boolean(true), "firstName": Str("John")})
```

Although we may at times want to insert such overrides, most of the
automatically generated portions remain usable.

Here are links to the [grammar](https://cs.hofstra.edu/~cscccl/rustlr_project/jsontypes/json.grammar), [parser](https://cs.hofstra.edu/~cscccl/rustlr_project/jsontypes/src/jsonparser.rs), [ast types](https://cs.hofstra.edu/~cscccl/rustlr_project/jsontypes/src/json_ast.rs) and [main](https://cs.hofstra.edu/~cscccl/rustlr_project/jsontypes/src/main.rs) of the project, which may differ slightly
from the above.


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
[chap4]:  https://cs.hofstra.edu/~cscccl/rustlr_project/chapter4.html
[lexsource]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.LexSource.html
[drs]:https://docs.rs/rustlr/latest/rustlr/index.html
[tktrait]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/trait.Tokenizer.html
[tt]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html
[rtk]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/enum.RawToken.html
[fromraw]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html#method.from_raw
[nextsymfun]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/trait.Tokenizer.html#tymethod.nextsym
[zcp]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.ZCParser.html
[ttnew]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html#method.new
[take]:https://docs.rs/rustlr/latest/rustlr/generic_absyn/struct.LBox.html#method.take
[c11]:https://cs.hofstra.edu/~cscccl/rustlr_project/cparser/cauto.grammar
[apnd]:  https://cs.hofstra.edu/~cscccl/rustlr_project/appendix.html
