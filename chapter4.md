## Chapter 4: Automatically Generating the AST and Using Regex in Grammars

Rustlr is capable of automatically generating the data types
(enums and structs) for the abstract syntax of a language as well as
the semantic actions required to create instances of those types.
For beginners new to writing grammars and parsers, we **do not**
recommend starting with an automatically generated AST.  The user must
understand clearly the relationship between concrete and abstract
syntax and the best way to learn this relationship is by writing ASTs
by hand, as demonstrated in the previous chapters.  Even with Rustlr
capable of generating nearly everything one might need from a parser,
it is still possible that careful fine tuning will be required.

Automatically creating the AST from the grammar (or the grammar from
the AST) is not a new idea and can cause problems if not done
carefully.  For example, one usually wants the AST to stay relatively
stable even when the grammar changes. Also, the grammar is usually
written to the contraints of LR parsing, and may not reflect what one
wants the AST to look like.  However, Rustlr essentially defines a
language, embedded with the grammar syntax, for describing how ASTs
should be generated.  The language will allow one to create ASTs that
are relatively stable and independent of the format of the grammar.
It is also possible to override the types and semantic actions of any
non-terminal, and use a hybrid approach between automatic and manually
written AST generation.

We redo the enhanced calculator example from [Chapter 2][chap2].
Although some form of abstract syntax can be generated for any
grammar, the format of the grammar can greatly influence the form of
the AST types.  To illustrate the various choices, this grammar is a
hybrid between the purely unambiguous grammar of [Chapter
1][chap1] and the one in [Chapter 2][chap2] in that operator
precedence declarations are only given for the binary arithmetic
operators.  For the unary minus and the `=` sign in let-expressions,
we choose to define different syntactic categories in the form of
extra non-terminal symbols `UnaryExpr` and `LetExpr`.  Along with
`Expr` they define three levels of precedence from weakest to
strongest: `LetExpr`, `Expr`, and `UnaryExpr`.  Writing ambiguous
grammars with operator precedence/associativity declarations is
convenient and can make the grammar more readable.  They also lead to
more reasonable abstract syntax.  Symbols such as `T` in `E --> T`
often have no meaning at the abstract syntax level.  However, when
there are a large number of operators and precedence levels, using such
declarations alone may be problematic (See the original [ANSI C][c11] grammar).
Besides, these categories sometimes have genuine semantic meaning,
such as the distinction between lvalues and rvalues.  

The following grammar is found
[here](https://cs.hofstra.edu/~cscccl/rustlr_project/autocalc/calcauto.grammar).
```
# the auto directive means AST types and semantic actions will be generated
auto
lifetime 'lt
terminals + - * / ( ) = ;
terminals let in
valueterminal int ~ i64 ~ Num(n) ~ n
valueterminal var ~ &'lt str ~ Alphanum(n) ~ n
lexattribute set_line_comment("#")

nonterminals Expr ExprList
nonterminal UnaryExpr : Expr
nonterminal LetExpr : Expr

topsym ExprList
resync ;

left * 500
left / 500
left + 400
left - 400

UnaryExpr:Val --> int
UnaryExpr:Var --> var
UnaryExpr:Neg --> - UnaryExpr
UnaryExpr --> ( LetExpr )

Expr --> UnaryExpr
Expr:Plus --> Expr + Expr
Expr:Minus --> Expr - Expr
Expr:Div --> Expr / Expr
Expr:Times --> Expr * Expr

LetExpr --> Expr
LetExpr:Let --> let var = Expr in LetExpr

ExprList:nil -->
ExprList:cons --> LetExpr ; ExprList

EOF
```

Note the following, further differences between this grammar and the one presented in [Chapter 2][chap2]:
1. There are no semantic actions
2. There is no "absyntype" or "valuetype" declaration; any such declaration would be ignored when using the `auto` option, which is enabled by the `auto`
directive at the top of the grammar, or by the `-auto` flag given to the
rustlr executable.
3. Only the types of values carried by certain terminal symbols must be declared (with `typedterminal` or `valueterminal`).
A `valueterminal` declaration is
just a combination of a `typedterminal` and a `lexvalue` declaration, with `~` separating the components.  The other terminals all have type () (unit).

4. The non-terminal symbol on the left-hand side of a production rule may carry a label.  These labels will become the names of enum variants to be created.

Process the grammar with **`rustlr calcauto.grammar`** (without the `auto` directive inside the grammar, run rustlr with the `-auto` option).   Two files are created.  Besides **[calcautoparser.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/autocalc/src/calcautoparser.rs)** there will be, in the
same folder as the parser, a **[calcauto_ast.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/autocalc/src/calcauto_ast.rs)** with the following (principal) contents:
```rust
#[derive(Debug)]
pub enum ExprList<'lt> {
  nil,
  cons(Expr<'lt>,LBox<ExprList<'lt>>),
  ExprList_Nothing,
}
impl<'lt> Default for ExprList<'lt> { fn default()->Self { ExprList::ExprList_No
thing } }

#[derive(Debug)]
pub enum Expr<'lt> {
  Plus(LBox<Expr<'lt>>,LBox<Expr<'lt>>),
  Minus(LBox<Expr<'lt>>,LBox<Expr<'lt>>),
  Div(LBox<Expr<'lt>>,LBox<Expr<'lt>>),
  Times(LBox<Expr<'lt>>,LBox<Expr<'lt>>),
  Var(&'lt str),
  Neg(LBox<Expr<'lt>>),
  Val(i64),
  Let(&'lt str,LBox<Expr<'lt>>,LBox<Expr<'lt>>),
  Expr_Nothing,
}
impl<'lt> Default for Expr<'lt> { fn default()->Self { Expr::Expr_Nothing } }
```
Compare these types with the manually written ones in [Chapter 2][chap2]: they
are not so different.
For example, the expression `5 - 7 - -9` will be represented in
abstract syntax as `Minus(Minus(Val(5), Val(7)), Neg(Val(9)))`.  This is
exactly what we want.

Generally speaking, a new type is created for each non-terminal symbol
of the grammar, which will also share the same name as the
non-terminal itself.  But this would mean that separate types would be
created for `LetExpr` and `UnaryExpr` as well, which would lead to
convoluted types that serve no purpose.  Their creation was avoided
with the following declarations:
```
nonterminal UnaryExpr : Expr
nonterminal LetExpr : Expr
```
The syntax means that instead of generating new types, the ASTs representing
the rules for `UnaryExpr` and `LetExpr` would *extend* the enum that would
be created for `Expr`.  The type created for Expr must be an enum for this
to work (it would not work if it was a struct).
Leave out the `: Expr` portion from the declarations and we will get
[these types](https://cs.hofstra.edu/~cscccl/rustlr_project/autocalc/src/calcb_ast.rs) instead.  They would be more cumbersome to work with.  


#### Rules of AST Generation

An enum is created for each non-terminal symbol of the grammar that appears on the left-hand side of multiple production rules, unless the type of the non-terminal is declared to "extend" another type as explained above. The name of the enum is the
same as the name of the non-terminal.
The names of the variants are derived from the labels given to the left-hand side nonterminal, or are automatically generated from the nonterminal name and the rule number (e.g. `Expr_8`).  A special `Nothing` variant is also created to represent a default.
There is normally an enum variant for each production rule of this non-terminal.  Each variant is composed of the right-hand side
symbols of the rule that are associated with non-unit types.  Unit typed values
can also become part of the enum if the symbol is given a label.  For example:
  **` E:acase -->  a E `**  where terminal symbol `a` is of unit type, will result in a enum variant
`acase(LBox<E>)`. whereas
  **` E:acase -->  a:m E `**
will result in a variant `acase((),LBox<E>)`

A struct is created for any non-terminal symbol that appears on the
left-hand side of exactly one production rule, unless the type of that
nonterminal is declared to extend another type.
You can also force an enum to be created instead of a struct by
giving the singleton rule a left-hand side label, in which case the label
will name the sole variant of the enum (besides the `_Nothing` default).
This would be required when you know that the type will be extended with
other variants, as demonstrated above.

The name of the struct is the same as the non-terminal.  If any of the grammar symbols
on the right-hand side of the rule is given a label, it would create a struct
with the fields of each struct named by these labels, or
with `_item{i}_` if
no labels are given.  For example, a nonterminal `Ifelse` with a singleton rule
  ```
  Ifelse --> if Expr:condition Expr else Expr
  ```
will result in the generation of:
  ```
  #[derive(Default,Debug)]
  pub struct Ifelse {
    pub condition: LBox<Expr>,
    pub _item2_: LBox<Expr>,
    pub _item4_: LBox<Expr>,
  }
  ```
The `_item{i}_` labels are generated from the positions of the corresponding
symbols on the right-hand side of the production rule.
If none of the symbols on the right have labels, rustlr creates a tuple
struct.  For Example a singleton rule such as **`whileloop --> while ( expr ) expr`**
will produce an a `struct whileloop(expr,expr);`  Be careful to avoid
using Rust keywords as the names of non-terminals.

The struct may be empty if all right-hand-side symbols of the single production
rule are associated with the unit type and do not have labels. Rustlr will
generate code to derive the Debug and Default traits for all structs (this
works fine for recursive structs).

Rustlr also calculates a reachability closure so it is aware of which
non-terminals are mutually recursive.  It uses this information to
determine where smart pointer are required when defining these
recursive types.  Rustlr always uses its [LBox][2] custom smartpointer
to also include line/column information.  Notice that the variant
`enum::cons` has only the second component in an LBox.  One can, for
the sake of recording position information, always create an LBox
regardless of reachability by giving the component a "boxed label".
That is, `ExprList:cons --> Expr:[car] SEMICOLON ExprList` will
generate a variant that also has its first component in an LBox.  The
reachability relation also determines if a type requires a lifetime
parameter.

Although the generated parser code may not be very readable, rustlr also generated semantic actions that create instances of these AST types.  For example, the rule `Expr:Plus --> Expr + Expr` will have semantic action equivalent to one created from:

```
Expr --> Expr:[a] + Expr:[b] {Plus(a,b)}
```

Recall from [Chapter 2][chap2] that a label of the form `[a]` means that the semantic value associated with the symbol is enclosed in an [LBox][2].

##### 'Passthru'

There are three production rules in the grammar that do not
correspond to enum variants: `Expr --> UnaryExpr`, `LetExpr --> Expr`
and `UnaryExpr --> ( LetExpr )`. 
Rustlr infers from the fact that
  1. there is no left-hand side label for any of these rules
  2. There is exactly one grammar symbol on the right-hand side that has a non-unit
     type, and that type is the same as the type of the left-hand side symbol.
     The other symbols, if any, are of unit type
  3. There are no labels nor operator precedence/associativity declarations for the other symbols.
     
For the rule `UnaryExpr --> ( LetExpr )`, it therefore infers that the parentheses on the right hand side carry no meaning at the AST level, and thus generates a semantic action for this rule
that would be equivalent to:
```
  UnaryExpr --> ( LetExpr:e ) { e }
```
We refer to such cases as "pass-thru" cases.  If the automatically
inferred "meaning" of this rule is not what's desired, it can be
altered by using an explicit left-side label: this will generate a
separate enum variant (at the cost of an extra LBox) that
distinguishes the presence of the parentheses.  Note that the 
rule `UnaryExpr:Neg --> - UnaryExpr`, was not recognized as a pass-thru
case by virtue of the left-hand side label `Neg`.  Unlike the parentheses,
the minus symbol certain has meaning beyond the syntactic level.
We can also force the minus sign to be
included in the AST by giving it an explicit lable such as `-:minus UnaryExpr`.
This would create an enum variant that includes a unit type value.


##### 'Flattening' Structs

Rustlr provides another way to control the generation of ASTs so that
it is not always dependent on the structure of the grammar, although
it is not illustrated in the calculator example.  When writing a
grammar, we sometimes create extra non-terminal symbols and rules for the
purpose of organization.  As an abstract example:
```
A --> a Threebs c
Threebs --> b b b
```
Rustlr will create two tuple structs for these types. Assuming that a, b, c
are not of unit type, there will be a `struct A(a,Threebs,c)` and a
`struct Threebs(b,b,b)`.  However, it is possible to declare in the grammar,
once the non-terminals `A` and `Threebs` have been declared, that the
type `Threebs` can be **flattened** into other structures:
```
flatten Threebs
```
This means that the AST for Threebs should be absorbed into other types if
possible.
This will still create a `struct A(a,Threebs,c)`, but it will create for A:
**`struct A(a,b,b,b,c)`**.

Both structs and enums can absorb 'flatten' types.  However, there
are several enforced rules governing the flattening of types:
  1. Only struct types can be flattened: thus only nonterminals that has but a
  single production rule can have its AST absorbed into other types. Enum
  types can absorb 'flatten' structs but cannot be absorbed into other types.
  2. Types already defined to 'extend' the enum of another type cannot be
  flattened
  3. A tuple struct can only absorb the flattened form of another tuple struct.
  In the above example, if `Threeb` was a non-tuple struct with named fields (which can be created
  by giving of the the b's a label), then it cannot be absorted into `A`.
  4. Recursive structs, directly or indirectly, cannot be flattened.
  5. A boxed-labeled field cannot absorb a 'flatten' type.  That is, if
  the rule for `A` above was written `A --> a:a Threebs:[b] c:c` then the AST
  for A would become `pub struct A{a:a, b:LBox<Threebs>, c:c}`.  This is
  the only way to prevent the absorption of a 'flatten' type on a case-by-case
  basis.


##### Importance of Labels

In general, the usage of labels greatly affect how the AST datatype is
generated.  Labels on the left-hand side of a production rule give
names to enum variants.  Their presence also cancel "pass-thru"
recognition by always generating an enum variant for the rule.
A left-hand side label will also prevent a struct from being generated even
when a nonterminal has but a single production rule.
Labels on the right-hand side give names to struct components.  Their
presence on unit-typed grammar symbols means that the symbols won't be
ignored and will be included in the the type.  If a non-terminal has a
single production rule, the lack of any labels left or right leads
to the creation of a simpler tuple struct.  The use of boxed
labels such as `[e]` forces the semantic value to be wrapped inside an LBox
whether or not it is required to define recursive types.  Boxed labels also
prevent the absorption of 'flatten' types.


#### Overriding Types and Actions

It is always possible to override the automatically generated types and actions.
In case of ExprList, the labels 'nil' and 'cons' are sufficient for rustlr to create a linked-list data structure.  However, the right-recursive grammar rule is slightly non-optimal for LR parsing (the parse stack grows until the last element of the list before ExprList-reductions take place).  One might wish to use a left-recursive rule and a Rust vector to represent a sequence of expressions.  This can be done in several ways, one of which is by making the following changes to the grammar.  First, change the declaration of the non-terminal symbol `ExprList` as follows:

```
nonterminal ExprList Vec<LBox<Expr<'lt>>>
```
You probably want to use an LBox even inside a Vec to record the line/column
position information.
Then replace the two production rules for `ExprList` with the following:

```rust
ExprList --> { vec![] }
ExprList --> ExprList:ev LetExpr:[e] ; { ev.push(e); ev }
```
When writing your own types and actions alongside automatically generated ones,
it's best to examine the types that are generated to determine their correct
usage: for example, whether a lifetime parameter is required for `Expr`.

The presence of a non-empty semantic action will override automatic AST generation. It is also possible to inject custom code into the
automatically generated code:
```
ExprList -->  {println!("starting a new ExprList sequence"); ... }
```
The ellipsis are allowed only before the closing right-brace.  This indicates
that the automatically generated portion of the semantic action should follow.
The ellipsis cannot appear anywhere else.

An easier way to parse a sequence of expressions separated by ; and to
create a vector is to
use the special suffixes `+`, `*`, `?`, `<_*>` and `<_+>`.  These are described below.

```

```


   -------------------

### Automatically Adding New Rules with *, + and ?

Rustlr allows the use of regular-expression style symbols *, + and ?
to automatically generate new production rules.  However, these
symbols cannot be used unrestrictedly to form arbitrary regular
expressions. They cannot be nested.  They are also guaranteed to only
fully work in the -auto mode.

Another way to achieve the same effects as the above (to derive a vector for symbol ExprList) is to use the following alternative grammar declarations:

```
nonterminal ExprList Vec<LBox<Expr<'lt>>>
ExprList --> (LetExpr ;)*
```
This would lead to the generation of a tuple struct for type ExprList:
```
#[derive(Default,Debug)]
pub struct ExprList<'lt>(pub Vec<LBox<Expr<'lt>>>,);
```
You can also eliminate the extra type for `ExprList` entirely by manually
defining the type as:
```
nonterminal ExprList Vec<LBox<Expr<'lt>>>
```
This would cause

The operator **`*`** means a sequence of zero or more.  This is done by generating several new non-terminal symbols internally.  Essentially, these correspond to

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

Other alternatives are possible:
```
nonterminal ExprList
ExprList:Sequence --> (Expr ;)*
```
This would generate a new struct type for the AST of ExprList, with a component of
type
`Vec<LBox<Expr<'lt>>>`.  If the type of ExprList is declared manually
as above, rustlr infers that the appropriate semantic action is equivalent to
`ExprList --> (Expr ;)*:v {v}`  because there is only one symbol (the internally
generated ES1) on the right-hand side, and it is of the same type.
**The label given for such an expression cannot be a pattern that includes `@...@`.**  It can only be a simple alphanumeric label or a boxed label (`[x]`).  These restrictions may eventually be eliminated in future releases.

Another restriction is that the symbols `(`, `)`, `?`, `*` and `+` may not
be separated by white spaces since that would confuse their interpretation
as independent terminal symbols.  For example, `( Expr ; ) *` is not valid.

Yet another alternative is to manually define the type of ExprList, from which Rustlr will infer that no struct/enum needs to be created for it:
```
nonterminal ExprList Vec<LBox<Expr<'lt>>>
ExprList --> (LetExpr ;)*
```
This is because rustlr generates an internal non-terminal to represent the right-hand side `*` expression and assigns it type `Vec<LBox<Expr<'lt>>>`.
It then recognizes that this is the only symbol on the
right, which is of the same type as the left-hand side nonterminal `ExprList`
as declared. This rule will again be given an action equivalent to
`ExprList --> (Expr ;)*:v {v}`

In addition to the `*`, `+` and `?` suffixes, rustlr also recognizes (non-nested)
suffixes such as **`<Comma*>`** or **`<;+>`**.  Assuming that
`Comma` is a declared terminal symbol of the grammer, the expression
`Expr<Comma+>` represents a sequence of one or more Expr separated by Comma,
but not ending in Comma, e.g *a,b,c* but not *a,b,c,*.  In contrast,
`(Expr Comma)+` means that the expression must end in a Comma.  <Comma*>
allows the sequence to be empty. The AST generator will also create vectors
as the semantic values of such expressions.  Please avoid whitespaces in
these expressions: `<Comma *>` is not recognized.

Be warned that overusing `+`, `*` and `?`, especially in the same
production rule, can easily lead to new ambiguities in the grammar, especially
with `*` and `?`.  The new productions generated for these operators
could lead to additional Shift-Reduce and even
Reduce-Reduce conflicts.  For example, a production with right-hand side
**`Expr<Comma*> Comma?`** will lead to a shift-reduce conflict. However,
**`Expr<Comma+> Comma?`** will not.

Generally speaking, context free grammars do not have the same
properties as regular expressions.  Regex operators including
`*`, `+` and `?` can be used in any combination such as `(a+a?)*` and we
can always create an optimized, deterministic finite state machine to
recognize it.  But grammars cannot be composed without losing their
properties.  A rule like `A --> B* B*` is hopelessly ambiguous as
there is no way to determine where the first sequence should stop and
the second one begins.  In such a situation, rustlr will still create a
parser by using defaults such as shift-over-reduce, but not before printing
out a warning.

The motivation for adding regex-like operators to a parser generator
are both usability and improved ability in creating abstract syntax
automatically.  But much work needs to be done before we can parse
arbitrary EBNF syntax.  We are currently exploring extensions of LR
parsing including *delayed reductions*, which can potentially allow
non-ambiguous grammars to be more easily composed without running into
new conflicts: see the [Appendix][apnd] of this tutorial for experimental
features.


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

Please note that using [LBox][2] is already included in all parsers generated with the `-genabsyn` or `-auto` option, so do not use `!use ...` to include
it again.

   ----------------

### JSON Parser

To give another, complete example of the features described in this chapter,
we build a parser for JSON.  The grammar is as follows
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

*The former section named "Generating a Parser for C" is being updated and
moved to a new chapter.*

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
[take]:https://docs.rs/rustlr/latest/rustlr/generic_absyn/struct.LBox.html#method.take
[c11]:https://cs.hofstra.edu/~cscccl/rustlr_project/cparser/cauto.grammar
[apnd]:  https://cs.hofstra.edu/~cscccl/rustlr_project/appendix.html
