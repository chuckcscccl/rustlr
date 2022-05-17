## Chapter 3: A Larger Example, with Multiple Abstract Syntax Types

The principal new feature, available since rustlr version 0.2.5,
demonstrated by the third sample grammar is ability to have more than
a singled 'absyntype' that all semantic actions must returned.  Only
the 'topsym' of the grammar needs to return the absyntype.  Each
terminal as well as non-terminal symbol can have a different type
attached as its semantic value.  The semantic actions for each
non-terminal must return the type as declared for for that
non-terminal.  All types must still implement the Default trait.

A grammar declaration such as
  *`nonterminal E Expr`* or `typedterminal int i32` associates types with
grammar symbols.  If no type is associated, they will be assigned the declared
absyntype/valuetype.  The type associated with the 'topsym' is forced to be
the same as the absyntype.  

In demonstrating this feature we will also take the opportunity to define a
larger language.  The grammar below defines a scaled-down version of Java
similar to the "Tiger" language in Andrew Appel's compiler textbooks.

We present the definition of the abstract syntax along with the grammar.
The files to examine are:

1. [Abstract Syntax Structures](https://cs.hofstra.edu/~cscccl/rustlr_project/minijava/src/enumabsyn.rs)
2. [Grammar](https://cs.hofstra.edu/~cscccl/rustlr_project/minijava/mjenum.grammar)
3. [Generated Parser/Lexer](https://cs.hofstra.edu/~cscccl/rustlr_project/minijava/src/mjenumparser.rs)
4. [main.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/minijava/src/main.rs)
5. Sample "minijava" programs [QuickSort.mj](https://cs.hofstra.edu/~cscccl/rustlr_project/minijava/QuickSort.mj) and
[BinaryTree.mj](https://cs.hofstra.edu/~cscccl/rustlr_project/minijava/BinaryTree.mj)

The grammar is as follows:

```
!use rustlr::LBox;
!use crate::enumabsyn::*;
!use crate::enumabsyn::Declaration::*;
!use crate::enumabsyn::Expr::*;
!use crate::enumabsyn::Stat::*;

lifetime 'lt
absyntype Program<'lt>
typedterminal ID &'lt str
typedterminal STRING &'lt str
typedterminal BOOL bool
typedterminal INTEGER i32
terminal class public static void main String extends return length
terminal ( ) [ ] ; DOT ! , new this
terminal LBR RBR OROR
terminal int boolean if else while == = + - * / < && MOD
nonterminal Program Program<'lt>
nonterminal MainCl Mainclass<'lt>
nonterminal ClassDec ClassDec<'lt>
nonterminal ClassDecl Vec<LBox<ClassDec<'lt>>> 
nonterminal Extension &'lt str
nonterminal VarDec VarDec<'lt>
nonterminal MethodDec MethodDec<'lt>
nonterminal Decl Vec<LBox<Declaration<'lt>>>
nonterminal FormalLst Vec<LBox<VarDec<'lt>>>
nonterminal FormalRst Vec<LBox<VarDec<'lt>>>
nonterminal Type &'lt str
nonterminal Stat Stat<'lt>
nonterminal Stats Vec<LBox<Stat<'lt>>>
nonterminal Exp Expr<'lt>
nonterminal ExpLst Vec<LBox<Expr<'lt>>>
nonterminal ExpRst Vec<LBox<Expr<'lt>>>
topsym Program
resync ;

left + 500
left - 510
left * 700
left / 710
left && 400
left OROR 350
left ! 450
left == 310
left = 800
left < 300
left MOD 705
left DOT 810
# to deal with the dangling-else problem, else is given higher precedence
# than if. Reduction by (Stat --> if (Exp) Stat) will be delayed if the
# the lookahead symbol is 'else'.
left if 30
left else 40


Program --> MainCl:[mc]  ClassDecl:cs  { Program {mainclass:mc, otherclasses:cs } }
   
MainCl ==> class ID:cn LBR public static void main ( String [ ] ID:an ) LBR Stats:thebody RBR RBR  {
   Mainclass{classname:cn,
             argvname:an,
             body: Blockst(thebody),
	    }
  } <==

ClassDecl --> { Vec::new() }
ClassDecl --> ClassDecl:cs  ClassDec:[cl]  { cs.push(cl); cs }
ClassDec ==> class ID:name Extension:sup LBR Decl:ds RBR {
  let mut vdecs=Vec::new();
  let mut mdecs=Vec::new();
  separatedecs(ds,&mut vdecs,&mut mdecs); /*split var and method declarations*/
  ClassDec {superclass:sup,
            classname:name,
            vars:vdecs,
            methods:mdecs}
  } <==
  
Extension --> extends Type:sup { sup }
Extension --> { "Object" }
VarDec --> Type:t ID:v ;  { VarDec{dname:v,dtype:t,initval:Nothing,} }
VarDec --> Type:t ID:v = Exp:e ; {VarDec{dname:v,dtype:t,initval:e}}
  
MethodDec ==> public Type:ty ID:name ( FormalLst:args ) LBR Stats:mbody RBR {
  MethodDec{ formals:args,
             body: mbody,
             classname:ty,
	     methodname:name, }
  } <==
Decl -->  { Vec::new() }
Decl --> Decl:ds VarDec:v { ds.push(parser.lbx(1,Vdec(v))); ds }
Decl --> Decl:ds MethodDec:m { ds.push(parser.lbx(1,Mdec(m))); ds }
FormalLst --> { Vec::new() }
# warning: list constructed backwards:
FormalLst ==> Type:ty ID:a FormalRst:frs {
  frs.push(parser.lb(VarDec{dname:a,dtype:ty,initval:Nothing}));
  frs 
  } <==
FormalRst --> { Vec::new() }
FormalRst ==> , Type:ty ID:a FormalRst:frs {
  frs.push(parser.lb(VarDec{dname:a,dtype:ty,initval:Nothing}));
  frs 
  } <==
Type --> int [ ] { return "int[]"; }
Type --> boolean { return "boolean"; }
Type --> String  { return "String"; }
Type --> int     { return "int"; }
Type --> void     { return "void"; }
Type --> ID:c    { c }    
Stats --> { Vec::new() }
Stats --> Stats:sv Stat:[s] { sv.push(s); sv }
Stat --> LBR Stats:sv RBR { Blockst(sv) }
Stat --> if ( Exp:[c] ) Stat:[a] else Stat:[b] { Ifstat(c, a, b) }
Stat --> if ( Exp:[c] ) Stat:[a] { Ifstat(c,a,parser.lb(Nopst)) }
Stat --> while ( Exp:[c] ) Stat:[s] { Whilest(c,s) }
Stat --> ID:v = Exp:[e] ; { Assignst(v,e) }

Stat --> Exp:[v] [ Exp:[i] ] = Exp:[e] ; { ArAssignst(v,i,e) }
Stat --> Exp:[obj] DOT ID:m ( ExpLst:args ) ; {Callstat(obj,m,args)}
Stat --> return Exp:[e] ; { Returnst(e) }
Stat --> VarDec:v  {Vardecst(v.dname,v.dtype,parser.lb(v.initval))}

Exp --> Exp:[a] * Exp:[b]  { Binop("*",a,b) }
Exp --> Exp:[a] + Exp:[b]  { Binop("+",a,b) }
Exp --> Exp:[a] / Exp:[b]  { Binop("/",a,b) }
Exp --> Exp:[a] - Exp:[b]  { Binop("-",a,b) }
Exp --> Exp:[a] && Exp:[b]  { Binop("&&",a,b) }
Exp --> Exp:[a] OROR Exp:[b]  { Binop("OROR",a,b) }
Exp --> ! Exp:[a]  { Notexp(a) }
Exp --> Exp:[a] < Exp:[b]  { Binop("<",a,b) }
Exp --> Exp:[a] MOD Exp:[b]  { Binop("%",a,b) }
Exp --> Exp:[a] == Exp:[b]  { Binop("==",a,b) }
Exp --> Exp:[a] [ Exp:[i] ] { Binop("[]",a,i) } 
Exp --> Exp:[obj] DOT ID:field { Field(field,obj) }

Exp --> Exp:[obj] DOT ID:f ( ExpLst:args ) { Callexp(obj,f,args) }
Exp --> INTEGER:i { Int(i) }
Exp --> STRING:s { Strlit(s) }
Exp --> BOOL:b { Bool(b) }
Exp --> ID:x { Var(x) }
Exp --> this { Thisptr }
Exp --> new int [ Exp:[s] ] { Newarray(s) }
Exp --> new Type:x ( ) { Newobj(x) }
Exp --> ( Exp:e ) { e }


# warning: backwards list:
ExpLst --> { Vec::new() }
ExpLst --> Exp:[e] ExpRst:er { er.push(e); er }
ExpRst --> { Vec::new() }
ExpRst --> , Exp:[e] ExpRst:er { er.push(e); er }


# Lexical scanner setup using StrTokenizer
lexname DOT .
lexname MOD %
lexname LBR {
lexname RBR }
lexname OROR ||
lexvalue BOOL Alphanum("true") true
lexvalue BOOL Alphanum("false") false
lexvalue ID Alphanum(x) x
lexvalue INTEGER Num(n) (n as i32)
lexvalue STRING Strlit(s) s

EOF
```

     -------------------

[1]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.StrTokenizer.html
[2]:https://docs.rs/rustlr/latest/rustlr/generic_absyn/struct.LBox.html
[3]:https://docs.rs/rustlr/latest/rustlr/generic_absyn/struct.LRc.html
[4]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.ZCParser.html#method.lbx
[5]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.StackedItem.html#method.lbox
[sitem]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.StackedItem.html
[chap1]:https://cs.hofstra.edu/~cscccl/rustlr_project/chapter1.html
[lexsource]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.LexSource.html
[drs]:https://docs.rs/rustlr/latest/rustlr/index.html
[tktrait]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/trait.Tokenizer.html
[tt]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html
[rtk]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/enum.RawToken.html
[fromraw]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html#method.from_raw
[nextsymfun]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/trait.Tokenizer.html#tymethod.nextsym
[zcp]:https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.ZCParser.html
[ttnew]:https://docs.rs/rustlr/latest/rustlr/lexer_interface/struct.TerminalToken.html#method.new