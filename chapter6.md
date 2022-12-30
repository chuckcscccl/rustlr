##  Generating [Bump][bumpalo]-Allocated ASTs

Abstract syntax trees are usually defined recursively.  This normally means
some form of smart pointer is requried to implement them.  The problem with
this aspect of Rust is that the smart pointers block *nested* patterns from
matching against AST expressions.  It is possible to define recursive structures
using references (borrows) but all references in an AST structure must have
compatible lifetimes.  An "arena" structure is required to hold the values that
they reference.  While it's possible to define such an arena manually, rustlr
provides support for using the [bumpalo][bumpalo] crate.

A grammar can begin with the declaration
```
auto-bump
```
in place of `auto`, which enables the generation of bump-allocated ASTs.
**It is also necessary to place `bumpalo = "3"` in your crate dependencies**
(in Cargo.toml). Although user code do not need to reference the crate
directly, the generated parser code does.

The disadvantage of bumpalo is that it bipasses some of the memory
safety checks of Rust. Bump-allocation is not recommended if frequent
changes are made to the allocated structures.  They are appropriate if
the AST remains relatively stable once created, with few changes if
any, until the entire structure can be dropped.

The advantage of bump-allocation, besides an increase in speed, is
that it enables the matching of nested patterns against recursive
types.  Consider the following recursive type that tries to avoid
smart pointers:
```
enum Expr<'t> {
  Var(&'t str),
  Negative(&'t Expr<'t>),
  Plus(&'t Expr<'t>, &'t Expr<'t>),
  Minus(&'t Expr<'t>, &'t Expr<'t>),
}
```
and a function that "pretty-prints" such expressions. The function will print
`x` instead of `--x`, `x-y` instead of `x+-y`, and `x+y` instead of `x--y`.
It also pushes negation inside plus/minus, and only prints parentheses as
required by the non-associative minus symbol.
```
fn pprint<'t>(expr:&'t Expr<'t>,) {
  use crate::Expr::*;
  match expr {
    Negative(Negative(n)) => pprint(n),
    Negative(Plus(a,b)) => pprint(&Minus(&Negative(a),b)),
    Negative(Minus(a,b)) => pprint(&Plus(&Negative(a),b)),
    Negative(n) => {print!("-"); pprint(n)},
    Plus(a,Negative(b)) => pprint(&Minus(a,b)),
    Minus(a,Negative(b)) => pprint(&Plus(a,b)),
    Minus(a,p@Minus(_,_)) => { pprint(a); print!("-("); pprint(p); print!(")")},
    Minus(a,p@Plus(_,_)) => { pprint(a); print!("+("); pprint(p); print!(")")},
    Plus(a,b) => {pprint(a); print!("+"); pprint(b);},
    Minus(a,b) => {pprint(a); print!("-"); pprint(b)},    
    Var(x) => print!("{}",x),
  }//match expr
}//pprint
```
Then given
```
    let q = Plus(&Negative(&Negative(&Var("x"))),&Negative(&Var("y")));
    pprint(&q);
```
will print `x-y`. Such a "declarative" style of programming is not
possible if the type is defined using Box (or [LBox][2]).

Notice that the references to new structures created by the function are
passed recursively "up-stack", and so are safe.
However, allocating such structures temporarily on the stack is impractical.
We would not be able to return references to them.
You may also get the dreaded compiler error *"creates a temporary which is freed while still in use"* when writing expressions such as `Plus(&q,&Negative(&q))`.

The [bumpalo][bumpalo] crate offers an "arena", a kind of *simulated heap*,
that allow us to access these structures within the lifetime of the arena:
```
 let bump = bumpalo::Bump::new(); // creates new bump arena
 let p = bump.alloc(Negative(bump.alloc(Negative(bump.alloc(Var("z"))))));
```
The [Bump::alloc](https://docs.rs/bumpalo/latest/bumpalo/struct.Bump.html#method.alloc) function returns a reference to the allocated memory within
the arena.  We can pass a reference to the "bump" to a function, which would
then be able to construct and return bump-allocated structures using references
of the same lifetime.

Rustlr (since version 0.3.93) can generate such structures automatically.
Most of bumpalo's usage is well-encapsulated, although
`bumpalo = "3"` does need to be added to the crate's
dependencies.  The easiest way to enable bumpalo is through enhancements to
the [Lexsource][lexsource] structure.  The following code fragment
demonstrates how to envoke a parser generated from an `auto-bump` grammar,
[bautocalc.grammar](https://cs.hofstra.edu/~cscccl/rustlr_project/bumpcalc/bautocalc.grammar),
```
   let srcfile = "test1.txt"; // srcfile names file to parse
   let source=LexSource::with_bump(srcfile).unwrap();
   let mut scanner = bautocalcparser::bautocalclexer::from_source(&source);   
   let mut parser = bautocalcparser::make_parser();
   let result = bautocalcparser::parse_with(&mut parser, &mut scanner);
```
A Rustlr Lexsource object containing a [Bump](https://docs.rs/bumpalo/latest/bumpalo/struct.Bump.html) is created with [Lexsource::with_bump][withbump].
The `parse_with` function, which is generated for individual parsers, will place
a reference to the bump arena inside the parser.  The automatically
generated semantic actions will call [Bump::alloc](https://docs.rs/bumpalo/latest/bumpalo/struct.Bump.html#method.alloc) to create ASTS that will have **the
same lifetime as the Lexsource structure.**

The link to the entire sample crate is [here](https://cs.hofstra.edu/~cscccl/rustlr_project/bumpcalc/).


#### **Replace the LBox**

Although the [LBox][2] is no longer needed, a device to capture the
lexical position (line/column) information in the AST in a
non-intrusive manner is still re required.  Along with the `auto-bump`
option is the struct [LC][lc].  This is just a tuple with open fields
for the value, and an inner tuple with line and column numbers. We've
implemented The Deref/DerefMut traits so the value can be exposed in
the manner of a smart pointer, but there's in fact no pointer
underneath.  Inside a grammar, right-hand side labels such as `[a]`
will automatically bind `a` to an LC struct and generate AST types
using LC.  For example,
```
Expr:Div --> Expr:[e1] / Expr:[e2]
```
will generate an `Expr<'lt>` enum with a variant
```
Div{e1:&'lt LC<Expr<'lt>>,e2:&'lt LC<Expr<'lt>>},
```
as well as semantic actions that inserts line/column information into the
AST as LC enclosures.

Note that a lifetime argument is required for all recursive types.

#### **Dealing with Recursive Structs**

There is currently one minor limitation with the `auto-bump` option.  When
a struct is recursive, it may not be possible to generate code that
`#[derive(Default)]`, which is required for all types in Rustlr, without some
help from the user.  A reference type `&A` does not have a default and so
a type `A` that contains a reference to itself also does not have a default. In
such cases, help from the user is required to implement the trait for
these reference types.  For example, 
```
A1 --> B1 int
B1 --> var var A1
flatten B1
```
would generate the types
```
#[derive(Default,Debug)]
pub struct A1<'lt>(pub &'lt str,pub &'lt str,pub &'lt A1<'lt>,pub i64,);

#[derive(Default,Debug)]
pub struct B1<'lt>(pub &'lt str,pub &'lt str,pub &'lt A1<'lt>,);
```
but will fail to compile. The following warning will be given by rustlr:
```
WARNING: Recursive structs may require the manual implementation of the Default trait for reference types, as in
  impl<'lt> Default for &'lt A1<'lt> ...
```
The warning should be heeded by including the following in the grammar:
```
$static A1DEFAULT:A1<'static> = A1("","",&A1DEFAULT,0);
$impl<'t> Default for &'t A1<'t> { fn default() -> Self { &A1DEFAULT } }
```
Lines that begin with `$` are injected into the `_ast.rs` file created from the
grammar.  These definitions will allow the `#[derive(Default)]` tags for the
`A1` and `B1` types to be effective.  Currently, such lines are not generated
automatically because of Rust's restrictions to what static definitions can
reference.

This problem can also be avoided by just using enums instead of
structs in such cases: just assign a left-side label to the sole
production rules for `A1` and `B1`.  Enums always include a `_Nothing` variant
so that a default can always be defined.


#### **Requisitioning the 'External State'**

There is another implemention-related detail that may be of occassional
concern.

To integrate the bumpalo option into Rustlr with minimal disturbance to its
other components, the [Bumper](https://docs.rs/rustlr/0.3.97/rustlr/generic_absyn/struct.Bumper.html) struct was introduced.  A grammar with the `auto-bump`
option will generate a parser with a Bumper struct as its `exstate` field.
This struct contains procedures to access the encapsulated bump arena.  One
can still declare and use an arbitrary `externtype`
for a *Bumper* will also include
a field of this type, which is accessible as a ref mut via the
[Bumper::state](https://docs.rs/rustlr/0.3.97/rustlr/generic_absyn/struct.Bumper.html#method.state) procedure.

In most cases, this detail is only required to be understood if the
`exstate` is to be used for another purpose, in which case all references
to `parser.exstate` should be replaced by `parser.exstate.state()`.
In addition to `exstate` each parser also contains a separate,
reference-counted `shared_state` field of the same type.  
The only other situation that requires understanding of this implementation
detail is when writing semantic actions manually that creates
bump-allocated structures.  They should be created by calling
`parser.exstate.make(...)`.

#### **Yet another calculator...**

We conclude this Chapter with a full grammar with the `auto-bump` option.
Two sample productions with manually written semantic actions are included
after `EOF` to illustrate accessing the `parser.exstate` as a "Bumper".
```
# auto-generate bump-allocated ASTs with refs instead of smart pointers
auto-bump
lifetime 'lt
nonterminals E ES
nonterminals A1 B1 C1 A2
terminals + - * / ( ) = ;
terminals let in
valueterminal int ~ i64 ~ Num(n) ~ n
valueterminal var ~ &'lt str ~ Alphanum(x) ~ x
topsym ES
resync ;

left * 500
left / 500
left + 400
left - 400
nonassoc = 300

lexattribute set_line_comment("#")

E:Val --> int
E:Var --> var
E:Plus --> E + E
E:Minus --> E - E
E:Times --> E * E
E:Div --> E:[e1] / E:[e2]
E(600):Neg --> - E:[e]
E:Let --> let var = E in E
E --> ( E )
ES --> E<;+> ;?

A1 --> B1 int
B1 --> var var A1
flatten B1
flatten A2
C1 --> A2 A1 B1 ; E
A2 --> var int

$static A1DEFAULT:A1<'static> = A1("","",&A1DEFAULT,0);
$static B1DEFAULT:B1<'static> = B1("","",&A1DEFAULT);
$impl<'t> Default for &'t A1<'t> { fn default() -> Self { &A1DEFAULT } }
$impl<'t> Default for &'t B1<'t> { fn default() -> Self { &B1DEFAULT } }

EOF

#Unused productions for example:
ES ==> E:n ; {
  let bump = &parser.exstate;
  let mut v1 = Vec::new(); /* not bump-allocated */
  v1.push(bump.make(parser.lc(0,n)));
  Seq(v1)
  } <==
  
ES ==> ES:@Seq(mut v)@  E:e ;  {
   v.push(parser.exstate.make(parser.lc(1,e)));
   Seq(v)
   } <==
```

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
[bumpalo]: https://docs.rs/bumpalo/latest/bumpalo/index.html
[withbump]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.LexSource.html#method.with_bump
[lc]:https://docs.rs/rustlr/latest/rustlr/generic_absyn/struct.LC.html