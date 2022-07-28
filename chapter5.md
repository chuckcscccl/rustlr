## Advance Tutorial: Generating a Parser for C

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


The AST types are found [here](https://cs.hofstra.edu/~cscccl/rustlr_project/cparser/src/cauto_ast.rs) and the generated parser [here](https://cs.hofstra.edu/~cscccl/rustlr_project/cparser/src/cautoparser.rs).

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
