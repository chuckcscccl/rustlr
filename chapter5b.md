##  Deep Pattern Matching with Bump-Allocated ASTs

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
types.  Consider the following type:
```
enum Exp<'t> {
  Var(&'t str),
  Negative(&'t Exp<'t>),
  Plus(&'t Exp<'t>, &'t Exp<'t>),
  Minus(&'t Exp<'t>, &'t Exp<'t>),
}
```
and a function that tries to "pretty-print" such expressions
