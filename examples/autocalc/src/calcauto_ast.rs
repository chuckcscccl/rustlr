//Abstract syntax types generated by rustlr for grammar calcauto
    
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(dead_code)]
extern crate rustlr;
pub use rustlr::LC;
use rustlr::LBox;

#[derive(Debug)]
pub enum Expr<'lt> {
  Plus(LBox<Expr<'lt>>,LBox<Expr<'lt>>),
  Expr_16(Aexpr<'lt>),
  Binop(&'static str,LBox<Expr<'lt>>,LBox<Expr<'lt>>),
  Minus(LBox<Expr<'lt>>,LBox<Expr<'lt>>),
  Expr_1(Sxpr,LC<Sxpr>),
  Val(i64),
  Neg(LBox<Expr<'lt>>),
  Var(&'lt str),
  Let{let_var:&'lt str,init_value:LBox<Expr<'lt>>,let_body:LBox<Expr<'lt>>},
  Expr_Nothing,
}
impl<'lt> Default for Expr<'lt> { fn default()->Self { Expr::Expr_Nothing } }

#[derive(Debug)]
pub enum Xxpr<'lt> {
  X1{a:i64,b:LC<i64>},
  X2{_item0_:i64,u__item0_:&'lt str,u__item2_:&'lt str,_item3_:&'lt str},
  Xxpr_Nothing,
}
impl<'lt> Default for Xxpr<'lt> { fn default()->Self { Xxpr::Xxpr_Nothing } }

#[derive(Debug)]
pub enum CC {
  int_11(i64),
  CC_10,
  let_12,
  CC_Nothing,
}
impl Default for CC { fn default()->Self { CC::CC_Nothing } }

#[derive(Debug)]
pub enum ExprList<'lt> {
  nil,
  cons{car:LC<Expr<'lt>>,cdr:LBox<ExprList<'lt>>},
  ExprList_Nothing,
}
impl<'lt> Default for ExprList<'lt> { fn default()->Self { ExprList::ExprList_Nothing } }

#[derive(Debug)]
pub enum AA<'lt> {
  AA_7{p_a:&'lt str,p_b:LBox<AA<'lt>>,p_c:LBox<AA<'lt>>,q:i64,r_a:&'lt str,r_b:LBox<AA<'lt>>,r_c:LBox<AA<'lt>>},
  var_8(&'lt str,&'lt str,CC,&'lt str),
  AA_Nothing,
}
impl<'lt> Default for AA<'lt> { fn default()->Self { AA::AA_Nothing } }

#[derive(Debug)]
pub enum Aexpr<'lt> {
  ae(i64),
  be(&'lt str),
  Aexpr_Nothing,
}
impl<'lt> Default for Aexpr<'lt> { fn default()->Self { Aexpr::Aexpr_Nothing } }

#[derive(Default,Debug)]
pub struct Yxpr<'lt>(pub &'lt str,pub &'lt str,);

#[derive(Default,Debug)]
pub struct DD();

#[derive(Default,Debug)]
pub struct BB<'lt> {
  pub a:&'lt str,
  pub b:LBox<AA<'lt>>,
  pub c:LBox<AA<'lt>>,
}

#[derive(Default,Debug)]
pub struct Txpr {
  pub ai:i64,
  pub bi:i64,
}

#[derive(Default,Debug)]
pub struct Zxpr<'lt> {
  pub x:i64,
  pub a__item0_:&'lt str,
  pub a__item2_:&'lt str,
  pub b__item0_:&'lt str,
  pub b__item2_:&'lt str,
  pub y:i64,
  pub xx:LC<Xxpr<'lt>>,
  pub _item7_:Expr<'lt>,
  pub yy:Xxpr<'lt>,
}

#[derive(Default,Debug)]
pub struct Sxpr {
  pub a1:Txpr,
  pub b:Txpr,
}

