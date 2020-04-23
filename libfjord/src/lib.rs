//! A library that parses and evaluates Fjord code. Through [an FFI](ffi), Rust code can be called
//! from Fjord.

#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms)]
pub mod eval;
mod expr;
pub mod ffi;
mod func;
mod ident_name;
mod item;
mod misc;
pub mod params;

use misc::*;
pub use {
    expr::Expr,
    func::Func,
    ident_name::IdentName,
    item::{BindingVal, Item},
};

/// An error type for all the kinds of errors that can occur when parsing and evaluating some code.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// an error occurred during evaluation
    #[error("evaluation error")]
    Eval(#[from] eval::Error),
    /// an error occurred during parsing
    #[error("parsing error")]
    Parse,
}

/// A simple wrapper that parses some source code, and returns the result of evaluating it.
pub fn eval(s: &str, state: &mut eval::State<'_>) -> Result<eval::OutputExpr, Error> {
    let (_, expr) = match Item::new(s) {
        Ok(e) => e,
        _ => return Err(Error::Parse),
    };

    Ok(expr.eval(state)?)
}
