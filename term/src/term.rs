// Copyright 2015 Adrien Champion. See the COPYRIGHT file at the top-level
// directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Terms.

use std::io ;
use std::fmt ;

use errors::* ;

use base::{
  StateWritable, Writable, SVarWriter, PrintSmt2, PrintVmt, SymWritable,
  Offset2, HConsed, HConsign, HConser, State, SymPrintStyle
} ;
use typ::Type ;
use sym::Sym ;
use cst::Cst ;
use var::{ Var, VarMaker } ;
use self::RealTerm::* ;

/// Standard operators.
#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub enum Operator {
  /// Equality.
  Eq,
  /// If then else operator.
  Ite,
  /// Negation operator.
  Not,
  /// Conjunction operator.
  And,
  /// Disjunction operator.
  Or,
  /// Implication operator.
  Impl,
  /// Exclusive disjunction operator.
  Xor,
  /// Distinct operator.
  Distinct,
  /// Plus operator.
  Add,
  /// Minus operator.
  Sub,
  /// Multiplication operator.
  Mul,
  /// Division operator.
  Div,
  /// Less or equal operator.
  Le,
  /// Greater or equal operator.
  Ge,
  /// Less than operator.
  Lt,
  /// Greater than operator.
  Gt,
}

impl Operator {

  /// The arity of the operator. `None` for n-ary operators.
  pub fn arity(& self) -> Option<u8> {
    use self::Operator::* ;
    match * self {
      // Unary.
      Not => Some(1u8),
      // Binary.
      Div | Le | Ge | Lt | Gt => Some(2),
      // Ternary.
      Ite => Some(3),
      // N-ary.
      Eq | And | Or | Impl | Xor |
      Distinct |
      Add | Sub | Mul => None,
    }
  }

  /// Returns its return type if its arguments type check.
  pub fn type_check(& self, sig: & [Type]) -> Result<
    Type, (Option<Vec<usize>>, String)
  > {
    use Operator::* ;
    match * self {

      Eq => {
        let mut sig = sig.iter() ;
        if let Some(first) = sig.next() {
          let mut cpt = 0 ;
          for typ in sig {
            if typ != first {
              return Err( (
                Some( vec![cpt] ),
                format!(
                  "parameter {} of equality: \
                    first parameter(s) have type {}, got {}",
                  cpt + 1, first, typ
                )
              ) )
            } ;
            cpt = cpt + 1 ;
          } ;
          Ok(Type::Bool)
        } else {
          Ok(Type::Bool)
        }
      },

      Ite => {
        if sig.len() != 3 {
          return Err( (
            None,
            format!("operator ite expects 3 arguments, got {}", sig.len())
          ) )
        } ;
        if sig[0] != Type::Bool {
          return Err( (
            Some(vec![0]),
            format!(
              "first argument of ite should have sort Bool, got {}", sig[0]
            )
          ) )
        } ;
        if sig[1] != sig[2] {
          return Err( (
            Some(vec![1, 2]),
            format!(
              "second and third argument of ite are incompatible: {} and {}",
              sig[1], sig[2]
            )
          ) )
        } ;
        Ok( sig[1].clone() )
      },

      Not => {
        if sig.len() != 1 {
          return Err( (
            None,
            format!("operator not expects 1 argument, got {}", sig.len())
          ) )
        } else {
          if sig[0] != Type::Bool {
            return Err( (
              Some(vec![0]),
              format!(
                "first argument of not should have sort Bool, got {} ", sig[0]
              )
            ) )
          } ;
          Ok( Type::Bool )
        }
      },

      And | Or | Impl | Xor => {
        let mut cpt = 0 ;
        for typ in sig.iter() {
          if * typ != Type::Bool {
            return Err( (
              Some( vec![cpt] ),
              format!(
                "parameter {} of operator `{}`: \
                  expected Bool, got {}",
                cpt + 1, self, typ
              )
            ) )
          } ;
          cpt = cpt + 1 ;
        } ;
        Ok( Type::Bool )
      },

      Distinct => {
        let mut sig = sig.iter() ;
        if let Some(first) = sig.next() {
          let mut cpt = 1 ;
          for typ in sig {
            if typ != first {
              return Err( (
                Some( vec![cpt] ),
                format!(
                  "argument {} of operator distinct: \
                    first argument(s) have type {}, got {}",
                  cpt + 1, first, typ
                )
              ) )
            } ;
            cpt = cpt + 1 ;
          } ;
          Ok(Type::Bool)
        } else {
          Ok(Type::Bool)
        }
      },

      Add | Sub | Mul | Div => {
        let mut sig = sig.iter() ;
        if let Some(first) = sig.next() {
          match * first {
            Type::Int | Type::Rat => (),
            _ => return Err( (
              Some( vec![0] ),
              format!(
                "first argument of operator {}: \
                  expected Int or Real but got {}",
                self, first
              )
            ) ),
          } ;
          let mut cpt = 1 ;
          for typ in sig {
            if typ != first {
              return Err( (
                Some( vec![cpt] ),
                format!(
                  "argument {} as incompatible type \
                    expected {}, got {}",
                  cpt + 1, first, typ
                )
              ) )
            } ;
            cpt = cpt + 1 ;
          } ;
          Ok(first.clone())
        } else {
          return Err( (
            None,
            format!("operator {} is applied to nothing", self)
          ) )
        }
      },

      Le | Ge | Lt | Gt => {
        let mut sig = sig.iter() ;
        if let Some(first) = sig.next() {
          match * first {
            Type::Int | Type::Rat => (),
            _ => return Err( (
              Some( vec![0] ),
              format!(
                "first argument of operator {}: \
                  expected Int or Real but got {}",
                self, first
              )
            ) ),
          } ;
          let mut cpt = 1 ;
          for typ in sig {
            if typ != first {
              return Err( (
                Some( vec![cpt] ),
                format!(
                  "argument {} as incompatible type \
                    expected {}, got {}",
                  cpt + 1, first, typ
                )
              ) )
            } ;
            cpt = cpt + 1 ;
          } ;
          Ok(Type::Bool)
        } else {
          return Err( (
            None,
            format!("operator {} is applied to nothing", self)
          ) )
        }
      },

    }
  }

  /// Evaluates itself given some arguments.
  pub fn eval(
    & self, factory: & ::Factory, mut args: Vec<Cst>
  ) -> Res<Cst> {
    use Operator::* ;
    use ::real_term::Cst as RCst ;
    match * self {

      Eq => {
        let mut args = args.iter() ;
        if let Some(first) = args.next() {
          for arg in args {
            if arg != first { return Ok( factory.cst(false) ) }
          }
        } ;
        Ok( factory.cst(true) )
      },

      Ite => if args.len() != 3 {
        Err(
          ErrorKind::OpArityError(Ite, args.len(), "3").into()
        )
      } else {
        args.reverse() ;
        match * args.pop().unwrap().get() {
          RCst::Bool(true) => Ok( args.pop().unwrap() ),
          RCst::Bool(false) => {
            args.pop() ;
            Ok( args.pop().unwrap() )
          },
          ref arg => Err(
            ErrorKind::OpTypeError(
              Ite, arg.typ(), Type::Bool, Some("for first argument".into())
            ).into()
          )
        }
      },

      Not => if args.len() != 1 {
        Err(
          ErrorKind::OpArityError(Not, args.len(), "1").into()
        )
      } else {
        match * args[0].get() {
          RCst::Bool(b) => Ok( factory.cst(! b) ),
          ref arg => Err(
            ErrorKind::OpTypeError(
              Not, arg.typ(), Type::Bool, None
            ).into()
          )
        }
      },

      And => {
        let mut cpt = 0 ;
        let mut res = true ;
        for arg in args.iter() {
          match * arg.get() {
            RCst::Bool(b) => res = res && b,
            ref arg => return Err(
              ErrorKind::OpTypeError(
                And, arg.typ(), Type::Bool, Some(
                  format!("(found {} for argument {})", arg, cpt + 1)
                )
              ).into()
            )
          } ;
          cpt = cpt + 1 ;
        } ;
        Ok( factory.cst(res) )
      },

      Or => {
        let mut cpt = 0 ;
        let mut res = true ;
        for arg in args.iter() {
          match * arg.get() {
            RCst::Bool(b) => res = res || b,
            ref arg => return Err(
              ErrorKind::OpTypeError(
                Or, arg.typ(), Type::Bool, Some(
                  format!("(found {} for argument {})", arg, cpt + 1)
                )
              ).into()
            )
          } ;
          cpt = cpt + 1 ;
        } ;
        Ok( factory.cst(res) )
      },

      Impl => {
        let mut cpt = 0 ;
        let mut so_far = false ;
        for arg in args.iter() {
          match * arg.get() {
            RCst::Bool(b) => if so_far {
              if ! b {
                return Ok( factory.cst(false) )
              }
            } else {
              if b { so_far = true }
            },
            ref arg => return Err(
              ErrorKind::OpTypeError(
                Impl, arg.typ(), Type::Bool, Some(
                  format!("(found `{}` for argument {})", arg, cpt + 1)
                )
              ).into()
            )
          } ;
          cpt = cpt + 1 ;
        } ;
        Ok( factory.cst(true) )
      },

      Xor => {
        let mut cpt = 0 ;
        let mut trues = 0 ;
        for arg in args.iter() {
          match * arg.get() {
            RCst::Bool(b) => if b { trues = trues + 1 },
            ref arg => return Err(
              ErrorKind::OpTypeError(
                Xor, arg.typ(), Type::Bool, Some(
                  format!("(found `{}` for argument {})", arg, cpt + 1)
                )
              ).into()
            )
          } ;
          cpt = cpt + 1 ;
        } ;
        Ok( factory.cst(trues == 1) )
      },

      Distinct => {
        Eq.eval(factory, args).and_then(
          |cst| match * cst.get() {
            RCst::Bool(b) => Ok(factory.cst(! b)),
            _ => Err(
              "evaluation of equality returned a non-boolean value".into()
            ),
          }
        ).chain_err(
          || format!("in evaluation of Distinct as `(not (= ...))`")
        )
      },

      Add => {
        let mut args = args.into_iter() ;
        if let Some(arg) = args.next() {
          let mut res = arg.get().clone() ;
          for arg in args {
            match res.add(& arg) {
              Ok(cst) => res = cst,
              Err(cst) => return Err(
                ErrorKind::OpTypeError(
                  Add, res.typ(), cst.typ(), None
                ).into()
              ),
            }
          } ;
          Ok( factory.mk_rcst(res) )
        } else {
          Err( ErrorKind::OpArityError(Add, 0, "> 0").into() )
        }
      },

      Sub => {
        let mut args = args.into_iter() ;
        if let Some(arg) = args.next() {
          let mut res = arg.get().clone() ;
          let mut unary = true ;
          for arg in args {
            unary = false ;
            match res.sub(& arg) {
              Ok(cst) => res = cst,
              Err(cst) => return Err(
                ErrorKind::OpTypeError(
                  Sub, res.typ(), cst.typ(), None
                ).into()
              ),
            }
          } ;
          if unary {
            res = res.neg().unwrap()
          }
          Ok(
            factory.mk_rcst(res)
          )
        } else {
          Err( ErrorKind::OpArityError(Sub, 0, "> 0").into() )
        }
      },

      Mul => {
        let mut args = args.into_iter() ;
        if let Some(arg) = args.next() {
          let mut res = arg.get().clone() ;
          for arg in args {
            match res.mul(& arg) {
              Ok(cst) => res = cst,
              Err(cst) => return Err(
                ErrorKind::OpTypeError(
                  Mul, res.typ(), cst.typ(), None
                ).into()
              ),
            }
          } ;
          Ok( factory.mk_rcst(res) )
        } else {
          Err( ErrorKind::OpArityError(Mul, 0, "> 0").into() )
        }
      },

      Div => {
        let mut args = args.into_iter() ;
        if let Some(arg) = args.next() {
          let mut res = arg.get().clone() ;
          for arg in args {
            match res.div(& arg) {
              Ok(cst) => res = cst,
              Err(cst) => return Err(
                ErrorKind::OpTypeError(
                  Sub, res.typ(), cst.typ(), None
                ).into()
              ),
            }
          } ;
          Ok( factory.mk_rcst(res) )
        } else {
          Err( ErrorKind::OpArityError(Mul, 0, "> 0").into() )
        }
      },

      Le => if args.len() == 2 {
        match * args[0].get() {
          RCst::Int(ref lhs) => match * args[1].get() {
            RCst::Int(ref rhs) => Ok( factory.cst( lhs <= rhs) ),
            ref rhs => Err(
              ErrorKind::OpTypeError(
                Le, Type::Int, args[1].typ(), Some(
                  format!("(found `{}`)", rhs)
                )
              ).into()
            ),
          },
          RCst::Rat(ref lhs) => match * args[1].get() {
            RCst::Rat(ref rhs) => Ok( factory.cst( lhs <= rhs) ),
            ref rhs => Err(
              ErrorKind::OpTypeError(
                Le, Type::Rat, args[1].typ(), Some(
                  format!("(found `{}`)", rhs)
                )
              ).into()
            ),
          },
          ref lhs => Err(
            ErrorKind::OpTypeError(
              Le, lhs.typ(), Type::Int, Some(
                format!("or {} (found `{}`)", Type::Rat, lhs)
              )
            ).into()
          ),
        }
      } else {
          Err( ErrorKind::OpArityError(Le, args.len(), "2").into() )
      },

      Ge => if args.len() == 2 {
        match * args[0].get() {
          RCst::Int(ref lhs) => match * args[1].get() {
            RCst::Int(ref rhs) => Ok( factory.cst( lhs >= rhs) ),
            ref rhs => Err(
              ErrorKind::OpTypeError(
                Ge, Type::Int, args[1].typ(), Some(
                  format!("(found `{}`)", rhs)
                )
              ).into()
            ),
          },
          RCst::Rat(ref lhs) => match * args[1].get() {
            RCst::Rat(ref rhs) => Ok( factory.cst( lhs >= rhs) ),
            ref rhs => Err(
              ErrorKind::OpTypeError(
                Ge, Type::Rat, args[1].typ(), Some(
                  format!("(found `{}`)", rhs)
                )
              ).into()
            ),
          },
          ref lhs => Err(
            ErrorKind::OpTypeError(
              Ge, lhs.typ(), Type::Int, Some(
                format!("or {} (found `{}`)", Type::Rat, lhs)
              )
            ).into()
          ),
        }
      } else {
          Err( ErrorKind::OpArityError(Ge, args.len(), "2").into() )
      },

      Lt => if args.len() == 2 {
        match * args[0].get() {
          RCst::Int(ref lhs) => match * args[1].get() {
            RCst::Int(ref rhs) => Ok( factory.cst( lhs < rhs) ),
            ref rhs => Err(
              ErrorKind::OpTypeError(
                Lt, Type::Int, args[1].typ(), Some(
                  format!("(found `{}`)", rhs)
                )
              ).into()
            ),
          },
          RCst::Rat(ref lhs) => match * args[1].get() {
            RCst::Rat(ref rhs) => Ok( factory.cst( lhs < rhs) ),
            ref rhs => Err(
              ErrorKind::OpTypeError(
                Lt, Type::Rat, args[1].typ(), Some(
                  format!("(found `{}`)", rhs)
                )
              ).into()
            ),
          },
          ref lhs => Err(
            ErrorKind::OpTypeError(
              Lt, lhs.typ(), Type::Int, Some(
                format!("or {} (found `{}`)", Type::Rat, lhs)
              )
            ).into()
          ),
        }
      } else {
          Err( ErrorKind::OpArityError(Lt, args.len(), "2").into() )
      },

      Gt => if args.len() == 2 {
        match * args[0].get() {
          RCst::Int(ref lhs) => match * args[1].get() {
            RCst::Int(ref rhs) => Ok( factory.cst( lhs > rhs) ),
            ref rhs => Err(
              ErrorKind::OpTypeError(
                Gt, Type::Int, args[1].typ(), Some(
                  format!("(found `{}`)", rhs)
                )
              ).into()
            ),
          },
          RCst::Rat(ref lhs) => match * args[1].get() {
            RCst::Rat(ref rhs) => Ok( factory.cst( lhs > rhs) ),
            ref rhs => Err(
              ErrorKind::OpTypeError(
                Gt, Type::Rat, args[1].typ(), Some(
                  format!("(found `{}`)", rhs)
                )
              ).into()
            ),
          },
          ref lhs => Err(
            ErrorKind::OpTypeError(
              Gt, lhs.typ(), Type::Int, Some(
                format!("or {} (found `{}`)", Type::Rat, lhs)
              )
            ).into()
          ),
        }
      } else {
          Err( ErrorKind::OpArityError(Gt, args.len(), "2").into() )
      },

    }
  }
}

impl fmt::Display for Operator {
  fn fmt(& self, fmt: & mut fmt::Formatter) -> fmt::Result {
    use std::str::from_utf8 ;
    let mut s: Vec<u8> = vec![] ;
    self.write(& mut s).unwrap() ;
    write!(fmt, "{}", from_utf8(& s).unwrap())
  }
}

impl Writable for Operator {
  fn write(
    & self, writer: & mut io::Write
  ) -> io::Result<()> {
    write!(
      writer,
      "{}",
      match * self {
        Operator::Eq => "=",
        Operator::Ite => "ite",
        Operator::Not => "not",
        Operator::And => "and",
        Operator::Or => "or",
        Operator::Impl => "=>",
        Operator::Xor => "xor",
        Operator::Distinct => "distinct",
        Operator::Add => "+",
        Operator::Sub => "-",
        Operator::Mul => "*",
        Operator::Div => "/",
        Operator::Le => "<=",
        Operator::Ge => ">=",
        Operator::Lt => "<",
        Operator::Gt => ">",
      }
    )
  }
}

/// Underlying representation of terms.
#[derive(
  Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone
)]
pub enum RealTerm {
  /// A variable.
  V(Var),
  /// A constant value.
  C(Cst),
  /// An application of an operator.
  Op(Operator, Vec<Term>),
  /// A universal quantification.
  Forall(Vec<(Sym, Type)>, Term),
  /// An existential quantification.
  Exists(Vec<(Sym, Type)>, Term),
  /// A let-binding.
  Let(Vec<(Sym, Term)>, Term),
  /// An application of a function symbol.
  App(Sym, Vec<Term>),
}
impl RealTerm {
  /// Returns true iff the term is the constant `true`.
  #[inline]
  pub fn is_true(& self) -> bool {
    if let RealTerm::C(ref cst) = * self {
      if let ::real_term::Cst::Bool(b) = ** cst { b } else { false }
    } else {
      false
    }
  }
  /// Returns true iff the term is the constant `true`.
  #[inline]
  pub fn is_false(& self) -> bool {
    if let RealTerm::C(ref cst) = * self {
      if let ::real_term::Cst::Bool(b) = ** cst { ! b } else { false }
    } else {
      false
    }
  }
}

impl fmt::Display for RealTerm {
  fn fmt(& self, fmt: & mut fmt::Formatter) -> fmt::Result {
    match * self {
      V(ref v) => write!(fmt, "{}", v),
      C(ref c) => write!(fmt, "{}", c),
      Op(ref op, ref terms) => {
        try!( write!(fmt, "({}", op) ) ;
        for t in terms.iter() {
          try!( write!(fmt, " {}", t) )
        } ;
        write!(fmt, ")")
      },
      Forall(ref bindings, ref term) => {
        try!( write!(fmt, "(forall (") ) ;
        for & (ref sym, ref typ) in bindings.iter() {
          try!( write!(fmt, " ({} {})", sym, typ) )
        } ;
        try!( write!(fmt, " ) ") ) ;
        try!( write!(fmt, "{}", term) ) ;
        write!(fmt, ")")
      },
      Exists(ref bindings, ref term) => {
        try!( write!(fmt, "(exists (") ) ;
        for & (ref sym, ref typ) in bindings.iter() {
          try!( write!(fmt, " ({} {})", sym, typ) )
        } ;
        try!( write!(fmt, " ) ") ) ;
        try!( write!(fmt, "{}", term) ) ;
        write!(fmt, ")")
      },
      Let(ref bindings, ref term) => {
        try!( write!(fmt, "(let (") ) ;
        for & (ref sym, ref term) in bindings.iter() {
          try!( write!(fmt, " ({} {})", sym, term) )
        } ;
        try!( write!(fmt, " ) ") ) ;
        try!( write!(fmt, "{}", term) ) ;
        write!(fmt, ")")
      },
      App(ref sym, ref args) => {
        try!( write!(fmt, "({}", sym) ) ;
        for term in args.iter() {
          try!( write!(fmt, " {}", term) )
        } ;
        write!(fmt, ")")
      },
    }
  }
}

/// Hash consed term.
pub type Term = HConsed<RealTerm> ;

/// A stateful term. Either one-state or two-state.
#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub enum STerm {
  /// A one-state term. Stores the state (init) and next (trans) version. That
  /// is, the second element is the bump of the first.
  One(Term,Term),
  /// A two-state term. Stores the next (trans) version. Understood as true in
  /// the initial state.
  Two(Term),
}

impl STerm {
  /// The state version of a term.
  #[inline(always)]
  pub fn state(& self) -> Option<& Term> {
    match * self {
      STerm::One(ref t, _) => Some(t),
      STerm::Two(_) => None,
    }
  }
  /// The next version of a term.
  #[inline(always)]
  pub fn next(& self) -> & Term {
    match * self {
      STerm::One(_, ref t) => t,
      STerm::Two(ref t) => t,
    }
  }
}

impl fmt::Display for STerm {
  fn fmt(& self, fmt: & mut fmt::Formatter) -> fmt::Result {
    match * self {
      STerm::One(ref t,_) => write!(fmt, "{}", t),
      STerm::Two(ref t) => write!(fmt, "{}", t),
    }
  }
}

/// Hash cons table for terms.
pub type TermConsign = HConsign<RealTerm> ;

impl<Svw: SVarWriter<Sym>> StateWritable<Sym, Svw> for Term {
  fn write(
    & self, writer: & mut io::Write, sv_writer: & Svw, style: SymPrintStyle
  ) -> io::Result<()> {
    let mut stack = vec![ (true, vec![ self.clone() ]) ] ;
    loop {
      if let Some( (is_first, mut to_do) ) = stack.pop() {

        if let Some( term ) = to_do.pop() {
          stack.push( (false, to_do) ) ;
          if ! is_first { try!( write!(writer, " ") ) } ;
          match term.get() {
            & V(ref var) => {
              try!( var.write(writer, sv_writer, style) )
            },
            & C(ref cst) => {
              try!( cst.write(writer) )
            },
            & App(ref sym, ref args) => {
              try!( write!(writer, "(|") ) ;
              try!( sym.write(writer, style) ) ;
              try!( write!(writer, "| ") ) ;
              let mut args = args.clone() ;
              args.reverse() ;
              stack.push( (true, args) )
            },
            & Op(ref op, ref args) => {
              try!( write!(writer, "(") ) ;
              try!( op.write(writer) ) ;
              try!( write!(writer, " ") ) ;
              let mut args = args.clone() ;
              args.reverse() ;
              stack.push( (true, args) )
            },
            & Let(ref binding, ref term) => {
              try!( write!(writer, "(let ( ") ) ;
              for & (ref sym, ref term) in binding {
                try!( write!(writer, "(|") ) ;
                try!( sym.write(writer, style) ) ;
                try!( write!(writer, "| ") ) ;
                try!( term.write(writer, sv_writer, style) ) ;
                try!( write!(writer, ") ") ) ;
              } ;
              try!( write!(writer, ") ") ) ;
              stack.push( (true, vec![term.clone()]) )
            }
            _ => unimpl!(),
          } ;
        } else {
          // Don't close paren for the last element of the stack.
          if ! stack.is_empty() {
            try!( write!(writer, ")") )
          }
        }

      } else {
        break
      }
    } ;
    Ok(())
  }
}

impl PrintVmt for Term {
  fn to_vmt(
    & self, writer: & mut io::Write
  ) -> io::Result<()> {
    self.write(writer, & (), SymPrintStyle::External)
  }
}

impl PrintSmt2 for Term {
  fn to_smt2(
    & self, writer: & mut io::Write, offset: & Offset2
  ) -> io::Result<()> {
    self.write(writer, offset, SymPrintStyle::Internal)
  }
}

/// Can create variables.
pub trait VariableMaker {
  /// Creates a variable.
  #[inline]
  fn var(& self, Var) -> Term ;
}
impl VariableMaker for TermConsign {
  fn var(& self, var: Var) -> Term {
    self.mk( V(var) )
  }
}

/// Can create a constant value.
pub trait CstMaker<Const, Out> {
  /// Creates a constant value.
  #[inline]
  fn cst(& self, Const) -> Out ;
}
impl<
  'a, Const: Clone, T: Sized + CstMaker<Const, Term>
> CstMaker<& 'a Const, Term> for T {
  fn cst(& self, c: & 'a Const) -> Term {
    self.cst(c.clone())
  }
}
impl CstMaker<Cst, Term> for TermConsign {
  fn cst(& self, c: Cst) -> Term {
    self.mk( C(c) )
  }
}

/// Can create an application of an operator.
pub trait OpMaker {
  /// Creates an application of an operator.
  #[inline]
  fn op(& self, Operator, Vec<Term>) -> Term ;
}
impl OpMaker for TermConsign {
  fn op(& self, op: Operator, mut args: Vec<Term>) -> Term {
    args.shrink_to_fit() ;
    self.mk( Op(op, args) )
  }
}

/// Can create an application of a function symbol.
pub trait AppMaker<Id> {
  /// Creates an application of a function symbol.
  #[inline]
  fn app(& self, Id, Vec<Term>) -> Term ;
}
impl<
  'a, Id: Clone, T: Sized + AppMaker<Id>
> AppMaker<& 'a Id> for T {
  fn app(& self, id: & 'a Id, args: Vec<Term>) -> Term {
    (self as & AppMaker<Id>).app(id.clone(), args)
  }
}
impl AppMaker<Sym> for TermConsign {
  fn app(& self, id: Sym, mut args: Vec<Term>) -> Term {
    args.shrink_to_fit() ;
    self.mk( App(id, args) )
  }
}

/// Can create quantified terms and let-bindings.
pub trait BindMaker<Trm> {
  /// Creates a universal quantification over some symbols.
  #[inline]
  fn forall(& self, Vec<(Sym, Type)>, Trm) -> Term ;
  /// Creates an existential quantification over some symbols.
  #[inline]
  fn exists(& self, Vec<(Sym, Type)>, Trm) -> Term ;
  /// Creates a let-binding.
  #[inline]
  fn let_b(& self, Vec<(Sym, Term)>, Trm) -> Term ;
}
impl<
  'a, Trm: Clone, T: Sized + BindMaker<Trm>
> BindMaker<& 'a Trm> for T {
  #[inline(always)]
  fn forall(& self, mut bind: Vec<(Sym, Type)>, term: & 'a Trm) -> Term {
    bind.shrink_to_fit() ;
    self.forall( bind, term.clone() )
  }
  #[inline(always)]
  fn exists(& self, mut bind: Vec<(Sym, Type)>, term: & 'a Trm) -> Term {
    bind.shrink_to_fit() ;
    self.exists( bind, term.clone() )
  }
  #[inline(always)]
  fn let_b(& self, mut bind: Vec<(Sym, Term)>, term: & 'a Trm) -> Term {
    bind.shrink_to_fit() ;
    self.let_b( bind, term.clone() )
  }
}
impl BindMaker<Term> for TermConsign {
  fn forall(& self, mut bind: Vec<(Sym, Type)>, term: Term) -> Term {
    bind.shrink_to_fit() ;
    if bind.is_empty() { term } else {
      self.mk( Forall(bind, term) )
    }
  }
  fn exists(& self, mut bind: Vec<(Sym, Type)>, term: Term) -> Term {
    bind.shrink_to_fit() ;
    if bind.is_empty() { term } else {
      self.mk( Exists(bind, term) )
    }
  }
  fn let_b(& self, mut bind: Vec<(Sym, Term)>, term: Term) -> Term {
    bind.shrink_to_fit() ;
    if bind.is_empty() { term } else {
      self.mk( Let(bind, term) )
    }
  }
}

/// A trait aggregating variable, constant, and term making traits.
pub trait Factory :
  VarMaker<Sym, Term> +
  CstMaker<Cst, Term> +
  OpMaker +
  AppMaker<Sym> +
  BindMaker<Term> {
}

pub fn bump<F: Factory>(f: & F, term: Term) -> Res<Term> {
  use var::RealVar::* ;
  zip::var_map(
    f,
    |factory, t| match * t.get() {
      V(ref var) => match * var.get() {
        SVar(ref s, State::Curr) => {
          let nu = factory.svar(s.clone(), State::Next) ;
          Ok( Some(nu) )
        },
        SVar(_,_) => Err(
          format!("[bump] illegal svar {}", var).into()
        ),
        _ => Ok(None),
      },
      _ => Ok(None),
    },
    term
  )
}

pub fn debump<F: Factory>(f: & F, term: Term) -> Res<Term> {
  use var::RealVar::* ;
  zip::var_map(
    f,
    |factory, t| match * t.get() {
      V(ref var) => match * var.get() {
        SVar(ref s, State::Next) => {
          let nu = factory.svar(s.clone(), State::Curr) ;
          Ok( Some(nu) )
        },
        SVar(_,_) => Err(
          format!("[debump] illegal svar {}", var).into()
        ),
        _ => Ok(None),
      },
      _ => Ok(None),
    },
    term
  )
}




/// Zipper stuff.
mod zip {
  use super::{ Operator, Term, RealTerm, Factory } ;
  use ::sym::Sym ;
  use ::typ::Type ;

  use self::Res::* ;
  use self::Step::* ;

  /// Result of going up in a zipper.
  enum Res {
    /// Zipper is done, contains the resulting term.
    Done(Term),
    /// Zipper is not done, contains the new state of the zipper.
    NYet(Zip)
  }

  /// A zipper step.
  enum Step {
    /// We're below an operator application.
    Op(
      Operator, Vec<Term>, Vec<Term>
    ),
    /// We're below a function symbol application.
    App(
      Sym, Vec<Term>, Vec<Term>
    ),
    /// We're below a universal quantifier.
    Forall(
      Vec<(Sym, Type)>
    ),
    /// We're below an existential quantifier.
    Exists(
      Vec<(Sym, Type)>
    ),
    /// We're below a let-binding, in the terms symbols are binded to.
    Let1(
      Vec<(Sym, Term)>
    ),
    /// We're below a let-binding, in the term the let ranges over.
    Let2(
      Vec<(Sym, Term)>, Sym, Vec<(Sym, Term)>, Term
    ),
  }

  /// A zipper on terms.
  struct Zip {
    /// Path of steps leading to the current term.
    path: Vec<Step>,
    /// Current term.
    curr: Term,
  }

  impl Zip {
    /// Goes down the current term stops when it reaches a leaf.
    ///
    /// That is, a variable or a constant.
    pub fn go_down(mut self) -> Self {
      loop {
        let update = match * self.curr.get() {

          RealTerm::Op(ref op, ref terms) => {
            let mut terms = terms.clone() ;
            terms.reverse() ;
            if let Some(term) = terms.pop() {
              self.path.push( Op(op.clone(), vec![], terms) ) ;
              Some( term.clone() )
            } else {
              panic!("operator applied to nothing: {:?}", op)
            }
          },

          RealTerm::App(ref sym, ref terms) => {
            let mut terms = terms.clone() ;
            terms.reverse() ;
            if let Some(term) = terms.pop() {
              self.path.push( App(sym.clone(), vec![], terms) ) ;
              Some( term.clone() )
            } else {
              panic!("application to nothing: {:?}", sym)
            }
          },

          RealTerm::Forall(ref syms, ref term) => {
            self.path.push( Forall(syms.clone()) ) ;
            Some( term.clone() )
          },

          RealTerm::Exists(ref syms, ref term) => {
            self.path.push( Exists(syms.clone()) ) ;
            Some( term.clone() )
          },

          RealTerm::Let(ref syms, ref term) => {
            self.path.push( Let1(syms.clone()) ) ;
            Some( term.clone() )
          },

          _ => None,
        } ;

        match update {
          None => return self,
          Some(t) => self.curr = t,
        }
      }
    }

    /// Goes up in the zipper recursively.
    ///
    /// Stops if going up an empty path, or unexplored siblings are found.
    pub fn go_up<F: Factory>(mut self, cons: & F) -> Res {
      loop {
        match self.path.pop() {

          Some( Op(op, mut lft, mut rgt) ) => {
            lft.push(self.curr) ;
            if let Some(term) = rgt.pop() {
              // Not done if `rgt` is not empty.
              self.curr = term ;
              self.path.push( Op(op, lft, rgt) ) ;
              return NYet(self)
            } else {
              // Otherwise go up.
              self.curr = cons.op(op, lft)
            }
          },

          Some( App(sym, mut lft, mut rgt) ) => {
            lft.push(self.curr) ;
            if let Some(term) = rgt.pop() {
              // Not done if `rgt` is not empty.
              self.curr = term ;
              self.path.push( App(sym, lft, rgt) ) ;
              return NYet(self)
            } else {
              // Otherwise go up.
              self.curr = cons.app(sym, lft)
            }
          },

          Some( Forall(syms) ) =>
            self.curr = cons.forall(syms, self.curr),

          Some( Exists(syms) ) =>
            self.curr = cons.exists(syms, self.curr),

          Some( Let1(mut syms) ) => {
            if let Some( (sym, term) ) = syms.pop() {
              self.path.push( Let2(vec![], sym, syms, self.curr) ) ;
              self.curr = term ;
              return NYet(self)
            } else {
              // We're in a let of nothing, skipping it.
              ()
            }
          },

          Some( Let2(mut lft, sym, mut rgt, t) ) => {
            lft.push( (sym, self.curr) ) ;
            if let Some( (sym, term) ) = rgt.pop() {
              // Not done if `rgt` is not empty.
              self.curr = term ;
              self.path.push( Let2(lft, sym, rgt, t) ) ;
              return NYet(self)
            } else {
              // Otherwise go up.
              self.curr = cons.let_b(lft, t)
            }
          },

          None => return Done(self.curr),
        }
      }
    }
  }

  // pub fn fold<Out, F>(cons: TermConsign, f: F, term: Term, init: Out) -> Out
  // where F: Fn(Out, & Term) -> Out {
  //   let mut zip = Zip { path: vec![], curr: term, cons: cons } ;
  //   let mut out = init ;
  //   loop {
  //     zip = zip.go_down() ;
  //     out = f(out, & zip.curr) ;
  //     zip = match zip.go_up() {
  //       Done(term) => return out,
  //       NYet(zip) => zip,
  //     }
  //   }
  // }

  /// Applies some function to the variables in a term.
  pub fn var_map<'a, F: Factory, Fun, E>(
    cons: & 'a F, f: Fun, term: Term
  ) -> Result<Term,E>
  where Fun: Fn(& 'a F, & Term) -> Result<Option<Term>,E> {
    let mut zip = Zip { path: vec![], curr: term } ;
    loop {
      zip = zip.go_down() ;
      zip.curr = match f(cons, & zip.curr) {
        Ok( Some(term) ) => term,
        Ok( None ) => zip.curr,
        Err(e) => return Err(e),
      } ;
      zip = match zip.go_up(cons) {
        Done(term) => return Ok(term),
        NYet(zip) => zip,
      }
    }
  }
}





pub mod zip2 {
  use std::collections::HashMap ;

  use ::sym::Sym ;
  use ::typ::Type ;
  use ::cst::Cst ;
  use ::var::Var ;

  use super::{ Operator, Term, RealTerm } ;

  use self::ZipStep::* ;
  use self::Res::* ;

  enum ZipStep<T> {
    App(Sym, Vec<T>, Vec<Term>),
    Op(Operator, Vec<T>, Vec<Term>),
    Let1(
      Vec<(Sym, T)>, Sym, Vec<(Sym, Term)>, Term
    ),
    Let2(
      Vec<(Sym, T)>
    ),
    Forall(
      Vec<(Sym, Type)>
    ),
    Exists(
      Vec<(Sym, Type)>
    ),
  }

  /// A step upward in the zipper.
  pub enum Step<T> {
    /// Application.
    App(Sym, Vec<T>),
    /// Operator.
    Op(Operator, Vec<T>),
    /// Let binding.
    Let(Vec<(Sym,T)>, T),
    /// Universal quantifier.
    Forall(Vec<(Sym, Type)>, T),
    /// Existential quantifier.
    Exists(Vec<(Sym, Type)>, T),
    /// Constant.
    C(Cst),
    /// Variable.
    V(Var),
  }

  enum Res<T> {
    NYet(Step<T>),
    Done(T),
  }

  struct Zip<T> {
    path: Vec<ZipStep<T>>,
    bindings: Vec<HashMap<Sym, T>>,
    quantified: Vec<HashMap<Sym, Type>>,
  }

  impl<T: Clone> Zip<T> {

    #[inline(always)]
    fn add_binding(& mut self, sym: Sym, t: T) {
      if self.bindings.is_empty() {
        panic!(
          "[term::zip] trying to add binding on empty list of binding maps"
        )
      } else {
        let last = self.bindings.len() - 1 ;
        self.bindings[last].insert(sym, t) ;
        ()
      }
    }

    #[inline(always)]
    fn push(& mut self, step: ZipStep<T>) {
      self.path.push(step)
    }

    #[inline(always)]
    fn pop(& mut self) -> Option<ZipStep<T>> {
      self.path.pop()
    }

    fn zip_down(& mut self, mut term: Term) -> Step<T> {
      loop {
        term = match * term.get() {

          RealTerm::Op(ref op, ref terms) => {
            let mut terms = terms.clone() ;
            terms.reverse() ;
            if let Some(kid) = terms.pop() {
              self.push(
                Op(op.clone(), Vec::with_capacity(terms.len() + 1), terms)
              ) ;
              kid.clone()
            } else {
              panic!("zipping down an operator ({}) applied to nothing", op)
            }
          },

          RealTerm::App(ref sym, ref terms) => {
            let mut terms = terms.clone() ;
            terms.reverse() ;
            if let Some(kid) = terms.pop() {
              self.push(
                App(sym.clone(), Vec::with_capacity(terms.len() + 1), terms)
              ) ;
              kid.clone()
            } else {
              panic!("zipping down an application ({}) to nothing", sym)
            }
          },

          RealTerm::Forall(ref syms, ref kid) => {
            self.push( Forall(syms.clone()) ) ;
            let mut map = HashMap::new() ;
            for & (ref sym, ref typ) in syms.iter() {
              map.insert(sym.clone(), typ.clone()) ;
            } ;
            self.quantified.push(map) ;
            kid.clone()
          },

          RealTerm::Exists(ref syms, ref kid) => {
            self.push( Exists(syms.clone()) ) ;
            let mut map = HashMap::new() ;
            for & (ref sym, ref typ) in syms.iter() {
              map.insert(sym.clone(), typ.clone()) ;
            } ;
            self.quantified.push(map) ;
            kid.clone()
          },

          RealTerm::Let(ref syms, ref kid) => {
            self.bindings.push(HashMap::new()) ;
            let mut syms = syms.clone() ;
            syms.reverse() ;
            if let Some( (sym, fst) ) = syms.pop() {
              self.push(
                Let1(
                  Vec::with_capacity(syms.len() + 1), sym, syms, kid.clone()
                )
              ) ;
              fst.clone()
            } else {
              panic!("[term::zip] zipping down a let-binding with no bindings")
            }
          },

          RealTerm::C(ref cst) => return Step::C(cst.clone()),

          RealTerm::V(ref var) => return Step::V(var.clone()),

        }
      }
    }


    fn zip_up(& mut self, t: T) -> Res<T> {
      match self.pop() {

        None => Done(t),

        Some( App(sym, mut lft, mut rgt) ) => {
          lft.push(t) ;
          if let Some(term) = rgt.pop() {
            self.push( App(sym, lft, rgt) ) ;
            NYet( self.zip_down(term) )
          } else {
            NYet( Step::App(sym, lft) )
          }
        },

        Some( Op(op, mut lft, mut rgt) ) => {
          lft.push(t) ;
          if let Some(term) = rgt.pop() {
            self.push( Op(op, lft, rgt) ) ;
            NYet( self.zip_down(term) )
          } else {
            NYet( Step::Op(op, lft) )
          }
        },

        Some( Let1(mut lft, sym, mut rgt, kid) ) => {
          self.add_binding(sym.clone(), t.clone()) ;
          lft.push( (sym, t) ) ;
          if let Some( (sym, term) ) = rgt.pop() {
            self.push( Let1(lft, sym, rgt, kid) ) ;
            NYet( self.zip_down(term) )
          } else {
            self.push( Let2(lft) ) ;
            NYet( self.zip_down(kid) )
          }
        },

        Some( Let2(syms) ) => {
          match self.bindings.pop() {
            Some(_) => (),
            None => panic!(
              "[term::zip] going up Let2 but list of bindings is empty"
            ),
          } ;
          NYet( Step::Let(syms, t) )
        },

        Some( Forall(syms) ) => {
          match self.quantified.pop() {
            Some(_) => (),
            None => panic!(
              "[term::zip] going up Forall but list of quantifieds is empty"
            ),
          } ;
          NYet( Step::Forall(syms, t) )
        },

        Some( Exists(syms) ) => {
          match self.quantified.pop() {
            Some(_) => (),
            None => panic!(
              "[term::zip] going up Exists but list of quantifieds is empty"
            ),
          } ;
          NYet( Step::Exists(syms, t) )
        },

      }
    }

  }

  /// Bottom-up, left-to-right fold.
  pub fn fold<
    T: Clone, Fun: Fn(Step<T>) -> T
  >(f: Fun, term: Term) -> T {
    let mut zip = Zip {
      path: vec![], bindings: vec![], quantified: vec![]
    } ;
    let first = zip.zip_down(term) ;
    let mut t = f(first) ;
    loop {
      match zip.zip_up(t) {
        NYet(step) => t = f(step),
        Done(t) => return t,
      }
    }
  }

  /// Bottom-up, left-to-right fold with information.
  pub fn fold_info<
    T: Clone, E,
    Fun: Fn(
      Step<T>, & [ HashMap<Sym, T> ], & [ HashMap<Sym, Type> ]
    ) -> Result<T,E>
  >(f: Fun, term: & Term) -> Result<T, E> {
    let term = term.clone() ;
    let mut zip = Zip {
      path: vec![], bindings: vec![], quantified: vec![]
    } ;
    let first = zip.zip_down(term) ;
    match f(first, & zip.bindings, & zip.quantified) {
      Ok(mut t) => loop {
        match zip.zip_up(t) {
          NYet(step) => match f(step, & zip.bindings, & zip.quantified) {
            Ok(nu_t) => t = nu_t,
            e => return e,
          },
          Done(t) => return Ok(t),
        }
      },
      e => return e,
    }
  }

  /// Extracts the value associated to a symbol in a hash map.
  pub fn extract<'a, T>(
    sym: & Sym, maps: & 'a [ HashMap<Sym, T> ]
  ) -> Option<& 'a T> {
    let maps = maps.iter().rev() ;
    for map in maps {
      match map.get(sym) {
        None => (),
        Some(t) => return Some(t),
      } ;
    } ;
    None
  }

}




/// Term evaluator.
pub mod eval {
  use ::{
    Type, Cst, Sym, Term, Offset2, Factory, UnTermOps
  } ;
  use ::errors::* ;
  use std::collections::HashMap ;
  use ::zip::{ Step, fold_info, extract } ;
  use ::zip::Step::* ;

  /// Function passed to fold to evaluate a term.
  fn eval_term(
    factory: & Factory,
    model: & HashMap<Term, & Cst>,
    step: Step<Cst>,
    bindings: & [ HashMap<Sym, Cst> ],
    quantified: & [ HashMap<Sym, Type> ],
    scope: & Sym
  ) -> Res<Cst> {
    match step {

      App(_, _) => Err(
        "evaluation of applications is not implemented".into()
      ),

      Op(op, args) => op.eval(factory, args),

      Let(_, cst) => Ok(cst),

      C(cst) => Ok(cst),

      V(r_var) => {
        let sym = r_var.sym().clone() ;
        let var = factory.mk_var(r_var) ;
        match model.get(& var) {
          Some(cst) => Ok( (* cst).clone() ),
          None => match extract(& sym, bindings) {
            Some(cst) => Ok( cst.clone() ),
            None => match extract(& sym, quantified) {
              Some(_) => Err(
                format!("cannot evaluate quantified variable {}", var).into()
              ),
              None => match factory.type_of(& var, Some(scope.clone())) {
                Ok(typ) => Ok(
                  factory.mk_rcst(typ.default())
                ),
                Err(e) => Err(
                  format!(
                    "variable {} not found in model \
                    or in type cache\n{}", var, e
                  ).into()
                ),
              },
            },
          },
        }
      },

      _ => Err("evaluation of quantifiers is not implemented".into()),
    }
  }

  /// Evaluates a term.
  pub fn eval(
    factory: & Factory, term: & Term, offset: & Offset2,
    model: & ::Model, scope: Sym
  ) -> Res<Cst> {
    let mut map = HashMap::new() ;
    for & ( (ref v, ref o), ref cst ) in model.iter() {
      if let Some(ref o) = * o {
        if o == offset.curr() {
          let v = factory.mk_var( v.clone() ) ;
          map.insert( v, cst ) ;
        } else {
          let v = factory.mk_var( v.clone() ) ;
          if o == offset.next() {
            map.insert( factory.bump(v).unwrap(), cst ) ;
          }
        }
      } else {
        let v = factory.mk_var( v.clone() ) ;
        map.insert( v, cst ) ;
      }
    } ;
    fold_info(
      |step, bindings, quantified| eval_term(
        factory, & map, step, bindings, quantified, & scope
      ),
      term
    )
  }
}