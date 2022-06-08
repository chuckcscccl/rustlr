# RustLr: LR Parser Generator

## Introduction

It's been decades since Donald Knuth introduced LR parsing and proved that
*every deterministic context free language has an LR(1) grammar.*  That is
quite an impressive result, and yet the question as to what is the best
approach to creating parsers has never been settled.  The flavors of parser
generator compete like the kinds of programming languages they help to
implement.  Convincing someone that LR is better than LL (or vice-versa)
is as hard as convincing someone that functional programming is better than
OOP.

Despite the strongest theoretical results, LR parsing requires a
rather steep learning curve when compared to top-down parsing
techniques such as recursive descent.  Recent developments such as
Parsing Expression Grammars (PEGs) has made top-down parsing even more
attractive despite the ever-present problem of left-recursion.

goal1 towards EBNF
goal2 towards 
