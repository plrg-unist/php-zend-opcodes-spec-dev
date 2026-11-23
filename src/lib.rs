//! Convert one Zend opcode listing to and from a P4-SpecTec program value.

use std::fmt;

use p4spectec::lang::data::value::ValueError;
pub use p4spectec::lang::data::{arena::Arena, value::flat::Value};

mod ast;
mod bodies;
mod instructions;
mod metadata;
mod parse;
mod print;
mod units;
mod validate;
mod values;

use ast::case_fields;
use parse::Parser;
pub use parse::parse_zop;
use print::Printer;
pub use print::print_zop;
pub use validate::{validate_unit, validate_unit_and_collect_bodies};

/// A malformed listing or a value that cannot be printed as a Zend opcode unit.
#[derive(Debug)]
pub struct Error(String);

impl Error {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

impl From<ValueError> for Error {
    fn from(error: ValueError) -> Self {
        Self::new(error.to_string())
    }
}

impl From<fmt::Error> for Error {
    fn from(error: fmt::Error) -> Self {
        Self::new(error.to_string())
    }
}

type Result<T> = std::result::Result<T, Error>;

fn with_stack<T>(operation: impl FnOnce() -> T) -> T {
    stacker::maybe_grow(128 * 1024, 2 * 1024 * 1024, operation)
}

#[cfg(test)]
mod tests;
