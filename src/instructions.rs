use std::{collections::HashSet, fmt::Write, sync::LazyLock};

use p4spectec::{
    frontend::parse::parse_text,
    lang::{
        common::notation::atom::Atom,
        el::ast::{DefKind, DefTypKind, NotTypKind, Typ},
    },
};

use crate::{Error, Parser, Printer, Result, Value};

fn opcode_names() -> Result<HashSet<String>> {
    let spec = parse_text(
        "spec/syntax.watsup".into(),
        include_str!("../spec/syntax.watsup"),
    )
    .map_err(|error| Error::new(format!("cannot parse embedded opcode syntax: {error}")))?;
    for definition in spec {
        let DefKind::Typ(typ) = definition.node else {
            continue;
        };
        if typ.id.node.as_ref() != "opcode" {
            continue;
        }
        let DefTypKind::Variant(cases) = typ.def_typ.node else {
            break;
        };
        let mut names = HashSet::with_capacity(cases.len());
        for case in cases {
            if let Typ::Notation(notation) = case.typ
                && let NotTypKind::Atom(atom) = notation.node
                && let Atom::Keyword(name) = atom.node
            {
                names.insert(name);
            } else {
                return Err(Error::new("expected a mnemonic in the opcode syntax"));
            }
        }
        return Ok(names);
    }
    Err(Error::new("missing opcode variant in the spec"))
}

fn known_opcode(name: &str) -> Result<bool> {
    static NAMES: LazyLock<Result<HashSet<String>>> = LazyLock::new(opcode_names);
    NAMES
        .as_ref()
        .map(|names| names.contains(name))
        .map_err(|error| Error::new(error.to_string()))
}

impl Parser<'_> {
    fn operand(&mut self) -> Result<Option<Value>> {
        if self.take("_") {
            return Ok(None);
        }
        let tag = if self.take("@") {
            "TARGET"
        } else if self.take("lit[") {
            "LIT"
        } else if self
            .rest()
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_digit)
        {
            "IMM"
        } else {
            let Some((prefix, tag)) =
                [("CV", "CV"), ("T", "TMP"), ("V", "VAR")]
                    .into_iter()
                    .find(|(prefix, _)| {
                        self.rest()
                            .strip_prefix(prefix)
                            .and_then(|tail| tail.as_bytes().first())
                            .is_some_and(u8::is_ascii_digit)
                    })
            else {
                return Ok(None);
            };
            self.expect(prefix)?;
            tag
        };
        let index = self.number()?;
        if tag == "LIT" {
            self.expect("]")?;
        }
        Ok(Some(self.case("operand", tag, [index])?))
    }

    pub(crate) fn instruction(&mut self, index: usize) -> Result<Value> {
        if self.u32()? as usize != index {
            return Err(self.error(format!("expected instruction {index}")));
        }
        self.expect(":")?;
        let unused = self.case("operand", "UNUSED", [])?;
        let result = self.operand()?;
        if result.is_some() {
            self.expect("=")?;
        }
        let mnemonic = self.scan(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == b'_');
        if !known_opcode(mnemonic)? {
            return Err(self.error(format!("unknown opcode '{mnemonic}'")));
        }
        let opcode = self.case("opcode", mnemonic, [])?;
        let op1 = self.operand()?.unwrap_or(unused);
        let op2 = if self.take(",") {
            self.operand()?.unwrap_or(unused)
        } else {
            unused
        };

        let (mut line, mut ext, mut cache, mut default_cache, mut mask, mut smart, mut numeric) =
            (None, None, None, None, None, None, None);
        if self.take("|") {
            loop {
                let (slot, value) = if self.take("line=") {
                    (&mut line, self.number()?)
                } else if self.take("ext=") {
                    let tag = if self.take("@") {
                        "EXT_TARGET"
                    } else {
                        "EXT_VALUE"
                    };
                    let n = self.number()?;
                    (&mut ext, self.case("extension", tag, [n])?)
                } else if self.take("builtin=") {
                    let name = self.bytes()?;
                    (&mut ext, self.case("extension", "BUILTIN", [name])?)
                } else if self.take("cache=") {
                    (&mut cache, self.number()?)
                } else if self.take("default-cache=") {
                    (&mut default_cache, self.number()?)
                } else if self.take("bound-mask=") {
                    (&mut mask, self.number()?)
                } else if self.take("smart=") {
                    let tag = if self.take("JMPNZ") {
                        "SMART_JMPNZ"
                    } else {
                        self.expect("JMPZ")?;
                        "SMART_JMPZ"
                    };
                    (&mut smart, self.case("smartbranch", tag, [])?)
                } else if self.take("numeric-string-key") {
                    (&mut numeric, self.boolean(true)?)
                } else {
                    break;
                };
                if slot.replace(value).is_some() {
                    return Err(self.error("duplicate instruction field"));
                }
            }
        }
        self.eol()?;
        let line = match line {
            Some(n) => n,
            None => self.nat(0)?,
        };
        let ext = self.opt("extension", ext)?;
        let cache = self.opt("nat", cache)?;
        let default_cache = self.opt("nat", default_cache)?;
        let mask = self.opt("nat", mask)?;
        let smart = self.opt("smartbranch", smart)?;
        let numeric = match numeric {
            Some(flag) => flag,
            None => self.boolean(false)?,
        };
        self.case(
            "instruction",
            "OPLINE",
            [
                opcode,
                result.unwrap_or(unused),
                op1,
                op2,
                line,
                ext,
                cache,
                default_cache,
                mask,
                smart,
                numeric,
            ],
        )
    }
}

impl Printer<'_> {
    fn operand_text(&self, value: Value) -> Result<Option<String>> {
        let (tag, fields) = self.case(value)?;
        if tag == "UNUSED" && fields.is_empty() {
            return Ok(None);
        }
        let [value] = fields else {
            return Err(Error::new("expected an indexed operand"));
        };
        let n = self.u32(*value)?;
        let text = match tag {
            "IMM" => n.to_string(),
            "TARGET" => format!("@{n}"),
            "LIT" => format!("lit[{n}]"),
            "CV" => format!("CV{n}"),
            "TMP" => format!("T{n}"),
            "VAR" => format!("V{n}"),
            _ => return Err(Error::new(format!("unknown operand {tag}"))),
        };
        Ok(Some(text))
    }

    pub(crate) fn instruction(&mut self, index: usize, value: Value) -> Result<()> {
        let [
            opcode,
            result,
            op1,
            op2,
            line,
            ext,
            cache,
            default_cache,
            mask,
            smart,
            numeric,
        ] = self.fields(value, "OPLINE")?;
        let (mnemonic, args) = self.case(opcode)?;
        if !args.is_empty() || !known_opcode(mnemonic)? {
            return Err(Error::new(format!("invalid opcode constructor {mnemonic}")));
        }
        write!(self.out, "    {index}: ")?;
        if let Some(result) = self.operand_text(result)? {
            write!(self.out, "{result} = ")?;
        }
        self.out.push_str(mnemonic);
        let first = self.operand_text(op1)?;
        let second = self.operand_text(op2)?;
        if first.is_some() || second.is_some() {
            write!(self.out, " {}", first.as_deref().unwrap_or("_"))?;
            if let Some(second) = second {
                write!(self.out, ", {second}")?;
            }
        }
        write!(self.out, " | line={}", self.u32(line)?)?;
        if let Some(ext) = self.opt(ext)? {
            match self.case(ext)? {
                ("EXT_VALUE", [n]) => write!(self.out, " ext={}", self.u32(*n)?)?,
                ("EXT_TARGET", [n]) => write!(self.out, " ext=@{}", self.u32(*n)?)?,
                ("BUILTIN", [name]) => {
                    self.out.push_str(" builtin=");
                    self.bytes(*name)?;
                }
                _ => return Err(Error::new("invalid extended instruction operand")),
            }
        }
        for (name, field) in [
            ("cache", cache),
            ("default-cache", default_cache),
            ("bound-mask", mask),
        ] {
            if let Some(n) = self.opt(field)? {
                write!(self.out, " {name}={}", self.u32(n)?)?;
            }
        }
        if let Some(smart) = self.opt(smart)? {
            let flag = match self.case(smart)? {
                ("SMART_JMPZ", []) => " smart=JMPZ",
                ("SMART_JMPNZ", []) => " smart=JMPNZ",
                _ => return Err(Error::new("invalid smart branch")),
            };
            self.out.push_str(flag);
        }
        if self.boolean(numeric)? {
            self.out.push_str(" numeric-string-key");
        }
        self.out.push('\n');
        Ok(())
    }
}
