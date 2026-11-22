use std::fmt::Write;

use p4spectec::lang::{common::prim::num, data::value::flat::get};

use crate::{Arena, Error, Result, Value};

/// Prints a complete Zend compilation unit. The value must belong to `arena`.
pub fn print_zop(arena: &Arena, unit: &Value) -> Result<String> {
    let mut printer = Printer {
        arena,
        out: String::new(),
    };
    printer.unit(*unit)?;
    Ok(printer.out)
}

pub(crate) struct Printer<'a> {
    arena: &'a Arena,
    pub(crate) out: String,
}

impl<'a> Printer<'a> {
    pub(crate) fn case(&self, value: Value) -> Result<(&'a str, &'a [Value])> {
        crate::case_fields(self.arena, value)
    }

    pub(crate) fn fields<const N: usize>(&self, value: Value, tag: &str) -> Result<[Value; N]> {
        crate::ast::fields(self.arena, value, tag)
    }

    pub(crate) fn list(&self, value: Value) -> Result<&'a [Value]> {
        Ok(get::list(self.arena, &value)?)
    }

    pub(crate) fn opt(&self, value: Value) -> Result<Option<Value>> {
        Ok(get::opt(self.arena, &value)?)
    }

    pub(crate) fn nat(&self, value: Value) -> Result<u64> {
        let number = get::num(self.arena, &value)?;
        u64::try_from(num::to_int(number))
            .map_err(|_| Error::new("expected an unsigned 64-bit integer"))
    }

    pub(crate) fn u32(&self, value: Value) -> Result<u32> {
        u32::try_from(self.nat(value)?)
            .map_err(|_| Error::new("expected an unsigned 32-bit integer"))
    }

    pub(crate) fn int(&self, value: Value) -> Result<i64> {
        let number = get::num(self.arena, &value)?;
        i64::try_from(num::to_int(number))
            .map_err(|_| Error::new("expected a signed 64-bit integer"))
    }

    pub(crate) fn boolean(&self, value: Value) -> Result<bool> {
        Ok(get::bool(self.arena, &value)?)
    }

    pub(crate) fn bytes(&mut self, value: Value) -> Result<()> {
        self.out.push('"');
        for &element in self.list(value)? {
            let byte = u8::try_from(self.nat(element)?)
                .map_err(|_| Error::new("byte string element is outside the octet range"))?;
            match byte {
                b'"' | b'\\' => {
                    self.out.push('\\');
                    self.out.push(char::from(byte));
                }
                0x20..=0x7e => self.out.push(char::from(byte)),
                _ => write!(self.out, "\\x{byte:02x}")?,
            }
        }
        self.out.push('"');
        Ok(())
    }

    pub(crate) fn optional_bytes(&mut self, value: Value) -> Result<()> {
        match self.opt(value)? {
            Some(bytes) => self.bytes(bytes),
            None => {
                self.out.push('-');
                Ok(())
            }
        }
    }
}
