// Abstract syntax for minijava (adopted from 2014 java program)
#![allow(dead_code)]
extern crate rustlr;
use rustlr::LBox;
use crate::Construct::*;
use crate::Expr::*;
use crate::Stat::*;

pub enum Construct
{
   Id(String),
   Stm(Stat),
   Stms(Vec<LBox<Stat>>),
   Tyexpr(String),
   Exp(Expr),
   Exps(Vec<LBox<Expr>>),
   Vdec(VarDec),
   Vdecs(Vec<VarDec>),
   Method(MethodDec),
   Methods(Vec<MethodDec>),
   Methodcall(LBox<Expr>,String,Vec<LBox<Expr>>),   
   Decs(Vec<LBox<Construct>>),
   Class(ClassDec),
   Classes(Vec<ClassDec>),
   Maincl(Mainclass),
   Program(Mainclass,Vec<ClassDec>),
}
impl Default for Construct
{
  fn default() -> Self { Exp(Nothing) }
}

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
   Nothing,
}

pub enum Stat
{
  Whilest(LBox<Expr>,LBox<Stat>),
  Ifstat(LBox<Expr>,LBox<Stat>,LBox<Stat>),
  Vardecst(String,String,LBox<Expr>),  //name, type, initial val
  Returnst(LBox<Expr>),
  Assignst(String,LBox<Expr>),
  ArAssignst(LBox<Expr>,LBox<Expr>,LBox<Expr>), //a[i]=e
  Nopst,  // nop
  Blockst(Vec<LBox<Stat>>),
}


pub struct VarDec
{
   pub dname:String,
   pub dtype:String,
   pub initval:Expr,
}

pub struct MethodDec   // method declaration
{
   pub formals:Vec<VarDec>,  // formal args
   pub body: Vec<LBox<Stat>>,  // should be a Blockst
   pub classname: String, // added later
   pub methodname: String,
}

pub struct ClassDec // class declaration
{
  pub superclass:String,
  pub classname:String,
  pub vars: Vec<VarDec>,
  pub methods: Vec<MethodDec>,
}

pub struct Mainclass  // main class can only contain a main
{
  pub classname:String,
  pub argvname: String,
  pub body : Stat,
}

