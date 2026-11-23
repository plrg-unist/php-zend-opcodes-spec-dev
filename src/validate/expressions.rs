use super::{
    Arena, Error, HashSet, Result, Value, case_fields, check_constant_literal, fields, get, index,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    Expression,
    String(usize),
    Absent,
    Element,
    Unpack,
    Named { placeholder: bool },
    Placeholder { variadic: bool },
    Arguments { placeholders: bool, full: bool },
    Callable,
}

impl Role {
    fn expression(self) -> bool {
        matches!(self, Self::Expression | Self::String(_))
    }

    fn class_name(self) -> bool {
        matches!(self, Self::String(0 | 4 | 8))
    }
}

fn valid_flags(tag: &str, flags: usize) -> bool {
    match tag {
        "VALUE" => flags <= u16::MAX.into(),
        "CONSTANT" => matches!(flags, 0 | 2048),
        "BINARY" => matches!(flags, 1..=12 | 15..=21 | 170),
        "UNARY" => matches!(flags, 13 | 14),
        "CAST" => matches!(flags, 4..=8 | 18),
        "CLASS_CONSTANT" => flags == 512,
        "CLASS_NAME" => matches!(flags, 1 | 2),
        "ARRAY" => matches!(flags, 0 | 2 | 3),
        "CONDITIONAL" | "DIMENSION" | "ARGUMENTS" | "PLACEHOLDER" => flags <= 1,
        _ => flags == 0,
    }
}

struct Check<'a> {
    arena: &'a Arena,
    allow_objects: bool,
    closure_depth: usize,
    pending: &'a mut Vec<(Value, usize)>,
    literals: &'a mut HashSet<Value>,
}

impl Check<'_> {
    fn node(&mut self, v: Value, depth: usize) -> Result<Role> {
        use Role::*;
        if depth > 256 {
            return Err(Error::new("constant expression nesting exceeds 256"));
        }
        let arena = self.arena;
        let (tag, args) = case_fields(arena, v)?;
        if (tag, args) == ("ABSENT", &[]) {
            return Ok(Absent);
        }
        let &[flags, _, ref children @ ..] = args else {
            return Err(Error::new("invalid constant expression fields"));
        };
        let flags = index(arena, flags)?;
        if !valid_flags(tag, flags) {
            return Err(Error::new(format!(
                "invalid constant expression flags {flags} for {tag}"
            )));
        }
        if !self.allow_objects && (tag == "NEW" || (tag == "CAST" && flags == 8)) {
            return Err(Error::new(
                "dynamic object creation in enum backing expression",
            ));
        }
        let mut child = |v| crate::with_stack(|| self.node(v, depth + 1));
        match (tag, children) {
            ("VALUE", &[v]) => {
                let kind = case_fields(arena, v)?.0;
                if kind == "UNDEF" {
                    return Err(Error::new("undefined value in a constant expression"));
                }
                check_constant_literal(arena, v, self.literals)?;
                self.pending.push((v, self.closure_depth));
                Ok(if kind == "STRING" {
                    String(flags)
                } else {
                    Expression
                })
            }
            ("CONSTANT", &[name]) => {
                let name = get::list(arena, &name)?;
                if name.is_empty()
                    || name
                        .iter()
                        .any(|&b| !matches!(index(arena, b), Ok(1..=255)))
                {
                    return Err(Error::new("invalid constant name"));
                }
                Ok(Expression)
            }
            ("CLOSURE", &[body]) => {
                let [decl, _, _, _, _, _, _] = fields(arena, body, "BODY")?;
                let [name, flags, _, _, _, _, _] = fields(arena, decl, "BODYDECL")?;
                let name = get::opt(arena, &name)?
                    .map(|v| get::list(arena, &v))
                    .transpose()?;
                // ZEND_ACC_CLOSURE | ZEND_ACC_STATIC on the pinned engine.
                const STATIC_CLOSURE: usize = (1 << 22) | (1 << 4);
                if name.is_none_or(|name| name.is_empty())
                    || index(arena, flags)? & STATIC_CLOSURE != STATIC_CLOSURE
                {
                    return Err(Error::new(
                        "constant closure requires a named static closure body",
                    ));
                }
                if self.closure_depth >= 64 {
                    return Err(Error::new("constant closure nesting exceeds 64"));
                }
                self.pending.push((body, self.closure_depth + 1));
                Ok(Expression)
            }
            ("ARRAY", &[items]) => {
                for &item in get::list(arena, &items)? {
                    if !matches!(child(item)?, Element | Unpack) {
                        return Err(Error::new("expected array element or unpack"));
                    }
                }
                Ok(Expression)
            }
            ("ARGUMENTS", &[items]) => {
                let items = get::list(arena, &items)?;
                let (mut named, mut placeholders, mut full) = (false, false, false);
                for (i, &item) in items.iter().enumerate() {
                    match child(item)? {
                        Named { placeholder } => {
                            named = true;
                            placeholders |= placeholder;
                        }
                        Placeholder { variadic: true } if i + 1 == items.len() => {
                            placeholders = true;
                            full = items.len() == 1;
                        }
                        Placeholder { variadic: false } if !named => placeholders = true,
                        v if v.expression() && !named => {}
                        _ => return Err(Error::new("invalid constant expression argument")),
                    }
                }
                if named != (flags == 1) {
                    return Err(Error::new("inconsistent named argument flag"));
                }
                Ok(Arguments { placeholders, full })
            }
            ("CALLABLE", &[file, name, args]) => {
                let Arguments {
                    placeholders: true,
                    full,
                } = child(args)?
                else {
                    return Err(Error::new("expected a callable placeholder"));
                };
                if get::opt(arena, &file)?.is_none() != full
                    || get::opt(arena, &name)?.is_none() != full
                {
                    return Err(Error::new(
                        "inconsistent partial application name or filename",
                    ));
                }
                Ok(Callable)
            }
            ("PLACEHOLDER", []) => Ok(Placeholder {
                variadic: flags == 1,
            }),
            ("CURRENT_CLASS", []) => Ok(Expression),
            (_, children) => {
                let roles = children
                    .iter()
                    .copied()
                    .map(child)
                    .collect::<Result<Vec<_>>>()?;
                match (tag, roles.as_slice()) {
                    ("CLASS_NAME", [Absent]) => Ok(Expression),
                    ("CLASS_CONSTANT", [String(_), value]) if value.expression() => Ok(Expression),
                    ("NAMED", [String(_), Placeholder { variadic: false }]) => {
                        Ok(Named { placeholder: true })
                    }
                    ("NAMED", [String(_), value]) if value.expression() => {
                        Ok(Named { placeholder: false })
                    }
                    (
                        "NEW",
                        [
                            class,
                            Arguments {
                                placeholders: false,
                                ..
                            },
                        ],
                    ) if class.class_name() => Ok(Expression),
                    ("CALL", [String(0..=2), Callable]) => Ok(Expression),
                    ("STATIC_CALL", [class, String(_), Callable]) if class.class_name() => {
                        Ok(Expression)
                    }
                    ("ELEMENT", [val, key])
                        if val.expression() && (*key == Absent || key.expression()) =>
                    {
                        Ok(Element)
                    }
                    ("CONDITIONAL", [a, b, c])
                        if a.expression() && (*b == Absent || b.expression()) && c.expression() =>
                    {
                        Ok(Expression)
                    }
                    ("UNPACK", [val]) if val.expression() => Ok(Unpack),
                    ("UNARY" | "PLUS" | "MINUS" | "CAST", [val]) if val.expression() => {
                        Ok(Expression)
                    }
                    (
                        "BINARY" | "GREATER" | "GREATER_EQUAL" | "AND" | "OR" | "COALESCE"
                        | "DIMENSION" | "PROPERTY" | "NULLSAFE_PROPERTY",
                        [a, b],
                    ) if a.expression() && b.expression() => Ok(Expression),
                    _ => Err(Error::new(format!(
                        "invalid constant expression child for {tag}"
                    ))),
                }
            }
        }
    }
}

pub(super) fn check(
    arena: &Arena,
    root: Value,
    allow_objects: bool,
    closure_depth: usize,
    pending: &mut Vec<(Value, usize)>,
    literals: &mut HashSet<Value>,
) -> Result<()> {
    let mut check = Check {
        arena,
        allow_objects,
        closure_depth,
        pending,
        literals,
    };
    if !check.node(root, 0)?.expression() {
        return Err(Error::new("expected a constant expression"));
    }
    Ok(())
}
