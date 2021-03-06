// Copyright 2015 Adrien Champion. See the COPYRIGHT file at the top-level
// directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/*! Chain (result of splitting an equivalence class. */

use std::fmt ;

use term::{ Term, TermSet } ;
use common::errors::* ;

use Domain ;

/** A chain is an increasing-ordered list containing values and
representative / equivalence class pairs.

It is ordered on the values. */
#[derive(PartialEq, Eq, Clone)]
pub enum Chain< Val: Domain, Info: PartialEq + Eq + Clone > {
  /** Empty chain. */
  Nil,
  /** Chain constructor. */
  Cons(Val, Term, Info, Box< Chain<Val, Info> >),
}
impl<
  Val: Domain, Info: PartialEq + Eq + Clone
> fmt::Display for Chain<Val, Info> {
  fn fmt(& self, fmt: & mut fmt::Formatter) -> fmt::Result {
    use self::Chain::* ;
    let mut chain = self ;
    try!( write!(fmt, "[") ) ;
    loop {
      match * chain {
        Nil => break,
        Cons(ref val, ref trm, _, ref tail) => {
          chain = & ** tail ;
          try!( write!(fmt, " {}<{}>", trm, val) )
        },
      }
    }
    write!(fmt, "]")
  }
}
impl<Val: Domain, Info: PartialEq + Eq + Clone> Chain<Val, Info> {
  /** Empty chain. */
  #[inline]
  pub fn nil() -> Self { Chain::Nil }
  /** Chain constructor. */
  #[inline]
  pub fn cons(self, v: Val, t: Term, s: Info) -> Self {
    Chain::Cons(v, t, s, Box::new(self))
  }
  /// Returns a pointer to the last element in the chain.
  pub fn last(& self) -> Option<(& Val, & Term)> {
    use self::Chain::* ;
    let mut chain = self ;
    let mut res = None ;
    loop {
      match * chain {
        Cons(ref val, ref term, _, ref tail) => {
          res = Some( (val, term) ) ;
          chain = & ** tail
        },
        Nil => return res,
      }
    }
  }
  /// Returns a pointer to the first element in the chain.
  pub fn first(& self) -> Option<(& Val, & Term)> {
    use self::Chain::* ;
    match * self {
      Cons(ref val, ref term, _, _) => Some( (val, term) ),
      Nil => None,
    }
  }
  /** Checks if a chain is empty. */
  #[inline]
  pub fn is_empty(& self) -> bool { * self == Chain::Nil }
  /** Returns the top value of a chain, if any. */
  #[inline]
  pub fn top_value(& self) -> Option<(Val, Term)> {
    use self::Chain::* ;
    match * self {
      Cons(ref v, ref rep, _, _) => Some( (v.clone(), rep.clone()) ),
      Nil => None,
    }
  }

  /// Fold on a chain.
  pub fn fold<
    T, F: Fn(T, & Val, & Term, & Info) -> T
  >(& self, init: T, f: F) -> T {
    use self::Chain::* ;
    let mut chain = self ;
    let mut val = init ;
    while let Cons(ref v, ref trm, ref inf, ref tail) = * chain {
      val = f(val, v, trm, inf) ;
      chain = & * tail
    }
    val
  }

  /** Returns the longest subchain of a chain the values of which are
  all greater than or equal to some value, and the rest of the chain.

  First subchain is a vector of representatives and is sorted in **increasing
  order** on their value (which have been removed at this point).
  The second subchain is an actual `Chain` and is still sorted in **decreasing
  order**. */
  pub fn split_at(mut self, value: & Val) -> (Vec<Term>, Self) {
    use self::Chain::* ;
    let mut res = Vec::with_capacity(3) ;
    loop {
      if let Cons(val, rep, set, tail) = self {
        if value <= & val {
          res.push(rep) ;
          self = * tail
        } else {
          // We have `val < value`, stop here.
          self = Cons(val, rep, set, tail) ;
          break
        }
      } else {
        // Chain is empty, we done.
        break
      }
    }
    res.reverse() ;
    (res, self)
  }

  /** Reverses the first chain and appends it to the second one. */
  #[inline]
  pub fn rev_append(mut self, mut that: Self) -> Self {
    use self::Chain::* ;
    while let Cons(val, term, set, tail) = self {
      that = Cons( val, term, set, Box::new(that) ) ;
      self = * tail
    }
    that
  }
  /** Reverses a chain. */
  #[inline]
  pub fn rev(self) -> Self {
    self.rev_append(Chain::Nil)
  }
}
impl<Val: Domain> Chain<Val, TermSet> {
  /** Maps to `Chain<Val, ()>`, calling a function on each element. */
  pub fn map_to_unit<
    Input, F: Fn(& mut Input, Val, Term, TermSet)
  >(mut self, f: F, i: & mut Input) -> Chain<Val, ()> {
    use self::Chain::* ;
    let mut res = Nil ;
    while let Cons(val, rep, set, tail) = self {
      self = * tail ;
      f(i, val.clone(), rep.clone(), set) ;
      res = res.cons(val, rep, ())
    }
    res.rev()
  }

  /** Inserts a term in a chain given its value. */
  pub fn insert(mut self, v: Val, t: Term) -> Res<Self> {
    use self::Chain::* ;
    use std::cmp::Ordering::* ;
    let mut prefix = Nil ;
    loop {
      if let Cons(val, term, mut set, tail) = self {
        match val.cmp(& v) {
          Less => return Ok(
            // Insert term found as a new node in the chain.
            prefix.rev_append(
              Cons(val, term, set, tail).cons(v, t, TermSet::new())
            )
          ),
          Equal => {
            // Insert term in the set of this node.
            debug_assert!( ! set.contains(& t) ) ;
            let _ = set.insert(t) ;
            return Ok( prefix.rev_append( Cons(val, term, set, tail) ) )
          },
          Greater => {
            // Need to go deeper, iterating.
            prefix = prefix.cons(val, term, set) ;
            self = * tail
          },
        }
      } else {
        // Reached end of list, inserting.
        return Ok(
          prefix.rev_append( Nil.cons(v, t, TermSet::new()) )
        )
      }
    }
  }
}