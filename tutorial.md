## RUSTLR: Bottom-Up Parser Generator for Rust, Version 0.4.x

<HR>

[Rustlr](https://crates.io/crates/rustlr) was originally created as a
research and educational platform that explores advanced ideas in
formal grammars and parsing in a practical setting.  It is also
designed to be usable and simplifies the process of parsing and the
construction of abstract syntax structures (ASTs).
From its origins as a barely functional parser generator rustlr has
gradually acquired more advanced capabilities that allow it to compete
with other parser generators in terms of practicality. 

 1. The option of automatically generating the abstract syntax datatypes 
and required semantic actions from hints given in the grammar, 
in whole or in part.  The AST types do not necessarily mirror the format
of the grammar: one can declare dependencies that allow multiple non-terminal
symbols to share a common type (see Chapters 4 and 6 of the tutorial).

 2. The option of generating recursive AST types using the "bumpalo" crate,
which enables deep pattern matching on recursive structures by replacing
smart pointers with transparent references.

 3. The possibility of using regular-expression style operators such as
*, + and ? directly inside the grammar.  This is not as easy to do as it
sounds as the additional grammar rules required to
support them may lead to new ambiguities.

 4. The option of using a larger class of unambiguous grammars 
compared to traditional LR and LALR, based on [Selective Marcus-Leermakers] delayed reductions. See the [Appendix][appendix].
Other experimental features include an "unexpected wildcard" symbol.

 5. The ability to interactively train the parser to give better error messages.

 6. Automatically generates a lexical scanner from the declaration of
   terminal symbols.


Rustlr also implements capabilities found in traditional parsing tools
such as operator precedence and associativity declarations. 
The goal of [Rustlr](https://crates.io/crates/rustlr) is to
*round up all the hoofed herbivores* into a tool that's both usable
and enjoys the expressive power of LR grammars and beyond.  
It's been decades since Donald Knuth proved that <i>every
deterministic context free language has an LR(1) grammar</i>.  However, such
theoretical properties never settled the dispute between LR-style parer 
generators and those based on other techniques.  One reason is that users of
LR parsing have always faced a steep learning curve.  How an LR parser works
is not as intuitive as, for example, a recursive descent parser.  To alleviate 
this problem Rustlr implements a collection of features including:

It is designed for the parsing of programming language
syntax.  It is not designed to parse natural languages, or binary
data, although there's also nothing that prevents it from used for
those purposes.  Rustlr generates parsers for Rust and for F\#
(Microsoft's version of OCaml) and will target other typed, functional
programming languages as these languages lack choices in parsing
tools.

  

Rustlr defines a trait that allows the use of any lexical analyzer as long as it's
adopted to the trait.  However, rustlr also provides a general
purpose, zero-copy lexer that suffices for many examples.  A lexer specific 
to a grammar can be automatically generated from a
minimal set of declarations.
<p>
With future releases, Rustlr will also be able to generate parsers for 
languages other than Rust.  With version 0.3.7, it
can generate a parser for F#, although not all capabilities are yet
available.  F# is the .Net version of Ocaml and lacks options when it comes
to parser generation.
Rustlr will mainly target typed, functional languages that support
algebraic types and pattern matching. 

The documentation format on 
<a href="https://docs.rs/rustlr/latest/rustlr/">docs.rs</a>
is a good technical reference but does not serve as a tutorial. 
<p>
<b>This tutorial is evolving as Rustlr is being enhanced with new features.</b>
The project aims to be backwards compatible as much as possible.
<p>
<HR>
<H2>  Tutorial by Examples</H2>
<p>
The tutorial is organized around a set of examples, starting
with the simplest, with each example explaining a set of more advanced
features.  All features of rustlr will
eventually be explained as you progress through the examples. It would be
helpful if the reader is familiar with some basic bottom-up
parsing concepts, such as those covered in typical compiler texts.
<p>
The chapters in <b>bold</b> listed below are complete.  The others provide additional examples and generally contain enough comments to be readable.
The latest and most advanced features of Rustlr are described in Chapter 4 and
in the Appendix.

<p>
<ol>
<li> **[Chapter 1][chap1]** <br>
<li> <b>Chapter 2: <a href="chapter2.html">Enhanced calculator</a></b> with more advanced features, including interactive training for error reporting.
<li> <b>Chapter 3: <a href="chapter3.html">Semantic actions returning multiple value types</a></b>.  (<a href="lbany.html">older version</a>)
<li> <b>Chapter 4: <a href="chapter4.html">Automatically Generating the AST</a></b>
<li> <b>Chapter 5: <a href="chapter5.html">Using Regex Operators *, + and ? in Grammars</a></b>
<li> <b>Chapter 6: <a href="chapter6.html">Generating Bump-allocated ASTs that enable recursive pattern matching</a></b>
<li> <b>Chapter 7: <a href="errors.html">Error Recovery Options</a></b>
<li> Advanced Example: <a href="cparser/c11.grammar">Building a Parser for C</a>. (under construction).  <a href="cparser/">link to crate</a>
<li> <b>Special Example: <a href="noncf/ncf.grammar">Non-context free language</a>, using External State.</b> Link to <a href="noncf/">full crate</a>
<li> <b>Special Chapter: <a href="chapterfs.html">Generating a Parser for F#</a></b>

<li> <b>Appendix: <a href="appendix.html">Experimental Features (Delayed Reductions and the Wildcard Token)</a></b>



<li> Additional Full Example: <a href="yacc/">Yacc converter</a>.  Create with rustlr grammar that builds a parser for converting Yacc grammars to Rustlr format, stripping away all C declarations and actions.

<li> Additional Full Example: <a href="https://crates.io/crates/lambdascript">Lambdascript</a>.  Program implementing and tracing beta-reduction
steps for the untyped lambda calculus.  This crate was created using rustlr 
and <a href="lambdascript/untyped.grammar">this grammar</a>.
<li> (Deprecated versions of <a href="test1grammar0.html">chapter 1</a>
and <a href="calculatorgrammar0.html">chapter 2</a>)
<p>
<b>Additional Grammars</b> (with no semantic actions)
<li> <a href="nonlalr.grammar">LR(1) but non-LALR(1) grammar</a>
<li> <a href="java14.grammar">LALR Grammar for full Java version 1.4</a> 
<li> <a href="ansic.grammar">ANSI C grammar</a> (adopted from yacc syntax)
</ol>

<p>
<H3>References</H3>

<p>
</BODY> 
</HTML>


[1]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.StrTokenizer.html
[2]:https://docs.rs/rustlr/latest/rustlr/generic_absyn/struct.LBox.html
[3]:https://docs.rs/rustlr/latest/rustlr/generic_absyn/struct.LRc.html
[4]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.ZCParser.html#method.lbx
[5]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.StackedItem.html#method.lbox
[sitem]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.StackedItem.html
[chap1]:https://github.com/chuckcscccl/rustlr/blob/main/chapter1.md
[chap2]:https://cs.hofstra.edu/~cscccl/rustlr_project/chapter2.html
[appendix]: https://github.com/chuckcscccl/rustlr/blob/main/appendix.md
[lexsource]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.LexSource.html
[drs]:https://docs.rs/rustlr/latest/rustlr/index.html
[tktrait]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/trait.Tokenizer.html
[tt]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html
[rtk]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/enum.RawToken.html
[fromraw]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html#method.from_raw
[nextsymfun]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/trait.Tokenizer.html#tymethod.nextsym
[zcp]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.ZCParser.html
[ttnew]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html#method.new
[getslice]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/trait.Tokenizer.html#tymethod.get_slice
[bns]:https://hal.archives-ouvertes.fr/hal-00769668/document
