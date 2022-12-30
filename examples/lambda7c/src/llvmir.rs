// LLVM IR abstract representation in Rust
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(unused_assignments)]
#![allow(dead_code)]
use std::rc::Rc;
use std::cell::{RefCell,Ref,RefMut};
extern crate rustlr;
use rustlr::LC;
use std::collections::{HashMap,HashSet,BTreeMap,BTreeSet};
use crate::bump7c_ast;
use crate::bump7c_ast::*;
use crate::btyping::lrtype;
use crate::btyping::lrtype::*;
use fixedstr::{str8,str16,str24,str32,str64,str128,str256};
//use std::hash::{Hash,Hasher};

// labels, register names can be at most 15 characters long (str24)
// function names can be 31 chars long (str32)
// operation type (add) can be 7 chars long (str8)

pub type RegisterRep = str16;
pub type LabelRep = str16;
pub type FuncnameRep = str32;

#[derive(Debug,Clone,Hash,Eq,PartialEq)]
pub enum LLVMtype {
  Basic(&'static str),
  Pointer(Box<LLVMtype>),
  Array_t(usize, Box<LLVMtype>),
  Func_t(Vec<LLVMtype>),  // last type is return type
  Userstruct(str32),
  Ellipsis,
  Void_t,
}//LLVMtype

#[derive(Debug,Clone)]
pub enum LLVMexpr {
  Iconst(i32),
  Sconst(String),
  Aconst(LLVMtype,Vec<LLVMexpr>),  // array constants
  Fconst(f64),
  I1const(bool),
  Register(str24),
  Global(str32),
  Novalue,
}//LLVMtype
impl PartialEq for LLVMexpr {
  fn eq(&self,other:&Self) -> bool {
    use crate::llvmir::LLVMexpr::*;
    match (self,other) {
      (Fconst(a), Fconst(b)) => {
          ((*a*1000000.0) as i64) == ((*b*1000000.0) as i64)
      },
      (Iconst(a), Iconst(b)) => a==b,
      (Sconst(a), Sconst(b)) => &a==&b,
      (Aconst(ta,va), Aconst(tb,vb)) => {
         if va.len()!=vb.len() || ta!=tb {return false;}
         for i in 0..va.len() {
           if va[i] != vb[i] {return false;}
         }
         true
      },
      (I1const(a), I1const(b)) => a==b,
      (Register(a), Register(b)) => &a==&b,
      (Global(a),Global(b)) => &a==&b,      
      (Novalue,Novalue) => true,
      _ => false,
    }//match
  }
}
impl Eq for LLVMexpr {}
/*
impl Hash for LLVMexpr {
  fn hash<H:Hasher>(&self, state:&mut H) {
    match self {
      LLVMexpr::Fconst(c) => {
        let ci = (c*1000000.0) as i64;
        ci.hash(state)
      },
      x => x.hash(state),
    }//match
  }
}//hash trait of LLVMexpr
*/

#[derive(Debug,Clone)]
pub enum Instruction {
  // terminators
  Ret(LLVMtype,LLVMexpr),
  Ret_noval,
  Br_uc(str24),
  Bri1(LLVMexpr,str24,str24),
  // mem ops
  Load(str24,LLVMtype,LLVMexpr,Option<str32>),
  Store(LLVMtype,LLVMexpr,LLVMexpr,Option<str32>),
  Alloca(str24,LLVMtype,Option<u8>),
  Binaryop(str24,str8,LLVMtype,LLVMexpr,LLVMexpr),
  Unaryop(str24,str8,Option<str64>,LLVMtype,LLVMexpr),
  Cast(str24,str8,LLVMtype,LLVMexpr,LLVMtype),
  Icmp(str24,str8,LLVMtype,LLVMexpr,LLVMexpr),
  Fcmp(str24,str8,LLVMtype,LLVMexpr,LLVMexpr),
  SelectTrue(str24,LLVMtype,LLVMexpr,LLVMexpr),
  Phi2(str24,LLVMtype,LLVMexpr,str24,LLVMexpr,str24),
  Phi(str24,LLVMtype,Vec<(LLVMexpr,str24)>),
  Call(Option<str24>,LLVMtype,Vec<LLVMtype>,str32,Vec<(LLVMtype,LLVMexpr)>),
  Arrayindex(str24,usize,LLVMtype,LLVMexpr,LLVMexpr),
  Vectorindex(str24,LLVMtype,LLVMexpr,LLVMexpr),  
  Structfield(str24,LLVMtype,LLVMexpr,LLVMexpr),
  Verbatim(String),
  Nop, // dummy op
}//Instruction


#[derive(Debug,Clone)]
pub enum LLVMdeclaration {
  Globalconst(str24,LLVMtype,LLVMexpr,Option<str32>),
  Externfunc(LLVMtype,str32,Vec<LLVMtype>),
  Structdec(str32,Vec<LLVMtype>),
  Verbatim_dec(String) // cases not covered by above
}//LLVMdeclaration

#[derive(Debug,Clone)]
pub struct BasicBlock {
   pub label: str24,
   pub phis: Vec<Instruction>, // phi instructions, added at the end
   pub body: Vec<Instruction>, // last instruction must be a terminator
   pub predecessors : Vec<str24>, // control flow graph (by labels)
   pub ssamap : HashMap<str24,(LLVMexpr,LLVMtype)>, //current version of var
   pub needphi : HashSet<(str24,str24,LLVMtype)>,
   // needphi: (originalname, newname, type)
   pub onphis:HashSet<str24>,
   pub visits: i32,
}//BasicBlock

#[derive(Debug,Clone)]
pub struct LLVMFunction {
   pub name: str32,
   pub formal_args : Vec<(LLVMtype,str24)>,
   pub return_type : LLVMtype,  // only basic and pointer types can be returned
   pub bblocks: Vec<BasicBlock>,
   pub attributes : Vec<String>, // like "dso_local", "#1", or ["","#1"]
   pub bblocator : HashMap<str24,usize>, //index of BB in vector by label
}//LLVMFunction

#[derive(Debug,Clone)]
pub struct LLVMProgram   {
   pub preamble : String,   // arbitrary stuff like target triple
   pub global_declarations : Vec<LLVMdeclaration>,
   pub functions: Vec<LLVMFunction>,
   pub postamble : String,  // stuff you don't want to know about
   pub strconsts : HashMap<String,str24>,  //record size also
}//LLVMprogram


//////////////////////////////////////////////

impl Instruction {
  pub fn destination(&self) -> Option<str24>
  { use crate::llvmir::Instruction::*;
    match self {
      Load(s,_,_,_) | Alloca(s,_,_) | Unaryop(s,_,_,_,_) => Some(*s),
      Binaryop(s,_,_,_,_) | Call(Some(s),_,_,_,_) => Some(*s),
      Cast(s,_,_,_,_) | Icmp(s,_,_,_,_) | Fcmp(s,_,_,_,_) => Some(*s),
      SelectTrue(s,_,_,_) | Phi2(s,_,_,_,_,_) | Phi(s,_,_) => Some(*s),
      Arrayindex(s,_,_,_,_) | Structfield(s,_,_,_) => Some(*s),
      _ => None,  // includes case for Verbatim
    }//match
  }//destination
}//impl Instruction


impl BasicBlock
{
   pub fn new(lb:str24,preds:Vec<str24>) -> BasicBlock {
     BasicBlock { label: lb, body:Vec::new(), predecessors:preds,ssamap:HashMap::new(),phis:Vec::new(), needphi:HashSet::new(),visits:0,onphis:HashSet::new(),}
   }
   pub fn add(&mut self, inst:Instruction) {self.body.push(inst);}
   pub fn append(&mut self, vs:&mut Vec<Instruction>) {self.body.append(vs);}
   pub fn last_dest(&self) -> Option<str24> {
     if self.body.len()==0 {None}
     else {self.body[self.body.len()-1].destination()}
   }

   pub fn add_predecessor(&mut self, pred:str24) {
     self.predecessors.push(pred);
   }
   // call this for the body of the while loop, so it will construct
   // ssamap differently

   pub fn getssa(&self, key:&str24) -> Option<&(LLVMexpr,LLVMtype)>//only local!
   {
      self.ssamap.get(key)
   }//getssa
   pub fn setssa(&mut self, key:str24, exp:LLVMexpr, ty:LLVMtype)
   {
     self.ssamap.insert(key,(exp,ty));
   }//setssa

}//impl BasicBlock
impl Default for BasicBlock {
  fn default() -> Self { BasicBlock::new(str24::default(),vec![]) }
}


impl LLVMFunction
{
 pub fn addBB_owned(&mut self, mut bb:BasicBlock) {
     self.bblocator.insert(bb.label,self.bblocks.len());
     self.bblocks.push(bb);
 }
 pub fn addBB(&mut self, bb:&mut BasicBlock) {
     let mut tmp = BasicBlock::default();
     std::mem::swap(&mut tmp, bb);
     self.bblocator.insert(tmp.label,self.bblocks.len());     
     self.bblocks.push(tmp);
 }

 pub fn currentBBopt(&mut self) -> Option<&mut BasicBlock> {
   if self.bblocks.len()==0 {return None;}
   let last = self.bblocks.len()-1;
   if self.bblocks[last].body.len()>0 {
     let li = self.bblocks[last].body.len()-1;
     if is_terminator(&self.bblocks[last].body[li]) {return None;}
   }
   Some(&mut self.bblocks[last])
 }

 pub fn last_instruction(&self) -> Option<&Instruction>
 {
   if self.bblocks.len()==0 {return None;}
   let last = self.bblocks.len()-1;
   if self.bblocks[last].body.len()>0 {
     let li = self.bblocks[last].body.len()-1;
     Some(&self.bblocks[last].body[li])
   } else {None}
 }//last_instruction

 pub fn pop_last(&mut self) -> Option<Instruction>
 {
   if self.bblocks.len()==0 {return None;}
   let last = self.bblocks.len()-1;
   if self.bblocks[last].body.len()>0 {
      self.bblocks[last].body.pop()
    }
   else {None}
 }//pop_last

 //version that creates a new BB if needed  (not used)
 pub fn currentBB(&mut self, index:usize) -> usize {  //returns index of BB
   let mut neednew = false;
   if self.bblocks.len()==0 { neednew=true; }
   let mut last = self.bblocks.len()-1;
   if self.bblocks[last].body.len()>0 {
     let li = self.bblocks[last].body.len()-1;
     if is_terminator(&self.bblocks[last].body[li]) {neednew=true;}
   }
   if neednew {
     let newlabel = str24::from(format!("newBB_{}",index));
     let newBB = BasicBlock::new(newlabel,vec![]);
     self.addBB_owned(newBB);
   }
   self.bblocks.len()-1
 }//currentBB - creates new one if required

 pub fn add_inst(&mut self, inst:Instruction) { // add to current BB
   let lasti = self.currentBB(0);
   self.bblocks[lasti].body.push(inst);
 }//add_inst

 pub fn add_phi(&mut self, inst:Instruction) { // add to current BB
   let lasti = self.currentBB(0);
   self.bblocks[lasti].phis.push(inst);
 }//add_inst

 pub fn add_need(&mut self, xvar:str24,xvar2:str24,xtype:LLVMtype) {
   let lasti = self.bblocks.len()-1;
   self.bblocks[lasti].needphi.insert((xvar,xvar2,xtype));
 }

 pub fn currentBBlabel(&self) -> str24 {
   if self.bblocks.len()==0 {str24::new()}
   else {self.bblocks[self.bblocks.len()-1].label}
 }//currentBBlabel

pub fn getlastSSA(&self, key:&str24) -> Option<&(LLVMexpr,LLVMtype)>
{
   let blen = self.bblocks.len();
   if (blen==0) {return None;}
   self.bblocks[blen-1].getssa(key)
}//lastSSA
pub fn setlastSSA(&mut self, key:str24, exp:LLVMexpr, ty:LLVMtype)
{
   let bblen = self.bblocks.len();
   self.bblocks[bblen-1].setssa(key,exp,ty);
//   let BB = self.currentBBopt();
//   BB.map(|bb|bb.setssa(key,exp,ty));
}//setlastSSA

/*
//getSSA by BB label
pub fn getSSA(&self, bbl:&str24, key:&str24) -> Option<&LLVMexpr>
{
  if let Some(bbi) = self.bblocator.get(bbl) {
     self.bblocks[*bbi].getssa(key)
  } else {None}
}//getSSA by BB label
*/
pub fn lastBB(&self) -> &BasicBlock  // doesn't check index
{
   &self.bblocks[self.bblocks.len()-1]
}//getBB
pub fn getBB(&self, bbl:&str24)-> &BasicBlock  // doesn't check index
{
  let bbi = *self.bblocator.get(bbl).unwrap();
  &self.bblocks[bbi]
}//getBB by label

// divide this into inherit_ssamap and resolve_ssamap
pub fn inherit_ssamap(&mut self,bbl:&str24)
{  use crate::llvmir::Instruction::*;
   use crate::llvmir::LLVMexpr::*;
   let bblen = self.bblocks.len();
   let bbi = *self.bblocator.get(bbl).unwrap();
   let mapcopy = self.bblocks[bbi].ssamap.clone();
   self.bblocks[bblen-1].ssamap = mapcopy;
}//inherit_ssamap

  // based on predecessors
pub fn fillneed(&mut self, targetbbl:&str24)
{ use crate::llvmir::Instruction::*;
  use crate::llvmir::LLVMexpr::*;
  use crate::llvmir::LLVMtype::*;  
  let tbbi = *self.bblocator.get(targetbbl).unwrap();
  if self.bblocks[tbbi].visits>1 {return;} else {self.bblocks[tbbi].visits+=1;}
  let preds = self.bblocks[tbbi].predecessors.clone();
  let needphi = self.bblocks[tbbi].needphi.clone();
  /*
  if preds.len()==0 {
    for (xvar,xvar2,xtype) in needphi.iter() {
      let xdestopt = self.bblocks[tbbi].ssamap.get(xvar);
      if let Some((xdest,_)) = xdestopt {
        if !self.bblocks[tbbi].onphis.contains(xvar2) {
          let movinst = Cast(*xvar2,str8::from("bitcast"),xtype.clone(),xdest.clone(),xtype.clone());
          self.bblocks[tbbi].phis.push(movinst);
          self.bblocks[tbbi].onphis.insert(*xvar2);
        }
      }// ssamap dontains value for xvar, just need to transfer to xvar2
    }//for each need
    //self.bblocks[tbbi].needphi.clear();    
    return;
  }// no preds
  */
  if preds.len()==0 {return;}
  let bbi0 = *self.bblocator.get(&preds[0]).unwrap();
  let bbl0 = self.bblocks[bbi0].label;
  /*
  if preds.len()==1 {
    for (xvar,xvar2,xtype) in needphi.iter() {
      let xdestopt = self.bblocks[bbi0].ssamap.get(xvar);
      if xdestopt.is_none() {
        let xvar3 = str24::from(&format!("{}_0",&xvar2));
        let need=self.bblocks[bbi0].needphi.insert((*xvar,xvar3,xtype.clone()));
        if need {self.fillneed(&bbl0);}
      }
      else if !self.bblocks[tbbi].onphis.contains(xvar2) { //mov
        let xdest = &xdestopt.unwrap().0;
        let movinst = Cast(*xvar2,str8::from("bitcast"),xtype.clone(),xdest.clone(),xtype.clone());
        self.bblocks[tbbi].phis.push(movinst);
        self.bblocks[tbbi].onphis.insert(*xvar2);
      }
    } //for each need in single predecessor case
  }////single predecessor case
  */
  if preds.len()!=2 {/*self.bblocks[tbbi].needphi.clear();*/ return;}  
  let bbi1 = *self.bblocator.get(&preds[1]).unwrap();
  let bbl1 = self.bblocks[bbi1].label;  
  for (xvar,xvar2,xtype) in needphi.iter() {
    let dest0;
    let dest0opt=self.bblocks[bbi0].ssamap.get(xvar);
    if dest0opt.is_none() {
      let xvar3 = str24::from(&format!("{}_0",&xvar2));
      self.bblocks[bbi0].needphi.insert((*xvar,xvar3,xtype.clone()));
      self.fillneed(&bbl0);
      dest0 = Register(xvar3);
    }// recursive fillneed
    else { dest0 = dest0opt.unwrap().0.clone(); }

    let dest1;
    let dest1opt=self.bblocks[bbi1].ssamap.get(xvar);
    if dest1opt.is_none() {
      let xvar3 = str24::from(&format!("{}_1",&xvar2));
      self.bblocks[bbi1].needphi.insert((*xvar,xvar3,xtype.clone()));
      self.fillneed(&bbl1);
      dest1 = Register(xvar3);
    }// recursive fillneed
    else { dest1 = dest1opt.unwrap().0.clone(); }
    if !self.bblocks[tbbi].onphis.contains(&xvar2) {
      let phiinst = Phi2(*xvar2,xtype.clone(),dest0,preds[0],dest1,preds[1]);
      self.bblocks[tbbi].phis.push(phiinst);
      self.bblocks[tbbi].onphis.insert(*xvar2);
    }
  }// for each need, x is original var, xvar2 must be set by phi inst
  //self.bblocks[tbbi].needphi.clear();
}//fillneed

   


//resolve current ssamap with two predecessor bb's
pub fn resolve_ssamap(&mut self, bbl0:&str24, bbl1:&str24,lindex:&mut usize)
{  use crate::llvmir::Instruction::*;
   use crate::llvmir::LLVMexpr::*;
   let bblen = self.bblocks.len();
   let bb0i = *self.bblocator.get(bbl0).unwrap();
   let bb1i = *self.bblocator.get(bbl1).unwrap();
   let mut map0 = self.bblocks[bb0i].ssamap.clone();
   let mut map1 = self.bblocks[bb1i].ssamap.clone();
    // merge two maps with phi's
   for (v,(e0,t0)) in map0.iter() {
      *lindex += 1;
      let v2 = str24::from(&format!("{}_{}",v,lindex));
      let getv = map1.get(v);
      match getv {
        Some((e1,t1)) if e0==e1 => {
          self.setlastSSA(*v,e0.clone(),t1.clone());
        },
        Some((e1,t1)) if !self.bblocks[bblen-1].onphis.contains(&v2) => {
          let inst = Phi2(v2,t0.clone(),e0.clone(),*bbl0,e1.clone(),*bbl1);
          self.bblocks[bblen-1].phis.push(inst);
          self.bblocks[bblen-1].onphis.insert(v2);
          self.setlastSSA(*v,Register(v2),t1.clone());
        },
        None => { panic!("SHOULD NOT HAPPEN1");
          /*
          let inst = Phi2(*v,t0.clone(),e0.clone(),*bbl0,Register(*v),*bbl1);
          self.bblocks[bblen-1].phis.push(inst);
          */
        },
        _ => {},
      }//match
   }// for each entry in map0
   for (v,(e1,t1)) in map1.iter() {
     *lindex += 1;
     let v2 = str24::from(&format!("{}_{}",v,lindex));   
     if self.bblocks[bb0i].ssamap.contains_key(v) {continue;}
     if let Some((e0,t0)) = map0.get(v) {
      if !self.bblocks[bblen-1].onphis.contains(&v2) {
        let inst = Phi2(v2,t1.clone(),e0.clone(),*bbl0,e1.clone(),*bbl1);
        self.setlastSSA(*v,Register(v2),t1.clone());
        self.bblocks[bblen-1].phis.push(inst);
        self.bblocks[bblen-1].onphis.insert(v2);
      }
     } else { panic!("SHOULD NOT HAPPEN2");
        /*
        let inst = Phi2(*v,t1.clone(),Register(*v),*bbl0,e1.clone(),*bbl1);
        self.bblocks[bblen-1].phis.push(inst);
        */
     } // no entry in map0, 
   }//for each entry in map1
}//resolve_ssamap
/*
pub fn set_ssamap(&mut self) //call before adding to func
{  use crate::llvmir::Instruction::*;
   use crate::llvmir::LLVMexpr::*;
   let bbpreds = self.bblocks[self.bblocks.len()-1].predecessors.clone();
   let bbl = self.bblocks[self.bblocks.len()-1].label;
   if bbpreds.len()==0 { return; }
   for i in 0..bbpreds.len() {
     self.resolve_ssamap(&bbl,&bbpreds[i]);
   }
}//setssamap
*/

}//impl LLVMFunction


impl LLVMProgram
{
  pub fn new(name:&str) -> Self {
    LLVMProgram {
      preamble:format!(";Output of LLVM compiler for Lambda7c source {}\n",name),
      global_declarations:Vec::new(),
      functions: Vec::new(),
      postamble:String::new(),
      strconsts:HashMap::new(),
    }
  }//new
}//impl LLVMProgram


//////////////////////////////////// standalone functions /////////////

pub fn is_terminator(inst:&Instruction) -> bool
{ use crate::llvmir::Instruction::*;
  match inst {
    Br_uc(_) | Bri1(_,_,_) | Ret(_,_) | Ret_noval => true,
    _ => false,
  }
}//is_terminator

pub fn isfloat(t:&LLVMtype) -> bool {
   if let LLVMtype::Basic("double") = t {true} else {false}
}

// translate lambda7c operators to operator names in LLVM
pub fn oprep<'t>(e:&'t Expr<'t>, isfloat:bool) -> &'static str
{  use crate::bump7c_ast::Expr::*;
   match e {
     Mult{a,b} => if isfloat {"fmul"} else {"mul"},
     Div{a,b} => if isfloat {"fdiv"} else {"sdiv"},
     Plus{a,b} => if isfloat {"fadd"} else {"add"},
     Minus{a,b} => if isfloat {"fsub"} else {"sub"},
     Mod{a,b} =>  if isfloat {"frem"} else {"srem"},
     Eq{a,b} => if isfloat {"oeq"} else {"eq"},
     Neq{a,b} => if isfloat {"one"} else {"ne"},
     Lt{a,b} => if isfloat {"olt"} else {"slt"},
     Leq{a,b} => if isfloat {"ole"} else {"sle"},
     Gt{a,b} => if isfloat {"ogt"} else {"sgt"},
     Geq{a,b} => if isfloat {"oge"} else {"sge"},     
     And(_,_) if !isfloat => "and",
     Or(_,_) if !isfloat => "or",
     _ => "INVALID OP",
   }//match
}//oprep

// note: LLVM and/or are bitwise operations


//////////////////////////////////////////// to_string ///////////////

impl LLVMtype {
  pub fn to_string(&self) -> String
  { use crate::llvmir::LLVMtype::*;
    match self {
      Basic(x) => x.to_string(),
      Pointer(t) => format!("{}*",&t.to_string()),
      Array_t(n,ty) => format!("[{} x {}]",n,&ty.to_string()),
      Userstruct(s) => format!("%struct.{}",&s),
      Ellipsis => "...".to_string(),
      Void_t => "void".to_string(),
      Func_t(ts) => {
        let mut ty = format!("{} (",&ts[ts.len()-1].to_string());
        for i in 0..ts.len()-1 {
          ty.push_str(&ts[i].to_string()); ty.push(',')
        }
        if ty.ends_with(',') {ty.pop();}
        ty.push(')');
        ty
      },
    }//match
  }
}//type

impl LLVMexpr {
  pub fn to_string(&self) -> String
  { use crate::llvmir::LLVMexpr::*;
    match self {
      Iconst(n) => n.to_string(),
      Sconst(s) => s.to_string(),
      Fconst(n) => {
        if ((*n as i64) as f64)==*n { format!("{}.0",n) }
        else {n.to_string()}
      },
      Aconst(ty,vs) => {
        let mut sa = String::from("[");
        let tys = ty.to_string();
        for v in vs {
          sa.push_str(&format!("{} {},",&tys,v.to_string()));
        }
        if sa.ends_with(',') {sa.pop();}
        sa.push(']');
        sa
      },
      I1const(b) => b.to_string(), // LLVM true/false
      Register(r) => format!("%{}",&r),
      Global(s) => format!("@{}",&s),
      Novalue => String::new(), //String::from(";NOVALUE!!"),
    }//match
  }
}//expr

impl Instruction
{
  pub fn to_string(&self) -> String
  { use crate::llvmir::LLVMexpr::*;
    use crate::llvmir::LLVMtype::*;
    use crate::llvmir::Instruction::*;
    match self {
      Ret(t,e) => format!("ret {} {}\n",t.to_string(),e.to_string()),
      Ret_noval => "ret void\n".to_string(),
      Br_uc(dest) => format!("br label %{}\n",&dest),
      Bri1(e,d1,d2) => format!("br i1 {}, label %{}, label %{}\n",e.to_string(),&d1,&d2),
      Load(dst,ty,ex,_) => format!("%{0} = load {1}, {1}* {2}\n",&dst,&ty.to_string(),&ex.to_string()),
      Store(ty,src,dst,_) => format!("store {} {}, {}* {}\n",&ty.to_string(),&src.to_string(),&ty.to_string(),&dst.to_string()),
      Alloca(dst,ty,_) => format!("%{} = alloca {}\n",&dst,&ty.to_string()),
      Binaryop(dst,op,ty,a,b) => format!("%{} = {} {} {}, {}\n",&dst,&op,&ty.to_string(),&a.to_string(),&b.to_string()),
      Unaryop(dst,op,ff,ty,e) => format!("%{} = {} {} {}\n",&dst,&op,&ty.to_string(),&e.to_string()),
      Cast(dst,op,t1,e,t2) => format!("%{} = {} {} {} to {}\n",&dst,&op,&t1.to_string(),&e.to_string(),&t2.to_string()),
      Icmp(dst,op,ty,a,b) => format!("%{} = icmp {} {} {}, {}\n",&dst,&op,&ty.to_string(),&a.to_string(),&b.to_string()),
      Fcmp(dst,op,ty,a,b) => format!("%{} = fcmp {} {} {}, {}\n",&dst,&op,&ty.to_string(),&a.to_string(),&b.to_string()),
      SelectTrue(dst,ty,a,b) => format!("%{} = select i1 true, {} {}, {} {}\n",&dst,&ty.to_string(),&a.to_string(),&ty.to_string(),&b.to_string()),
      Phi2(dst,ty,e1,lb1,e2,lb2) => format!("%{} = phi {} [{}, %{}], [{}, %{}]\n",&dst,&ty.to_string(),&e1.to_string(),&lb1,&e2.to_string(),&lb2),
      Phi(dst,ty,choices) => {
        let mut inst = format!("%{} = phi {} ",&dst,&ty.to_string());
        for (e,label) in choices.iter() {
          inst.push_str(&format!("[{}, %{}],",&e.to_string(),&label));
        }
        if inst.ends_with(',') {inst.pop();}
        inst.push('\n');
        inst
      },
      Call(dopt,ty,tyopts,fname,aargs) => {
        let mut cs = String::new();
        if let Some(dst) = &dopt { cs.push_str(&format!("%{} = ",dst)); }
        cs.push_str(&format!("call {} ",&ty.to_string()));
        if tyopts.len()>0 {
          cs.push('(');
          for t in tyopts {
            cs.push_str(&format!("{},",t.to_string()));
          }
          if cs.ends_with(',') {cs.pop();}
          cs.push_str(") ");
        }//optional types
        cs.push_str(&format!("@{}(",&fname));
        for (ta,aa) in aargs {
          cs.push_str(&format!("{} {},",&ta.to_string(),&aa.to_string()));
        }
        if cs.ends_with(',') {cs.pop();}
        cs.push_str(")\n");
        cs
      },
      Arrayindex(dst,n,ty,ar,ai) => format!("%{0} = getelementptr inbounds [{1} x {2}], [{1} x {2}]* {3}, i64 0, i32 {4}\n",&dst,n,&ty.to_string(),&ar.to_string(),&ai.to_string()),
      Vectorindex(dst,ty,ar,ai) => format!("%{0} = getelementptr inbounds {1}, {1}* {2}, i64 0, i32 {3}\n",&dst,&ty.to_string(),&ar.to_string(),&ai.to_string()),      
      Structfield(dst,ty,se,fe) => format!("%{0} = getelementptr inbounds {1}, {1}* {2}, i32 0, i32 {3}\n",&dst,&ty.to_string(),&se.to_string(),&fe.to_string()),
      Verbatim(v) => v.clone(),
      Nop => String::new(),
    }//match
  }//to_string()
}//instruction

impl LLVMdeclaration {
  pub fn to_string(&self) -> String
  { use crate::llvmir::LLVMdeclaration::*;
    use crate::llvmir::LLVMexpr::*;
    use crate::llvmir::LLVMtype::*;
    match self {
      Globalconst(dst,ty,ge,_) => {
        let mut copt = "";
        if let Array_t(_,t) = &*ty {
            if let Basic("i8") = &**t {
              copt = "c";
            }
        }
        format!("@{} = constant {} {}{}\n",&dst,&ty.to_string(),copt,&ge.to_string())
      },
      Externfunc(ty,fname,fargs) => {
        let mut ds = format!("declare {} @{}(",&ty.to_string(),&fname);
        for t in fargs {ds.push_str(&format!("{},",&t.to_string()));}
        if ds.ends_with(',') {ds.pop();}
        ds.push_str(")\n");
        ds
      },
      Structdec(sname,ftypes) => {
        let mut st = format!("%struct.{} = type {{",&sname);
        for t in ftypes { st.push_str(&format!("{},",&t.to_string())); }
        if st.ends_with(',') {st.pop();}
        st.push_str("}\n");
        st
      },
      Verbatim_dec(s) => s.clone(),
    }//match
  }
}//declaration

impl BasicBlock
{
  pub fn to_string(&self) -> String
  {
    let mut bb = format!("  {}:\n",&self.label);
    for inst in &self.phis {
      bb.push_str("    ");
      bb.push_str(&inst.to_string());      
    }
    for inst in &self.body {
      bb.push_str("    ");
      bb.push_str(&inst.to_string());
    }
    bb
  }
}//BB tostring

impl LLVMFunction
{
  pub fn to_string(&self) -> String
  {
     let mut fun = format!("define {} @{}(",&self.return_type.to_string(),&self.name);
     for (ty,ta) in &self.formal_args {
       fun.push_str(&format!("{} %{},",&ty.to_string(),ta));
     }
     if fun.ends_with(',') {fun.pop();}
     fun.push_str(")  {\n");
     for bb in &self.bblocks { fun.push_str(&bb.to_string()); }
     fun.push_str(&format!("}} ;function {}\n",&self.name));
     fun
  }
}//function tostring

impl LLVMProgram
{
  pub fn to_string(&self) -> String
  {
    let mut prog = self.preamble.clone();
    for gc in &self.global_declarations { prog.push_str(&gc.to_string());}
    for fun in &self.functions {
      prog.push('\n');
      prog.push_str(&fun.to_string());
    }
    prog.push('\n');
    prog.push_str(&self.postamble);
    prog
  }
} // to_string for LLVMProgram
