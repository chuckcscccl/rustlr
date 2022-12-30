## Error Recovery

Rustlr supports two methods of error recovery.  The first method
requires the designation of a special terminal symbol as an error
recovery symbol, using the directive "errsym" or "errorsymbol".  This
symbol is assumed to not conflict with actual input tokens.  The error
symbol can appear at most once on the right-hand side of a production
rule.  When an error is encountered (a lookahead symbol with no
defined transition in the LR finite-state machine), the parser
looks down the parse stack for a state that has a transition defined
on the error symbol.  It truncates the stack and performs all possible
reductions until a state that can "shift" the error symbol
is found. It then simuluates the shifting of the next state
associated with the error symbol, with the default semantic value of its type,
onto the stack.  It then skips
lookheads until a valid transition is found for the new state.

The second method of error recovery is rather straightforward: the
grammar can define one or more "resynchronization" terminal symbols
using the "resynch" directive.  When an error is encountered, the
parser skips ahead past the first resynchronization symbol.  Then it
looks down the parse stack for a state that has a valid transition on
the next symbol.  For languages that ends statements with a ;
(semicolon), the ; is the natural choice as the resynch symbol.  The
parser will report an error message for the current statement, then
skip over to the next statement.

The second (resynch) method of error recovery is attempted if the
first method fails or if no error symbol is defined.

If both methods of error-recovery fail, the parser simply skips input
tokens until a suitable action is found.

The following grammar can be used to experiment with these error-recovery
methods.  It parses "cout" statements in "C+-".    The grammar contains an injected `main` function and can replace the
"main.rs" of a crate.  One can test error recovery behavior with input
such as
```
cout << x; cout << y x; cout z; cout << y << z ;
```

```
# Grammar for testing error-recovery
auto
grammarname cpm
nonterminals STAT STATLIST EXPR EXPRLIST
terminals x y z cin cout ; ( ) >> ERROR
lexterminal LLANGLE <<
topsym STATLIST
errsym ERROR
#resync ;

STATLIST --> STAT+
STAT --> cout LLANGLE EXPRLIST:s ; 

EXPR --> x | y | z
EXPR --> ( EXPR )
EXPRLIST --> EXPR<LLANGLE+>

STAT:ErrorStat --> ERROR ;
#STATLIST --> ERROR STAT
EXPR:ErrorExpr --> ( ERROR )
#EXPR --> EXPR ERROR )

!//injected into parser:
!mod cpm_ast;
!use std::io::{Write};
!
!fn readln()-> String {
!  let mut s = String::new();
!  let r = std::io::stdin().read_line(&mut s);
!  s
!}
!
!fn main() {
!   print!("Write something in C+- : ");
!   std::io::stdout().flush().unwrap();
!   let input = readln();
!   let mut lexer1 = cpmlexer::from_str(&input);
!   let mut parser1 = make_parser();
!   let res = parse_with(&mut parser1, &mut lexer1);
!   //let res = parse_train_with(&mut parser1, &mut lexer1);
!   println!("parsing success: {}",!parser1.error_occurred());
!   println!("Result: {:?}",res);
!}//main
```

[1]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.StrTokenizer.html
[2]:https://docs.rs/rustlr/latest/rustlr/generic_absyn/struct.LBox.html
[3]:https://docs.rs/rustlr/latest/rustlr/generic_absyn/struct.LRc.html
[4]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.ZCParser.html#method.lbx
[5]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.StackedItem.html#method.lbox
[sitem]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.StackedItem.html
[chap1]:https://cs.hofstra.edu/~cscccl/rustlr_project/chapter1.html
[chap2]:https://cs.hofstra.edu/~cscccl/rustlr_project/chapter2.html
[chap3]:https://cs.hofstra.edu/~cscccl/rustlr_project/chapter3.html
[chap5]:https://cs.hofstra.edu/~cscccl/rustlr_project/chapter5.html
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
