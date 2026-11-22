use std::fmt::Write;

use crate::{Error, Parser, Printer, Result, Value};

impl Parser<'_> {
    pub(crate) fn unit(&mut self) -> Result<Value> {
        self.expect("zend-opcodes")?;
        self.eol()?;
        self.expect("file")?;
        let file = self.name_row()?;
        let halt = if self.take("halt") {
            let offset = u64::try_from(self.i64()?)
                .map_err(|_| self.error("expected a nonnegative halt offset"))?;
            self.eol()?;
            Some(self.nat(offset)?)
        } else {
            None
        };
        let halt = self.opt("nat", halt)?;
        let globals = self.unit_rows("autoglobal", "bytes", Self::name_row)?;
        self.expect("main")?;
        self.eol()?;
        let main = self.body()?;
        let functions = self.unit_rows("function", "namedBody", Self::named_body)?;
        let classes = self.unit_rows("class", "classDecl", Self::class_decl)?;
        self.expect("end")?;
        self.eol()?;
        self.case(
            "unit",
            "UNIT",
            [file, halt, globals, main, functions, classes],
        )
    }

    fn unit_rows(
        &mut self,
        keyword: &str,
        element_type: &str,
        read: fn(&mut Self) -> Result<Value>,
    ) -> Result<Value> {
        let mut rows = Vec::new();
        while self.take(keyword) {
            rows.push(read(self)?);
        }
        self.list(element_type, rows)
    }

    fn name_row(&mut self) -> Result<Value> {
        let name = self.bytes()?;
        self.eol()?;
        Ok(name)
    }

    fn named_body(&mut self) -> Result<Value> {
        let key = self.name_row()?;
        let body = self.body()?;
        self.case("namedBody", "NAMEDBODY", [key, body])
    }

    fn class_decl(&mut self) -> Result<Value> {
        let header = self.class_header()?;
        let attributes = self.attributes()?;
        let backing = if self.take("enum") {
            let tag = if self.take("pure") {
                "PURE"
            } else if self.take("int") {
                "INTBACKING"
            } else {
                self.expect("string")?;
                "STRINGBACKING"
            };
            self.eol()?;
            Some(self.case("enumBacking", tag, [])?)
        } else {
            None
        };
        let backing = self.opt("enumBacking", backing)?;
        let interfaces = self.unit_rows("implements", "bytes", Self::name_row)?;
        let traits = self.class_traits()?;
        let properties = self.unit_rows("property", "propertyDecl", Self::property_decl)?;
        let mut constants = Vec::new();
        loop {
            if self.take("case") {
                constants.push(self.enum_case()?);
            } else if self.take("constant") {
                let member = self.class_member()?;
                constants.push(self.case("classConstant", "CONSTANT", [member])?);
            } else {
                break;
            }
        }
        let constants = self.list("classConstant", constants)?;
        let methods = self.unit_rows("method", "namedBody", Self::named_body)?;
        self.expect("}")?;
        self.eol()?;
        self.case(
            "classDecl",
            "CLASS",
            [
                header, attributes, backing, interfaces, traits, properties, constants, methods,
            ],
        )
    }

    fn class_header(&mut self) -> Result<Value> {
        let key = self.bytes()?;
        let name = self.bytes()?;
        let link_tag = if self.take("linked") {
            "LINKED"
        } else {
            self.expect("unlinked")?;
            "UNLINKED"
        };
        let link = self.case("classLink", link_tag, [])?;
        let flags = self.number()?;
        let first = self.number()?;
        let last = self.number()?;
        let parent = self.optional_bytes()?;
        let doc = self.optional_bytes()?;
        self.expect("{")?;
        self.eol()?;
        self.case(
            "classHeader",
            "CLASSHEADER",
            [key, name, link, flags, first, last, parent, doc],
        )
    }

    fn class_traits(&mut self) -> Result<Value> {
        let names = self.unit_rows("use", "bytes", Self::name_row)?;
        let aliases = self.unit_rows("alias", "traitAlias", Self::trait_alias)?;
        let precedences = self.unit_rows("instead", "traitPrecedence", Self::trait_precedence)?;
        self.case("traitUse", "TRAITS", [names, aliases, precedences])
    }

    fn trait_alias(&mut self) -> Result<Value> {
        let owner = self.optional_bytes()?;
        let name = self.bytes()?;
        let flags = self.number()?;
        let alias = self.optional_bytes()?;
        self.eol()?;
        self.case("traitAlias", "ALIAS", [owner, name, flags, alias])
    }

    fn trait_precedence(&mut self) -> Result<Value> {
        let owner = self.bytes()?;
        let method = self.bytes()?;
        let mut excluded = Vec::new();
        for _ in 0..self.u32()? {
            excluded.push(self.bytes()?);
        }
        self.eol()?;
        let excluded = self.list("bytes", excluded)?;
        self.case("traitPrecedence", "INSTEAD", [owner, method, excluded])
    }

    fn class_member(&mut self) -> Result<Value> {
        let name = self.bytes()?;
        let flags = self.number()?;
        let ty = self.ztype()?;
        let doc = self.optional_bytes()?;
        self.expect("=")?;
        let value = self.zval()?;
        self.eol()?;
        let attributes = self.attributes()?;
        self.case(
            "classMember",
            "MEMBER",
            [name, flags, ty, doc, value, attributes],
        )
    }

    fn property_hook(&mut self, keyword: &str) -> Result<Value> {
        let body = if self.take(keyword) {
            self.eol()?;
            Some(self.body()?)
        } else {
            None
        };
        self.opt("body", body)
    }

    fn property_decl(&mut self) -> Result<Value> {
        let member = self.class_member()?;
        let get = self.property_hook("get")?;
        let set = self.property_hook("set")?;
        self.case("propertyDecl", "PROPERTY", [member, get, set])
    }

    fn enum_case(&mut self) -> Result<Value> {
        let id = self.number()?;
        let name = self.bytes()?;
        let flags = self.number()?;
        let line = self.number()?;
        let doc = self.optional_bytes()?;
        self.expect("=")?;
        let backing = self.expr()?;
        self.eol()?;
        let attributes = self.attributes()?;
        self.case(
            "classConstant",
            "ENUMCASE",
            [id, name, flags, line, doc, backing, attributes],
        )
    }
}

impl Printer<'_> {
    pub(crate) fn unit(&mut self, value: Value) -> Result<()> {
        let [file, halt, globals, main, functions, classes] = self.fields(value, "UNIT")?;
        self.out.push_str("zend-opcodes\nfile ");
        self.bytes(file)?;
        self.out.push('\n');
        if let Some(offset) = self.opt(halt)? {
            let offset = i64::try_from(self.nat(offset)?)
                .map_err(|_| Error::new("halt offset exceeds the signed integer range"))?;
            writeln!(self.out, "halt {offset}")?;
        }
        self.name_rows("autoglobal", globals)?;
        self.out.push_str("main\n");
        self.body(main)?;
        self.named_bodies("function", functions)?;
        for &class in self.list(classes)? {
            self.class_decl(class)?;
        }
        self.out.push_str("end\n");
        Ok(())
    }

    fn name_rows(&mut self, keyword: &str, rows: Value) -> Result<()> {
        for &name in self.list(rows)? {
            write!(self.out, "{keyword} ")?;
            self.bytes(name)?;
            self.out.push('\n');
        }
        Ok(())
    }

    fn named_bodies(&mut self, keyword: &str, rows: Value) -> Result<()> {
        for &row in self.list(rows)? {
            let [key, body] = self.fields(row, "NAMEDBODY")?;
            write!(self.out, "{keyword} ")?;
            self.bytes(key)?;
            self.out.push('\n');
            self.body(body)?;
        }
        Ok(())
    }

    fn class_decl(&mut self, value: Value) -> Result<()> {
        let [
            header,
            attributes,
            backing,
            interfaces,
            traits,
            properties,
            constants,
            methods,
        ] = self.fields(value, "CLASS")?;
        self.class_header(header)?;
        self.attributes(attributes)?;
        if let Some(backing) = self.opt(backing)? {
            let label = match self.case(backing)? {
                ("PURE", []) => "pure",
                ("INTBACKING", []) => "int",
                ("STRINGBACKING", []) => "string",
                _ => return Err(Error::new("expected an enum backing type")),
            };
            writeln!(self.out, "enum {label}")?;
        }
        self.name_rows("implements", interfaces)?;
        self.class_traits(traits)?;
        for &property in self.list(properties)? {
            let [member, get, set] = self.fields(property, "PROPERTY")?;
            self.out.push_str("property ");
            self.class_member(member)?;
            for (keyword, hook) in [("get", get), ("set", set)] {
                if let Some(body) = self.opt(hook)? {
                    writeln!(self.out, "{keyword}")?;
                    self.body(body)?;
                }
            }
        }
        for &constant in self.list(constants)? {
            self.class_constant(constant)?;
        }
        self.named_bodies("method", methods)?;
        self.out.push_str("}\n");
        Ok(())
    }

    fn class_header(&mut self, header: Value) -> Result<()> {
        let [key, name, link, flags, first, last, parent, doc] =
            self.fields(header, "CLASSHEADER")?;
        let link_name = match self.case(link)? {
            ("LINKED", []) => "linked",
            ("UNLINKED", []) => "unlinked",
            _ => return Err(Error::new("expected class linkage")),
        };
        let flags = self.u32(flags)?;
        let first = self.u32(first)?;
        let last = self.u32(last)?;
        self.out.push_str("class ");
        self.bytes(key)?;
        self.out.push(' ');
        self.bytes(name)?;
        write!(self.out, " {link_name} {flags} {first} {last} ")?;
        self.optional_bytes(parent)?;
        self.out.push(' ');
        self.optional_bytes(doc)?;
        self.out.push_str(" {\n");
        Ok(())
    }

    fn class_traits(&mut self, value: Value) -> Result<()> {
        let [names, aliases, precedences] = self.fields(value, "TRAITS")?;
        self.name_rows("use", names)?;
        for &alias in self.list(aliases)? {
            let [owner, name, modifiers, alias] = self.fields(alias, "ALIAS")?;
            let modifiers = self.u32(modifiers)?;
            self.out.push_str("alias ");
            self.optional_bytes(owner)?;
            self.out.push(' ');
            self.bytes(name)?;
            write!(self.out, " {modifiers} ")?;
            self.optional_bytes(alias)?;
            self.out.push('\n');
        }
        for &precedence in self.list(precedences)? {
            let [owner, method, excluded] = self.fields(precedence, "INSTEAD")?;
            self.out.push_str("instead ");
            self.bytes(owner)?;
            self.out.push(' ');
            self.bytes(method)?;
            let excluded = self.list(excluded)?;
            write!(self.out, " {}", excluded.len())?;
            for &name in excluded {
                self.out.push(' ');
                self.bytes(name)?;
            }
            self.out.push('\n');
        }
        Ok(())
    }

    fn class_member(&mut self, member: Value) -> Result<()> {
        let [name, flags, ty, doc, value, attributes] = self.fields(member, "MEMBER")?;
        let flags = self.u32(flags)?;
        self.bytes(name)?;
        write!(self.out, " {flags} ")?;
        self.ztype(ty)?;
        self.out.push(' ');
        self.optional_bytes(doc)?;
        self.out.push_str(" = ");
        self.zval(value)?;
        self.out.push('\n');
        self.attributes(attributes)
    }

    fn class_constant(&mut self, constant: Value) -> Result<()> {
        match self.case(constant)? {
            ("CONSTANT", &[member]) => {
                self.out.push_str("constant ");
                self.class_member(member)
            }
            ("ENUMCASE", &[id, name, flags, line, doc, expr, attributes]) => {
                let id = self.u32(id)?;
                let flags = self.u32(flags)?;
                let line = self.u32(line)?;
                write!(self.out, "case {id} ")?;
                self.bytes(name)?;
                write!(self.out, " {flags} {line} ")?;
                self.optional_bytes(doc)?;
                self.out.push_str(" = ");
                self.expr(expr)?;
                self.out.push('\n');
                self.attributes(attributes)
            }
            _ => Err(Error::new("expected a class constant or enum case")),
        }
    }
}
