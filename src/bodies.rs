use std::fmt::Write;

use crate::{Error, Parser, Printer, Result, Value, with_stack};

impl Parser<'_> {
    pub(crate) fn body(&mut self) -> Result<Value> {
        with_stack(|| self.body_inner())
    }

    fn body_inner(&mut self) -> Result<Value> {
        self.expect("body")?;
        let name = self.optional_bytes()?;
        self.expect("{")?;
        self.eol()?;
        let (flags, flags2) = if self.take("flags") {
            let flags = self.number()?;
            let flags2 = self.number()?;
            self.eol()?;
            (flags, flags2)
        } else {
            (self.nat(0)?, self.nat(0)?)
        };
        let scope = self.body_bytes_row("scope")?;
        self.expect("lines")?;
        let start = self.number()?;
        let end = self.number()?;
        self.eol()?;
        let doc = self.body_bytes_row("doc")?;
        let source = self.case("bodysource", "LINES", [start, end, doc])?;

        self.expect("frame")?;
        let temps = self.number()?;
        self.expect("cache")?;
        let cache = self.number()?;
        self.eol()?;
        let locals = self.locals()?;
        let signature = self.signature()?;
        let attributes = self.attributes()?;
        let declaration = self.case(
            "bodydecl",
            "BODYDECL",
            [name, flags, flags2, scope, source, signature, attributes],
        )?;
        let statics = if self.take("statics") {
            let table = self.zval()?;
            self.eol()?;
            Some(table)
        } else {
            None
        };
        let statics = self.opt("zval", statics)?;
        let frame = self.case("bodyframe", "FRAME", [temps, cache, locals, statics])?;

        self.expect("size")?;
        let code_count = self.u32()?;
        self.expect("literals")?;
        let pool_count = self.u32()?;
        self.expect("children")?;
        let child_count = self.u32()?;
        self.eol()?;
        let mut pool = Vec::new();
        for index in 0..pool_count {
            pool.push(self.pool_entry(index)?);
        }
        let pool = self.list("poolentry", pool)?;
        let tries = self.try_regions()?;
        let ranges = self.live_ranges()?;
        let mut children = Vec::new();
        for _ in 0..child_count {
            children.push(self.body()?);
        }
        let children = self.list("body", children)?;
        self.expect("code")?;
        self.eol()?;
        let mut code = Vec::new();
        for index in 0..code_count {
            code.push(self.instruction(index as usize)?);
        }
        let code = self.list("instruction", code)?;
        self.expect("}")?;
        self.eol()?;
        self.case(
            "body",
            "BODY",
            [declaration, frame, pool, tries, ranges, children, code],
        )
    }

    fn body_bytes_row(&mut self, name: &str) -> Result<Value> {
        let value = if self.take(name) {
            let value = self.bytes()?;
            self.eol()?;
            Some(value)
        } else {
            None
        };
        self.opt("bytes", value)
    }

    fn locals(&mut self) -> Result<Value> {
        let mut locals = Vec::new();
        if self.take("locals") {
            let count = self.u32()?;
            self.expect("[")?;
            for i in 0..count {
                if i != 0 {
                    self.expect(",")?;
                }
                locals.push(self.bytes()?);
            }
            self.expect("]")?;
            self.eol()?;
        }
        self.list("bytes", locals)
    }

    fn signature(&mut self) -> Result<Value> {
        if !self.take("args") {
            return self.opt("signature", None);
        }
        let declared = self.number()?;
        self.expect("required")?;
        let required = self.number()?;
        self.eol()?;
        let returns = if self.take("returns") {
            let ty = self.ztype()?;
            self.eol()?;
            Some(ty)
        } else {
            None
        };
        let returns = self.opt("ztype", returns)?;
        let mut args = Vec::new();
        while self.take("arg") {
            let name = self.bytes()?;
            let ty = self.ztype()?;
            let doc = if self.take("doc") {
                Some(self.bytes()?)
            } else {
                None
            };
            let doc = self.opt("bytes", doc)?;
            self.eol()?;
            args.push(self.case("arginfo", "ARG", [name, ty, doc])?);
        }
        let args = self.list("arginfo", args)?;
        let signature = self.case("signature", "ARGS", [declared, required, returns, args])?;
        self.opt("signature", Some(signature))
    }

    fn pool_entry(&mut self, index: u32) -> Result<Value> {
        self.expect("lit[")?;
        if self.u32()? != index {
            return Err(self.error(format!("expected literal index {index}")));
        }
        self.expect("]")?;
        self.expect("=")?;
        let literal = if self.take("attributes") {
            self.expect("{")?;
            self.eol()?;
            let attributes = self.attributes()?;
            self.expect("}")?;
            self.case("poolentry", "ATTRIBUTES", [attributes])?
        } else {
            let value = self.zval()?;
            self.case("poolentry", "LITERAL", [value])?
        };
        self.eol()?;
        Ok(literal)
    }

    fn try_regions(&mut self) -> Result<Value> {
        let mut regions = Vec::new();
        if self.take("tries") {
            let count = self.u32()?;
            self.eol()?;
            for _ in 0..count {
                self.expect("try")?;
                self.expect("@")?;
                let start = self.number()?;
                self.expect("catch")?;
                self.expect("@")?;
                let catch = self.number()?;
                self.expect("finally")?;
                self.expect("@")?;
                let finally = self.number()?;
                self.expect("end")?;
                self.expect("@")?;
                let end = self.number()?;
                self.eol()?;
                regions.push(self.case("tryregion", "TRY", [start, catch, finally, end])?);
            }
        }
        self.list("tryregion", regions)
    }

    fn live_ranges(&mut self) -> Result<Value> {
        let mut ranges = Vec::new();
        if self.take("ranges") {
            let count = self.u32()?;
            self.eol()?;
            for _ in 0..count {
                self.expect("live")?;
                self.expect("T")?;
                let slot = self.number()?;
                self.expect("kind")?;
                let kind = self.number()?;
                self.expect("@")?;
                let start = self.number()?;
                self.expect("@")?;
                let end = self.number()?;
                self.eol()?;
                ranges.push(self.case("liverange", "LIVE", [slot, kind, start, end])?);
            }
        }
        self.list("liverange", ranges)
    }
}

impl Printer<'_> {
    pub(crate) fn body(&mut self, body: Value) -> Result<()> {
        with_stack(|| self.body_inner(body))
    }

    fn body_inner(&mut self, body: Value) -> Result<()> {
        let [decl, frame, pool, tries, ranges, children, code] = self.fields(body, "BODY")?;
        let [name, flags, flags2, scope, source, signature, attributes] =
            self.fields(decl, "BODYDECL")?;
        let [start, end, doc] = self.fields(source, "LINES")?;
        let [temps, cache, locals, statics] = self.fields(frame, "FRAME")?;
        self.out.push_str("body ");
        self.optional_bytes(name)?;
        self.out.push_str(" {\n");
        let (flags, flags2) = (self.u32(flags)?, self.u32(flags2)?);
        if flags != 0 || flags2 != 0 {
            writeln!(self.out, "  flags {flags} {flags2}")?;
        }
        self.body_bytes_row("  scope ", scope)?;
        writeln!(self.out, "  lines {} {}", self.u32(start)?, self.u32(end)?)?;
        self.body_bytes_row("  doc ", doc)?;
        writeln!(
            self.out,
            "  frame {} cache {}",
            self.u32(temps)?,
            self.u32(cache)?
        )?;
        let locals = self.list(locals)?;
        if !locals.is_empty() {
            write!(self.out, "  locals {} [", locals.len())?;
            for (index, &local) in locals.iter().enumerate() {
                if index != 0 {
                    self.out.push_str(", ");
                }
                self.bytes(local)?;
            }
            self.out.push_str("]\n");
        }
        if let Some(signature) = self.opt(signature)? {
            self.signature(signature)?;
        }
        self.attributes(attributes)?;
        if let Some(statics) = self.opt(statics)? {
            self.out.push_str("  statics ");
            self.zval(statics)?;
            self.out.push('\n');
        }
        let pool = self.list(pool)?;
        let children = self.list(children)?;
        let code = self.list(code)?;
        writeln!(
            self.out,
            "  size {} literals {} children {}",
            code.len(),
            pool.len(),
            children.len()
        )?;
        for (index, &entry) in pool.iter().enumerate() {
            self.pool_entry(index, entry)?;
        }
        self.try_regions(tries)?;
        self.live_ranges(ranges)?;
        for &child in children {
            self.body(child)?;
        }
        self.out.push_str("  code\n");
        for (index, &instruction) in code.iter().enumerate() {
            self.instruction(index, instruction)?;
        }
        self.out.push_str("}\n");
        Ok(())
    }

    fn body_bytes_row(&mut self, label: &str, value: Value) -> Result<()> {
        if let Some(bytes) = self.opt(value)? {
            self.out.push_str(label);
            self.bytes(bytes)?;
            self.out.push('\n');
        }
        Ok(())
    }

    fn signature(&mut self, signature: Value) -> Result<()> {
        let [declared, required, returns, args] = self.fields(signature, "ARGS")?;
        writeln!(
            self.out,
            "  args {} required {}",
            self.u32(declared)?,
            self.u32(required)?
        )?;
        if let Some(ty) = self.opt(returns)? {
            self.out.push_str("  returns ");
            self.ztype(ty)?;
            self.out.push('\n');
        }
        for &arg in self.list(args)? {
            let [name, ty, doc] = self.fields(arg, "ARG")?;
            self.out.push_str("  arg ");
            self.bytes(name)?;
            self.out.push(' ');
            self.ztype(ty)?;
            if let Some(doc) = self.opt(doc)? {
                self.out.push_str(" doc ");
                self.bytes(doc)?;
            }
            self.out.push('\n');
        }
        Ok(())
    }

    fn pool_entry(&mut self, index: usize, entry: Value) -> Result<()> {
        write!(self.out, "  lit[{index}] = ")?;
        match self.case(entry)? {
            ("LITERAL", &[value]) => self.zval(value)?,
            ("ATTRIBUTES", &[attributes]) => {
                self.out.push_str("attributes {\n");
                self.attributes(attributes)?;
                self.out.push('}');
            }
            _ => return Err(Error::new("expected a literal pool entry")),
        }
        self.out.push('\n');
        Ok(())
    }

    fn try_regions(&mut self, regions: Value) -> Result<()> {
        let regions = self.list(regions)?;
        if !regions.is_empty() {
            writeln!(self.out, "  tries {}", regions.len())?;
        }
        for &region in regions {
            let [start, catch, finally, end] = self.fields(region, "TRY")?;
            writeln!(
                self.out,
                "  try @{} catch @{} finally @{} end @{}",
                self.u32(start)?,
                self.u32(catch)?,
                self.u32(finally)?,
                self.u32(end)?,
            )?;
        }
        Ok(())
    }

    fn live_ranges(&mut self, ranges: Value) -> Result<()> {
        let ranges = self.list(ranges)?;
        if !ranges.is_empty() {
            writeln!(self.out, "  ranges {}", ranges.len())?;
        }
        for &range in ranges {
            let [slot, kind, start, end] = self.fields(range, "LIVE")?;
            writeln!(
                self.out,
                "  live T{} kind {} @{} @{}",
                self.u32(slot)?,
                self.u32(kind)?,
                self.u32(start)?,
                self.u32(end)?,
            )?;
        }
        Ok(())
    }
}
