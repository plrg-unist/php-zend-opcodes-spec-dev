use crate::{Error, Parser, Printer, Result, Value, with_stack};
use std::fmt::Write;

enum Shape {
    Leaf,
    Unary,
    Binary,
    Ternary,
    List,
    Literal,
    Name,
    Body,
    Callable,
}

fn shape(tag: &str) -> Option<Shape> {
    Some(match tag {
        "CURRENT_CLASS" | "PLACEHOLDER" => Shape::Leaf,
        "CLASS_NAME" | "UNARY" | "PLUS" | "MINUS" | "CAST" | "UNPACK" => Shape::Unary,
        "CLASS_CONSTANT" | "BINARY" | "GREATER" | "GREATER_EQUAL" | "AND" | "OR" | "COALESCE"
        | "DIMENSION" | "PROPERTY" | "NULLSAFE_PROPERTY" | "ELEMENT" | "NEW" | "NAMED" | "CALL" => {
            Shape::Binary
        }
        "CONDITIONAL" | "STATIC_CALL" => Shape::Ternary,
        "ARRAY" | "ARGUMENTS" => Shape::List,
        "VALUE" => Shape::Literal,
        "CONSTANT" => Shape::Name,
        "CLOSURE" => Shape::Body,
        "CALLABLE" => Shape::Callable,
        _ => return None,
    })
}

impl Parser<'_> {
    fn metadata_bytes(&mut self) -> Result<Value> {
        let bytes = if self.take("none") {
            None
        } else {
            Some(self.bytes()?)
        };
        self.opt("bytes", bytes)
    }

    pub(crate) fn attributes(&mut self) -> Result<Value> {
        let mut attrs = Vec::new();
        while self.take("attribute") {
            let name = self.bytes()?;
            let offset = self.number()?;
            let line = self.number()?;
            let flags = self.number()?;
            let count = self.u32()?;
            let error = self.metadata_bytes()?;
            self.eol()?;
            let mut args = Vec::new();
            for _ in 0..count {
                self.expect("attrarg")?;
                let name = self.metadata_bytes()?;
                let arg = self.zval()?;
                self.eol()?;
                args.push(self.case("attrarg", "ATTRARG", [name, arg])?);
            }
            let args = self.list("attrarg", args)?;
            attrs.push(self.case(
                "attribute",
                "ATTRIBUTE",
                [name, offset, line, flags, error, args],
            )?);
        }
        self.list("attribute", attrs)
    }

    pub(crate) fn expr(&mut self) -> Result<Value> {
        with_stack(|| self.expression_node())
    }

    fn expression_node(&mut self) -> Result<Value> {
        if self.take("none") {
            return self.case("expr", "ABSENT", []);
        }
        let name = self.scan(|b| b.is_ascii_lowercase() || b == b'_');
        let tag = name.to_ascii_uppercase();
        let shape = shape(&tag).ok_or_else(|| self.error("expected a constant expression node"))?;
        self.expect("(")?;
        let flags = self.number()?;
        let line = self.number()?;
        let node = match shape {
            Shape::Leaf => self.case("expr", &tag, [flags, line])?,
            Shape::Unary => {
                let child = self.expr()?;
                self.case("expr", &tag, [flags, line, child])?
            }
            Shape::Binary => {
                let left = self.expr()?;
                let right = self.expr()?;
                self.case("expr", &tag, [flags, line, left, right])?
            }
            Shape::Ternary => {
                let first = self.expr()?;
                let second = self.expr()?;
                let third = self.expr()?;
                self.case("expr", &tag, [flags, line, first, second, third])?
            }
            Shape::List => {
                let count = self.u32()?;
                let mut children = Vec::new();
                for _ in 0..count {
                    children.push(self.expr()?);
                }
                let items = self.list("expr", children)?;
                self.case("expr", &tag, [flags, line, items])?
            }
            Shape::Literal => {
                let value = self.zval()?;
                self.case("expr", &tag, [flags, line, value])?
            }
            Shape::Name => {
                let name = self.bytes()?;
                self.case("expr", &tag, [flags, line, name])?
            }
            Shape::Body => {
                self.eol()?;
                let body = self.body()?;
                self.case("expr", &tag, [flags, line, body])?
            }
            Shape::Callable => {
                let filename = self.metadata_bytes()?;
                let name = self.metadata_bytes()?;
                let arguments = self.expr()?;
                self.case("expr", &tag, [flags, line, filename, name, arguments])?
            }
        };
        self.expect(")")?;
        Ok(node)
    }
}

impl Printer<'_> {
    fn metadata_bytes(&mut self, value: Value) -> Result<()> {
        if let Some(bytes) = self.opt(value)? {
            self.bytes(bytes)?;
        } else {
            self.out.push_str("none");
        }
        Ok(())
    }

    pub(crate) fn attributes(&mut self, attrs: Value) -> Result<()> {
        for &attr in self.list(attrs)? {
            let [name, offset, line, flags, validation_error, args] =
                self.fields(attr, "ATTRIBUTE")?;
            let args = self.list(args)?;
            let offset = self.u32(offset)?;
            let line = self.u32(line)?;
            let flags = self.u32(flags)?;
            self.out.push_str("attribute ");
            self.bytes(name)?;
            write!(self.out, " {offset} {line} {flags} {} ", args.len())?;
            self.metadata_bytes(validation_error)?;
            self.out.push('\n');
            for &arg in args {
                let [name, value] = self.fields(arg, "ATTRARG")?;
                self.out.push_str("attrarg ");
                self.metadata_bytes(name)?;
                self.out.push(' ');
                self.zval(value)?;
                self.out.push('\n');
            }
        }
        Ok(())
    }

    pub(crate) fn expr(&mut self, value: Value) -> Result<()> {
        with_stack(|| self.expression_node(value))
    }

    fn expression_node(&mut self, value: Value) -> Result<()> {
        let (tag, fields) = self.case(value)?;
        if tag == "ABSENT" && fields.is_empty() {
            self.out.push_str("none");
            return Ok(());
        }
        let Some(shape) = shape(tag) else {
            return Err(Error::new("expected a constant expression constructor"));
        };
        let [flags, line, children @ ..] = fields else {
            return Err(Error::new(
                "constant expression requires flags and source line",
            ));
        };
        let flags = self.u32(*flags)?;
        let line = self.u32(*line)?;
        write!(self.out, "{}({flags} {line}", tag.to_ascii_lowercase())?;
        match (shape, children) {
            (Shape::Leaf, [])
            | (Shape::Unary, [_])
            | (Shape::Binary, [_, _])
            | (Shape::Ternary, [_, _, _]) => {
                for &child in children {
                    self.out.push(' ');
                    self.expr(child)?;
                }
            }
            (Shape::List, [items]) => {
                let items = self.list(*items)?;
                write!(self.out, " {}", items.len())?;
                for &child in items {
                    self.out.push(' ');
                    self.expr(child)?;
                }
            }
            (Shape::Literal, [value]) => {
                self.out.push(' ');
                self.zval(*value)?;
            }
            (Shape::Name, [name]) => {
                self.out.push(' ');
                self.bytes(*name)?;
            }
            (Shape::Body, [body]) => {
                self.out.push('\n');
                self.body(*body)?;
            }
            (Shape::Callable, [filename, name, args]) => {
                self.out.push(' ');
                self.metadata_bytes(*filename)?;
                self.out.push(' ');
                self.metadata_bytes(*name)?;
                self.out.push(' ');
                self.expr(*args)?;
            }
            _ => return Err(Error::new("incorrect constant expression fields")),
        }
        self.out.push(')');
        Ok(())
    }
}
