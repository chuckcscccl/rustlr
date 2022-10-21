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
use std::collections::{HashMap,HashSet,BTreeSet,BTreeMap};
use crate::l7c_ast;
use crate::l7c_ast::*;
use crate::typing;
use crate::typing::*;

impl<'t> SymbolTable<'t> // defined in l7c_ast module
{
/* moved to typing
   pub fn closure_vars(&mut self, expr:&Expr<'t>) -> VarSet<'t>
   {
     let mut fvs = VarSet::new();
     //let mut bvs = HashSet::new();
     self.collect_freevars(expr,&mut fvs);
     fvs
   }//closure_vars

   // collect freevariables, recursively on functions.
   fn collect_freevars(&mut self, expr:&Expr<'t>, fvs:&mut VarSet<'t>)
   { use crate::l7c_ast::Expr::*;
     use crate::typing::lrtype::*;
     let mut branches = Vec::new(); // branches of table to explore
     let mut oldbranches = HashSet::new();
     let totalframes = self.frames.len();
     if let TypedLambda{return_type,formal_args,body} = expr {
       branches.push(*self.frame_locate.get(&(body.line,body.column)).unwrap());
     } else {branches.push(self.current);}
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
         for (x,xentry) in frame.entries.iter() {
           if x.starts_with("_lrtlSelf") {continue;} // skip unnamed lambda
           if let LRfun(_) = &xentry.typefor {continue;} // skip functions**
           if fi0!=fi {
             fvs.insert(x,(xentry.gindex,xentry.typefor.clone()));
           }
           match(&xentry.ast_rep) {
             Some(TypedLambda{return_type,formal_args,body}) => {
               branches.push(*self.frame_locate.get(&(body.line,body.column)).unwrap());
             },
             _ => (),
           }//match
         }// for each in current entry
         fi = frame.parent_scope;
       }//while fi<self.frames.len()
     } // while there are branches
   }//collect_freevars
*/


   // constructs ordered set of free variables mapped to their global index,
   // bound vars are saved in bvs
   pub fn find_freevars(&mut self, expr:&Expr<'t>) -> VarSet<'t>
   {
     let mut fvs = VarSet::new();
     let mut bvs = HashSet::new();
     self.freevars(expr,&mut bvs,&mut fvs);
     fvs
   }//find_closure

   fn freevars(&mut self, expr:&Expr<'t>, bvs:&mut HashSet<&'t str>, fvs:&mut VarSet<'t>)
   {  use crate::l7c_ast::Expr::*;
      match expr {
        var(x) if !bvs.contains(x) => {
          if let Some(entry) = self.get_entry(x,0) {
            fvs.insert(x,(entry.gindex,entry.typefor.clone()));
          }
        },
        Define(x,_,e) => {
            self.freevars(&*e,bvs,fvs);
            bvs.insert(x); // may want to add outside?
          //}
        },
        TypedLambda{return_type,formal_args,body} => {
          let original_frame = self.current;
          if let Some(fi) = self.frame_locate.get(&(body.line,body.column)) {
            self.current = *fi;
          }
          let mut fargs= vec![];
          for fa in formal_args {
            let Varopt(a1,_) = &**fa;
            fargs.push(a1);
          }
          let mut bvs2 = bvs.clone();
          for fa in fargs { bvs2.insert(fa); }
          self.freevars(&*body,&mut bvs2,fvs);
          self.current = original_frame;
        },
        Let(x,_,v,body) => {
          self.freevars(&*v,bvs,fvs);
          let original_frame = self.current;
          if let Some(fi) = self.frame_locate.get(&(v.line,v.column)) {
            self.current = *fi;
          }
          let newbv = bvs.insert(x);
          self.freevars(&*body,bvs,fvs);
          if newbv {bvs.remove(x);}
          self.current = original_frame;
        },
        App(f,args) => {
          self.freevars(&*f,bvs,fvs);
          for a in args {self.freevars(&*a,bvs,fvs);}
        },
        Ifelse{condition,truecase,falsecase} => {
          self.freevars(&*condition,bvs,fvs);
          self.freevars(&*truecase,bvs,fvs);
          self.freevars(&*falsecase,bvs,fvs);
        },
        Whileloop{condition,body} => {
          self.freevars(&*condition,bvs,fvs);
          self.freevars(&*body,bvs,fvs);        
        },
        Eq(a,b) | Leq(a,b) | Neq(a,b) | Geq(a,b) | Gt(a,b) | Lt(a,b) |Mod(a,b) |
        Plus(a,b) | Minus(a,b) | Mult(a,b) | Div(a,b) | And(a,b) | Or(a,b) => {
          self.freevars(&*a,bvs,fvs);  self.freevars(&*b,bvs,fvs);
        },
        Neg(a) | Not(a) | Display(a) | Car(a) | Cdr(a) => {
          self.freevars(&*a,bvs,fvs);
        },
        Setq(x,e) => { self.freevars(&*e,bvs,fvs); },
        Beginseq(seq) => {
          for s in seq {self.freevars(&*s,bvs,fvs);}
        },
        _ => (),
      }//match
   }//freevars // not used

}//impl SymbolTable



//////////////////////// Compile to ANSI C ///////////////////////

struct Compout  // output of compile function
{
   code: String,  // generated code
   target: String, // variable that holds result of compilation
}
impl Compout {
  fn new(c:String, t:String) -> Self {
    Compout{code:c, target:t}
  }
}//impl Compout
fn newout(t:String,c:String) -> Compout {
  Compout{code:c, target:t}
}

pub struct CCompiler<'t>
{
  pub symbol_table : SymbolTable<'t>,
  functions: Vec<String>,  //BTreeMap<String,String>, // global functions
  cindex: usize, // compilation counter (variable index)
}//struct CCompiler


impl<'t> CCompiler<'t> // defined in l7c_ast module
{
  pub fn new() -> Self {
     CCompiler{ symbol_table:SymbolTable::new(),functions:Vec::new(),cindex:0 }
  }//new
  
  // compile with given set of bound variables.
  fn compile_expr(&mut self, expr:&'t Expr<'t>) -> Compout
  { use crate::l7c_ast::Expr::*;
    use crate::typing::lrtype::*;
    match expr {
      var(x) => {
        let (fi,eopt) = self.symbol_table.get_entry_locate(x,0);
        let xentry = eopt.unwrap();
        let staropt = if fi==self.symbol_table.current {""} else {"*"};
        let xalpha = format!("{}{}_{}",staropt,x,xentry.gindex);
        newout(xalpha.clone(),xalpha)
      }, //var case
      integer(x) => newout(x.to_string(),x.to_string()),
      floatpt(x) => newout(x.to_string(),x.to_string()),
      strlit(x) => newout(x.to_string(),x.to_string()),
      Mult(a,b) => self.compile_binop("*",&*a,&*b),
      Div(a,b) => self.compile_binop("/",&*a,&*b),
      Mod(a,b) => self.compile_binop("%",&*a,&*b),      
      Plus(a,b) => self.compile_binop("+",&*a,&*b),
      Eq(a,b) => self.compile_binop("==",&*a,&*b),
      Neq(a,b) => self.compile_binop("!=",&*a,&*b),
      Geq(a,b) => self.compile_binop(">=",&*a,&*b),
      Gt(a,b) => self.compile_binop(">",&*a,&*b),
      Leq(a,b) => self.compile_binop("<=",&*a,&*b),
      Lt(a,b) => self.compile_binop("<",&*a,&*b),
      And(a,b) => self.compile_binop("&&",&*a,&*b),
      Or(a,b) => self.compile_binop("||",&*a,&*b),      
      Define(x,tx,e) => self.compile_define(x,e),
      Display(e) => {
        let eout = self.compile_expr(&*e);
        let etype = self.symbol_table.check_type(e);
        let form = match etype {
            Int_t => "%d",
            Float_t => "%f",
            _ => "%s", // all other types must be converted to strings first
          };//match
        let code = format!("printf(\"{}\",{})",form,&eout.code);
        newout(eout.target,code)
      },
      Index(a,i) => { // a[i]
        let iout = self.compile_expr(&*i);
        let aout = self.compile_expr(&*a);
        let code = format!("{}[{}]",&aout.code,&iout.code);
        newout(code.clone(),code)
      },
      Setq(lv,e) => {
        let eout = self.compile_expr(&*e);
        match &**lv {
          var(x) => { 
            let xentry = self.symbol_table.get_entry(x,0).unwrap();
            let xalpha = format!("{}_{}",x,xentry.gindex);
            let code = format!("{} = {}",&xalpha,eout.code);
            newout(xalpha,code)
          },
          Index(a,i) => {
            let iout = self.compile_expr(&*i);
            let aout = self.compile_expr(&*a);
            let code = format!("{}[{}] = {}",&aout.code,&iout.code,&eout.code);
            newout(code.clone(),code)
          },
          _ => newout(String::new(),String::new()) // won't happen
        }//match
      },
      TypedLambda{return_type,formal_args,body} => {
        let fvs = self.symbol_table.get_current_closure(); //self.symbol_table.closure_vars(expr);
        self.cindex+=1;
        self.compile_fn("",self.cindex,return_type,formal_args,&fvs,body)
      },
      App(fun,aas) => { // apply function to actual args
        match &**fun {
          var(f) => {
            let fentry = self.symbol_table.get_entry(f,0).unwrap();
            let fname = format!("{}_{}",f,fentry.gindex);
            let ftype = &fentry.typefor;
            let ast_rep = fentry.ast_rep.unwrap();
            let mut fframei = self.symbol_table.current;

            if let TypedLambda{return_type,formal_args,body}=&ast_rep {
              self.symbol_table.current = *self.symbol_table.frame_locate.get(&(body.line,body.column)).unwrap();
            }
            let fvs = self.symbol_table.get_current_closure(); //self.symbol_table.closure_vars(&ast_rep);
            self.symbol_table.current=fframei;
            
            let mut aargs = String::new();
            for a in aas {
              let aout = self.compile_expr(&*a);
              aargs.push_str(&format!("{},",&aout.target));
            }
            for (f,(fi,_)) in &fvs {
              let local;
              match self.symbol_table.frames[fframei].entries.get(f) {
                Some(entry) if &entry.gindex==fi => {local=true;},
                _ => {local=false;}
              }//match
              if local {
//println!("{}_{} is local in frame {}",f,fi,fframei);
                 aargs.push_str(&format!("&{}_{},",f,fi));
              } else {
//println!("{}_{} is not local in frame {}",f,fi,fframei);              
                aargs.push_str(&format!("{}_{},",f,fi));
              }
            }
            if aargs.ends_with(',') {aargs.pop();}
            let code = format!("{}({})",&fname,&aargs);
            newout(code.clone(),code)
          },
          _ => newout(String::new(), String::from("/*NOT SUPPORTED*/"))
        }//match
      },
      Beginseq(seq) => {
        let mut code = String::from("{\n");
        let mut target = String::new();
        for i in 0..seq.len() {
          let eout = self.compile_expr(&*seq[i]);
          code.push_str(&format!("  {};\n",&eout.code));
          if i==seq.len()-1 {target= eout.target;}
        }
        code.push_str("}");
        newout(target,code)
      },
      Ifelse{condition,truecase,falsecase} => {
        let cout = self.compile_expr(&*condition);
        let tout = self.compile_expr(&*truecase);
        let fout = self.compile_expr(&*falsecase);
        let etype = self.symbol_table.check_type(truecase);
        if etype!=&Unit_t {
          let code = format!("({} ? {} : {})",&cout.code,&tout.code,&fout.code);
          newout(code.clone(),code)
        }//non-unit typed
        else {
          let code = format!("  if ({}) {{{}}} else {{{}}}",&cout.code,&tout.code,&fout.code);
          newout(String::new(),code)
        }
      },
      Whileloop{condition,body} => {
        let cout = self.compile_expr(&*condition);
        let bout = self.compile_expr(&*body);
        let code = format!("  while ({}) {{\n{}  }}\n",&cout.code,&bout.code);
        newout(String::new(),code)
      },
      _ => newout(String::new(),String::from("/*NOT SUPPORTED*/")), 
    }//match
  } //compile_expr


  fn compile_fn(&mut self,funn:&str,fnind:usize,rt:&Txpr,fas:&Vec<LBox<Varopt<'t>>>,fvs:&VarSet<'t>,body:&'t LBox<Expr<'t>>) -> Compout
  {
    self.cindex+=1;
    let bp = self.symbol_table.current;
    let fi = *self.symbol_table.frame_locate.get(&(body.line,body.column)).unwrap();
    self.symbol_table.current = fi;
    let mut fname = format!("{}_{}",funn,fnind);
    if fname.len()==0 {fname = format!("_nameless_lambda7c_{}",&self.cindex);}
    let mut fargs = String::new();
    for ai in 0..fas.len() {
      let Varopt(a,_) = &*fas[ai];
      let taentry = self.symbol_table.get_entry(a,0).unwrap();
      let ta = taentry.typefor.to_string();
      let ai = taentry.gindex;
      fargs.push_str(&format!("{} {}_{}",ta,a,ai));
      fargs.push(',');
    }//for each formal arg
    // for closure variables (freevars)
    for (fv,(vi,vt)) in fvs {
      fargs.push_str(&format!("{} *{}_{},",vt.to_string(),fv,vi));
    }//for each freevariable fv, type vt with global index vi
    if fargs.ends_with(',') {fargs.pop();} // get rid of trailing ,
    // find return type from symbol table
    let rtype = match self.symbol_table.geti_type(fi,funn) {
        lrtype::LRfun(ts) => ts[ts.len()-1].clone(),
        _ => lrtype::LRunknown,
      };//match
    
    let bout = self.compile_expr(&*body);
    self.cindex+=1;
    let rvar = format!("_l7c_tempvar_{}",&self.cindex);
    let mut code = String::new();
    if &rtype!=&lrtype::Unit_t {
      code.push_str(&format!("  {} {} = ",&rtype.to_string(),&rvar));
    }
    code.push_str(&format!("{};\n",&bout.code));
    let mut fdef=format!("\n{} {}({}) {{\n{}",&rtype.to_string(),&fname,fargs,&code);
    if &rtype!=&lrtype::Unit_t {
      fdef.push_str(&format!("  return {};\n}}//{}\n",&rvar,&fname));
    } else {fdef.push_str(&format!("\n}}//{}\n",&fname));}
    //fdef.push_str(&format!("  return {};\n}}\\\{}\n",&bout.target,&fname));
    self.functions.push(fdef); // add to global functions list
    self.symbol_table.current = bp; // restore symbol-table stack
    newout(fname,String::new())
  }//compile_fn


  fn compile_binop(&mut self,op:&'static str,a:&'t Expr<'t>,b:&'t Expr<'t>)-> Compout
  {
    let aout = self.compile_expr(a);
    let bout = self.compile_expr(b);
    let code = format!("({} {} {})",aout.code,op,bout.code);
    newout(code.clone(),code) // expressions don't have targets?
  }//compile_binop


  fn compile_define(&mut self, x:&'t str, expr:&'t LBox<Expr<'t>>) -> Compout
  { use crate::l7c_ast::Expr::*;
    match &**expr {
      TypedLambda{return_type,formal_args,body} => {
//print!("COMPILING LAMBDA FOR {}, frame {}..  ",x,self.symbol_table.current);
        let oldframei = self.symbol_table.current;
        let framei = *self.symbol_table.frame_locate.get(&(body.line,body.column)).unwrap();
        self.symbol_table.current = framei;
        let fvs = self.symbol_table.get_current_closure(); //self.symbol_table.closure_vars(&expr); //no ** here?
        let xind = self.symbol_table.get_entry(x,0).unwrap().gindex;
        //let fname = format!("{}_{}",x,xind);
        let result = self.compile_fn(x,xind,return_type,formal_args,&fvs,body);
        self.symbol_table.current = oldframei;
        result
      },
      Vector(vs) => {
        let xentry = self.symbol_table.get_entry(x,0).unwrap();
        let gi = xentry.gindex;
        if let lrtype::LRlist(bx) = &xentry.typefor {
          let atype = bx.to_string();
          let mut code = format!("{} {}_{}[] = {{",atype,x,gi);
          for v in vs {
            let vout = self.compile_expr(v);
            code.push_str(&format!("{},",&vout.code));
          }
          if code.ends_with(',') {code.pop();}
          code.push_str("};");
          newout(code.clone(),code)
        }
        else {newout(String::new(),String::new())} // shouldn't happen
      },
      Vector_make(init,size) => {
        match &**size {
          integer(_) => (),
          _ => {
            let s = format!("/*ANSI C requires a constant size for stack-allocated arrays, line {}*/",size.line);
            //println!("Error: {}",&s);
            return newout(String::new(),s);
          },
        }//match
        let icode = self.compile_expr(init);
        let scode = self.compile_expr(size);
        let xentry = self.symbol_table.get_entry(x,0).unwrap();
        let gi = xentry.gindex;
        if let lrtype::LRlist(bx) = &xentry.typefor {
          let atype = bx.to_string();
          let mut code = format!("{} {}_{}[{}];\n",atype,x,gi,&scode.code);
          code.push_str(&format!("  for(int i=0;i<{};i++) {}_{}[i]={}",scode.code,x,gi,icode.code));
          newout(code.clone(),code)
        }
        else {newout(String::new(),String::new())} // shouldn't happen        
      },
      exp => {
        // lookup symbol table for index
        let xentry = self.symbol_table.get_entry(x,0).unwrap();
        let xtype = xentry.typefor.to_string();
        let gi = xentry.gindex;
        let varname = format!("{}_{}",x,gi);
        let eout = self.compile_expr(expr);
        let dcode = format!("{} {}={}",&xtype,&varname,&eout.code);
        newout(varname,dcode)
      },
      // _ => newout(String::new(),String::new()), // temporary default
    }//match
  }//compile_define


  // assumes symbol_table.check_sequence already called (in main)
  pub fn compile_program(&mut self, prog:&'t Sequence<'t>) -> String
  {
    let mut program =String::from("#include<stdio.h>\n");
    let mut maincode =String::from("\nint main() {\n");
    let Sequence(seq) = prog;
    for se in seq {
//println!("... compiling {:?} in frame {}",&*se,self.symbol_table.current);    
      let seout = self.compile_expr(&*se);
      if seout.code.len()>0 {
        maincode.push_str(&format!("  {}",&seout.code));
//println!("line {}: {}",se.line,&seout.code); //echo trace
        if !seout.code.trim().ends_with("}") && !seout.code.trim().ends_with(';') {maincode.push_str(";\n");}
        else {maincode.push('\n');}
      }
    } // for each statement-expression in sequence
    maincode.push_str("  return 0;\n}//main\n");

    // write global functions before main, avoid duplicates(supports recursion)
    let mut defcount=HashMap::with_capacity(self.functions.len());
    for def in &self.functions { 
      // decode function name
      let fname = extract_fname(def);
      let defcx = *defcount.get(fname).unwrap_or(&0);
      defcount.insert(fname,defcx+1);
    }// count function defs

    // construct count of each function defined
    for fun in &self.functions {
    /*
      let fname= extract_fname(fun);
      let defcx = *defcount.get(fname).unwrap_or(&0);
      if defcx==1 { program.push_str(&fun); }
      else if defcx>1 { defcount.insert(fname,defcx-1); }
    */
      program.push_str(&fun);
    }

    program.push_str(&maincode);
    program
}//comppile_program

}// impl CCompiler


impl lrtype { // this is target-language specific
  fn to_string(&self) -> String
  { use crate::typing::lrtype::*;
    match self {
      String_t => String::from("string"),
      Int_t => String::from("int"),
      Float_t => String::from("float"),
      Unit_t => String::from("void"),
      LRlist(bx) => format!("{}*",bx.to_string()),
      _ => String::from("void*"),
      //_ => format!("NOT SUPPORTED TYPE FORM {:?}",self),
    }//match
  }//tostring
}//impl lrtype

fn extract_fname(def:&str) -> &str
{
  if let Some(spi) = def.find(' ') {
    if let Some(lpi) = def[spi..].find('(') {
        return def[spi+1..lpi].trim();
    }
  }
  ""
}//extract_fname

/*
how to change grammar, generate ast.
new Lxpr, Lvalue category, merge type with Expr?
extend of grammar granularity.  check array index out of bounds with grammar?!
no way. runtime check only! array type mismatch?  pushing it. 3[1] = 4?
probably, since easy to do.

trade off between grammar granularity and later type checking - strike a
balance...

show how to change, modify type checking
[4;a[2]]  allowed?  where to prevent?

where to report this type of error?

grammar?
type checker?
code gen?     *** this is the right answer


*/
