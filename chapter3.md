## Chapter 3: A Larger Example with Multiple Abstract Syntax Types

The principal new feature, available since Rustlr version 0.2.5 (set Cargo
dependency accordingly), and
demonstrated by the third sample grammar is the ability to have more than
a single 'absyntype' that all semantic actions must return.  Only
the 'topsym' of the grammar needs to return the absyntype.  Each
terminal as well as non-terminal symbol can have a different type
attached as its semantic value.  The semantic actions for each
non-terminal must return the type as declared for that
non-terminal.  

A grammar declaration such as *`nonterminal E Expr`* or
*`typedterminal int i32`* associate types with individual grammar symbols.  If
no type is associated, they will be assigned the declared
absyntype/valuetype.  The type associated with the 'topsym' must be the same as the absyntype.  **All types must still implement the
Default trait.**

In demonstrating this feature we will also take the opportunity to define a
larger language.  The grammar below defines a scaled-down version of Java
similar to the language in Andrew Appel's compiler textbooks.

For smaller grammars, using one abstract syntax type, along with the external state type, is
preferable.  **A downside of using different types is that it becomes
more difficult to use an alternative lexical analyzer that does not
come with Rustlr.** Semantic values of different types are
accommodated on the parse stack by generating for each grammar an enum
`RetTypeEnum` that exists only within the generated parser module.
All values must be encoded variants of the enum before being
stacked, and extracted when popped from the stack.  The generated parser becomes
less readable because of the extra coded needed.  Rustlr's
lexer generation directives `lexname` and `lexvalue` will generate code that automatically encode/decode with
respect to the enum.  Currently, no support is offered to translate
tokens produced by a lexer other than the built-in [StrTokenizer][1].  The best way to see how to
implement a different lexical analyzer is to examine the one that's
automatically generated for this grammar (look for `mjenumlexer`) and
follow what needs to be done.  One has to adopt the lexer to the
Tokenizer trait as well as the specific enum generated for the
grammar.

We present the definition of the abstract syntax along with the grammar.
The files to examine are:

1. [Abstract Syntax Structures](https://cs.hofstra.edu/~cscccl/rustlr_project/minijava/src/enumabsyn.rs)
2. [Grammar](https://cs.hofstra.edu/~cscccl/rustlr_project/minijava/mjenum.grammar)
3. [Generated Parser/Lexer](https://cs.hofstra.edu/~cscccl/rustlr_project/minijava/src/mjenumparser.rs)
4. [main.rs](https://cs.hofstra.edu/~cscccl/rustlr_project/minijava/src/main.rs)
5. Sample "minijava" programs [QuickSort.mj](https://cs.hofstra.edu/~cscccl/rustlr_project/minijava/QuickSort.mj) and
[BinaryTree.mj](https://cs.hofstra.edu/~cscccl/rustlr_project/minijava/BinaryTree.mj)


This time, before looking at the grammar first we first become
familiar with the abstract syntax structures defined [here](https://cs.hofstra.edu/~cscccl/rustlr_project/minijava/src/enumabsyn.rs) with the core parts
reproduced below:
```
pub enum Expr<'t>
{
   Int(i32),
   Strlit(&'t str),
   Bool(bool),
   Var(&'t str),
   Thisptr,
   Binop(&'static str,LBox<Expr<'t>>,LBox<Expr<'t>>), // includes index,
   Notexp(LBox<Expr<'t>>),
   Field(&'t str,LBox<Expr<'t>>),
   Newarray(LBox<Expr<'t>>),
   Newobj(&'t str),  // String is the class name
   Callexp(LBox<Expr<'t>>,&'t str,Vec<LBox<Expr<'t>>>), //expr version
   Nothing,
}

pub enum Stat<'t>
{
  Whilest(LBox<Expr<'t>>,LBox<Stat<'t>>),
  Ifstat(LBox<Expr<'t>>,LBox<Stat<'t>>,LBox<Stat<'t>>),
  Vardecst(&'t str,&'t str,LBox<Expr<'t>>),  //name, type, initial val
  Returnst(LBox<Expr<'t>>),
  Assignst(&'t str,LBox<Expr<'t>>),
  ArAssignst(LBox<Expr<'t>>,LBox<Expr<'t>>,LBox<Expr<'t>>), //a[i]=e
  Callstat(LBox<Expr<'t>>,&'t str,Vec<LBox<Expr<'t>>>), //stat version  
  Nopst,  // nop
  Blockst(Vec<LBox<Stat<'t>>>),
}

pub struct VarDec<'t>  // variable declaration
{
   pub dname:&'t str,
   pub dtype:&'t str,
   pub initval:Expr<'t>,
}

pub struct MethodDec<'t>   // method declaration
{
   pub formals:Vec<LBox<VarDec<'t>>>,  // formal args
   pub body: Vec<LBox<Stat<'t>>>,  // should be a Blockst
   pub classname: &'t str, // added later
   pub methodname: &'t str,
}

pub struct ClassDec<'t> // class declaration
{
  pub superclass:&'t str,
  pub classname:&'t str,
  pub vars: Vec<LBox<VarDec<'t>>>,
  pub methods: Vec<LBox<MethodDec<'t>>>,
}

pub enum Declaration<'t>
{
   Mdec(MethodDec<'t>),
   Vdec(VarDec<'t>),
   Cdec(ClassDec<'t>),
}

pub struct Mainclass<'t>  // main class can only contain a main
{
  pub classname:&'t str,
  pub argvname: &'t str,  // name of &'t str[] arg to main
  pub body : Stat<'t>,       // body of main
}

pub struct Program<'t>   // absyn value for TOPSYM
{
    pub mainclass:LBox<Mainclass<'t>>,
    pub otherclasses: Vec<LBox<ClassDec<'t>>>,
}
```

We've omitted the `impl Default` segments, which are required for all types.
Essentially, the types distinguish between expressions (Expr), statements
(Stat) and declarations (for variables, methods and classes).  
The final type that's returned by the parser is `Program` which contains
a list of class definitions, one of which contains `main`.

The grammar is found below. **Please note that the interpretation of semantic
labels of the form `E:a` has changed**: 'a' no longer represent a [StackedItem][sitem] but the semantic value that's been extracted from the .value field
of the StackedItem, which held the value as an enum variant.  However, labels
of the form `E:[a]` still means that a refers to an [LBox][2] holding the 
extracted value.
Rustlr will automatically detect if symbols of the grammar are declared to 
hold types that are different from the declared `absyntype`, and use different
routines to generate the parser if there is only a single type.  Thus grammars
written with a single absyntype will still work the same way they did 
before Rustlr 0.2.5.

```
# Grammar for "minijava"
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
nonterminal Rxp Expr<'lt>
nonterminal Dxp Expr<'lt>
nonterminal Bxp Expr<'lt>
nonterminal ExpLst Vec<LBox<Expr<'lt>>>
nonterminal ExpRst Vec<LBox<Expr<'lt>>>
topsym Program
resync ;

# precedence/associativity declarations for common binary operators
left * 700
left / 700
left MOD 700
left + 500
left - 500
left == 450
left < 450
left && 400
left OROR 350
right = 200
# other operators are defined by different levels of "Expression": from
# loosest to tightest: Exp, Bxp, Rxp, Dxp.  These include unary operators,
# array expressions and "." expressions.

# to deal with the dangling-else problem, else is given higher precedence
# than if. Reduction by (Stat --> if (Exp) Stat) will be delayed if the
# the lookahead symbol is 'else'.
nonassoc if 30
nonassoc else 40


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
FormalLst --> FormalRst:v {v}
FormalRst ==> Type:ty ID:a {
  vec![ parser.lb(VarDec{dname:a,dtype:ty,initval:Nothing}) ]
  } <==
FormalRst ==> FormalRst:v , Type:ty ID:a {
  v.push(parser.lb(VarDec{dname:a,dtype:ty,initval:Nothing})); v
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

Dxp --> Dxp:[obj] DOT ID:field  { Field(field,obj) }
Rxp --> Dxp:e {e}

Rxp --> Rxp:[a] [ Exp:[i] ] { Binop("[]",a,i) }
Bxp --> Rxp:e {e}

Bxp --> ! Bxp:[a] { Notexp(a) }
Exp --> Bxp:e {e}

Stat --> Rxp:[v] [ Exp:[i] ] = Exp:[e] ; { ArAssignst(v,i,e) }
Stat --> Dxp:[obj] DOT ID:m ( ExpLst:args ) ; {Callstat(obj,m,args)}
Stat --> return Exp:[e] ; { Returnst(e) }
Stat --> VarDec:v  {Vardecst(v.dname,v.dtype,parser.lb(v.initval))}

Exp --> Exp:[a] * Exp:[b]  { Binop("*",a,b) }
Exp --> Exp:[a] + Exp:[b]  { Binop("+",a,b) }
Exp --> Exp:[a] / Exp:[b]  { Binop("/",a,b) }
Exp --> Exp:[a] - Exp:[b]  { Binop("-",a,b) }
Exp --> Exp:[a] && Exp:[b]  { Binop("&&",a,b) }
Exp --> Exp:[a] OROR Exp:[b]  { Binop("OROR",a,b) }
Exp --> Exp:[a] < Exp:[b]  { Binop("<",a,b) }
Exp --> Exp:[a] MOD Exp:[b]  { Binop("%",a,b) }
Exp --> Exp:[a] == Exp:[b]  { Binop("==",a,b) }
Exp --> Dxp:[obj] DOT ID:f ( ExpLst:args ) { Callexp(obj,f,args) }

Bxp --> INTEGER:i { Int(i) }
Bxp --> STRING:s { Strlit(s) }
Bxp --> BOOL:b { Bool(b) }
Dxp --> ID:x { Var(x) }
Dxp --> this { Thisptr }

Exp --> new int [ Exp:[s] ] { Newarray(s) }
Exp --> new Type:x ( ) { Newobj(x) }
Dxp --> ( Exp:e ) { e }

# zero or more comma-separated expressions with no trailing comma:
ExpLst --> { Vec::new() }
ExpLst --> ExpRst:v {v}
ExpRst --> Exp:[e] { vec![e] }
ExpRst --> ExpRst:v , Exp:[e]  { v.push(e); v }

# With automatic AST generation, this can now be done with:
# ExpLst --> Exp<,*>   (see Chapter 4 and 5 of the tutorial).


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

Please note that the function
[ZCParser::lb](https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.ZCParser.html#method.lb)
had to be called to create certain [LBox][2] structures.  Each item on
the stack contains the staring line/column position of that construct.
The parser itself also maintains it's own current line/column
position, which are used to form the LBox when `parser.lb` is called.
The [ZCParser::lbx][4] function cannot be called when the right-hand
side of a production rule is empty.

Invoking the parser also requires a different procedure (find in main).
One can still call the .parse function on the generated parser but it
would return a value of the internal enum type.
The generated parser now contains two additional functions: **`parse_with`**
and **`parse_train_with`**.  These functions include code to 
extract the final semantic value from an enum variant.  The return type of
these functions is in both cases **`Result<absyntype,absyntype>`**: that is, a parse tree
is always returned, either as Ok(tree) or as Err(tree) if a parse error 
has occurred.
```
fn main()
{
  let args:Vec<String> = std::env::args().collect();
  let mut srcfile = "";
  if args.len()>1 {srcfile = &args[1];}
  let source = LexSource::new(srcfile).unwrap();
  let mut scanner3 = mjenumlexer::from_source(&source);
  let mut parser3 = make_parser();
  let result3 = parse_with(&mut parser3, &mut scanner3);
  let absyntree3 = result3.unwrap_or_else(|x|{println!("Parsing Errors Encountered"); x});
  println!("abstract syntax tree after parse: {:?}\n",absyntree3);
}//main

```

One could still call
[ZCParser::parse](https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.ZCParser.html#method.parse) and
[ZCParser::parse_train](https://docs.rs/rustlr/latest/rustlr/zc_parser/struct.ZCParser.html#method.parse_train) directly.
However, these function now return the abstract syntax enclosed within the
generated enum.  Each generated parser therefore contains its own
**`parse_with`** and **`parse_train_with`** functions.


### Alternatives to Consider

Using different types by generating an internal enum comes at the cost
of tightly coupling the built-in lexer [StrTokenizer][1] to the parser, because code
must be generated to translate the tokens produced by the lexer into
the enum.  There is an alternative way to have semantic actions produce
values of different types: specifying **`LBox<dyn Any>`** as the absyntype of
grammar will invoke a different parser generation routine that automatically
upcasts/downcasts each semantic value into this generic type.  This approach
allows the parser generation routines to stay generic, and does not
require a custom treatment of lexical tokens.  However, this object-oriented
approach also comes with its own costs: it's slower, does not accommodate
non-static references (they don't implement Any), and compromises static
type safety.  There is an **[original "Chapter 3"](https://cs.hofstra.edu/~cscccl/rustlr_project/lbany.html)** that explains how to use this approach,
A version of the "minijava" grammar that uses this approach
is found **[here](https://cs.hofstra.edu/~cscccl/rustlr_project/minijava/lbamj.grammar)**.

One can always create an enum manually and still use a single absyntype
declaration for the entire grammar.  Combined with pattern matching, this
approach is another viable alternative even with larger grammars.  This
version of the "minijava" grammar can be found **[here](https://cs.hofstra.edu/~cscccl/rustlr_project/minijava/mj.grammar)** along with its alternative
[abstract syntax structures](https://cs.hofstra.edu/~cscccl/rustlr_project/minijava/src/absyntax.rs).  The main difference between these structures and
the one shown here is the `Construct` enum that ties all the different types
together.

In addition to the absyntype, there also the 'externtype' that's carried
by each parser.  Thus one can use at least two types when creating
abstract syntax.  For example, the main absyntype can be called 'Expression'
while the externtype can be a 'Vec<Expression>'.  An example of this
approach is found **[here](https://cs.hofstra.edu/~cscccl/rustlr_project/lambdascript/untyped.grammar)**, which is the grammar for [lambdascript](https://crates.io/crates/lambdascript), an interpreter for the pure untyped lambda calculus.

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
