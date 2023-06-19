#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(dead_code)]


pub struct Continuation<'t,T>( Box<dyn Fn(T) -> T + 't> );
impl<'t,T> Default for Continuation<'t,T> {
  fn default() -> Self {
    Continuation (Box::new(|x|x))
  }
}// impl Default

impl<'t,T> Continuation<'t,T> {
  pub fn make(f:impl Fn(T) -> T + 't) -> Self {
    Continuation( Box::new(f) )
  }

  pub fn apply(&self, n:T) -> T {
    (self.0)(n)
  }
}//impl Continuation

/*
pub struct Continuation<'t>( Box<dyn Fn(i32) -> i32 + 't> );
impl<'t> Default for Continuation<'t> {
  fn default() -> Self {
    Continuation (Box::new(|x|x))
  }
}// impl Default

impl<'t> Continuation<'t> {
  pub fn make(f:impl Fn(i32) -> i32 + 't) -> Self {
    Continuation( Box::new(f) )
  }

  pub fn apply(&self, n:i32) -> i32 {
    let Continuation(f) = self;
    f(n)
  }
}//impl Continuation
*/

/*
pub struct Continuation<'t>( dyn Fn(i32) -> i32 + 't );
impl<'t> Default for Continuation<'t> {
  fn default() -> Self {
    Continuation (|x|x)
  }
}// impl Default

impl<'t> Continuation<'t> {
  pub fn make(f: impl Fn(i32) -> i32 + 't) -> Self {
    Continuation( f )
  }

  pub fn apply(&self, n:i32) -> i32 {
    let Continuation(f) = self;
    f(n)
  }
}//impl Continuation
*/

/*
pub struct Continuation( Box<dyn Fn(i32) -> i32> );
impl Default for Continuation {
  fn default() -> Self {
    Continuation (Box::new(|x|x))
  }
}// impl Default

impl Continuation {
  pub fn make(f:impl Fn(i32) -> i32 + 'static) -> Self {
    Continuation( Box::new(f) )
  }

  pub fn apply(&self, n:i32) -> i32 {
    let Continuation(f) = &self;
    f(n)
  }
}//impl Continuation
*/

//fn main() {}
