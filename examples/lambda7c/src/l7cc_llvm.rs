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
use crate::bump7c_ast;
use crate::bump7c_ast::*;
use crate::btyping;
use crate::btyping::*;
use crate::llvmir::*;
use fixedstr::{str8,str16,str24,str32,str64,str128,str256};
use rustlr::{LC,Bumper};
use bumpalo::Bump;
/*
impl<'t> SymbolTable<'t> // defined in l7c_ast module
{
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
   {  use crate::bump7c_ast::Expr::*;
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
   */



//////////////////////// Compile to Simplified LLVM ///////////////////////

//return value of compile_expr -- not used
//pub struct Compout<'t>(LLVMexpr,&'t mut BasicBlock);
// alternative, return ref to last bb

pub struct LLVMCompiler<'t>
{
  pub symbol_table : SymbolTable<'t>,
  program: LLVMProgram,  // to be built
  gindex: usize, // compilation counter
  lindex: usize, // local counter
  pub bumpopt: Option<&'t Bump>,
}//struct LLVMCompiler

impl<'t> LLVMCompiler<'t> // 
{
  pub fn new_skeleton(name:&str) -> Self {
     LLVMCompiler{ symbol_table:SymbolTable::new(),
       program:LLVMProgram::new(name),gindex:0, lindex:0,bumpopt:None,}
  }//new
  pub fn newid(&mut self, prefix:&str) -> str24 {
    self.lindex+=1;
    let mut reg = str24::from(prefix);
    reg.push(&format!("_{}",self.lindex));
    reg
  }//newreg
  pub fn newgid(&mut self, prefix:&str) -> str24 {
    self.gindex+=1;
    let mut reg = str24::from(prefix);
    reg.push(&format!("_{}",self.gindex));    
    reg
  }//newreg  
  pub fn newindex(&mut self) -> usize {
    self.lindex+=1; self.lindex
  }

  // compile with given set of bound variables.
  // symbol table must be set to correct frame when compiling a function
  // Rust doesn't allow multiple mut pointers to same structure, otherwise
  // this function can take a pointer to current function and current BB
  fn compile_expr(&mut self,expr:&'t Expr<'t>, func:&mut LLVMFunction) -> LLVMexpr  //returns expression (destination)
  { use crate::bump7c_ast::Expr::*;
    use crate::llvmir::Instruction::*;
    use crate::llvmir::LLVMtype::*;
    use crate::llvmir::LLVMexpr::*;
    //let bb = func.currentBB(); // mut pointer to current bb.
    match expr {
      var(x) => {
        let (fi,eopt) = self.symbol_table.get_entry_locate(x,0);
        let xentry = eopt.unwrap();
        //let staropt = fi!=self.symbol_table.current; // true=is pointer
        let xind = xentry.gindex;
        let xvar = str24::from(&format!("{}_{}",x,xind));
        let xtype = translate_type(&xentry.typefor);
        //let xtype = if staropt {Pointer(Box::new(xtypebase))}else {xtypebase};
        // generate a load instruction on %x_xind
        let x1 = self.newid(xvar.to_str());
        let inst = Load(x1,xtype,Register(xvar),None);
        func.add_inst(inst);
        Register(x1) // return value of compile_expr
      },
      integer(x) => Iconst(*x as i32),
      floatpt(x) => Fconst(*x),
      strlit(x0) => {
        let strid;
        let mut strsize = x0.len()-1;
        let mut x = x0.to_string();
        while let Some(pos) = x.rfind("\\n") {
          x.replace_range(pos..pos+2,"\\0a");
          strsize-=1;
        }
        x.replace_range(x.len()-1..x.len()-1, "\\00");
        if let Some(id) = self.program.strconsts.get(*x0) {
          strid = *id;
        } else {
          strid =self.newgid("str");
          let dec = LLVMdeclaration::Globalconst(strid,Array_t(strsize,Box::new(Basic("i8"))),Sconst(x.clone()),None);
          self.program.global_declarations.push(dec);
          self.program.strconsts.insert(x,strid);
        }// if new id
        let r1 = self.newid("r");
        let sinst = Arrayindex(r1,strsize,Basic("i8"),Global(strid.resize()),Iconst(0));
        func.add_inst(sinst);
        Register(r1)
      },
      Plus{a:e,b:LC(integer(0),_)} => self.compile_expr(e,func), // nop
      Mult{a,b} | Div{a,b} | Plus{a,b} | Minus{a,b} | Mod{a,b} => {
        let desta = self.compile_expr(a,func);
        let destb = self.compile_expr(&**b,func);
        let r1 = self.newid("r");
        // way to do more efficiently?
        // symbol table must be set to correct frame when compiling a function
        //let rtype = translate_type(self.symbol_table.check_type(a,0));
        let rtype = translate_type(self.symbol_table.get_checked_type(b.line(),b.column()).unwrap());
        let opstr = oprep(expr,isfloat(&rtype));
        func.add_inst(Binaryop(r1,str8::from(opstr),rtype,desta,destb));
        Register(r1)
      },
      Eq{a,b} | Neq{a,b} | Lt{a,b} | Leq{a,b} | Gt{a,b} | Geq{a,b} => {
        let desta = self.compile_expr(a,func);
        let destb = self.compile_expr(&**b,func);
        let r1 = self.newid("cm");
        //let rtype = translate_type(self.symbol_table.check_type(a,0));
        //println!("GETTING l/c {},{}",b.line(),b.column());
        let rtype = translate_type(self.symbol_table.get_checked_type(b.line(),b.column()).unwrap_or(&btyping::lrtype::Int_t));
        let floattype = isfloat(&rtype); //bool
        let opstr = str8::from(oprep(expr,floattype));
        if floattype {
          func.add_inst(Fcmp(r1,opstr,rtype,desta,destb));
        } else {
          func.add_inst(Icmp(r1,opstr,rtype,desta,destb));        
        }//integer type
        let r2 = self.newid("r");
        func.add_inst(Cast(r2,str8::from("zext"),Basic("i1"),Register(r1),Basic("i32")));
        Register(r2)
      },
      Neg{a} => { //compile as for  -1 * a
        let desta = self.compile_expr(&**a,func);
        let r1 = self.newid("r");
        //let rtype = translate_type(self.symbol_table.check_type(a,0));
        let rtype = translate_type(self.symbol_table.get_checked_type(a.line(),a.column()).unwrap());
        if isfloat(&rtype) {
          func.add_inst(Binaryop(r1,str8::from("fmul"),Basic("double"),desta,Fconst(-1.0)));
        }
        else {
          func.add_inst(Binaryop(r1,str8::from("mul"),Basic("i32"),desta,Iconst(-1)));
        }
        Register(r1)
      },
      // technically, booleans should be short-circuited
      And(a,b) => {  // should change source AST to Ifelse, but hard in rust
        let bump = self.bumpopt.unwrap();
        let bzero = bump.alloc(LC::new(integer(0),0,0));
        let lca = bump.alloc(LC::new(Eq{a:a,b:bzero},0,0));
        let lcb = bump.alloc(LC::new(Plus{a:&**b,b:bzero},0,0));
        let lcz = bump.alloc(LC::new(integer(0),0,0));
        let newast = Ifelse {
          condition: lca,
          truecase:  lcz,
          falsecase: lcb,
        };
        self.compile_expr(bump.alloc(newast),func)
      /*  non-short-circuited solution
        let desta = self.compile_expr(a,func);
        let destb = self.compile_expr(b,func);
        let r1 = self.newid("r");
        let r2 = self.newid("r");        
        let cinst = Icmp(r1,str8::from("eq"),Basic("i32"),desta,Iconst(0));
        let sinst = SelectTrue(r2,Basic("i32"),Iconst(0),destb);
        func.add_inst(cinst);  func.add_inst(sinst);
        Register(r2)             // non-short circuited solution
      */
      },
      Or(a,b) => { // should add Nop case to AST to replace Plus(b,0)
        let bump = self.bumpopt.unwrap();
        let bzero = bump.alloc(LC::new(integer(0),0,0));
        let lca = bump.alloc(LC::new(Neq{a:a,b:bzero},0,0));
        let lcb = bump.alloc(LC::new(Plus{a:&**b,b:bzero},0,0));
        let lcz = bump.alloc(LC::new(integer(1),0,0));
        let newast = Ifelse {
          condition: lca,
          truecase:  lcz,
          falsecase: lcb,
        };
        self.compile_expr(bump.alloc(newast),func)
      },
      Not(a) => { // same as a==0
        let bump = self.bumpopt.unwrap();
        let newast = Neq{a:a,b:bump.alloc(LC::new(integer(0),0,0))};
        self.compile_expr(bump.alloc(newast),func)
      },
      
       //SSA need another suffix for current manifestation of this var.
       // need control flow graph: need to know predecessors to create
       // phi - 
       // need to keep track of the current manifestation of variable
       // x.  pointer variables still need store/load.  apply SSA only
       // to local vars.

      // the compiler has a 'program' with global declarations
      Define{id:x,typeopt:tx,init_val:e} => self.compile_define(&**x,e,func),
      Display{e} => {
        let edest = self.compile_expr(&**e,func);
        //let etype = translate_type(self.symbol_table.check_type(e,0));
        let etype = translate_type(self.symbol_table.get_checked_type(e.line(),e.column()).unwrap()); 
        let cheatfun = match etype {
            Basic("i32") => "lambda7c_printint",
            Basic("double") | Basic("float") => "lambda7c_printfloat",
            _ => "lambda7c_printstr",
          };//match
        let inst = Call(None,Void_t,vec![],str32::from(cheatfun),vec![(etype,edest)]);
        func.add_inst(inst);
        Novalue
      },
      Setq{lvalue,rvalue} => {
        let rdest = self.compile_expr(&**rvalue,func);
        match &***lvalue {
          var(x) => { 
            let xentry = self.symbol_table.get_entry(x,0).unwrap();
            let xvar = str24::from(format!("{}_{}",x,xentry.gindex));
            let xtype = translate_type(&xentry.typefor);
            let storeinst = Store(xtype,rdest.clone(),Register(xvar),None);
            func.add_inst(storeinst);
            rdest
          },
          _ => Novalue,  // no index
        }//match
      },
      App(var("getint"),aargs) if aargs.len()==0 => {  //intrinsic function
        let r1 = self.newid("in");
        func.add_inst(Call(Some(r1),Basic("i32"),vec![],str32::from("lambda7c_cin"),vec![]));
        Register(r1)
      },
      App(var(fname0),aargs) => {  // only allow named functions for now..
        let fnentry = self.symbol_table.get_entry(fname0,0).unwrap();
        let fname = str32::from(&format!("{}_{}",fname0,fnentry.gindex));
        let TypedLambda{return_type,formal_args,body}=&fnentry.ast_rep.unwrap()
            else {return Novalue};
        let fnframe = *self.symbol_table.frame_locate.get(&(body.line() as u32,body.column() as u32)).unwrap();
        let fnclosure = self.symbol_table.frames[fnframe].closure.clone();
        let lrtype::LRfun(types0) = &fnentry.typefor else {return Novalue};
        // get frame infor
        let mut fntypes = Vec::new();
        for t in types0 {fntypes.push(translate_type(t));}
        let return_type = fntypes.pop().unwrap();
        let mut argsinfo = Vec::new();
        let mut i = 0;
        while i<fntypes.len()
        {
          let adest = self.compile_expr(&**aargs[i],func);
          let atype = fntypes[i].clone();
          argsinfo.push((atype,adest));
          i+=1;
        }// args loop
        // closure arguments
        for((v0,vi),tv0) in fnclosure.iter() {
          let v = str24::from(&format!("{}_{}",v0,vi));
          let vtype = translate_type(tv0);
          argsinfo.push((Pointer(Box::new(vtype)),Register(v)));
        }
        let r1 = self.newid("r");
        let cdest = if let Void_t = &return_type {None}
            else {Some(r1)};
        let cinst = Call(cdest.clone(),return_type,vec![],fname,argsinfo);
        func.add_inst(cinst);
        if cdest.is_some() {Register(r1)} else {Novalue}
      },
      Let{id:idc@LC(x,_),typeopt:txopt,init_val:ivc@LC(v,(vl,vc,_)),body} => {
        let ivdest = self.compile_expr(v,func);
        let bp = self.symbol_table.current;
        // locate frame for let
        let fi = *self.symbol_table.frame_locate.get(&(*vl,*vc)).unwrap();
        // find type, gindex for let-bound var x:
        let xentry = self.symbol_table.frames[fi].entries.get(x).unwrap();
        let xtype = translate_type(&xentry.typefor);
        let xvar = str24::from(&format!("{}_{}",x,xentry.gindex));
        //compile like a define:
        func.add_inst(Instruction::Alloca(xvar,xtype.clone(),None));
        func.add_inst(Instruction::Store(xtype,ivdest,Register(xvar),None));
        // compile body of let under this symbol table context
        self.symbol_table.current = fi;
        let bdest = self.compile_expr(&**body,func);
        self.symbol_table.current = bp;
        bdest
      },
      Beginseq(seq) => self.compile_seq(&seq,func),
      Ifelse{condition,truecase,falsecase} => {
        let cdest = self.compile_expr(&**condition,func);
        // cdest will be of type i32, not i1 because of lambda7c booleans
        // need to downcast cdest to an i1 before branch

        // optimization
        let ultdest;
        match func.last_instruction() {
          Some(Cast(csdest,castop,Basic("i1"),pdest,Basic("i32")))
          if castop=="zext" => {
            let Some(Cast(_,_,_,pd,_)) = func.pop_last() else {return Novalue};
            ultdest = pd;
          },
          _ => {
          let ccast = self.newid("r");
          func.add_inst(Cast(ccast,str8::from("trunc"),Basic("i32"),cdest,Basic("i1")));
          ultdest = Register(ccast);          
          },
        }//match, optimization

        let label1 = self.newid("iftrue");
        let label0 = self.newid("iffalse");
        let endif = self.newid("endif");
        let brinst = Bri1(ultdest,label1,label0);
        let predlabel = func.currentBBlabel(); // need to do before termination
        func.add_inst(brinst); // add to current BB of function
        // this BB is now complete and already inside func
        let mut BB1 = BasicBlock::new(label1,vec![predlabel]);
        func.addBB_owned(BB1);
        let dest1 = self.compile_expr(&**truecase,func);
        // this could terminate BB1 and create more BBs
        let realabel1 = func.currentBBlabel(); //must call before termination
        func.add_inst(Br_uc(endif)); // currentBB terminated

        let mut BB0 = BasicBlock::new(label0,vec![realabel1]);
        func.addBB_owned(BB0);
        let dest0 = self.compile_expr(&**falsecase,func);
        let realabel0 = func.currentBBlabel(); //must call before termination
        func.add_inst(Br_uc(endif)); // BB0 or whatever's current BB terminated

        let mut newBB = BasicBlock::new(endif,vec![label1,label0]);
        func.addBB_owned(newBB);
        // Each compile_expr should leave the last BB open!        
        //let desttype = translate_type(self.symbol_table.check_type(&**truecase,0));
        let desttype = translate_type(self.symbol_table.get_checked_type(truecase.line(),truecase.column()).unwrap_or(&lrtype::Int_t));        
        if let Void_t = &desttype { // do nothing
          Novalue        
        } else {
          let fdest = self.newid("r");
          let phiinst = Phi2(fdest,desttype,dest1,realabel1,dest0,realabel0);
          func.add_inst(phiinst);
          Register(fdest)
        }
      }, //if
      Whileloop{condition,body} => {
        let cdest1 = self.compile_expr(&**condition,func);
        let cast1 = self.newid("r");
        func.add_inst(Cast(cast1,str8::from("trunc"),Basic("i32"),cdest1,Basic("i1")));
        let startlabel = self.newid("loopstart");
        let endlabel = self.newid("loopend");
        func.add_inst(Bri1(Register(cast1),startlabel,endlabel)); // end of BB
        let label0 = func.currentBBlabel();
        func.addBB_owned(BasicBlock::new(startlabel,vec![label0,startlabel]));
        let bdest = self.compile_expr(&**body,func);
        // compare again
        let cdest2 = self.compile_expr(&**condition,func);
        let cast2 = self.newid("r");
        func.add_inst(Cast(cast2,str8::from("trunc"),Basic("i32"),cdest2,Basic("i1")));
        func.add_inst(Bri1(Register(cast2),startlabel,endlabel)); // end of BB
        let label1 = func.currentBBlabel();
        func.addBB_owned(BasicBlock::new(endlabel,vec![label0,label1]));
        // we'll assume that type of while loop will be void, so no destination
        Novalue
      },
      _ => Novalue,  // default
    }//match
  } //compile_expr

  fn compile_seq(&mut self,seq:&Vec<&'t LC<Expr<'t>>>, func:&mut LLVMFunction) -> LLVMexpr
  {
     let mut i = 0;
     let mut result = LLVMexpr::Novalue;
     while i<seq.len()
     {
       result = self.compile_expr(&**seq[i],func);
       i+=1;
     }//while i<seq.len()
     result
  }//compile_seq

  fn compile_define(&mut self, x:&'t str, expr:&'t Expr<'t>, func:&mut LLVMFunction) -> LLVMexpr
  { use crate::bump7c_ast::Expr::*;
    use crate::llvmir::LLVMexpr::*;
    if let TypedLambda{return_type,formal_args,body}=expr {
      return self.compile_fn(x,expr)
    }
    let edest = self.compile_expr(expr,func);
    let xentry = self.symbol_table.get_entry(x,0).unwrap();
    let xvar = str24::from(format!("{}_{}",x,xentry.gindex));
    let xtype = translate_type(&xentry.typefor);
    func.add_inst(Instruction::Alloca(xvar,xtype.clone(),None));
    func.add_inst(Instruction::Store(xtype,edest.clone(),Register(xvar),None));
    //LLVMexpr::Register(xvar)
    edest
  }//compile_define subprocedure of compile_expr

  //compile a function
  fn compile_fn(&mut self,funn:&str,expr:&'t Expr<'t>) -> LLVMexpr 
  { use crate::bump7c_ast::Expr::*;
    use btyping::lrtype;
    use crate::llvmir::LLVMexpr::*;
    use crate::llvmir::LLVMtype::*;
    use crate::llvmir::Instruction::*;
    let oldindex = self.lindex;
    self.lindex=0;
    let namelessindex = self.newindex();
    let TypedLambda{return_type,formal_args,body} = expr else {return Novalue};
    let bp = self.symbol_table.current;
    let fi = *self.symbol_table.frame_locate.get(&(body.line() as u32,body.column() as u32)).unwrap();
    self.symbol_table.current = fi;
    let fnentry = self.symbol_table.frames[fi].entries.get(funn).unwrap();
    let fnind = fnentry.gindex;
    let mut fname = format!("{}_{}",funn,fnind);
    if funn.len()==0 {fname= format!("_nameless_lambda7c_{}",namelessindex);}

    // gather information from symbol table for function signature.
    let lrtype::LRfun(fntypes) = &fnentry.typefor else {return Novalue};
    let rettype = translate_type(&fntypes[fntypes.len()-1]); //return type
    let mut fargs = Vec::new();
    let mut farginsts = Vec::new(); // alloc/store on each formal arg
    let mut i = 0;
    // the type info is in the symbol table but the arg name is in the AST***
    while i<fntypes.len()-1
    {
      let argn = (&**formal_args[i]).0; // src argument name (pure)
      let aindex = self.symbol_table.frames[fi].entries.get(argn).unwrap().gindex;
      let argname = str24::from(&format!("farg_{}_{}",argn,aindex));
      let argname0 = str24::from(&format!("{}_{}",argn,aindex));
      let argtype = translate_type(&fntypes[i]);
      // generate instruction at same time? YES
      fargs.push((argtype.clone(),argname));
      farginsts.push(Alloca(argname0,argtype.clone(),None));
      farginsts.push(Store(argtype,Register(argname),Register(argname0),None));
      i+=1;
    }// while through fntypes
    // additional, closure arguments
    let fnclosure = self.symbol_table.get_current_closure(); //Varset
    for((cvar0,cindex),ctype0) in fnclosure.iter() {
      let ctype = translate_type(ctype0);
      let cvar = str24::from(&format!("{}_{}",cvar0,cindex));
      fargs.push((Pointer(Box::new(ctype)),cvar));
    }//closure arguments

    let mut newfunc = LLVMFunction {
      name: str32::from(&fname),
      formal_args: fargs,
      return_type: rettype.clone(),
      bblocks: Vec::new(),
      attributes: Vec::new(),
      bblocator: HashMap::new(),
      //needphi:HashMap::new(),
    };
    newfunc.addBB_owned(BasicBlock::new(str24::from("funbegin"),vec![]));
    for inst in farginsts { newfunc.add_inst(inst); }

    let bdest = self.compile_expr(&**body,&mut newfunc);
    newfunc.add_inst(Ret(rettype,bdest));
    self.symbol_table.current = bp;

    self.program.functions.push(newfunc);
    self.lindex = oldindex;
    Global(str32::from(fname))    
  }//compile_fn

  // assumes symbol_table.check_sequence already called (in main)
  pub fn compile_program(&mut self, mainseq:&'t Sequence<'t>) -> String
  {
     // type check and build symbol table:
     let ptype = self.symbol_table.check_sequence(mainseq);
     if let lrtype::LRuntypable = ptype {
        return String::from(";Program failed to type check. No output produced\n");
     }
     self.program.preamble.push_str(&format!(r#"
target triple = "x86_64-pc-windows-msvc19.33.31629"
;target triple = "x86_64-pc-linux-gnu"

  declare void @lambda7c_printint(i32)
  declare void @lambda7c_printfloat(double)
  declare void @lambda7c_printstr(i8*)
  declare i32 @lambda7c_cin()
  declare void @lambda7c_newline()
  declare i32 @lambda7c_not(i32)
  declare i32 @lambda7c_neg(i32)
"#));
     //create a main function, but don't push onto program until end
     let mut mainfunc = LLVMFunction {
       name: str32::from("main"),
       formal_args: Vec::new(),
       return_type: LLVMtype::Basic("i32"),
       bblocks: Vec::new(),
       attributes: Vec::new(),
       bblocator: HashMap::new(),
       //needphi:HashMap::new(),
     };
     mainfunc.addBB_owned(BasicBlock::new(str24::from("beginmain"),vec![]));

     let mainres = self.compile_seq(&mainseq.0, &mut mainfunc);

     let ret = Instruction::Ret(LLVMtype::Basic("i32"),LLVMexpr::Iconst(0));
     mainfunc.add_inst(ret);
     self.program.functions.push(mainfunc);

     self.program.to_string()
     //println!("PROGRAM: {:?}",&self.program);  //debug
  }//compile_program

}// impl LLVMCompiler

// typemap: lrtype to LLVMIR type
pub fn translate_type(t:&lrtype) -> LLVMtype {
  use crate::btyping::lrtype::*;
  use crate::llvmir::LLVMtype::*;
  match t {
    Unit_t => Void_t,
    Int_t => Basic("i32"),
    Float_t => Basic("double"),
    String_t => Pointer(Box::new(Basic("i8"))),
    _ => Basic("INVALID TYPE"),
  }//match
}


//////////////////// overall function ////////////////////
