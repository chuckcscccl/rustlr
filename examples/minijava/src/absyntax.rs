/* Abstract syntax for minijava (adopted from 2014 java program)

   Unlike in the simple calculator example, when a single enum suffices to
   define the abstract syntax, here there are several enums and structs that
   are unified by the enum 'Construct'.  This is the absyntype of the minijava
   grammar.  There are some repetition in the reprsentation.  In particular,
   a variable declaration can appear as a Construct::Vdec(..) as well as a
   Construct::Stm(Vardecst(..)).  This is due to the fact variable declarations
   can appear in different places: in the definition of a class, the definition
   of the formal arguments of a method, as well as a statement inside a
   method body.  Similarly, a method call can appear as either a
   Construct::Exp or as a Construct::Stm.

   A list of variable and method declarations, found in the definition of a
   a class is represented as a vector of Constructs, then split into two
   lists using the separatedecs function.

   All vectors use LBox, including vectors of statements.  Thus every statement
   or declaration will be inside an LBox which will allow us to give proper
   error messages after the parsing state, such as during type checking.
   
*/
#![allow(dead_code)]
use rustlr::LBox;
use crate::Construct::*;
use crate::Expr::*;
use crate::Stat::*;

#[derive(Debug)]
pub enum Construct
{
   Id(String),
   Stm(Stat),
   Stms(Vec<LBox<Stat>>),
   //Tyexpr(String),
   Exp(Expr),
   Exps(Vec<LBox<Expr>>),
   Vdec(VarDec),
   Vdecs(Vec<LBox<VarDec>>),
   Method(MethodDec),
   Methods(Vec<LBox<MethodDec>>),
   Decs(Vec<LBox<Construct>>),
   Class(ClassDec),
   Classes(Vec<LBox<ClassDec>>),
   Maincl(Mainclass),
   Program(LBox<Mainclass>,Vec<LBox<ClassDec>>),
}
impl Default for Construct // required for Construct to be grammar absyntype
{
  fn default() -> Self { Exp(Nothing) }
}

#[derive(Debug)]
pub enum Expr
{
   Int(i32),
   Strlit(String),
   Bool(bool),
   Var(String),
   Thisptr,
   Binop(&'static str,LBox<Expr>,LBox<Expr>), // includes index,
   Notexp(LBox<Expr>),
   Field(String,LBox<Expr>),
   Newarray(LBox<Expr>),
   Newobj(String),  // String is the class name
   Callexp(LBox<Expr>,String,Vec<LBox<Expr>>), //expr version
   Nothing,
}

#[derive(Debug)]
pub enum Stat
{
  Whilest(LBox<Expr>,LBox<Stat>),
  Ifstat(LBox<Expr>,LBox<Stat>,LBox<Stat>),
  Vardecst(String,String,LBox<Expr>),  //name, type, initial val
  Returnst(LBox<Expr>),
  Assignst(String,LBox<Expr>),
  ArAssignst(LBox<Expr>,LBox<Expr>,LBox<Expr>), //a[i]=e
  Callstat(LBox<Expr>,String,Vec<LBox<Expr>>), //stat version  
  Nopst,  // nop
  Blockst(Vec<LBox<Stat>>),
}

#[derive(Debug)]
pub struct VarDec  // variable declaration
{
   pub dname:String,
   pub dtype:String,
   pub initval:Expr,
}
impl Default for VarDec {
 fn default() -> Self { VarDec{dname:String::new(),dtype:String::new(),initval:Nothing} }
}

#[derive(Debug)]
pub struct MethodDec   // method declaration
{
   pub formals:Vec<LBox<VarDec>>,  // formal args
   pub body: Vec<LBox<Stat>>,  // should be a Blockst
   pub classname: String, // added later
   pub methodname: String,
}
impl Default for MethodDec {
 fn default() -> Self { MethodDec{formals:Vec::new(),classname:String::new(),methodname:String::new(),body:Vec::new()} }
}

#[derive(Debug)]
pub struct ClassDec // class declaration
{
  pub superclass:String,
  pub classname:String,
  pub vars: Vec<LBox<VarDec>>,
  pub methods: Vec<LBox<MethodDec>>,
}

#[derive(Debug)]
pub struct Mainclass  // main class can only contain a main
{
  pub classname:String,
  pub argvname: String,  // name of String[] arg to main
  pub body : Stat,       // body of main
}

// separates a list containing both variable and method declarations as 
// "constructs" into two separate lists; for use when constructing a class
// declaration.
pub fn separatedecs(mut ds:Vec<LBox<Construct>>,vars:&mut Vec<LBox<VarDec>>,mths:&mut Vec<LBox<MethodDec>>)
{
  while ds.len()>0
  {
     let mut dec = ds.pop().unwrap(); // this is an lbox
     match &mut *dec {
       Vdec(vd) => {
         let vdec = std::mem::replace(vd,VarDec::default());
         vars.push(dec.transfer(vdec)); // transfers lexical info to new lbox
       },
       Method(md) => {
         let mdec = std::mem::replace(md,MethodDec::default());
         mths.push(dec.transfer(mdec));
       },
       _ => {},
     }//match
  }
}//separatedecs

