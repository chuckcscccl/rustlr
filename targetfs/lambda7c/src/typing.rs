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
use rustlr::LBox;
use std::collections::{HashMap,HashSet,BTreeMap};
use crate::l7c_ast;
use crate::l7c_ast::*;
use crate::typing::lrtype::*;

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
}// lrtype

// unification algorithm
type Equations = Vec<(lrtype,lrtype)>;

//returns success/failure followed by substitution map for unifier
pub fn unify_types(equations:&mut Equations) -> (bool,HashMap<i32,lrtype>)
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
  (!failure,unifier)
}//unify_types

pub fn unifytest()
{
  let t1 = LRlist(Box::new(LRvar(4)));
  let t2 = LRlist(Box::new(Int_t));
  let t3 = LRlist(Box::new(LRvar(5)));
  let mut eq1 = vec![(t1.clone(),t2), (t1.clone(),t3)];
  eq1.push((LRvar(1),LRvar(2)));
  eq1.push((LRvar(4),LRvar(2)));  
  let (nofailure,unifier) = unify_types(&mut eq1);
  if !nofailure {println!("unification failed"); return;}
  for (x,t) in unifier.iter() {
    println!("var_{} = {:?}", x, t);
  }
}//unifytest


///////////// type checking lambda7c (with minimal type inference)
// type checking returns a grounded type for all declarations.

//// Type checking stage will also construct symbol table for later use
// maps variable to global index and type

// maps freevars to (gindex,type), gindex for brute-force alpha-conversion
pub type VarSet<'t> = BTreeMap<&'t str,(usize,lrtype)>;

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
}//Global symbol table struct

impl<'t> SymbolTable<'t>
{
  pub fn new() -> Self  // start with one global frame
  { let mut st = SymbolTable{frames:Vec::with_capacity(128), types:HashSet::new(), current:0, gindex:0, frame_locate:HashMap::new(),};
    st.frames.push(table_frame{name:"global",entries:HashMap::new(),closure:VarSet::new(),parent_scope:usize::MAX}); // using usize::MAX to mean non-existent
    st
  }//new

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
     //let mut bvs = HashSet::new();
     self.collect_freevars(&mut fvs);
     fvs
   }//closure_vars

   // collect freevariables, recursively on functions.
   fn collect_freevars(&mut self, fvs:&mut VarSet<'t>)
   { use crate::l7c_ast::Expr::*;
     use crate::typing::lrtype::*;
     let mut branches = Vec::new(); // branches of table to explore
     let mut oldbranches = HashSet::new();
     let totalframes = self.frames.len();
     /*
     if let TypedLambda{return_type,formal_args,body} = expr {
       branches.push(*self.frame_locate.get(&(body.line,body.column)).unwrap());
     } else {branches.push(self.current);}
     */
     branches.push(self.current);
     // compute a kind of closure
     while branches.len()>0
     {
       let mut fi = branches.pop().unwrap();
       if oldbranches.contains(&fi) {continue;}
       else {oldbranches.insert(fi);}
       let fi0 = fi; // fixed
       while fi < totalframes
       {
         let mut frame = &self.frames[fi];
         for (x,xentry) in frame.entries.iter() { // local vars
           if x.starts_with("_lrtlSelf") {continue;} // skip unnamed lambda
           if let LRfun(_) = &xentry.typefor {continue;} // skip functions**
           if fi0!=fi {
             fvs.insert(x,(xentry.gindex,xentry.typefor.clone()));
           }
           /*
           match(&xentry.ast_rep) {
             Some(TypedLambda{return_type,formal_args,body}) => {
               branches.push(*self.frame_locate.get(&(body.line,body.column)).unwrap());
             }, // really need?
             _ => (),
           }//match
           */
         }// for each in current entry
         fi = frame.parent_scope;
       }//while fi<self.frames.len()
     } // while there are branches
   }//collect_freevars

// functions where not included in closure collection, so functions cannot
// be passed to functions (they can't anyway, since they're closures).


  ////// type checking with minimal type inference
pub fn check_type(&mut self, expr:&'t LBox<Expr<'t>>)-> &lrtype
  { use crate::l7c_ast::Expr::*;
    match &**expr {
      integer(_) => &Int_t,
      floatpt(_) => &Float_t,
      strlit(_) => &String_t,
      var(x) => {
        let xtype = self.get_type(x,0);
	if xtype==&LRuntypable {
	  eprintln!("Unknown Variable {}, line {} column {}",x,expr.line,expr.column);
	}
	xtype
      },
      Beginseq(se) => {
        let mut setype = Int_t;
        for i in 0..se.len() {
          setype = self.check_type(&se[i]).clone();
          if !setype.grounded() { break; }
        }
        if !setype.grounded() {return &LRuntypable;}
        self.types.insert(setype.clone());
        self.types.get(&setype).unwrap()
      },
      Neg(a) | Not(a) => {
        let ta = self.check_type(a).clone();
        if ta.numerical() {
          self.types.insert(ta.clone());
          self.types.get(&ta).unwrap()
        } else {&LRuntypable}
      },
      Display(e) => {
        let te = self.check_type(e);
        if te.grounded() {&Unit_t} else {&LRuntypable}
      },
      Eq(a,b) | Leq(a,b) | Neq(a,b) | Geq(a,b) | Gt(a,b) | Lt(a,b) | Mod(a,b) |
      Plus(a,b) | Minus(a,b) | Mult(a,b) | Div(a,b) | And(a,b) | Or(a,b) => {
        let ta = self.check_type(a).clone();
        let tb = self.check_type(b).clone();
        if &ta==&tb && ta.numerical() {
           self.types.insert(ta);
           self.types.get(&tb).unwrap_or(&LRuntypable)
        } else {&LRuntypable}
      },
      Vector(vs) => {
        let mut etype = Int_t; // default type   ***HACK
        if vs.len()>0 {
           etype = self.check_type(&vs[0]).clone();
           for i in 1 .. vs.len() {
             let itype = self.check_type(&vs[i]);
             if itype!=&etype {
               println!("Vector on line {} cannot contain values of different types",expr.line);
               return &LRuntypable;
             }
           }
        }// non-empty vector
        let vtype = LRlist(Box::new(etype));
        self.types.insert(vtype.clone());
        self.types.get(&vtype).unwrap()
      },
      Vector_make(init,size) => {  //  [0;4]
        let itype = self.check_type(&*init).clone();
        let stype = self.check_type(&*size);
        if stype != &Int_t {
          println!("array dimension not an integer, line {} column {}",size.line,size.column);
          return &LRuntypable;
        }
        let vtype = LRlist(Box::new(itype));
        self.types.insert(vtype.clone());
        self.types.get(&vtype).unwrap()
      },
      Index(ae,ai) => {  //ae[ai]
        let etype = self.check_type(ae).clone();
        let itype = self.check_type(ai);
        if itype!=&Int_t {
          println!("array index is not an integer, line {}",expr.line);
          return &LRuntypable;
        }
        if let LRlist(bx) = &etype {
          let ltype = &**bx;
          self.types.insert(ltype.clone());
          self.types.get(ltype).unwrap()
        } else {
          println!("expression on line {}, column {} is not of vector type",ae.line,ae.column);
          &LRuntypable
        }
      },
      Setq(lv,rv) => { // assign l-value to r-value
        let vtype = self.check_type(rv).clone();
        if !vtype.grounded() {return &LRuntypable;}
        match &**lv {
          Index(a,i) => {   // A[i] = rv
            let atype = self.check_type(a).clone();
            let itype = self.check_type(i);
            if itype != &Int_t {
              println!("array index is not an integer, line {}",expr.line);
              return &LRuntypable;
            }
            let expected_type = LRlist(Box::new(vtype.clone()));
            if &atype != &expected_type {
              println!("array assignment type mismatch, line {}",expr.line);
              return &LRuntypable;
            }
            self.types.insert(vtype.clone()); //pure-rust only!
            self.types.get(&vtype).unwrap()            
          },
          var(x) => {
            let xtype = self.get_type(x,0).clone();
            if !xtype.grounded() {
              println!("UNDECLARED VARIABLE {} CANNOT BE ASSIGNED TO, line {}",x,lv.line);
              return &LRuntypable;
            }
            if &xtype==&vtype {
              self.types.insert(xtype.clone());
              self.types.get(&xtype).unwrap()
            } else {
              println!("VALUE OF TYPE {:?} CANNOT BE ASSIGNED TO VARIABLE OF TYPE {:?}, line {}",vtype,&xtype,rv.line);
              &LRuntypable
            }
          },
          _ => &LRuntypable
        }//match
      },
      App(fun,aargs) => {
        let ftype = self.check_type(fun).clone();
        let mut result = &LRunknown;
        match ftype {
          LRfun(ts) if aargs.len()+1==ts.len() => {
            // collect actual argument types
            let mut atypes = vec![];
            for a in aargs {atypes.push(self.check_type(a).clone())}
            for i in 0..atypes.len() {
              if &atypes[i]!=&ts[i] || !atypes[i].grounded()
              {result=&LRuntypable; break;}
            }
            if result!=&LRuntypable {result=&ts[ts.len()-1];}
            self.types.insert(result.clone());
            self.types.get(result).unwrap()
          },
          _ => {&LRuntypable}
        }//match
      },
      // following only allows for first order functions
      Define(x,None,e) => {
        if let TypedLambda{return_type,formal_args,body} = &**e {
          let ti=self.check_tlambda(x,e,body.line,body.column);
          self.pop_frame();
          if ti==usize::MAX {return &LRuntypable;}
          //let tt = self.geti_type(ti,x);
          let tlentry = self.frames[ti].entries.get(x).unwrap();
          let inferred_type = tlentry.typefor.clone();
          //println!("/* inferred_type is {:?} */",&inferred_type);
          let gi = tlentry.gindex;
          self.add_entry(x,ti as i32,inferred_type,Some(&**e));
          let old_entry = self.get_entry_mut(x,0).unwrap();
          old_entry.gindex = gi;
          return self.get_type(x,0)
        }
        /*
        else if let Lambda(fargs,body) = &**e {
          let ti=self.check_tlambda(x,&convert);
          if ti==usize::MAX {return &LRuntypable;}
          self.pop_frame();
          let tt = self.geti_type(ti,"_lrtlSelf_");
          self.add_entry(x,ti as i32,tt.clone(),None);
          return self.get_type(x,0)          
        }
        */
        let etype = self.check_type(e).clone();
        if etype.grounded() {
println!("type {:?} inferred for {}",&etype,&x); //trace print
          self.add_entry(x,0,etype,Some(expr));
          self.get_type(x,0)
        }
        else {&LRuntypable}
      },
      Define(x,ty,e) => {
        let dty = LRty(ty.as_deref());
        let etype = self.check_type(e).clone();
        if etype == dty {
          self.add_entry(x,0,etype,Some(expr));
          self.get_type(x,0)
        }
        else {&LRuntypable}
      },
      Let(x,txopt,v,body) => {
        //create new symbol table frame
        let mut result = &LRunknown;
        self.push_frame("let",v.line,v.column); //new frame, locate info
        let vtype = self.check_type(v).clone();
        if let Some(tx) = txopt {
          if &vtype != &LRty(Some(&**tx)) {result=&LRuntypable} //err_message!
        }
        if result!=&LRuntypable && vtype.grounded() {
          self.add_entry(x,0,vtype,None);
          let btype = self.check_type(body).clone();
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
        let ctype = self.check_type(condition);
        if ctype!=&Int_t {&LRuntypable}  // need error message
        else {
          let ttype = self.check_type(truecase).clone();
          let ftype = self.check_type(falsecase);
          if &ttype==ftype {ftype} else {&LRuntypable}
        }
      },
      Whileloop{condition,body} => {
        let ctype = self.check_type(condition);
        if ctype!=&Int_t {&LRuntypable}  // need error message
        else {self.check_type(body)}
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
      stype = self.check_type(&es[ei]).clone();
      println!("type inferred for expression on line {}: {:?}",es[ei].line, stype);
    }
    if !stype.grounded() {return &LRuntypable;}
    self.types.insert(stype.clone());
    self.types.get(&stype).unwrap()
  }//check_sequence

  // returns index of its symbol_table frame
  fn check_tlambda(&mut self, s:&'t str, tl:&'t LBox<Expr<'t>>,ln:u32,cl:u32) -> usize
  { use crate::l7c_ast::Expr::*;
     match &**tl {
      TypedLambda{return_type,formal_args,body} => {
        // create new symbol table frame
        let rtype = LRty(Some(return_type));
        self.push_frame(s,ln,cl); // new frame by definition name
        // record formal argument types
        let mut fargs = vec![];
        for vi in 0..formal_args.len() {
          match &*formal_args[vi] {
            Varopt(x,txx@Some(tx)) => {
              let rt = LRty(txx.as_deref());
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
        let btype = self.check_type(body).clone();
        fargs.push(btype.clone()); // return type
        if &btype == &rtype || (&rtype==&LRunknown && btype.grounded()) {
          //self.add_entry("_lrtlSelf_",-1,LRfun(fargs),Some(tl));
          // change entry
          let old_entry = self.get_entry_mut(s,0).unwrap();
          old_entry.typefor = LRfun(fargs);
          self.current
        }
        else {
	  eprintln!("Function {} failed to typecheck, line {}",s,tl.line);
	  usize::MAX
	}
      },
      _ => usize::MAX,
    }//match
  }//check_tlambda   


}// impl SymbolTable


////////////////////////////////////////

// what exactly is a list expression?

pub fn LRty(t:Option<&Txpr>) -> lrtype
{ use crate::l7c_ast::Txpr::*;
  match t {
    None => LRunknown,
    Some(int_t) => Int_t,
    Some(float_t) => Float_t,
    Some(string_t) => String_t,
    Some(unit_t) => Unit_t,
    Some(Txpr_Nothing) => LRunknown,
    Some(vec_t(ty)) => LRlist(Box::new(LRty(Some(&*ty)))),
    //_ => LRuntypable,
  }
}


