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
   Id(&'t st),
   Stm(Stat<'t>),
   Stms(Vec<LBox<Stat<'t>>>),
   Exp(Expr<'t>),
   Exps(Vec<LBox<Expr<'t>>>),
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
impl<'t> Default for Construct<'t> // required for Construct to be grammar absyntype
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
impl Default for Expr { fn default()->Self {Nothing} }

#[derive(Debug)]
pub enum Stat<'t>
{
  Whilest(LBox<Expr<'t>>,LBox<Stat<'t>>),
  Ifstat(LBox<Expr<'t>>,LBox<Stat<'t>>,LBox<Stat<'t>>),
  Vardecst(LBox<VarDec>),
  Returnst(LBox<Expr<'t>>),
  Assignst(&'t str,LBox<Expr<'t>>),
  ArAssignst(LBox<Expr<'t>>,LBox<Expr<'t>>,LBox<Expr<'t>>), //a[i]=e
  Callstat(LBox<Expr<'t>>,&'t str,Vec<LBox<Expr<'t>>>), //stat version  
  Nopst,  // nop
  Blockst(Vec<LBox<Stat<'t>>>),
}
impl Default for Stat {fn default()->Self {Nopst} }

#[derive(Debug)]
pub struct VarDec<'t>  // variable declaration
{
   pub dname:&'t str,
   pub dtype:&'t str,
   pub initval:Expr<'t>,
}
impl Default for VarDec {
 fn default() -> Self { VarDec{dname:"",dtype:"",initval:Nothing} }
}

#[derive(Debug)]
pub struct MethodDec<'t>   // method declaration
{
   pub formals:Vec<LBox<VarDec>>,  // formal args
   pub body: Vec<LBox<Stat<'t>>>,  // should be a Blockst
   pub classname: &'t str, // added later
   pub methodname: &'t str,
}
impl Default for MethodDec {
 fn default() -> Self { MethodDec{formals:Vec::new(),classname:"",methodname:"",body:Vec::new()} }
}

#[derive(Debug)]
pub struct ClassDec<'t> // class declaration
{
  pub superclass:&'t str,
  pub classname:&'t str,
  pub vars: Vec<LBox<VarDec>>,
  pub methods: Vec<LBox<MethodDec>>,
}
impl Default for ClassDec {
 fn default()->Self { ClassDec{superclass:"Object",classname:"",vars:Vec::new(),methods:Vec::new()}}
}


#[derive(Debug)]
pub struct Mainclass<'t>  // main class can only contain a main
{
  pub classname:&'t str,
  pub argvname: &'t str,  // name of String[] arg to main
  pub body : Stat<'t>,       // body of main
}
impl Default for Mainclass {
  fn default()->Self { Mainclass {classname:"",argvname:"",body:Stat::default(),}}
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
