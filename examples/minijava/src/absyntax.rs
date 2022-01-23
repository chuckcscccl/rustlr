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
pub enum Construct<'t>
{
   Id(&'t str),
   Stm(Stat<'t>),
   Stms(Vec<LBox<Stat<'t>>>),
   Exp(Expr<'t>),
   Exps(Vec<LBox<Expr<'t>>>),
   Vdec(VarDec<'t>),
   Vdecs(Vec<LBox<VarDec<'t>>>),
   Method(MethodDec<'t>),
   Methods(Vec<LBox<MethodDec<'t>>>),
   Decs(Vec<LBox<Construct<'t>>>),
   Class(ClassDec<'t>),
   Classes(Vec<LBox<ClassDec<'t>>>),
   Maincl(Mainclass<'t>),
   Program(LBox<Mainclass<'t>>,Vec<LBox<ClassDec<'t>>>),
}
impl Default for Construct<'_> // required for Construct to be grammar absyntype
{
  fn default() -> Self { Exp(Nothing) }
}

#[derive(Debug)]
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
impl<'t> Default for Expr<'t> { fn default()->Self {Nothing} }

#[derive(Debug)]
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
impl<'t> Default for Stat<'t> {fn default()->Self {Nopst} }

#[derive(Debug)]
pub struct VarDec<'t>  // variable declaration
{
   pub dname:&'t str,
   pub dtype:&'t str,
   pub initval:Expr<'t>,
}
impl<'t> Default for VarDec<'t> {
 fn default() -> Self { VarDec{dname:"",dtype:"",initval:Nothing} }
}

#[derive(Debug)]
pub struct MethodDec<'t>   // method declaration
{
   pub formals:Vec<LBox<VarDec<'t>>>,  // formal args
   pub body: Vec<LBox<Stat<'t>>>,  // should be a Blockst
   pub classname: &'t str, // added later
   pub methodname: &'t str,
}
impl<'t> Default for MethodDec<'t> {
 fn default() -> Self { MethodDec{formals:Vec::new(),classname:"",methodname:"",body:Vec::new()} }
}

#[derive(Debug)]
pub struct ClassDec<'t> // class declaration
{
  pub superclass:&'t str,
  pub classname:&'t str,
  pub vars: Vec<LBox<VarDec<'t>>>,
  pub methods: Vec<LBox<MethodDec<'t>>>,
}
impl<'t> Default for ClassDec<'t> {
 fn default()->Self { ClassDec{superclass:"Object",classname:"",vars:Vec::new(),methods:Vec::new()}}
}


#[derive(Debug)]
pub struct Mainclass<'t>  // main class can only contain a main
{
  pub classname:&'t str,
  pub argvname: &'t str,  // name of &'t str[] arg to main
  pub body : Stat<'t>,       // body of main
}
impl<'t> Default for Mainclass<'t> {
  fn default()->Self { Mainclass {classname:"",argvname:"",body:Stat::default(),}}
}

// separates a list containing both variable and method declarations as 
// "constructs" into two separate lists; for use when constructing a class
// declaration.
pub fn separatedecs<'t>(mut ds:Vec<LBox<Construct<'t>>>,vars:&mut Vec<LBox<VarDec<'t>>>,mths:&mut Vec<LBox<MethodDec<'t>>>)
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
