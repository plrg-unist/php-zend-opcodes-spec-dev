use std::fmt::Write;

use crate::{Error, Parser, Printer, Result, Value, with_stack};

impl Parser<'_> {
    pub(crate) fn zval(&mut self) -> Result<Value> {
        with_stack(|| self.read_zval())
    }

    fn read_zval(&mut self) -> Result<Value> {
        for (word, tag) in [
            ("undef", "UNDEF"),
            ("null", "NULL"),
            ("false", "FALSE"),
            ("true", "TRUE"),
        ] {
            if self.take(word) {
                return self.case("zval", tag, []);
            }
        }
        if self.peek("\"") {
            let bytes = self.bytes()?;
            return self.case("zval", "STRING", [bytes]);
        }
        if self.take("int(") {
            let n = self.i64()?;
            self.expect(")")?;
            let value = self.int(n)?;
            return self.case("zval", "INT", [value]);
        }
        if self.take("float(") {
            let spelling = self.scan(|c| c.is_ascii_alphanumeric() || b"+-.:".contains(&c));
            let bits = binary64(spelling).ok_or_else(|| self.error("invalid binary64 literal"))?;
            self.expect(")")?;
            let value = self.nat(bits)?;
            return self.case("zval", "FLOAT", [value]);
        }
        if self.take("array(") {
            return self.read_array();
        }
        if self.take("array#") {
            let id = self.u32()?;
            self.expect("(")?;
            if self.shared_arrays.insert(id, None).is_some() {
                return Err(self.error(format!("duplicate shared array {id}")));
            }
            let array = self.read_array()?;
            let number = self.nat(id.into())?;
            let shared = self.case("zval", "SHARED_ARRAY", [number, array])?;
            self.shared_arrays.insert(id, Some(shared));
            return Ok(shared);
        }
        if self.take("array-ref(") {
            let id = self.u32()?;
            self.expect(")")?;
            return self
                .shared_arrays
                .get(&id)
                .copied()
                .flatten()
                .ok_or_else(|| self.error(format!("no completed shared array {id}")));
        }
        if self.take("ast") {
            self.expect("(")?;
            let node = self.expr()?;
            self.expect(")")?;
            return self.case("zval", "DEFERRED", [node]);
        }
        Err(self.error("expected a PHP literal"))
    }

    fn read_literal_layout(&mut self) -> Result<Option<Value>> {
        let layout = if self.take("packed=") {
            let capacity = self.number()?;
            self.expect(";")?;
            self.case("literalLayout", "PACKED_LITERAL", [capacity])?
        } else if self.take("mixed=") {
            let capacity = self.number()?;
            self.expect(";")?;
            let mut holes = Vec::new();
            if self.take("holes=") {
                self.expect("[")?;
                if !self.peek("]") {
                    loop {
                        holes.push(self.number()?);
                        if !self.take(",") {
                            break;
                        }
                    }
                }
                self.expect("]")?;
                self.expect(";")?;
            }
            let holes = self.list("nat", holes)?;
            self.case("literalLayout", "MIXED_TABLE", [capacity, holes])?
        } else if self.take("mixed") {
            self.expect(";")?;
            self.case("literalLayout", "MIXED_LITERAL", [])?
        } else {
            return Ok(None);
        };
        Ok(Some(layout))
    }

    fn read_array(&mut self) -> Result<Value> {
        let immutable = self.take("immutable");
        if immutable {
            self.expect(";")?;
        }
        let layout = self.read_literal_layout()?;
        let mut has_metadata = if immutable || layout.is_some() {
            self.peek("next=") || self.peek("cursor=") || self.peek("position=")
        } else {
            self.take(";")
        };
        let mut entries = Vec::new();
        if !has_metadata && !self.peek(")") {
            loop {
                let key = if self.peek("\"") {
                    let bytes = self.bytes()?;
                    self.case("arraykey", "KEY", [bytes])?
                } else {
                    let n = self.i64()?;
                    let index = self.int(n)?;
                    self.case("arraykey", "INDEX", [index])?
                };
                self.expect("=>")?;
                let value = self.zval()?;
                entries.push(self.case("arrayentry", "ENTRY", [key, value])?);
                if !self.take(",") {
                    break;
                }
            }
            has_metadata = self.take(";");
        }
        let mut next = None;
        let mut cursor = None;
        if has_metadata && self.take("next=") {
            let n = self.i64()?;
            next = Some(self.int(n)?);
            has_metadata = self.take(";");
        }
        if has_metadata {
            let tag = if self.take("position=") {
                "PHYSICAL_CURSOR"
            } else {
                self.expect("cursor=")?;
                "ORDINAL_CURSOR"
            };
            let position = self.number()?;
            cursor = Some(self.case("literalCursor", tag, [position])?);
        }
        self.expect(")")?;
        let entries = self.list("arrayentry", entries)?;
        let next = self.opt("int", next)?;
        let cursor = self.opt("literalCursor", cursor)?;
        let layout = self.opt("literalLayout", layout)?;
        let immutable = self.boolean(immutable)?;
        self.case("zval", "ARRAY", [entries, next, cursor, layout, immutable])
    }

    pub(crate) fn ztype(&mut self) -> Result<Value> {
        with_stack(|| self.read_ztype())
    }

    fn read_ztype(&mut self) -> Result<Value> {
        self.expect("type(")?;
        let mask = self.number()?;
        if !self.take(",") {
            self.expect(")")?;
            return self.case("ztype", "MASK", [mask]);
        }
        if self.take("name") {
            let name = self.bytes()?;
            self.expect(")")?;
            return self.case("ztype", "NAMED", [mask, name]);
        }
        let tag = if self.take("intersection") {
            "INTERSECTION"
        } else {
            self.expect("union")?;
            "UNION"
        };
        let count = self.u32()?;
        self.expect("[")?;
        let mut members = Vec::new();
        for index in 0..count {
            if index != 0 {
                self.expect(",")?;
            }
            members.push(self.ztype()?);
        }
        self.expect("]")?;
        self.expect(")")?;
        let members = self.list("ztype", members)?;
        self.case("ztype", tag, [mask, members])
    }
}

fn binary64(spelling: &str) -> Option<u64> {
    if let Some(hex) = spelling.strip_prefix("nan:0x") {
        if hex.len() != 16 || !hex.bytes().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let bits = u64::from_str_radix(hex, 16).ok()?;
        return f64::from_bits(bits).is_nan().then_some(bits);
    }
    match spelling {
        "inf" => Some(f64::INFINITY.to_bits()),
        "-inf" => Some(f64::NEG_INFINITY.to_bits()),
        _ => {
            let value: f64 = spelling.parse().ok()?;
            value.is_finite().then_some(value.to_bits())
        }
    }
}

impl Printer<'_> {
    pub(crate) fn zval(&mut self, value: Value) -> Result<()> {
        with_stack(|| self.write_zval(value))
    }

    fn write_zval(&mut self, value: Value) -> Result<()> {
        match self.case(value)? {
            ("UNDEF", []) => self.out.push_str("undef"),
            ("NULL", []) => self.out.push_str("null"),
            ("FALSE", []) => self.out.push_str("false"),
            ("TRUE", []) => self.out.push_str("true"),
            ("INT", [n]) => write!(self.out, "int({})", self.int(*n)?)?,
            ("STRING", [bytes]) => self.bytes(*bytes)?,
            ("FLOAT", [value]) => {
                let bits = self.nat(*value)?;
                let float = f64::from_bits(bits);
                if float.is_nan() {
                    write!(self.out, "float(nan:0x{bits:016x})")?;
                } else {
                    write!(self.out, "float({float:?})")?;
                }
            }
            ("ARRAY", _) => {
                self.out.push_str("array");
                self.write_array(value)?;
            }
            ("SHARED_ARRAY", &[id, array]) => {
                let id = self.u32(id)?;
                if let Some(known) = self.shared_arrays.get(&id) {
                    if !known.is_some_and(|value| {
                        self.arena.value.canon_id(value.node)
                            == self.arena.value.canon_id(array.node)
                    }) {
                        return Err(Error::new(format!("conflicting shared array {id}")));
                    }
                    write!(self.out, "array-ref({id})")?;
                } else {
                    self.shared_arrays.insert(id, None);
                    write!(self.out, "array#{id}")?;
                    self.write_array(array)?;
                    self.shared_arrays.insert(id, Some(array));
                }
            }
            ("DEFERRED", [node]) => {
                self.out.push_str("ast(");
                self.expr(*node)?;
                self.out.push(')');
            }
            _ => return Err(Error::new("expected a PHP literal")),
        }
        Ok(())
    }

    fn write_array(&mut self, array: Value) -> Result<()> {
        let [entries, next, cursor, layout, immutable] = self.fields(array, "ARRAY")?;
        let immutable = self.boolean(immutable)?;
        self.out.push('(');
        let entries = self.list(entries)?;
        let mut separator = "; ";
        let layout = self.opt(layout)?;
        if immutable {
            self.out.push_str("immutable;");
            if layout.is_some() {
                self.out.push(' ');
            }
        }
        if let Some(layout) = layout {
            match self.case(layout)? {
                ("PACKED_LITERAL", &[capacity]) => {
                    write!(self.out, "packed={};", self.u32(capacity)?)?
                }
                ("MIXED_LITERAL", []) => self.out.push_str("mixed;"),
                ("MIXED_TABLE", &[capacity, holes]) => {
                    write!(self.out, "mixed={};", self.u32(capacity)?)?;
                    let holes = self.list(holes)?;
                    if !holes.is_empty() {
                        self.out.push_str(" holes=[");
                        for (index, &hole) in holes.iter().enumerate() {
                            if index != 0 {
                                self.out.push_str(", ");
                            }
                            write!(self.out, "{}", self.u32(hole)?)?;
                        }
                        self.out.push_str("];");
                    }
                }
                _ => return Err(Error::new("invalid array literal layout")),
            }
        }
        if immutable || layout.is_some() {
            if entries.is_empty() {
                separator = " ";
            } else {
                self.out.push(' ');
            }
        }
        for (index, &entry) in entries.iter().enumerate() {
            if index != 0 {
                self.out.push_str(", ");
            }
            let [key, value] = self.fields(entry, "ENTRY")?;
            match self.case(key)? {
                ("KEY", [name]) => self.bytes(*name)?,
                ("INDEX", [n]) => write!(self.out, "{}", self.int(*n)?)?,
                _ => return Err(Error::new("expected integer or string array key")),
            }
            self.out.push_str(" => ");
            self.zval(value)?;
        }
        if let Some(value) = self.opt(next)? {
            write!(self.out, "{separator}next={}", self.int(value)?)?;
            separator = "; ";
        }
        if let Some(cursor) = self.opt(cursor)? {
            let (field, value) = match self.case(cursor)? {
                ("ORDINAL_CURSOR", &[value]) => ("cursor", value),
                ("PHYSICAL_CURSOR", &[value]) => ("position", value),
                _ => return Err(Error::new("invalid array cursor")),
            };
            write!(self.out, "{separator}{field}={}", self.u32(value)?)?;
        }
        self.out.push(')');
        Ok(())
    }

    pub(crate) fn ztype(&mut self, value: Value) -> Result<()> {
        with_stack(|| self.write_ztype(value))
    }

    fn write_ztype(&mut self, value: Value) -> Result<()> {
        match self.case(value)? {
            ("MASK", [mask]) => write!(self.out, "type({})", self.u32(*mask)?)?,
            ("NAMED", [mask, name]) => {
                write!(self.out, "type({}, name ", self.u32(*mask)?)?;
                self.bytes(*name)?;
                self.out.push(')');
            }
            (tag @ ("UNION" | "INTERSECTION"), [mask, members]) => {
                let name = if tag == "UNION" {
                    "union"
                } else {
                    "intersection"
                };
                let members = self.list(*members)?;
                write!(
                    self.out,
                    "type({}, {name} {} [",
                    self.u32(*mask)?,
                    members.len()
                )?;
                for (index, &member) in members.iter().enumerate() {
                    if index != 0 {
                        self.out.push_str(", ");
                    }
                    self.ztype(member)?;
                }
                self.out.push_str("])");
            }
            _ => return Err(Error::new("expected a Zend type")),
        }
        Ok(())
    }
}
