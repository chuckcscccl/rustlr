/* Abstract syntax for minijava (adopted from 2014 java program)
   Using internally generated RetTypeEnum
*/
#![allow(dead_code)]
use rustlr::LBox;
use crate::Expr::*;
use crate::Stat::*;
use crate::Declaration::*;

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
pub enum Declaration<'t>
{
   Mdec(MethodDec<'t>),
   Vdec(VarDec<'t>),
   Cdec(ClassDec<'t>),
}
// no default for this one.


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

#[derive(Debug)]
pub struct Program<'t>   // absyn value for TOPSYM
{
    pub mainclass:LBox<Mainclass<'t>>,
    pub otherclasses: Vec<LBox<ClassDec<'t>>>,
}
impl<'t> Default for Program<'t> {
  fn default()->Self { Program {mainclass:LBox::default(), otherclasses:Vec::new()}}
}


// separates a list containing both variable and method declarations as 
// "constructs" into two separate lists; for use when constructing a class
// declaration.
pub fn separatedecs<'t>(mut ds:Vec<LBox<Declaration<'t>>>,vars:&mut Vec<LBox<VarDec<'t>>>,mths:&mut Vec<LBox<MethodDec<'t>>>)
{
  while ds.len()>0
  {
     let mut dec = ds.pop().unwrap(); // this is an lbox
     match &mut *dec {
       Vdec(vd) => {
         let vdec = std::mem::replace(vd,VarDec::default());
         vars.push(dec.transfer(vdec)); // transfers lexical info to new lbox
       },
       Mdec(md) => {
         let mdec = std::mem::replace(md,MethodDec::default());
         mths.push(dec.transfer(mdec));
       },
       _ => {},
     }//match
  }
}//separatedecs
