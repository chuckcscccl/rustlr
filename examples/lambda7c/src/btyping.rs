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
use crate::btyping::lrtype::*;
use fixedstr::str32;

// Lambda7c type
#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub enum lrtype {
  String_t,
  Unit_t,
  Float_t,
  Int_t,
  LRlist(Box<lrtype>),
  LRtuple(Vec<lrtype>),
  LRfun(Vec<lrtype>), // last type is return type
  LRclosure(Vec<lrtype>,str32), //includes name of closure-struct
  LRvar(i32), // type variable, signed for later flexibility
  LRunknown,
  LRuntypable,
}//lrtype
impl Default for lrtype { fn default()->Self {lrtype::LRunknown} }

impl lrtype
{
  fn occurs_check(&self, x:i32) -> bool  // true means does not occur
  {  match self {
       LRvar(y) if x==*y => false,
       LRuntypable => false,
       LRlist(t) => t.occurs_check(x),
       LRtuple(ts) | LRfun(ts) => {
         let mut result = true;
         for t in ts {
           if !t.occurs_check(x) { result=false; break; }
         }
         result
       },
       _ => true,  
     }//match
  }// occurs_check


  // non-destructive substitution [t/x] to type
  fn apply_subst(&self, x:i32, t:&lrtype) -> lrtype
  { match self {
      LRvar(y) if *y==x => t.clone(),
      LRunknown => t.clone(),
      LRlist(ty) => LRlist(Box::new(ty.apply_subst(x,t))),
      LRtuple(ts) => LRtuple(ts.iter().map(|y|y.apply_subst(x,t)).collect()),
      LRfun(ts) => LRfun(ts.iter().map(|y|y.apply_subst(x,t)).collect()),
      _ => self.clone(),
    }//match
  }//apply_subst

  fn grounded(&self) -> bool
  {
    match self {
      LRuntypable | LRunknown | LRvar(_) => false,
      LRlist(t) => t.grounded(),
      LRtuple(tv) | LRfun(tv) => {
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
    if let LRfun(_) = self {true} else {false}
  }
}//impl lrtype

// unification algorithm
type Equations = Vec<(lrtype,lrtype)>;

//returns success/failure followed by substitution map for unifier
pub fn unify_types(equations:&mut Equations) -> Option<HashMap<i32,lrtype>>
{
  let mut unifier = HashMap::new();
  let mut failure = false;
  let mut eqi = 0; //indexes equations
  while eqi < equations.len()  // break when failure detected
  {
    let mut neweqs:BTreeMap<usize,(lrtype,lrtype)> = BTreeMap::new();
    let (ref s,ref t) = equations[eqi];
    if (s==t) { eqi+=1; continue; }
    match (s,t) {
      (LRvar(x),u) | (u,LRvar(x)) if u.occurs_check(*x) => {
        for i in 0..equations.len() {
          if i==eqi {continue;}
          let (ref ta,ref tb) = equations[i];
          let ta2 = ta.apply_subst(*x,u);
          let tb2 = tb.apply_subst(*x,u);
          //equations[i] = (ta2,tb2);  // mutation!
          neweqs.insert(i,(ta2,tb2));
        }//for
      },
      (LRlist(ta),LRlist(tb)) => {
        equations.push((*ta.clone(),*tb.clone())); // why no checker error?
      },      
      (LRtuple(tav),LRtuple(tbv)) | (LRfun(tav),LRfun(tbv)) if tav.len()==tbv.len() => {
        for i in 0..tav.len() {
          neweqs.insert(equations.len()+i,(tav[i].clone(),tbv[i].clone()));
          //equations.push((tav[i].clone(), tbv[i].clone()));
        }
      },
      _ => {failure=true; break; }
    }//match
    let originalen = equations.len();
    for (i,(a,b)) in neweqs {
      if i<originalen { equations[i] = (a,b); }
      else { equations.push((a,b));}
    }
    eqi += 1;
  }//while eqi<equations.len()
  // construct unifier
  eqi = equations.len();
  while eqi>0 
  {
     eqi -= 1;
     match &equations[eqi] {
       (LRvar(x), u) | (u,LRvar(x)) if !unifier.contains_key(x) => {
         unifier.insert(*x,u.clone());
       },
       _ => (),
     }//match
  }// while eqi>0
  if failure {None} else {Some(unifier)}
}//unify_types

pub fn grounded_unifer(unifier:&HashMap<i32,lrtype>) -> bool
{
   unifier.values().all(|t|t.grounded())
}//grounded_unifier

pub fn unifytest()
{
  let t1 = LRlist(Box::new(LRvar(4)));
  let t2 = LRlist(Box::new(Int_t));
  let t3 = LRlist(Box::new(LRvar(5)));
  let mut eq1 = vec![(t1.clone(),t2), (t1.clone(),t3)];
  eq1.push((LRvar(1),LRvar(2)));
  eq1.push((LRvar(4),LRvar(2)));  
  if let Some(unifier) = unify_types(&mut eq1) {
    for (x,t) in unifier.iter() {
      println!("var_{} = {:?}", x, t);
    }
  }
  else {println!("unification failed");}
}//unifytest


///////////// type checking lambda7c (with minimal type inference)
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
  pub index : i32,
  pub gindex: usize,  // global index for disambiguation
  pub ast_rep : Option<&'t Expr<'t>>,
} // table_entry

// a new frame is created for each lambda term or let-expression
pub struct table_frame<'t> {
  pub name : &'t str,  // frame id, don't know what to do yet
  pub entries : HashMap<&'t str, table_entry<'t>>, // for bound vars
  pub closure: VarSet<'t>,  // closure variables (possible freevars)
  pub parent_scope : usize, //Option<&'t mut table_frame<'t>>, //Option<Rc<RefCell<table_frame<'t>>>>,
}//symbol_table

pub struct SymbolTable<'t> {
  pub frames: Vec<table_frame<'t>>,
  pub types: HashSet<lrtype>,
  pub current: usize,  // current frame
  gindex: usize, // global index (for alpha-conversion)
  pub frame_locate: HashMap<(u32,u32),usize>, //locate frame by line,column
  pub typeshash: HashMap<(u32,u32), lrtype>,  // hash by location
}//Global symbol table struct

impl<'t> SymbolTable<'t>
{
  pub fn new() -> Self  // start with one global frame
  { let mut st = SymbolTable{frames:Vec::with_capacity(128), types:HashSet::new(), current:0, gindex:0, frame_locate:HashMap::new(),typeshash:HashMap::new(),};
    st.frames.push(table_frame{name:"global",entries:HashMap::new(),closure:VarSet::new(),parent_scope:usize::MAX}); // using usize::MAX to mean non-existent
    st
  }//new

  pub fn get_checked_type(&self,ln:usize,cl:usize) -> Option<&lrtype>
  {
     self.typeshash.get(&(ln as u32, cl as u32))
  }//get_checked_type

  pub fn push_frame(&mut self, name0:&'t str, ln:u32, cl:u32) 
  {
    self.frames.push(table_frame{name:name0,entries:HashMap::new(),closure:VarSet::new(), parent_scope:self.current});
    self.current = self.frames.len()-1;
    self.frame_locate.insert((ln,cl),self.current);
    let fvs = self.find_closure();
    let fi = self.current;
    self.frames[fi].closure = fvs;
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
  let fi = self.find_frame(key,gi);
  //if fi==usize::MAX { return None; }
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

pub fn get_current_closure(&self) -> VarSet<'t>
{
  self.frames[self.current].closure.clone()
}

////// find closure variables of current frame
pub fn find_closure(&mut self) -> VarSet<'t>
   {
     let mut fvs = VarSet::new();
     self.collect_freevars(&mut fvs);
     fvs
   }//find_closure

   fn collect_freevars(&mut self, fvs:&mut VarSet<'t>)
   { use crate::bump7c_ast::Expr::*;
     use crate::btyping::lrtype::*;
     let original_frame = self.current;
     let mut parent_frame = self.frames[original_frame].parent_scope;
     if parent_frame < usize::MAX // usize::MAX means None
     {
        let mut frame = &self.frames[parent_frame]; // rust only
        for ((x,y),z) in frame.closure.iter() {
          fvs.insert((*x,*y),z.clone());
        }
        for (x,xentry) in frame.entries.iter() { // local vars
           if x.starts_with("_lrtlSelf") {continue;} // skip unnamed lambda
           if let LRfun(_) = &xentry.typefor {continue;} // skip functions**
           fvs.insert((x,xentry.gindex),xentry.typefor.clone());
	}// for each table entry
     }//collect all non-function variables from this frame and parent frames
   }//simplified collect_freevars

// functions where not included in closure collection, so functions cannot
// be passed to functions (they can't anyway, since they're closures).

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
        let vtype = LRlist(Box::new(etype));
        self.newtype(&vtype)
      },
      Vector_make{ve:init,vi:size} => {  //  [0;4]
        let itype = self.check_type(init,line).clone();
        let stype = self.check_type(size,line);
        if stype != &Int_t {
          println!("array dimension not an integer, line {}",line);
          return &LRuntypable;
        }
        let vtype = LRlist(Box::new(itype));
        self.newtype(&vtype)
      },
      
      Index{ae,ai} => {  //ae[ai]
        let etype = self.check_type(&**ae,line).clone();
        let itype = self.check_type(ai,line);
        if itype!=&Int_t {
          println!("array index is not an integer, line {}",line);
          return &LRuntypable;
        }
        if let LRlist(bx) = &etype {
          let ltype = &**bx;
          self.newtype(ltype)
        } else {
          println!("expression on line {} is not of vector type",line);
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
            let expected_type = LRlist(Box::new(vtype.clone()));
            if &atype != &expected_type {
              println!("array assignment type mismatch, line {}",lvc.line());
              return &LRuntypable;
            }
            self.newtype(&vtype)
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
      App(fun,aargs) => {
        let ftype = self.check_type(fun,line).clone();
        let mut result = &LRunknown;
        match ftype {
          LRfun(ts) if aargs.len()+1==ts.len() => {
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
          old_entry.gindex = gi;
          return self.get_type(x,0);
        }
        let etype = self.check_type(e,ivc.line()).clone();
        if etype.grounded() {
println!(";//type {:?} inferred for {} on line {}",&etype,&x,ivc.line());
          self.add_entry(x,0,etype,Some(expr));
          self.get_type(x,0)
        }
        else {&LRuntypable}
      },
      Define{id:idc@LC(x,_),typeopt:Some(ty),init_val:ivc@LC(e,_)} => {
        let dty = LRty(Some(&**ty));
        let etype = self.check_type(e,ivc.line()).clone();
        if etype == dty {
          self.add_entry(x,0,etype,Some(expr));
          self.get_type(x,0)
        }
        else {&LRuntypable}
      },
      Let{id:idc@LC(x,_),typeopt:txopt,init_val:ivc@LC(v,(vl,vc,_)),body} => {
        //create new symbol table frame
        let mut result = &LRunknown;
        self.push_frame("let",*vl,*vc); //new frame, locate info
        let vtype = self.check_type(v,*vl as usize).clone();
        if let Some(tx) = txopt {
          if &vtype != &LRty(Some(&**tx)) {result=&LRuntypable} //err_message!
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
        else {self.check_type(&**body,body.line())}
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
    }
    if !stype.grounded() {return &LRuntypable;}
    self.newtype(&stype)
  }//check_sequence

  // returns index of its symbol_table frame
  fn check_tlambda(&mut self, s:&'t str, tl:&'t LC<Expr<'t>>,ln:usize,cl:usize) -> usize
  { use crate::bump7c_ast::Expr::*;
     match &**tl {
      TypedLambda{return_type,formal_args,body} => {
        // create new symbol table frame
        let rtype = LRty(Some(return_type));
        self.push_frame(s,ln as u32,cl as u32); // new frame by definition name
        // record formal argument types
        let mut fargs = vec![];
        for vi in 0..formal_args.len() {
          match &**formal_args[vi] {
            Varopt(x,txx@Some(tx)) => {
              //let rt = LRty(txx.as_deref());
              let rt = LRty(Some(tx));
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
        fargs2.push(LRty(Some(return_type)));
        let expected_type = LRfun(fargs2);
        self.add_entry(s,-2,expected_type,Some(tl));
        let btype = self.check_type(&**body,body.line()).clone();
        fargs.push(btype.clone()); // return type
        if &btype == &rtype || (&rtype==&LRunknown && btype.grounded()) {
          //self.add_entry("_lrtlSelf_",-1,LRfun(fargs),Some(tl));
          // change entry
          let old_entry = self.get_entry_mut(s,0).unwrap();
          old_entry.typefor = LRfun(fargs);
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

}// impl SymbolTable


////////////////////////////////////////

// what exactly is a list expression?

pub fn LRty(t:Option<&Txpr>) -> lrtype
{ use crate::bump7c_ast::Txpr::*;
  match t {
    None => LRunknown,
    Some(int_t) => Int_t,
    Some(float_t) => Float_t,
    Some(string_t) => String_t,
    Some(unit_t) => Unit_t,
    Some(Txpr_Nothing) => LRunknown,
    Some(vec_t(ty,None)) => LRlist(Box::new(LRty(Some(&*ty)))),
    _ => LRuntypable,
  }
}


