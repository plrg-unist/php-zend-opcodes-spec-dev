use p4spectec::lang::{
    common::notation::atom::Atom,
    data::{
        notation::{flat::Mixfix, tree::Mixop},
        typ::{self, Typ},
        value::flat::make,
    },
};
use p4spectec::phrase;

use crate::{Arena, Error, Result, Value};

/// Parses one complete listing into the caller's P4-SpecTec arena.
pub fn parse_zop(arena: &mut Arena, input: &str) -> Result<Value> {
    let mut parser = Parser {
        arena,
        input,
        line: 1,
    };
    let unit = parser.unit()?;
    if !parser.input.is_empty() {
        return Err(parser.error("unexpected data after unit"));
    }
    Ok(unit)
}

pub(crate) struct Parser<'a> {
    arena: &'a mut Arena,
    input: &'a str,
    line: usize,
}

fn spec_type(name: &str) -> Typ {
    match name {
        "nat" => typ::make::nat(),
        "int" => typ::make::int(),
        "bool" => typ::make::bool(),
        _ => typ::make::var(
            phrase!(node: name.into(), span: Default::default()),
            Vec::new(),
        ),
    }
}

impl<'a> Parser<'a> {
    pub(crate) fn error(&self, message: impl AsRef<str>) -> Error {
        Error::new(format!("line {}: {}", self.line, message.as_ref()))
    }

    pub(crate) fn rest(&self) -> &'a str {
        self.input.trim_start_matches([' ', '\t'])
    }

    pub(crate) fn peek(&self, token: &str) -> bool {
        self.rest().starts_with(token)
    }

    pub(crate) fn take(&mut self, token: &str) -> bool {
        self.input = self.rest();
        if let Some(tail) = self.input.strip_prefix(token) {
            self.input = tail;
            true
        } else {
            false
        }
    }

    pub(crate) fn expect(&mut self, token: &str) -> Result<()> {
        if self.take(token) {
            Ok(())
        } else {
            Err(self.error(format!("expected {token:?}")))
        }
    }

    pub(crate) fn scan(&mut self, accept: impl Fn(u8) -> bool) -> &'a str {
        self.input = self.rest();
        let end = self
            .input
            .bytes()
            .take_while(|byte| byte.is_ascii() && accept(*byte))
            .count();
        let (token, tail) = self.input.split_at(end);
        self.input = tail;
        token
    }

    pub(crate) fn eol(&mut self) -> Result<()> {
        self.input = self.rest();
        if self.input.is_empty() {
            return Ok(());
        }
        self.input = self
            .input
            .strip_prefix("\r\n")
            .or_else(|| self.input.strip_prefix('\n'))
            .ok_or_else(|| self.error("expected end of line"))?;
        self.line += 1;
        Ok(())
    }

    pub(crate) fn u32(&mut self) -> Result<u32> {
        self.scan(|b| b.is_ascii_digit())
            .parse()
            .map_err(|_| self.error("expected a u32 integer"))
    }

    pub(crate) fn i64(&mut self) -> Result<i64> {
        let input = self.rest();
        let digits = input.strip_prefix('-').unwrap_or(input);
        let n = digits.bytes().take_while(u8::is_ascii_digit).count();
        if n == 0 {
            return Err(self.error("expected a signed integer"));
        }
        self.input = &digits[n..];
        input[..input.len() - self.input.len()]
            .parse()
            .map_err(|_| self.error("integer outside signed 64-bit range"))
    }

    pub(crate) fn number(&mut self) -> Result<Value> {
        let value = self.u32()?;
        self.nat(value.into())
    }

    pub(crate) fn nat(&mut self, value: u64) -> Result<Value> {
        Ok(make::nat(self.arena, value.into(), Default::default())?)
    }

    pub(crate) fn int(&mut self, value: i64) -> Result<Value> {
        Ok(make::int(self.arena, value.into(), Default::default())?)
    }

    pub(crate) fn boolean(&mut self, value: bool) -> Result<Value> {
        Ok(make::bool(self.arena, value, Default::default())?)
    }

    pub(crate) fn list(&mut self, element_type: &str, values: Vec<Value>) -> Result<Value> {
        let ty = typ::make::list(spec_type(element_type));
        Ok(make::list(
            self.arena,
            ty.node.into(),
            values,
            Default::default(),
        )?)
    }

    pub(crate) fn opt(&mut self, element_type: &str, value: Option<Value>) -> Result<Value> {
        let ty = typ::make::opt(spec_type(element_type));
        Ok(make::opt(
            self.arena,
            ty.node.into(),
            value,
            Default::default(),
        )?)
    }

    pub(crate) fn case<const N: usize>(
        &mut self,
        type_name: &str,
        tag: &str,
        args: [Value; N],
    ) -> Result<Value> {
        let head = Mixop::Atom(phrase!(node: Atom::Keyword(tag.into()), span: Default::default()));
        let shape = if N == 0 {
            head
        } else {
            let mut parts = vec![head];
            parts.resize(N + 1, Mixop::Arg);
            Mixop::Seq(parts)
        };
        let mixop = self
            .arena
            .mixop
            .intern(&shape)
            .map_err(|e| self.error(e.to_string()))?;
        let case = Mixfix::new(&self.arena.mixop, mixop, Vec::from(args))
            .map_err(|e| self.error(e.to_string()))?;
        Ok(make::case_flat(
            self.arena,
            spec_type(type_name).node.into(),
            case,
            Default::default(),
        )?)
    }

    pub(crate) fn bytes(&mut self) -> Result<Value> {
        self.expect("\"")?;
        let mut values = Vec::new();
        loop {
            let (byte, rest) = self
                .input
                .as_bytes()
                .split_first()
                .ok_or_else(|| self.error("unterminated byte string"))?;
            match byte {
                b'"' => {
                    self.input = &self.input[1..];
                    return self.list("nat", values);
                }
                b'\\' => {
                    let (byte, consumed) = match rest {
                        [b'\\', ..] => (b'\\', 2),
                        [b'"', ..] => (b'"', 2),
                        [b'x', hi, lo, ..] if hi.is_ascii_hexdigit() && lo.is_ascii_hexdigit() => {
                            let digits = &self.input[2..4];
                            (
                                u8::from_str_radix(digits, 16).expect("two ASCII hex digits"),
                                4,
                            )
                        }
                        _ => return Err(self.error("invalid byte string escape")),
                    };
                    self.input = &self.input[consumed..];
                    values.push(self.nat(byte.into())?);
                }
                0x20..=0x7e => {
                    let byte = *byte;
                    self.input = &self.input[1..];
                    values.push(self.nat(byte.into())?);
                }
                _ => return Err(self.error("unescaped byte in quoted string")),
            }
        }
    }

    pub(crate) fn optional_bytes(&mut self) -> Result<Value> {
        let value = if self.take("-") {
            None
        } else {
            Some(self.bytes()?)
        };
        self.opt("bytes", value)
    }
}
