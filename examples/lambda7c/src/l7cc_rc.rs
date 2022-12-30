// Second order compiler

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
use crate::btypingso;
use crate::btypingso::*;
use crate::llvmir::*;
use fixedstr::{str8,str16,str24,str32,str64,str128,str256};
use rustlr::{LC,Bumper};
use bumpalo::Bump;


impl<'t> SymbolTable<'t> // defined in l7c_ast module
{
   // constructs ordered set of free variables mapped to their global index,
   // bound vars are saved in bvs

   pub fn expr_freevars(&mut self, expr:&Expr<'t>) -> VarSet<'t>
   {
     let mut fvs = VarSet::new();
     let mut bvs = HashSet::new();
     self.freevars(expr,&mut bvs,&mut fvs);
     fvs
   }//find_freevars of an expression

   fn freevars(&mut self, expr:&Expr<'t>, bvs:&mut HashSet<&'t str>, fvs:&mut VarSet<'t>)
   {  use crate::bump7c_ast::Expr::*;
      match expr {
        var(x) if !bvs.contains(x) => {
          /*
          let (fi,eopt) = self.get_entry_locate(x,0);
          let xentry = eopt.unwrap();
          let mut cli = self.current;
          let mut isfree = fi!=cli;
          while isfree && cli<usize::MAX && self.frames[cli].name=="let" {
            if fi==cli || self.frames[cli].name=="global"
              {isfree=false;}
            cli = self.frames[cli].parent_scope;
          }//while
          if isfree {
            fvs.insert((x,xentry.gindex),xentry.typefor.clone());
          }
          */
          if let Some(entry) = self.get_entry(x,0) {
            fvs.insert((x,entry.gindex),entry.typefor.clone());
//println!("adding {} to freevars",x);            
          }

        },
        //var(x) => {println!("var {} excluded!",x);},
        Define{id:x,typeopt:_,init_val:e} => {
            bvs.insert(x); // may want to add outside?
//println!("{} added to bvars",&***x);
            self.freevars(&**e,bvs,fvs);
        },
        TypedLambda{return_type,formal_args,body} => {
          /*
          let original_frame = self.current;
          if let Some(fi) = self.frame_locate.get(&(body.line() as u32,body.column() as u32)) {
            self.current = *fi;
          }
          */
          let mut fargs= vec![];
          for fa in formal_args {
            let Varopt(a1,_) = &***fa;
            fargs.push(a1);
          }
          let mut bvs2 = bvs.clone();
          for fa in fargs { bvs2.insert(fa); }
          self.freevars(&**body,&mut bvs2,fvs);
          //self.current = original_frame;
        },
        Let{id:x,typeopt:_,init_val:v,body} => {
          self.freevars(&**v,bvs,fvs);
          let original_frame = self.current;
          if let Some(fi) = self.frame_locate.get(&(v.line() as u32,v.column() as u32)) {
            self.current = *fi;
          }
          let newbv = bvs.insert(&**x);
          self.freevars(&**body,bvs,fvs);
          if newbv {bvs.remove(&***x);}
          self.current = original_frame;
        },
        App(var("getint"),_)  => (),
        App(var("free"),args) if args.len()==1 => {
          self.freevars(&**args[0],bvs,fvs);
        },
        App(var("weaken"),args) if args.len()==1 => {
          self.freevars(&**args[0],bvs,fvs);
        },
        App(f,args) => {
          self.freevars(&**f,bvs,fvs); //may take this out for 2nd order
          for a in args {self.freevars(&*a,bvs,fvs);}
        },
        Ifelse{condition,truecase,falsecase} => {
          self.freevars(&**condition,bvs,fvs);
          self.freevars(&**truecase,bvs,fvs);
          self.freevars(&**falsecase,bvs,fvs);
        },
        Whileloop{condition,body} => {
          self.freevars(&**condition,bvs,fvs);
          self.freevars(&**body,bvs,fvs);        
        },
        Eq{a,b} | Leq{a,b} | Neq{a,b} | Geq{a,b} | Gt{a,b} | Lt{a,b} |Mod{a,b} |
        Plus{a,b} | Minus{a,b} | Mult{a,b} | Div{a,b} => {
          self.freevars(&**a,bvs,fvs);  self.freevars(&**b,bvs,fvs);
        },
        Neg{a}  => { self.freevars(&**a,bvs,fvs); },
        Display{e}  => { self.freevars(&**e,bvs,fvs); },        
        And(a,b) | Or(a,b) => {
          self.freevars(&**a,bvs,fvs);  self.freevars(&**b,bvs,fvs);        
        },
        Not(a) | Car(a) | Cdr(a) => {
          self.freevars(&**a,bvs,fvs);
        },
        Setq{lvalue:LC(var(x),_),rvalue:e} => {
          self.freevars(&**e,bvs,fvs);
          if let Some(entry) = self.get_entry(x,0) {
            if !bvs.contains(x) {
              fvs.insert((*x,entry.gindex),entry.typefor.clone());
            } 
          }
        },
        Beginseq(seq) => {
          for s in seq {self.freevars(&**s,bvs,fvs);}
        },
        Vector(vs) => {
          for v in vs {self.freevars(&**v,bvs,fvs);}
        },
        Vector_make{ve:init,vi:size} => {
          self.freevars(&**init,bvs,fvs);
          self.freevars(&**size,bvs,fvs);
        },
        Index{ae,ai} => {
          self.freevars(&**ae,bvs,fvs);
          self.freevars(&**ai,bvs,fvs);          
        },
        _ => (),
      }//match
   }//freevars // not used

}//impl SymbolTable


//////////////////////// Compile to Simplified LLVM SO///////////////////////

pub struct LLVMCompiler<'t>
{
  pub symbol_table : SymbolTable<'t>,
  program: LLVMProgram,  // to be built
  gindex: usize, // compilation counter
  lindex: usize, // local counter
  pub bumpopt: Option<&'t Bump>,  // not used
  pub clsmaps: HashMap<str32,(usize,HashMap<str24,(usize,LLVMtype)>)>, 
  // function fn implies struct fn_i_closure, implies fn_i function that
  // takes LAST arg as self-closure
}//struct LLVMCompiler

// clsmap maps closure names to frame index, and maps each free var
// to struct field offset and type

impl<'t> LLVMCompiler<'t> // 
{
  pub fn new_skeleton(name:&str) -> Self {
     LLVMCompiler{ symbol_table:SymbolTable::new(),
       program:LLVMProgram::new(name),gindex:0, lindex:0,bumpopt:None,clsmaps:HashMap::new(),}
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
    use crate::btypingso::lrtype;
    //let bb = func.currentBB(); // mut pointer to current bb.
    match expr {
      var(x) => {
        self.compile_var(&str24::from(x),func,false,true)
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
        let r1 = self.newid("_r");
        let sinst = Arrayindex(r1,strsize,Basic("i8"),Global(strid.resize()),Iconst(0));
        func.add_inst(sinst);
        Register(r1)
      },
      Plus{a:e,b:LC(integer(0),_)} => self.compile_expr(e,func), // nop
      Mult{a,b} | Div{a,b} | Plus{a,b} | Minus{a,b} | Mod{a,b} => {
        let desta = self.compile_expr(a,func);
        let destb = self.compile_expr(&**b,func);
        let r1 = self.newid("_r");
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
        let rtype = translate_type(self.symbol_table.get_checked_type(b.line(),b.column()).unwrap_or(&btypingso::lrtype::Int_t));
        let floattype = isfloat(&rtype); //bool
        let opstr = str8::from(oprep(expr,floattype));
        if floattype {
          func.add_inst(Fcmp(r1,opstr,rtype,desta,destb));
        } else {
          func.add_inst(Icmp(r1,opstr,rtype,desta,destb));        
        }//integer type
        let r2 = self.newid("_r");
        func.add_inst(Cast(r2,str8::from("zext"),Basic("i1"),Register(r1),Basic("i32")));
        Register(r2)
      },
      Neg{a} => { //compile as for  -1 * a
        let desta = self.compile_expr(&**a,func);
        let r1 = self.newid("_r");
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
            let ldest = self.compile_var(&str24::from(x),func,false,false);
            let xentry = self.symbol_table.get_entry(x,0).unwrap();
            let xvar = str24::from(format!("{}_{}",x,xentry.gindex));
            let xtype = translate_type(&xentry.typefor);

            // Reference counter adjustment (move semantics)
            if is_closurestar(&xtype) {
              // decrecrement lvalue ref counter (must be bound to something)
              let rl1 = self.newid("_r");
              func.add_inst(Load(rl1,xtype.clone(),ldest.clone(),None));
              for i in Rc_dec(Register(rl1),xtype.clone(),self.newid("_r")) {
                func.add_inst(i);
              }
          
              // increase rvalue ref counter unless function call return
              match &rdest {
                Register(rc) if rc.to_str().starts_with("_call_") => {},
                _ => { // else increase reference counter
                 for i in Rc_inc(rdest.clone(),xtype.clone(),self.newid("_r")) {
                   func.add_inst(i);
                 }
                },
              }//match
            }// x is pointer to a closure

            let storeinst = Store(xtype,rdest.clone(),ldest,None);
            func.add_inst(storeinst);
            rdest
          },
          Index{ae,ai} => {    // ae[ai] = ..
            let edest = self.compile_expr(&**ae,func);
            let idest = self.compile_expr(&**ai,func);
            let r1 = self.newid("_r");
            let hashindex = (ae.1.0,ae.1.1+1);
            let etype = self.symbol_table.typeshash.get(&hashindex).unwrap();
            let lrtype::LRarray(atype,asize) = etype else {println!("HERE, etype is {:?}",&etype); return Novalue};
            func.add_inst(Call(None,Void_t,vec![],str32::from("check_index"),vec![(Basic("i32"),idest.clone()),(Basic("i32"),Iconst(*asize as i32)),(Basic("i32"),Iconst(ae.1.0 as i32))]));            
            func.add_inst(Arrayindex(r1,*asize,translate_type(&*atype),edest,idest));
            func.add_inst(Store(translate_type(&*atype),rdest.clone(),Register(r1),None));
            rdest
          },
          _ => Novalue,
        }//match
      },
      App(var("getint"),aargs) if aargs.len()==0 => {  //intrinsic function
        let r1 = self.newid("in");
        func.add_inst(Call(Some(r1),Basic("i32"),vec![],str32::from("lambda7c_cin"),vec![]));
        Register(r1)
      },
      App(var("free"),aargs) if aargs.len()==1 => {
        let atype= self.symbol_table.typeshash.get(&aargs[0].lncl()).unwrap().clone();
        if let lrtype::LRclosure(_,cn) = &atype  {
          let adest = self.compile_expr(&**aargs[0],func);
          let r1 = self.newid(cn.to_str());
          func.add_inst(Cast(r1,str8::from("bitcast"),Pointer(Box::new(Userstruct(cn.clone()))),adest,Pointer(Box::new(Basic("i8")))));
          func.add_inst(Call(None,Void_t,vec![],str32::from("free"),vec![(Pointer(Box::new(Basic("i8"))), Register(r1))]));
        }// is closure type, with name cn
        Novalue
      },
      App(var("weaken"),aargs) if aargs.len()==1 => {
        let atype= self.symbol_table.typeshash.get(&aargs[0].lncl()).unwrap().clone();
        if let lrtype::LRclosure(_,cn) = &atype  {
          let adest = self.compile_expr(&**aargs[0],func);
          let r1 = self.newid(cn.to_str());
          let latype = translate_type(&atype);
          for inst in Rc_dec(adest,latype,r1) {
            func.add_inst(inst);
          }
        }// is closure type, with name cn
        Novalue
      },      
      App(f@var(fname0),aargs) => {  // only allow named functions for now..
        //let mut fdest = self.compile_expr(&**f,func); // address of struct
        let fnentry = self.symbol_table.get_entry(fname0,0).unwrap();
        let fname = str32::from(&format!("{}_{}",fname0,fnentry.gindex));
        let lrtype::LRclosure(types00,cn)= &fnentry.typefor else {panic!("NO CLOSURE!");};
        let types0 = types00.clone();
        // cn is the name of the original function and closure
        let mut cnroot = cn.to_str();
        if let Some(pos)=cnroot.rfind('_') {cnroot = &cnroot[..pos];}
        let (sframei,structmap) = self.clsmaps.get(cn).expect("STILL NO");
        let cnentry = self.symbol_table.frames[*sframei].entries.get(cnroot).expect("NO ENTRY");
        
        let TypedLambda{return_type,formal_args,body}=&cnentry.ast_rep.unwrap()
            else {panic!("NO LAMBDA");};
        let fnframe = *self.symbol_table.frame_locate.get(&(body.line() as u32,body.column() as u32)).unwrap();

        let cname = cn.to_string();
        // detect recursive calls and treat as special case
        let fdest;
        if &func.name == &cname[..] {
            fdest = Register(str24::from("_self"));
        } else { fdest = self.compile_expr(&**f,func); } //address of struct

        
        // get frame information
        let mut fntypes = Vec::new();
        for t in &types0 {fntypes.push(translate_type(t));}
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
        // add _self argument (fdest)
        argsinfo.push((Pointer(Box::new(Userstruct(str32::from(&format!("{}",&cname))))), fdest));
        
        let r1 = self.newid("_call");
        let cdest = if let Void_t = &return_type {None}
            else {Some(r1)};
        let cinst = Call(cdest.clone(),return_type,vec![],str32::from(&cname),argsinfo);
        func.add_inst(cinst);
        if cdest.is_some() {Register(r1)} else {Novalue}
      }, // App case
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
      Index{ae,ai} => {    // ae[ai] = ..
        let edest = self.compile_expr(&**ae,func);
        let idest = self.compile_expr(&**ai,func);
        let r1 = self.newid("_r");
        let r2 = self.newid("_r");
        let hashindex = (ae.1.0,ae.1.1+1);
        let etype = self.symbol_table.typeshash.get(&hashindex).unwrap();

//println!("!!!!!etype: {:?}",&etype);
        let lrtype::LRarray(atype,asize) = etype else {return Novalue};
        func.add_inst(Call(None,Void_t,vec![],str32::from("check_index"),vec![(Basic("i32"),idest.clone()),(Basic("i32"),Iconst(*asize as i32)),(Basic("i32"),Iconst(ae.1.0 as i32))]));
        func.add_inst(Arrayindex(r1,*asize,translate_type(&*atype),edest,idest));
        func.add_inst(Load(r2,translate_type(&*atype),Register(r1),None));
        Register(r2)
      },
      _ => {
        //println!("expression {:?} returned Novalue",expr);
        Novalue  // default, includes case for Export
      },
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

  // compile a variable, ind flag indicates if index is known (x_5)
  // if load arg is false, then only the address is returned
  fn compile_var(&mut self, x:&str24, func:&mut LLVMFunction,ind:bool,load:bool)-> LLVMexpr
  {  use crate::bump7c_ast::Expr::*;
     use crate::llvmir::LLVMexpr::*;
     use crate::llvmir::LLVMtype::*;
     use crate::llvmir::Instruction::*;

     let mut xroot = x.to_str();
     if ind {
       if let Some(pos) = x.to_str().rfind('_') {xroot = &x.to_str()[..pos];}
     }
     let (fi,eopt) = self.symbol_table.get_entry_locate(xroot,0);
     let xentry = eopt.unwrap();
     let mut cfi = self.symbol_table.current;
     let mut isfree = fi != cfi && &func.name!="main";
     //make sure it's really free
     while isfree && cfi<usize::MAX && self.symbol_table.frames[cfi].name=="let" {
       if fi==cfi /*|| self.symbol_table.frames[cfi].name=="global"*/ {isfree=false;}
       cfi = self.symbol_table.frames[cfi].parent_scope;
     }
     let xind = xentry.gindex;
     let xvar = if !ind {str24::from(&format!("{}_{}",x,xind))}
       else {*x};
     let xtype = translate_type(&xentry.typefor);
     let r1 = self.newid(xvar.to_str());
     if isfree { // load from _self closure, must assume it exists
       let (sframei,structmap) = self.clsmaps.get(&func.name).expect("NO WAY");
       let (fieldi,_) = structmap.get(&xvar).expect("no entry!");
       func.add_inst(Structfield(r1,Userstruct(func.name),Register(str24::from("_self")),Iconst(*fieldi as i32)));
       if !load {return Register(r1);}
       let r2 = self.newid(xvar.to_str());
       func.add_inst(Load(r2,xtype,Register(r1),None));
       Register(r2)
     }// isfree
     else {
        if !load { return Register(xvar); }
        let inst = Load(r1,xtype,Register(xvar),None);
        func.add_inst(inst);
        Register(r1) // return value of compile_expr
     }// not free var
  }//compile_var

  fn compile_define(&mut self, x:&'t str, expr:&'t Expr<'t>, func:&mut LLVMFunction) -> LLVMexpr
  { use crate::bump7c_ast::Expr::*;
    use crate::llvmir::LLVMexpr::*;
    use crate::llvmir::LLVMtype::*;
    use crate::llvmir::Instruction::*;

//println!("DEFINING {} in frame {}",x,self.symbol_table.current);
//println!("{} in table:{}",x, self.symbol_table.get_entry(x,0).is_some());

    let xentry = self.symbol_table.get_entry(x,0).unwrap();
    let xvar = str24::from(format!("{}_{}",x,xentry.gindex));
    let xtype = translate_type(&xentry.typefor);
    
    if let TypedLambda{return_type,formal_args,body}=expr {
      let fdest = self.compile_fn(x,expr);
      // closure type created by compile_fn
      //form closure instance
      let structname = format!("{}",&xvar);
      let ssize = self.bsize(&Userstruct(xvar.resize()));
      
      // generate call malloc instruction to allocate on heap
      // register with reference counting hash
      let r3 = self.newid(xvar.to_str());
      let callinst = Call(Some(r3),Pointer(Box::new(Basic("i8"))),vec![],str32::from("rcmalloc"),vec![(Basic("i64"),LLVMexpr::Iconst(ssize))]);
      func.add_inst(callinst);      

//      let callinst = Call(Some(r3),Pointer(Box::new(Basic("i8"))),vec![],str32::from("malloc"),vec![(Basic("i64"),LLVMexpr::Iconst(ssize))]);
//      func.add_inst(Call(None,Basic("i8"),vec![],str32::from("reference_inc"),vec![(Pointer(Box::new(Basic("i8"))),Register(r3))]));


      let xvarstar = self.newid(xvar.to_str());
      func.add_inst(Cast(xvarstar,str8::from("bitcast"),Pointer(Box::new(Basic("i8"))),Register(r3),Pointer(Box::new(Userstruct(str32::from(&structname))))));
      // store copies of the closure variables into the struct
      // the structmap was created in compile_fn
      let (sframei,structmap) = self.clsmaps.get(&str32::from(xvar.to_str())).unwrap().clone();

      // prepare destructor function
      let mut destfunc = LLVMFunction {
        name: str32::from(&format!("destruct_{}",&xvar)),
        formal_args:vec![(Pointer(Box::new(Basic("i8"))),str24::from("_addr"))],
        return_type: Void_t,
        bblocks: Vec::new(),
        attributes: Vec::new(),
        bblocator: HashMap::new(),
      };
      destfunc.addBB_owned(BasicBlock::new(str24::from("funbegin"),vec![]));
      let r11 = self.newid("_r");
      destfunc.add_inst(Cast(r11,str8::from("bitcast"),Pointer(Box::new(Basic("i8"))),Register(str24::from("_addr")),Pointer(Box::new(Userstruct(xvar.resize())))));

      // store copies into closure, inc counter, write destructor
      for (fvar,(fvari,fvartype)) in structmap.iter() {
        let r4 = self.newid(fvar.to_str());
        func.add_inst(Structfield(r4,Userstruct(str32::from(&structname)),Register(xvarstar),Iconst(*fvari as i32))); ///// MUST CHANGE TYPE!
//println!("compiling define {}, var {}, current frame {}, structmap for {}",x,fvar,self.symbol_table.current,&xvar);  
        let fvardest = self.compile_var(fvar,func,true,true);
        func.add_inst(Store(fvartype.clone(),fvardest.clone(),Register(r4),None));

        // increase reference counter if needed:
        if is_closurestar(fvartype) {
          for inst in Rc_inc(fvardest.clone(),fvartype.clone(),self.newid("_r")){
            func.add_inst(inst)
          }

        // write destructor clause
        let r12a = self.newid("_r");         let r12 = self.newid("_r");
        let r13 = self.newid("_r");
        destfunc.add_inst(Structfield(r12a,Userstruct(xvar.resize()),Register(r11),Iconst(*fvari as i32))); // struct pointer within userstruct
        destfunc.add_inst(Load(r12,fvartype.clone(),Register(r12a),None));
        destfunc.add_inst(Cast(r13,str8::from("bitcast"),fvartype.clone(),Register(r12),Pointer(Box::new(Basic("i8")))));
        let destn = str32::from(&format!("destruct_{}",get_struct_name(fvartype).unwrap()));
        destfunc.add_inst(Call(None,Basic("i32"),vec![],str32::from("decrement_rc"),vec![(Pointer(Box::new(Basic("i8"))),Register(r13)),(Pointer(Box::new(Func_t(vec![Pointer(Box::new(Basic("i8"))),Void_t]))),Global(destn))]));

        }// increase rc counter if copied was pointer to closure.

      } // store copies into closure of xvar function (for)
      destfunc.add_inst(Ret_noval);
      self.program.functions.push(destfunc);
      
      func.add_inst(Instruction::Alloca(xvar,xtype.clone(),None));
      func.add_inst(Instruction::Store(xtype.clone(),Register(xvarstar),Register(xvar),None));
      return Register(xvar);
    }//lambda case
    
    if let Vector(vals) = expr {
      let vlen = vals.len();
      let mut vtype = Basic("i32");
      let r1 = self.newid("_r");
      if vlen>0 {vtype = translate_type(self.symbol_table.typeshash.get(&vals[0].lncl()).unwrap()); 
      }//find type
      let arraytype = Array_t(vlen,Box::new(vtype.clone()));
      func.add_inst(Alloca(r1,arraytype.clone(),None));
      for i in 0..vlen {
        let idest = self.compile_expr(&**vals[i],func);
        let r2 = self.newid("_r");
        func.add_inst(Arrayindex(r2,vlen,vtype.clone(),Register(r1),Iconst(i as i32)));
        func.add_inst(Store(vtype.clone(),idest,Register(r2),None));
      }//store initial values into array
      func.add_inst(Alloca(xvar,xtype.clone(),None)); // allocate **
      func.add_inst(Store(xtype.clone(),Register(r1),Register(xvar),None));
      return Register(xvar);
    }// vector case
    if let Vector_make{ve,vi:integer(vsize)} = expr {
      let r1 = self.newid("_r");
      let vetype = self.symbol_table.typeshash.get(&ve.lncl()).unwrap().clone();
      let arraytype= Array_t(*vsize as usize,Box::new(translate_type(&vetype)));
      let asize = self.bsize(&arraytype);
      func.add_inst(Alloca(r1,arraytype.clone(),None));
      let vedest = self.compile_expr(ve,func);
      let func_to_call;
      if let lrtype::Int_t = &vetype { func_to_call=str32::from("fillarray_int"); }
      else if let lrtype::Float_t=&vetype{func_to_call=str32::from("fillarray_double");}
      else {func_to_call=str32::from("fillarray_ptr"); }
      let lvtype = translate_type(&vetype);
      let r2 = self.newid("_r");
      func.add_inst(Arrayindex(r2,*vsize as usize,lvtype.clone(),Register(r1),Iconst(0)));
      // need bitcast ops if array of pointers
      func.add_inst(Call(None,Void_t,vec![],func_to_call,vec![(Pointer(Box::new(lvtype.clone())),Register(r2)), (lvtype.clone(),vedest), (Basic("i32"),Iconst(*vsize))]));
      func.add_inst(Alloca(xvar,xtype.clone(),None)); // allocate **
      func.add_inst(Store(xtype.clone(),Register(r1),Register(xvar),None));
      return Register(xvar);
    }//Vector_make case // can't have arrays of pointers right now
    
    // general case:

    let edest = self.compile_expr(expr,func);

    // see if reference counter needs increasing (define a = b...)
    if is_closurestar(&xtype) {
      match &edest {
        Register(rc) if rc.to_str().starts_with("_call_") => {}, //nothing
        _ => { // increase reference counter (no pointer arithmetic)
          for i in Rc_inc(edest.clone(),xtype.clone(),self.newid("_r")) {
            func.add_inst(i);
          }
        },
      }//match
    } // if x is closure pointer (but not define x = lambda...)

    func.add_inst(Instruction::Alloca(xvar,xtype.clone(),None));
    func.add_inst(Instruction::Store(xtype,edest.clone(),Register(xvar),None));
    //LLVMexpr::Register(xvar)
    edest
  }//compile_define subprocedure of compile_expr

  //compile a function (funn is function name, if one is given)
  fn compile_fn(&mut self,funn:&str,expr:&'t Expr<'t>) -> LLVMexpr 
  { use crate::bump7c_ast::Expr::*;
    use btypingso::lrtype;
    use btypingso::lrtype::*;    
    use crate::llvmir::LLVMexpr::*;
    use crate::llvmir::LLVMtype::*;
    use crate::llvmir::Instruction::*;
    use crate::llvmir::LLVMdeclaration::*;
    let oldindex = self.lindex; // for middle defs
    self.lindex=0;
    let namelessindex = self.newindex();
    let TypedLambda{return_type,formal_args,body} = expr else {return Novalue};
    let bp = self.symbol_table.current;
    let fi = *self.symbol_table.frame_locate.get(&(body.line() as u32,body.column() as u32)).unwrap();
    self.symbol_table.current = fi;
    let mut fnentry = self.symbol_table.frames[fi].entries.get(funn).unwrap();
    let fnind = fnentry.gindex;
    let mut fname = format!("{}_{}",funn,fnind);
    if funn.len()==0 {fname= format!("_nameless_lambda7c_{}",namelessindex);}

    let freevars = self.symbol_table.expr_freevars(expr);
    // these variables will be copied into closure

    // create closure struct type
    let mut struct_map = HashMap::new();
    let mut struct_fields = Vec::new();
    let mut cli = 0;
    for ((xvar,xind),xtype) in freevars.iter() {
      // insert info into Compiler's clsmap
      if &fname == &format!("{}_{}",xvar,xind) {continue;}
//println!("COMPILING fun {} in frame {}, inserting freevar {}_{}",&fname,self.symbol_table.current,xvar,xind);     
      let type2 = translate_type(xtype);
      struct_map.insert(str24::from(&format!("{}_{}",xvar,xind)),(cli,type2.clone()));
      struct_fields.push(type2);
      cli += 1;
    }
    self.clsmaps.insert(str32::from(&fname),(bp,struct_map));
    // create LLVM struct declaration
    let structname = str32::from(&format!("{}",&fname));
    let sdec = Structdec(structname,struct_fields);
    self.program.global_declarations.push(sdec);
    // create closure instance - at end?

    fnentry = self.symbol_table.frames[fi].entries.get(funn).unwrap(); //reborrow
    // gather information from symbol table for function signature.
    let lrtype::LRclosure(fntypes,cn) = &fnentry.typefor else {return Novalue};
    let lasttype = fntypes[fntypes.len()-1].clone();
    let mut rettype = translate_type(&lasttype); //return type
    // may have to change if return type is functional
    let mut fargs = Vec::new();
    let mut farginsts = Vec::new(); // alloc/store on each formal arg
    let mut i = 0;
    // the type info is in the symbol table but the arg name is in the AST***
    while i<fntypes.len()-1
    {
      let argn = (&**formal_args[i]).0; // src argument name (pure)
      let aindex= self.symbol_table.frames[fi].entries.get(argn).unwrap().gindex;
      let argname = str24::from(&format!("farg_{}_{}",argn,aindex));
      let argname0 = str24::from(&format!("{}_{}",argn,aindex));
      let argtype = translate_type(&fntypes[i]);
      // generate instruction at same time? YES
      fargs.push((argtype.clone(),argname));
      farginsts.push(Alloca(argname0,argtype.clone(),None));
      farginsts.push(Store(argtype.clone(),Register(argname),Register(argname0),None));

      // increment reference counter
      if is_closurestar(&argtype) {
        for inst in Rc_inc(Register(argname),argtype.clone(),str24::from(&format!("_rc_{i}",))) {
          farginsts.push(inst);
        }
      }

      i+=1;
    }// while through fntypes
    // last argument is self
    let closurearg = Pointer(Box::new(Userstruct(structname)));
//    fargs.push((closurearg,str24::from(&format!("{}_self",&fname)))); //self
    fargs.push((closurearg,str24::from("_self")));  // always _self
    
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

    let bdest = self.compile_expr(&**body,&mut newfunc); // function body

    // decrease reference counter for all local pointers to closures
    // will auto-free when counters reach zero,

     // do not increase counter of returned closure: may not be a local:
     // counter should, but if it's local, may get deallocated if not inc'ed
    if is_closurestar(&rettype) {
      for i in Rc_inc(bdest.clone(),rettype.clone(),self.newid("_r")) {
        newfunc.add_inst(i);
      }
    }// increase ref counter is closure is being returned

    // decrease ref counters of all local closure pointers (avoid recursives)
    let mut indi = self.lindex;
    for (evar,entry) in self.symbol_table.frames[fi].entries.iter() {
      if *evar==funn {continue;}  // doesn't apply to recursive def
      let letype = translate_type(&entry.typefor);
      if is_closurestar(&letype) {
        indi += 1;
        let re9 = str24::from(&format!("_r_{}",indi));
        newfunc.add_inst(Load(re9,letype.clone(),Register(str24::from(&format!("{}_{}",evar,entry.gindex))),None));
        indi+=1;
        let re9b = str24::from(&format!("_r_{}",indi));
        for i in Rc_dec(Register(re9),letype.clone(),re9b) {
          newfunc.add_inst(i);
        }
      }//is closure pointer
    }//for each entry - looking for closure pointers
    self.lindex = indi;

    newfunc.add_inst(Ret(rettype,bdest)); //terminal BB with ret
    
    // define will create the closure
    self.symbol_table.current = bp;
    self.program.functions.push(newfunc);
    self.lindex = oldindex;
    Global(str32::from(fname)) // change this to closure pointer? not here

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

  declare i8* @malloc(i64)
  declare void @free(i8*)
  declare void @check_index(i32,i32,i32)
  declare void @lambda7c_printint(i32)
  declare void @lambda7c_printfloat(double)
  declare void @lambda7c_printstr(i8*)
  declare i32 @lambda7c_cin()
  declare void @lambda7c_newline()
  declare void @fillarray_int(i32*, i32, i32)
  declare void @fillarray_double(double*, double, i32)
  declare void @fillarray_ptr(i8**, i8*, i32)
  declare i32 @reference_inc(i8*)
  ;declare i32 @reference_dec(i8*)
  declare i32 @decrement_rc(i8*, void (i8*)*)
  declare i8* @rcmalloc(i64)
  declare void @reference_map_init()

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

     // initial reference-counter global map
     mainfunc.add_inst(Instruction::Call(None,LLVMtype::Void_t,vec![],str32::from("reference_map_init"),vec![]));

     let mainres = self.compile_seq(&mainseq.0, &mut mainfunc);

     let ret = Instruction::Ret(LLVMtype::Basic("i32"),LLVMexpr::Iconst(0));
     mainfunc.add_inst(ret);
     self.program.functions.push(mainfunc);

     self.program.to_string()
     //println!("PROGRAM: {:?}",&self.program);  //debug
  }//compile_program


// calculate size of of a LLVM type
  pub fn bsize(&self, ty:&LLVMtype) -> i32
  { use crate::llvmir::LLVMtype::*;
    match ty {
      Basic("i8") => 1,
      Basic("i16") => 2,
      Basic("i32") | Basic("float") => 4,
      Basic("i64") | Basic("double") => 8,
      Pointer(_) => 8,
      Array_t(n,ty) => (*n as i32) * self.bsize(ty),
      Userstruct(s) => {
        let (sframei,fields) = self.clsmaps.get(s).unwrap();
        let mut sum = 0;
        for (_,(_,t)) in fields.iter() { sum += self.bsize(t); }
        sum
      },
      _ => 0, // default
    }//match    
  }//sizeof

}// impl LLVMCompiler

// typemap: lrtype to LLVMIR type
pub fn translate_type(t:&lrtype) -> LLVMtype {
  use crate::btypingso::lrtype::*;
  use crate::llvmir::LLVMtype::*;
  match t {
    Unit_t => Void_t,
    Int_t => Basic("i32"),
    Float_t => Basic("double"),
    String_t => Pointer(Box::new(Basic("i8"))),
    LRclosure(_,cn) => Pointer(Box::new(Userstruct(str32::from(&format!("{}",cn))))),
    //LRarray(ta,ts) => Array_t(*ts,Box::new(translate_type(ta))),    
    LRarray(ta,ts) => Pointer(Box::new(Array_t(*ts,Box::new(translate_type(ta))))),
    _ => Basic("INVALID TYPE"),
  }//match
}


//////////////////// special instructions ////////////////////

fn Rc_inc(e:LLVMexpr, t:LLVMtype, rnew:str24) -> Vec<Instruction> {
    use crate::bump7c_ast::Expr::*;
    use crate::llvmir::Instruction::*;
    use crate::llvmir::LLVMtype::*;
    use crate::llvmir::LLVMexpr::*;
    use crate::btypingso::lrtype;
    let mut vi = Vec::new();
    vi.push(Cast(rnew,str8::from("bitcast"),t,e.clone(),Pointer(Box::new(Basic("i8")))));
    vi.push(Call(None,Basic("i32"),vec![],str32::from("reference_inc"),vec![(Pointer(Box::new(Basic("i8"))),Register(rnew))]));
    vi
}//Rc_inc

fn Rc_dec(e:LLVMexpr, t:LLVMtype, rnew:str24) -> Vec<Instruction> {
    use crate::bump7c_ast::Expr::*;
    use crate::llvmir::Instruction::*;
    use crate::llvmir::LLVMtype::*;
    use crate::llvmir::LLVMexpr::*;
    use crate::btypingso::lrtype;
    let mut vi = Vec::new();
    vi.push(Cast(rnew,str8::from("bitcast"),t.clone(),e.clone(),Pointer(Box::new(Basic("i8")))));
    let dfname = str32::from(&format!("destruct_{}",get_struct_name(&t).unwrap())); // destructor function name
    vi.push(Call(None,Basic("i32"),vec![],str32::from("decrement_rc"),vec![(Pointer(Box::new(Basic("i8"))),Register(rnew)), (Pointer(Box::new(Func_t(vec![Pointer(Box::new(Basic("i8"))),Void_t]))),Global(dfname))]));
    //vi.push(Call(None,Basic("i32"),vec![],str32::from("reference_dec"),vec![(Pointer(Box::new(Basic("i8"))),Register(rnew))]));
    vi
}//Rc_inc

fn is_closurestar(t:&LLVMtype) -> bool {
  use crate::llvmir::LLVMtype::*;
  if let Pointer(p) = t {
    if let Userstruct(_) = &**p {true} else {false}
  } else {false}
}

fn get_struct_name(t:&LLVMtype) -> Option<str32> {
  use crate::llvmir::LLVMtype::*;
  if let Pointer(p) = t {
    if let Userstruct(x) = &**p {Some(*x)} else {None}
  } else {None}
}

fn deref_type(t:&LLVMtype) -> Option<LLVMtype> {
  use crate::llvmir::LLVMtype::*;
  if let Pointer(p) = t {Some((**p).clone())} else {None}
}
