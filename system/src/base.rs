// Copyright 2015 Adrien Champion. See the COPYRIGHT file at the top-level
// directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt ;
use std::hash::{ Hash, Hasher } ;
use std::cmp::{ PartialEq, Eq } ;
use std::collections::HashSet ;

use term::{ Sym, Type, Term } ;

/** Set of callables. */
pub type CallSet = HashSet<::Callable> ;
/** Set of properties. */
pub type PropSet = HashSet<::Prop> ;

/** A signature, a list of types. Used only in `Uf`. */
#[derive(Debug,Clone)]
pub struct Sig {
  /** Types of the signature. */
  types: Vec<Type>,
}
impl Sig {
  /** Creates a new signature. */
  #[inline(always)]
  pub fn mk(types: Vec<Type>) -> Self {
    Sig { types: types }
  }
  /** The types of a signature. */
  #[inline(always)]
  pub fn types(& self) -> & [Type] { & self.types }
}
impl fmt::Display for Sig {
  fn fmt(& self, fmt: & mut fmt::Formatter) -> fmt::Result {
    let mut iter = self.types.iter() ;
    if let Some(ref t) = iter.next() {
      try!( write!(fmt, "{}", t) ) ;
      for t in iter {
        try!( write!(fmt, " {}", t) )
      }
    } ;
    Ok(())
  }
}

/** A list of typed formal parameters. */
#[derive(Debug,Clone)]
pub struct Args {
  /** The symbol/type pair list. */
  args: Vec<(Sym, Type)>,
}
impl Args {
  /** Creates a new argument list. */
  #[inline(always)]
  pub fn mk(args: Vec<(Sym, Type)>) -> Self {
    Args { args: args }
  }
  /** The formal parameters. */
  #[inline(always)]
  pub fn args(& self) -> & [(Sym, Type)] { & self.args }
  /** Number of paramaters. */
  #[inline(always)]
  pub fn len(& self) -> usize { self.args.len() }
}
impl fmt::Display for Args {
  fn fmt(& self, fmt: & mut fmt::Formatter) -> fmt::Result {
    let mut iter = self.args.iter() ;
    if let Some( & (ref s, ref t) ) = iter.next() {
      try!( write!(fmt, "({} {})", s, t) ) ;
      for & ( ref s, ref t) in iter {
        try!( write!(fmt, " ({} {})", s, t) )
      }
    } ;
    Ok(())
  }
}

// pub struct StatelessDep {
//   funs: HashSet<Sym>,
// }
// impl StatelessDep {
//   pub fn mk(funs: HashSet<Sym>) -> Self {
//     StatelessDep { funs: funs }
//   }
// }

// pub struct StatefulDep {
//   funs: HashSet<Sym>,
//   inits: HashSet<Sym>,
//   transs: HashSet<Sym>,
// }
// impl StatefulDep {
//   pub fn mk(
//     funs: HashSet<Sym>, inits: HashSet<Sym>, transs: HashSet<Sym>
//   ) -> Self {
//     StatefulDep { funs: funs, inits: inits, transs: transs }
//   }
// }

/** An uninterpreted function. */
#[derive(Debug,Clone)]
pub struct Uf {
  /** Identifier of the function. */
  sym: Sym,
  /** Signature of the function. */
  sig: Sig,
  /** Return type of the function. */
  typ: Type,
}
impl Uf {
  /** Creates a new uninterpreted function. */
  #[inline(always)]
  pub fn mk(sym: Sym, sig: Sig, typ: Type) -> Self {
    Uf { sym: sym, sig: sig, typ: typ }
  }
  /** Identifier of a function. */
  #[inline(always)]
  pub fn sym(& self) -> & Sym { & self.sym }
  /** Signature of a function. */
  #[inline(always)]
  pub fn sig(& self) -> & [Type] { & self.sig.types() }
  /** Return type of a function. */
  #[inline(always)]
  pub fn typ(& self) -> & Type { & self.typ }
}
impl fmt::Display for Uf {
  fn fmt(& self, fmt: & mut fmt::Formatter) -> fmt::Result {
    write!(fmt, "{} ({}) -> {}", self.sym, self.sig, self.typ)
  }
}
impl PartialEq for Uf {
  fn eq(& self, other: & Uf) -> bool {
    self.sym == other.sym
  }
}
impl Eq for Uf {}
impl Hash for Uf {
  fn hash<H: Hasher>(& self, state: & mut H) {
    self.sym.hash(state)
  }
}

/** A function (actually as a macro in SMT-LIB). */
#[derive(Debug,Clone)]
pub struct Fun {
  /** Identifier of the function. */
  sym: Sym,
  /** Formal arguments of the function. */
  args: Args,
  /** Return type of the function. */
  typ: Type,
  /** Body of the function. */
  body: Term,
  /** Callables used by this function **recursively**. */
  calls: CallSet,
}
impl Fun {
  /** Creates a new function. */
  #[inline(always)]
  pub fn mk(
    sym: Sym, args: Args, typ: Type, body: Term, calls: CallSet
  ) -> Self {
    Fun { sym: sym, args: args, typ: typ, body: body, calls: calls }
  }
  /** Identifier of a function. */
  #[inline(always)]
  pub fn sym(& self) -> & Sym { & self.sym }
  /** Formal arguments of a function. */
  #[inline(always)]
  pub fn args(& self) -> & [(Sym, Type)] { & self.args.args() }
  /** Return type of a function. */
  #[inline(always)]
  pub fn typ(& self) -> & Type { & self.typ }
  /** Body of a function. */
  #[inline(always)]
  pub fn body(& self) -> & Term { & self.body }
  /** Calls of a function. */
  #[inline(always)]
  pub fn calls(& self) -> & CallSet { & self.calls }
}
impl fmt::Display for Fun {
  fn fmt(& self, fmt: & mut fmt::Formatter) -> fmt::Result {
    write!(
      fmt, "{} ({}) -> {} {{ {} }}", self.sym, self.args, self.typ, self.body
    )
  }
}
impl PartialEq for Fun {
  fn eq(& self, other: & Fun) -> bool {
    self.sym == other.sym
  }
}
impl Eq for Fun {}
impl Hash for Fun {
  fn hash<H: Hasher>(& self, state: & mut H) {
    self.sym.hash(state)
  }
}

/** Wraps an (uninterpreted) function. */
#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub enum Callable {
  /** Wraps an uninterpreted function. */
  Dec(Uf),
  /** Wraps a function. */
  Def(Fun),
}
impl Callable {
  /** The symbol of a function. */
  pub fn sym(& self) -> & Sym {
    match * self {
      Callable::Def(ref f) => f.sym(),
      Callable::Dec(ref f) => f.sym(),
    }
  }
}
impl fmt::Display for Callable {
  fn fmt(& self, fmt: & mut fmt::Formatter) -> fmt::Result {
    match * self {
      Callable::Dec(ref f) => write!(fmt, "declaration : {}", f),
      Callable::Def(ref f) => write!(fmt, "definition  : {}", f),
    }
  }
}

/** A property. */
#[derive(Debug,Clone)]
pub struct Prop {
  /** Identifier of the property. */
  sym: Sym,
  /** System the property is over. */
  sys: ::Sys,
  /** Body of the property. */
  body: Term,
  /** Calls in the property. */
  calls: CallSet,
}
impl Prop {
  /** Creates a new property. */
  #[inline(always)]
  pub fn mk(sym: Sym, sys: ::Sys, body: Term, calls: CallSet) -> Self {
    Prop { sym: sym, sys: sys, body: body, calls: calls }
  }
  /** Identifier of a property. */
  #[inline(always)]
  pub fn sym(& self) -> & Sym { & self.sym }
  /** System a property ranges over. */
  #[inline(always)]
  pub fn sys(& self) -> & ::Sys { & self.sys }
  /** Body of a property. */
  #[inline(always)]
  pub fn body(& self) -> & Term { & self.body }
  /** Calls of a property. */
  #[inline(always)]
  pub fn calls(& self) -> & CallSet { & self.calls }
}
impl fmt::Display for Prop {
  fn fmt(& self, fmt: & mut fmt::Formatter) -> fmt::Result {
    write!(
      fmt, "{} ({}) {{ {} }}", self.sym, self.sys.sym(), self.body
    )
  }
}
impl PartialEq for Prop {
  fn eq(& self, other: & Prop) -> bool {
    self.sym == other.sym
  }
}
impl Eq for Prop {}
impl Hash for Prop {
  fn hash<H: Hasher>(& self, state: & mut H) {
    self.sym.hash(state)
  }
}

/** A transition system. */
#[derive(Debug,Clone)]
pub struct Sys {
  /** Identifier of the system. */
  sym: Sym,
  /** State of the system. */
  state: Args,
  /** Local variables of the system. */
  locals: Vec<(Sym, Type, Term)>,
  /** Identifier of the init predicate of the system. */
  init: Term,
  /** Identifier of the transition relation of the system. */
  trans: Term,
  /** Calls of the system. */
  subsys: Vec<(::Sys, Vec<Term>)>,
  /** Callables used by this system **recursively**. */
  calls: CallSet,
}
impl Sys {
  /** Creates a new system. */
  #[inline(always)]
  pub fn mk(
    sym: Sym, state: Args, locals: Vec<(Sym, Type, Term)>,
    init: Term, trans: Term,
    subsys: Vec<(::Sys, Vec<Term>)>,
    calls: CallSet,
  ) -> Self {
    Sys {
      sym: sym, state: state, locals: locals,
      init: init, trans: trans,
      subsys: subsys, calls: calls,
    }
  }
  /** Identifier of a system. */
  #[inline(always)]
  pub fn sym(& self) -> & Sym { & self.sym }
  /** State of a system. */
  #[inline(always)]
  pub fn state(& self) -> & Args { & self.state }
  /** Locals variables of a system. */
  #[inline(always)]
  pub fn locals(& self) -> & [ (Sym, Type, Term) ] { & self.locals }
  /** Init predicate of a system. */
  #[inline(always)]
  pub fn init(& self) -> & Term { & self.init }
  /** Transition relation of a system. */
  #[inline(always)]
  pub fn trans(& self) -> & Term { & self.trans }
  /** Sub-systems of a system. */
  #[inline(always)]
  pub fn subsys(& self) -> & [ (::Sys, Vec<Term>) ] { & self.subsys }
  /** Calls of a system. */
  #[inline(always)]
  pub fn calls(& self) -> & CallSet { & self.calls }

  /** String representation of a system as lines. */
  pub fn lines(& self) -> String {
    let mut s = format!(
      "{} ({})\n  init:  {}\n  trans: {}",
      self.sym, self.state, self.init, self.trans
    ) ;
    if ! self.subsys.is_empty() {
      s = format!("{}\n  sub-systems:", s) ;
      for & (ref sub_sym, ref params) in self.subsys.iter() {
        s = format!("{}\n    {} (", s, sub_sym.sym()) ;
        for param in params {
          s = format!("{}\n      {}", s, param) ;
        } ;
        s = format!("{}\n    )", s) ;
      } ;
    } ;
    if ! self.calls.is_empty() {
      s = format!("{}\n  calls:", s) ;
      for callable in self.calls.iter() {
        s = format!("{}\n    {}", s, callable.sym()) ;
      } ;
    } ;
    s
  }
}
impl fmt::Display for Sys {
  fn fmt(& self, fmt: & mut fmt::Formatter) -> fmt::Result {
    write!(
      fmt, "{} ({}) {{ {} -> {} }}",
      self.sym, self.state, self.init, self.trans
    )
  }
}


