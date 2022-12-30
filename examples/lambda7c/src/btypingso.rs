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
use crate::btypingso::lrtype::*;
use fixedstr::str32;

// Lambda7c type
#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub enum lrtype {
  String_t,
  Unit_t,
  Float_t,
  Int_t,
  LRlist(Box<lrtype>),
  LRarray(Box<lrtype>,usize),  // just added
  LRtuple(Vec<lrtype>),
  //LRfun(Vec<lrtype>), // last type is return type
  LRclosure(Vec<lrtype>,str32), //includes name of function-struct
  LRvar(i32), // type variable, signed for later flexibility
  LRunknown,
  LRuntypable,
}//lrtype
impl Default for lrtype { fn default()->Self {lrtype::LRunknown} }
impl lrtype
{
  fn grounded(&self) -> bool
  {
    match self {
      LRuntypable | LRunknown | LRvar(_) => false,
      LRlist(t) => t.grounded(),
      LRarray(t,_) => t.grounded(),      
      LRtuple(tv) | LRclosure(tv,_) => {
        let mut result = true;
        for t in tv { if !t.grounded() {result=false; break;} }
        result
      },
      _ => true,
    }//match
  }//grounded

  fn numerical(&self) -> bool
  {
     match self {
       Int_t | Float_t => true,
       _ => false,
     }
  }//numerical

  fn funtype(&self) -> bool {
    if let LRclosure(_,_) = self {true} else {false}
  }
}//impl lrtype

  /* unification code not used, removed */


///////////// type checking second order lambda7c (with minimal type inference)
// type checking returns a grounded type for all declarations.

//// Type checking stage will also construct symbol table for later use
// maps variable to global index and type

// maps freevars to (gindex,type), gindex for brute-force alpha-conversion
pub type VarSet<'t> = BTreeMap<(&'t str,usize),lrtype>;

// This method of keeping the closure won't work if we returned closures as
// objects: that would require heap allocated memory.  For this version of
// the language, a function with free variables can only be called from another
// function that includes those same variables: closures cannot be passed
// around as objects.

//// Symbol table
pub struct table_entry<'t> {
  pub  typefor : lrtype,
  pub index : i32,    //frame number, array size, etc...
  pub gindex: usize,  // global index for disambiguation
  pub ast_rep : Option<&'t Expr<'t>>,
} // table_entry

// a new frame is created for each lambda term or let-expression
pub struct table_frame<'t> {
  pub name : &'t str,  // frame id, don't know what to do yet
  pub entries : HashMap<&'t str, table_entry<'t>>, // for bound vars
  //pub closure: VarSet<'t>,  // closure variables (possible freevars)
  pub parent_scope : usize, //Option<&'t mut table_frame<'t>>, //Option<Rc<RefCell<table_frame<'t>>>>,
}//symbol_table

pub struct SymbolTable<'t> {
  pub frames: Vec<table_frame<'t>>,
  pub types: HashSet<lrtype>,
  pub current: usize,  // current frame
  gindex: usize, // global index (for alpha-conversion)
  pub frame_locate: HashMap<(u32,u32),usize>, //locate frame by line,column
  pub typeshash: HashMap<(u32,u32), lrtype>,  // hash by location
  pub exported: HashMap<&'t str, lrtype>,  // closure names to LRclosures
}//Global symbol table struct

impl<'t> SymbolTable<'t>
{
  pub fn new() -> Self  // start with one global frame
  { let mut st = SymbolTable{frames:Vec::with_capacity(128), types:HashSet::new(), current:0, gindex:0, frame_locate:HashMap::new(),typeshash:HashMap::new(),exported:HashMap::new(),};
    st.frames.push(table_frame{name:"global",entries:HashMap::new(),/*closure:VarSet::new(),*/parent_scope:usize::MAX}); // using usize::MAX to mean non-existent
    st
  }//new

  pub fn get_checked_type(&self,ln:usize,cl:usize) -> Option<&lrtype>
  {
     self.typeshash.get(&(ln as u32, cl as u32))
  }//get_checked_type

  pub fn push_frame(&mut self, name0:&'t str, ln:u32, cl:u32) 
  {
    self.frames.push(table_frame{name:name0,entries:HashMap::new(),/*closure:VarSet::new(),*/ parent_scope:self.current});
    self.current = self.frames.len()-1;
    self.frame_locate.insert((ln,cl),self.current);
    //let fvs = self.find_closure();
    let fi = self.current;
    //self.frames[fi].closure = fvs;
  }//push_frame

  pub fn pop_frame(&mut self)
  {
    let previous = self.frames[self.current].parent_scope;
    self.current = previous;
  }// pop_frame

  // add entry to symbol table, overwriting existing entry, returns gindex
  pub fn add_entry(&mut self, s:&'t str, i:i32, t:lrtype, a:Option<&'t Expr<'t>>) -> usize
  {
    self.gindex+=1; // 0 not used (can serve special purpose)
    self.frames[self.current].entries.insert(s,table_entry{typefor:t,index:i,gindex:self.gindex,ast_rep:a});
    self.gindex
  }//add_entry

// find frame index with entry for key, or usize::MAX if not found
// gi index correct gindex of entry, if 0 then not used, (0 gindex impossible)
pub fn find_frame(&self, key:&'t str, gi:usize) -> usize 
  {
    let mut psi = self.current;
    while psi < usize::MAX
    {
      if self.frames[psi].entries.contains_key(key) {
        if gi==0 || self.frames[psi].entries.get(key).unwrap().gindex==gi
        {break;}
      }
      psi = self.frames[psi].parent_scope;
    }//while
    psi
  }//
/*
Rust lesson learnt: separate finding with immutable borrows from final
mutable borrow
*/

pub fn get_entry(&self, key:&'t str,gi:usize)-> Option<&table_entry<'t>>
{
  let fi = self.find_frame(key,gi);
  if fi==usize::MAX { return None; }
  self.frames[fi].entries.get(key)
}//get_entry_mut

// also returns frame where variable was found
pub fn get_entry_locate(&self, key:&'t str,gi:usize)-> (usize,Option<&table_entry<'t>>)
{
//println!("finding key {}",key);
  let fi = self.find_frame(key,gi);
  (fi,self.frames[fi].entries.get(key))
}//get_entry_mut

pub fn get_entry_mut(&mut self, key:&'t str,gi:usize)-> Option<&mut table_entry<'t>>
{
  let fi = self.find_frame(key,gi);
  if fi==usize::MAX { return None; }
  self.frames[fi].entries.get_mut(key)
}//get_entry_mut

pub fn get_type(&self, s:&'t str,gi:usize) -> &lrtype
  {
    let ti = self.find_frame(s,gi);
    self.geti_type(ti,s)
  }//get_type

pub fn geti_type(&self, i:usize, s:&'t str) -> &lrtype //have frame number
  {
    if i>=self.frames.len() {return &LRuntypable;}
    let entryopt = self.frames[i].entries.get(s);
    if let Some(entry) = entryopt { &entry.typefor }
    else {&LRuntypable}
  }//get_type

pub fn set_type(&mut self, s:&'t str, ty:lrtype) -> bool
  {
    let entryopt = self.get_entry_mut(s,0);
    if let Some(entry) = entryopt {
      entry.typefor = ty;
      true
    }
    else {false}
  }//set_type

/* find closure stuff removed */

  ////// type checking with minimal type inference
pub fn check_type(&mut self, expr:&'t Expr<'t>, line:usize)-> &lrtype
  { use crate::bump7c_ast::Expr::*;
    match expr {
      integer(_) => &Int_t,
      floatpt(_) => &Float_t,
      strlit(_) => &String_t,
      var(x) => {
        let xtype = self.get_type(x,0);
	if xtype==&LRuntypable {
	  eprintln!("Unknown Variable {}, line {}",x,line);
	}
	xtype
      },
      Beginseq(se) => {
        let mut setype = Int_t;
        for i in 0..se.len() {
          setype = self.check_type(&se[i],se[i].line()).clone(); // Deref on LC
          if !setype.grounded() { break; }
        }
        if !setype.grounded() {return &LRuntypable;}
        self.newtype(&setype)
        //self.types.insert(setype.clone());
        //self.types.get(&setype).unwrap()
      },
      Not(a) => {
        let ta = self.check_type(a,line).clone();
        if &ta==&Int_t {
          self.types.insert(ta.clone());
          self.types.get(&ta).unwrap()
        } else {&LRuntypable}
      },
      Neg{a} => {
        let ta = self.check_type(&**a,line).clone();
        if ta.numerical() {
          self.typeshash.insert((a.1.0,a.1.1),ta.clone());
          self.newtype(&ta)
          //self.types.insert(ta.clone());
          //self.types.get(&ta).unwrap()
        } else {&LRuntypable}
      },      
      Display{e} => {
        let te = self.check_type(&**e,line).clone();
        if te.grounded() {
          self.typeshash.insert((e.1.0,e.1.1),te);
          &Unit_t
        }
        else {&LRuntypable}
      },
      Eq{a,b} | Leq{a,b} | Neq{a,b} | Geq{a,b} | Gt{a,b} | Lt{a,b} | Mod{a,b} |
      Plus{a,b} | Minus{a,b} | Mult{a,b} | Div{a,b} => {
        let ta = self.check_type(a,line).clone();
        let tb = self.check_type(&**b,line).clone();
        if &ta==&tb && ta.numerical() {
           self.typeshash.insert((b.1.0,b.1.1),tb);
           //println!("hashing l/c {},{}",b.1.0,b.1.1);           
           self.newtype(&ta)           
        } else {&LRuntypable}
      },
      And(a,b) | Or(a,b) => {
        let ta = self.check_type(a,line).clone();
        let tb = self.check_type(b,line).clone();
        if &ta==&tb && &ta==&Int_t {
           self.newtype(&ta)           
        } else {&LRuntypable}
      },
      Vector(vs) => {
        let mut etype = Int_t; // default type   ***HACK
        if vs.len()>0 {
           etype = self.check_type(&vs[0],vs[0].line()).clone();
           for i in 1 .. vs.len() {
             let itype = self.check_type(&vs[i],vs[i].line());
             if itype!=&etype {
               println!("Vector on line {} cannot contain values of different types",vs[i].line());
               return &LRuntypable;
             }
           }
        }// non-empty vector
        self.typeshash.insert(vs[0].lncl(),etype.clone());
        //let vtype = LRlist(Box::new(etype));
        let vtype = LRarray(Box::new(etype),vs.len());
        self.newtype(&vtype)
      },
      Vector_make{ve:init,vi:integer(size)} if *size>0 => {  //  [0;4]
        let itype = self.check_type(&init,line).clone();
        self.typeshash.insert(init.lncl(),itype.clone());
        /*
        let stype = self.check_type(size,line);
        if stype != &Int_t {
          println!("array dimension not an integer, line {}",line);
          return &LRuntypable;
        }
        */
        //let vtype = LRlist(Box::new(itype));
        let vtype = LRarray(Box::new(itype),*size as usize);
        self.newtype(&vtype)
      },
      Vector_make{ve,vi} => {
        eprintln!("This version of Lambda7c does not support dynamically sized or zero-sized arrays, line {}",line);
        &LRuntypable
      },
      Index{ae,ai} => {  //ae[ai]
        let etype = self.check_type(&**ae,line).clone();
        let itype = self.check_type(ai,line);
        if itype!=&Int_t {
          println!("array index is not an integer, line {}",line);
          return &LRuntypable;
        }
        self.typeshash.insert((ae.1.0,ae.1.1+1),etype.clone());
        if let LRarray(bx,_) = &etype {
          let ltype = &**bx;
          self.newtype(ltype)
        } else {
          println!("expression on line {} is not of array type",line);
          &LRuntypable
        }
      },
      Setq{lvalue:lvc@LC(Index{ae:a,ai:i},_),rvalue:lrc@LC(rv,_)} => { 
        let vtype = self.check_type(rv,lrc.line()).clone();
        if !vtype.grounded() {return &LRuntypable;}
        let atype = self.check_type(&**a,lvc.line()).clone();
        let itype = self.check_type(i,lrc.line());
        if itype != &Int_t {
              println!("array index is not an integer, line {}",lvc.line());
              return &LRuntypable;
        }
        self.typeshash.insert((a.1.0,a.1.1+1),atype.clone());        
        match &atype {
          LRarray(vt,_) if &**vt==&vtype => {self.newtype(&vtype)},
          _ => {
              eprintln!("array assignment type mismatch, line {}",lvc.line());
              &LRuntypable       
          },
        }//match
        /*
            let expected_type = LRlist(Box::new(vtype.clone()));
            if &atype != &expected_type {

            }
            self.newtype(&vtype)
        */
      },
      Setq{lvalue:lvc@LC(var(x),_),rvalue:lrc@LC(rv,_)} => {           
        let vtype = self.check_type(rv,lrc.line()).clone();
        if !vtype.grounded() {return &LRuntypable;}          
        let xtype = self.get_type(x,0).clone();
        if !xtype.grounded() {
              println!("UNDECLARED VARIABLE {} CANNOT BE ASSIGNED TO, line {}",x,lvc.line());
              return &LRuntypable;
            }
        if &xtype==&vtype {
              self.newtype(&xtype)
        } else {
              println!("VALUE OF TYPE {:?} CANNOT BE ASSIGNED TO VARIABLE OF TYPE {:?}, line {}",vtype,&xtype,lrc.line());
              &LRuntypable
        }
      },
      App(var("getint"),aargs) if aargs.len()==0 => &Int_t,
      App(var("free"),aargs) | App(var("weaken"),aargs) if aargs.len()==1 => {
        let atype = self.check_type(&**aargs[0],aargs[0].line()).clone();
        if let LRclosure(_,_) = &atype {
          self.typeshash.insert(aargs[0].lncl(),atype);
          &Unit_t
        } else {&LRuntypable}
      },
      App(fun,aargs) => {
        let ftype = self.check_type(fun,line).clone();
        let mut result = &LRunknown;
        match ftype {
          LRclosure(ts,tn) if aargs.len()+1==ts.len() => {
            // collect actual argument types
            let mut atypes = vec![];
            for a in aargs {atypes.push(self.check_type(&*a,a.line()).clone())}
            for i in 0..atypes.len() {
              if &atypes[i]!=&ts[i] || !atypes[i].grounded()
              {result=&LRuntypable; break;}
            }
            if result!=&LRuntypable {result=&ts[ts.len()-1];}
            self.newtype(&result)
          },
          _ => {&LRuntypable}
        }//match
      },
      // following only allows for first order functions
      Define{id:idc@LC(x,_),typeopt:None,init_val:ivc@LC(e,_)} => {
        if let TypedLambda{return_type,formal_args,body} = e {
          let ti=self.check_tlambda(x,ivc,body.line(),body.column());
          self.pop_frame();
          if ti==usize::MAX {return &LRuntypable;}
          let tlentry = self.frames[ti].entries.get(x).unwrap();
          let inferred_type = tlentry.typefor.clone();
println!(";//type {:?} inferred for {} on line {}",&inferred_type,&x,ivc.line());
          let gi = tlentry.gindex;
          self.add_entry(x,ti as i32,inferred_type,Some(e));
          let old_entry = self.get_entry_mut(x,0).unwrap();
          let mut oldtype = old_entry.typefor.clone();
          if let LRclosure(ts,cn) = &oldtype {
            let cn2 = str32::from(&format!("{}_{}",cn,gi));
            // change not made inside
            oldtype = LRclosure(ts.clone(),cn2);
          }
          old_entry.gindex = gi;
          old_entry.typefor = oldtype.clone();
          let tmentry = self.frames[ti].entries.get_mut(x).unwrap();
          tmentry.typefor = oldtype;
          return self.get_type(x,0);
        } // lambda case
        let etype = self.check_type(e,ivc.line()).clone();
        if etype.grounded() {
println!(";//type {:?} inferred for {} on line {}",&etype,&x,ivc.line());
          self.add_entry(x,0,etype,Some(expr));
          self.get_type(x,0)
        }
        else {&LRuntypable}
      },
      Define{id:idc@LC(x,_),typeopt:Some(ty),init_val:ivc@LC(e,_)} => {
        let dty = self.LRty(Some(&**ty));
        let etype = self.check_type(e,ivc.line()).clone();
        match (&etype, &dty) {
          (LRarray(t1,_), LRlist(t2))  => {
            self.add_entry(x,0,etype,Some(expr));
            self.get_type(x,0)
          },
          _ if &etype==&dty => {
            self.add_entry(x,0,etype,Some(expr));
            self.get_type(x,0)          
          },
          _ => &LRuntypable,
        }//match
        /*
        if etype == dty {
          self.add_entry(x,0,etype,Some(expr));
          self.get_type(x,0)
        }
        else {&LRuntypable}
        */
      },
      Export(closure_name) => {
        // only exports from local scope where closure was defined is allowed
        if let Some(centry) = self.frames[self.current].entries.get(closure_name) {
          if let cty@LRclosure(_,cn) = &centry.typefor {
             self.exported.insert(closure_name,cty.clone());
          }
        }// found in local scope - export to global
        &Unit_t
      },
      Let{id:idc@LC(x,_),typeopt:txopt,init_val:ivc@LC(v,(vl,vc,_)),body} => {
        //create new symbol table frame
        let mut result = &LRunknown;
        self.push_frame("let",*vl,*vc); //new frame, locate info
        let vtype = self.check_type(v,*vl as usize).clone();
        if let Some(tx) = txopt {
          if &vtype != &self.LRty(Some(&**tx)) {result=&LRuntypable} //err_message!
        }
        if result!=&LRuntypable && vtype.grounded() {
          self.add_entry(x,0,vtype,None);
          let btype = self.check_type(&**body,body.line()).clone();
          if !btype.grounded() {}
          else {
             self.types.insert(btype.clone());
             result = self.types.get(&btype).unwrap();
          }
        } else {result = &LRuntypable;}
        self.current = self.frames[self.current].parent_scope; //pop
        result
      },
      Ifelse{condition,truecase,falsecase} => {
        let ctype = self.check_type(&**condition,condition.line());
        if ctype!=&Int_t {&LRuntypable}  // need error message
        else {
          let ttype = self.check_type(&**truecase,truecase.line()).clone();
          let ftype = self.check_type(&**falsecase,falsecase.line()).clone();
          if &ttype==&ftype {
            self.typeshash.insert((truecase.1.0,truecase.1.1),ttype);
            self.newtype(&ftype)
          } else {&LRuntypable}
        }
      },
      Whileloop{condition,body} => {
        let ctype = self.check_type(&**condition,condition.line());
        if ctype!=&Int_t {&LRuntypable}  // need error message
        else {
          let btype = self.check_type(&**body,body.line());
          if btype.grounded() {&Unit_t} else {&LRuntypable}
        }
      },
      _ =>  &LRuntypable,
    }//match expr
  }//check_type


  //check sequence function: program top level
  pub fn check_sequence(&mut self, seq:&'t Sequence<'t>)-> &lrtype
  {
    let es = &seq.0;
    let mut stype = LRuntypable;
    for ei in 0..es.len() {
      stype = self.check_type(&es[ei],es[ei].line()).clone();
      println!(";//type inferred for expression on line {}: {:?}",es[ei].line(), stype);
      if !stype.grounded() {return &LRuntypable;}
    }
    self.newtype(&stype)
  }//check_sequence

  // returns index of its symbol_table frame
  fn check_tlambda(&mut self, s:&'t str, tl:&'t LC<Expr<'t>>,ln:usize,cl:usize) -> usize
  { use crate::bump7c_ast::Expr::*;
     match &**tl {
      TypedLambda{return_type,formal_args,body} => {
        // create new symbol table frame
        let rtype = self.LRty(Some(return_type));
        self.push_frame(s,ln as u32,cl as u32); // new frame by definition name
        // record formal argument types
        let mut fargs = vec![];
        for vi in 0..formal_args.len() {
          match &**formal_args[vi] {
            Varopt(x,txx@Some(tx)) => {
              //let rt = self.LRty(txx.as_deref());
              let rt = self.LRty(Some(tx));
              self.add_entry(x,vi as i32,rt.clone(),None);
              fargs.push(rt);
            },
            Varopt(x,None) => {
              self.add_entry(x,vi as i32,Int_t,None);  // TEMP HACK***
              fargs.push(Int_t);            
            },
            //_ => {return usize::MAX;}, // need to print message
          }//match vt
        }//for each formal_arg

        // for recursion, insert type into own frame
        let mut fargs2 = fargs.clone();
        fargs2.push(self.LRty(Some(return_type)));
        let expected_type = LRclosure(fargs2,str32::from(s));
        self.add_entry(s,-2,expected_type,Some(tl));
        let btype = self.check_type(&**body,body.line()).clone();
        fargs.push(btype.clone()); // return type
        if &btype == &rtype || (&rtype==&LRunknown && btype.grounded()) {
          // change entry
          let old_entry = self.get_entry_mut(s,0).unwrap();
          old_entry.typefor = LRclosure(fargs,str32::from(s));
          self.current
        }
        else {
	  eprintln!("Function {} failed to typecheck, line {}",s,tl.line());
	  usize::MAX
	}
      },
      _ => usize::MAX, // this means "None"
    }//match
  }//check_tlambda   


  pub fn newtype(&mut self, ty:&lrtype) -> &lrtype
  {
    self.types.insert(ty.clone());
    self.types.get(&ty).unwrap()
  }//newtype


pub fn LRty(&self, t:Option<&Txpr>) -> lrtype
{ use crate::bump7c_ast::Txpr::*;
  match t {
    None => LRunknown,
    Some(int_t) => Int_t,
    Some(float_t) => Float_t,
    Some(string_t) => String_t,
    Some(unit_t) => Unit_t,
    Some(Txpr_Nothing) => LRunknown,
    Some(vec_t(ty,Some(n))) => LRarray(Box::new(self.LRty(Some(&*ty))),*n as usize),
    Some(vec_t(ty,None)) => LRlist(Box::new(self.LRty(Some(&*ty)))),
    Some(closure_t(cn)) => {
      if let Some(cty) = self.exported.get(cn) {cty.clone()}
      else {LRuntypable}
    },
  }
}//LRty moved here.

}// impl SymbolTable
