use p4spectec::lang::{
    common::notation::atom::Atom,
    data::{notation::flat::MixopKind, value::flat::get},
};

use crate::{Arena, Error, Result, Value};

pub(crate) fn case_fields(arena: &Arena, value: Value) -> Result<(&str, &[Value])> {
    let case = get::case(arena, &value)?;
    let mixops = &arena.mixop;
    let head = match mixops.kind(*case.mixop()) {
        MixopKind::Atom(atom) => atom,
        MixopKind::Seq(parts) => {
            let Some((head, args)) = parts.split_first() else {
                return Err(Error::new("empty constructor notation"));
            };
            if args.is_empty()
                || !args
                    .iter()
                    .all(|arg| matches!(mixops.kind(*arg), MixopKind::Arg))
            {
                return Err(Error::new("expected positional constructor arguments"));
            }
            match mixops.kind(*head) {
                MixopKind::Atom(atom) => atom,
                _ => return Err(Error::new("expected a constructor name")),
            }
        }
        _ => return Err(Error::new("expected a Zend AST constructor")),
    };
    match &head.node {
        Atom::Keyword(tag) => Ok((tag, case.args())),
        _ => Err(Error::new("expected a constructor keyword")),
    }
}

pub(crate) fn fields<const N: usize>(arena: &Arena, value: Value, tag: &str) -> Result<[Value; N]> {
    let (actual, args) = case_fields(arena, value)?;
    if tag != actual {
        return Err(Error::new(format!("expected {tag}, found {actual}")));
    }
    args.try_into()
        .map_err(|_| Error::new(format!("{tag} expects {N} fields, got {}", args.len())))
}
