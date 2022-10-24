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

The disadvantage of bumpalo is that it bipasses some of the memory
safety checks of Rust. Bump-allocation is not recommended if frequent
changes are made to the allocated structures.  They are appropriate if
the ASTs remain relatively stable once created, with few changes if
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

Rustlr (since version 0.3.93) can now generate such structures automatically.
To use this feature, it is necessary to include `bumpalo = "3"` in a crate's
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
generated semantic actions will call [Bump::alloc](https://docs.rs/bumpalo/latest/bumpalo/struct.Bump.html#method.alloc) to create ASTS that will have the
same lifetime as the Lexsource structure.

The link to the entire sample crate is [here](https://cs.hofstra.edu/~cscccl/rustlr_project/bumpcalc/).


#### Replace the LBox

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