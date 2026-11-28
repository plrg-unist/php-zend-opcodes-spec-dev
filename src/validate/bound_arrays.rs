//! Validate array literals embedded as constants by CALLABLE_CONVERT_PARTIAL.

use std::collections::{HashMap, HashSet};

use super::{Instruction, Operand, index};
use crate::{Arena, Error, Result, Value, ast::fields, case_fields};
use p4spectec::lang::data::value::flat::get;

#[derive(Clone, Copy)]
pub(super) struct Declarations<'a> {
    pub(super) functions: &'a [Value],
    pub(super) classes: &'a [Value],
    pub(super) scope: Option<Value>,
}

fn bytes(arena: &Arena, value: Value) -> Result<Vec<u8>> {
    get::list(arena, &value)?
        .iter()
        .map(|&byte| u8::try_from(index(arena, byte)?).map_err(|_| Error::new("invalid name byte")))
        .collect()
}

fn name_literal(arena: &Arena, pool: &[Value], index: usize) -> Result<Option<Vec<u8>>> {
    if let Some(&entry) = pool.get(index)
        && let ("LITERAL", &[value]) = case_fields(arena, entry)?
        && let ("STRING", &[name]) = case_fields(arena, value)?
    {
        return Ok(Some(bytes(arena, name)?));
    }
    Ok(None)
}

fn named_body(arena: &Arena, bodies: &[Value], name: &[u8]) -> Result<Option<Value>> {
    for &item in bodies {
        let [key, body] = fields(arena, item, "NAMEDBODY")?;
        if bytes(arena, key)? == name {
            return Ok(Some(body));
        }
    }
    Ok(None)
}

fn returns_never(arena: &Arena, body: Value) -> Result<bool> {
    let [decl, _, _, _, _, _, _] = fields(arena, body, "BODY")?;
    let [_, flags, _, _, _, signature, _] = fields(arena, decl, "BODYDECL")?;
    const ZEND_ACC_HAS_RETURN_TYPE: usize = 1 << 13;
    const MAY_BE_NEVER: usize = 1 << 17;
    if index(arena, flags)? & ZEND_ACC_HAS_RETURN_TYPE != 0
        && let Some(signature) = get::opt(arena, &signature)?
    {
        let [_, _, result, _] = fields(arena, signature, "ARGS")?;
        if let Some(result) = get::opt(arena, &result)?
            && let Some(&mask) = case_fields(arena, result)?.1.first()
        {
            return Ok(index(arena, mask)? & MAY_BE_NEVER != 0);
        }
    }
    Ok(false)
}

enum TargetClass {
    Declaration(Value),
    Key(Vec<u8>),
}

enum Member {
    Method(Vec<u8>),
    Hook(Vec<u8>, usize),
}

fn linked_parent(arena: &Arena, header: Value) -> Result<Option<TargetClass>> {
    let [_, _, link, _, _, _, parent, _] = fields(arena, header, "CLASSHEADER")?;
    if case_fields(arena, link)?.0 == "LINKED"
        && let Some(parent) = get::opt(arena, &parent)?
    {
        return Ok(Some(TargetClass::Key(
            bytes(arena, parent)?.to_ascii_lowercase(),
        )));
    }
    Ok(None)
}

impl Declarations<'_> {
    fn member_is_never(&self, arena: &Arena, class: TargetClass, member: Member) -> Result<bool> {
        let mut pending = vec![class];
        let mut visited = HashSet::new();
        while let Some(class) = pending.pop() {
            let decl = match class {
                TargetClass::Declaration(decl) => decl,
                TargetClass::Key(key) => {
                    // These are the only internal methods with a never return type on the pinned CLI.
                    if key == b"emptyiterator"
                        && let Member::Method(ref name) = member
                    {
                        match name.as_slice() {
                            b"current" | b"key" => return Ok(true),
                            b"next" | b"valid" | b"rewind" => return Ok(false),
                            _ => continue,
                        }
                    }
                    let mut found = None;
                    for &decl in self.classes {
                        let [header, _, _, _, _, _, _, _] = fields(arena, decl, "CLASS")?;
                        let [name, _, _, _, _, _, _, _] = fields(arena, header, "CLASSHEADER")?;
                        if bytes(arena, name)? == key {
                            found = Some(decl);
                            break;
                        }
                    }
                    let Some(decl) = found else { continue };
                    decl
                }
            };
            if !visited.insert(decl) {
                continue;
            }
            let [header, _, _, interfaces, _, properties, _, methods] =
                fields(arena, decl, "CLASS")?;
            let body = match &member {
                Member::Method(name) => named_body(arena, get::list(arena, &methods)?, name)?,
                Member::Hook(name, kind) => {
                    let mut found = None;
                    for &property in get::list(arena, &properties)? {
                        let [member, get_hook, set_hook] = fields(arena, property, "PROPERTY")?;
                        let [key, _, _, _, _, _] = fields(arena, member, "MEMBER")?;
                        if bytes(arena, key)? == *name {
                            found =
                                get::opt(arena, if *kind == 0 { &get_hook } else { &set_hook })?;
                            break;
                        }
                    }
                    found
                }
            };
            if let Some(body) = body {
                return returns_never(arena, body);
            }
            let [_, _, link, _, _, _, _, _] = fields(arena, header, "CLASSHEADER")?;
            if case_fields(arena, link)?.0 == "LINKED" {
                for &interface in get::list(arena, &interfaces)?.iter().rev() {
                    pending.push(TargetClass::Key(
                        bytes(arena, interface)?.to_ascii_lowercase(),
                    ));
                }
                if let Some(parent) = linked_parent(arena, header)? {
                    pending.push(parent);
                }
            }
        }
        Ok(false)
    }

    fn call_is_never(&self, arena: &Arena, pool: &[Value], init: &Instruction<'_>) -> Result<bool> {
        let scope = self.scope;
        let parent = |scope| {
            let [header, _, _, _, _, _, _, _] = fields(arena, scope, "CLASS")?;
            linked_parent(arena, header)
        };
        if init.name == "INIT_PARENT_PROPERTY_HOOK_CALL"
            && let (Some(scope), Operand::Literal(prop), Operand::Immediate(kind @ 0..=1)) =
                (scope, init.left, init.right)
            && let (Some(class), Some(name)) = (parent(scope)?, name_literal(arena, pool, prop)?)
        {
            return self.member_is_never(arena, class, Member::Hook(name, kind));
        }
        let Operand::Literal(index) = init.right else {
            return Ok(false);
        };
        if init.name == "INIT_FCALL"
            && let Some(name) = name_literal(arena, pool, index)?
        {
            if matches!(name.as_slice(), b"exit" | b"die") {
                return Ok(true);
            }
            return match named_body(arena, self.functions, &name)? {
                Some(body) => returns_never(arena, body),
                None => Ok(false),
            };
        }
        let class = match (init.name, init.left) {
            ("INIT_METHOD_CALL", Operand::Unused) => scope.map(TargetClass::Declaration),
            ("INIT_STATIC_METHOD_CALL", Operand::Literal(index)) => {
                name_literal(arena, pool, index + 1)?.map(TargetClass::Key)
            }
            ("INIT_STATIC_METHOD_CALL", Operand::Immediate(flags)) if flags & 15 == 1 => {
                scope.map(TargetClass::Declaration)
            }
            ("INIT_STATIC_METHOD_CALL", Operand::Immediate(flags)) if flags & 15 == 2 => {
                scope.map(parent).transpose()?.flatten()
            }
            _ => None,
        };
        if let Some(class) = class
            && let Some(name) = name_literal(arena, pool, index + 1)?
        {
            return self.member_is_never(arena, class, Member::Method(name));
        }
        Ok(false)
    }
}

fn enclosing_call_after(
    name: &str,
    pc: usize,
    parent: Option<usize>,
    parents: &[Option<Option<usize>>],
) -> Option<usize> {
    match name {
        "INIT_FCALL"
        | "INIT_FCALL_BY_NAME"
        | "INIT_NS_FCALL_BY_NAME"
        | "INIT_DYNAMIC_CALL"
        | "INIT_METHOD_CALL"
        | "INIT_STATIC_METHOD_CALL"
        | "INIT_PARENT_PROPERTY_HOOK_CALL"
        | "INIT_USER_CALL"
        | "NEW" => Some(pc),
        "DO_ICALL"
        | "DO_UCALL"
        | "DO_FCALL"
        | "DO_FCALL_BY_NAME"
        | "CALLABLE_CONVERT"
        | "CALLABLE_CONVERT_PARTIAL" => parent.and_then(|init| parents[init].flatten()),
        _ => parent,
    }
}

fn mutable_argument(arena: &Arena, pool: &[Value], op: &Instruction<'_>) -> Result<u32> {
    if matches!(op.name, "SEND_VAL" | "SEND_VAL_EX" | "SEND_USER")
        && let (Operand::Literal(index), Operand::Immediate(pos @ 1..=32)) = (op.left, op.right)
        && let ("LITERAL", &[value]) = case_fields(arena, pool[index])?
    {
        let array = match case_fields(arena, value)? {
            ("SHARED_ARRAY", &[_, array]) => array,
            _ => value,
        };
        if let ("ARRAY", &[_, _, _, _, flag]) = case_fields(arena, array)?
            && !get::bool(arena, &flag)?
        {
            return Ok(1 << (pos - 1));
        }
    }
    Ok(0)
}

pub(super) fn check(
    arena: &Arena,
    pool: &[Value],
    code: &[Instruction<'_>],
    tries: Value,
    declarations: &Declarations<'_>,
) -> Result<()> {
    // A named-placeholder literal carries its last bound-mask write for every user of that pool slot.
    let mut named_masks = HashMap::new();
    for op in code {
        if op.name == "CALLABLE_CONVERT_PARTIAL"
            && let Operand::Literal(index) = op.right
        {
            named_masks.insert(index, op.bound_mask);
        }
    }
    let mask = |op: &Instruction<'_>| match (op.name, op.right) {
        ("CALLABLE_CONVERT_PARTIAL", Operand::Literal(index)) => named_masks[&index],
        ("CALLABLE_CONVERT_PARTIAL", Operand::Immediate(_)) => op.bound_mask,
        _ => 0,
    };
    let selected = code.iter().map(mask).fold(0, |a, b| a | b);
    if selected == 0 {
        return Ok(());
    }
    let mut sends = Vec::with_capacity(code.len());
    for op in code {
        sends.push(mutable_argument(arena, pool, op)? & selected);
    }
    if sends.iter().all(|&positions| positions == 0) {
        return Ok(());
    }

    // An instruction reached by a jump uses the open call from that path.
    // A handler starts with no open call. A never-returning direct call has no fallthrough.
    let mut parents: Vec<Option<Option<usize>>> = vec![None; code.len()];
    let mut work = vec![(0, None)];
    for &region in get::list(arena, &tries)? {
        let [_, catch, finally, _] = fields(arena, region, "TRY")?;
        for entry in [catch, finally] {
            let pc = index(arena, entry)?;
            if pc != 0 {
                work.push((pc, None));
            }
        }
    }
    while let Some((pc, parent)) = work.pop() {
        if parents[pc].is_some() {
            continue;
        }
        parents[pc] = Some(parent);
        let op = &code[pc];
        if matches!(
            op.name,
            "RETURN"
                | "RETURN_BY_REF"
                | "GENERATOR_RETURN"
                | "FAST_RET"
                | "THROW"
                | "MATCH_ERROR"
                | "VERIFY_NEVER_TYPE"
        ) {
            continue;
        }
        if matches!(op.name, "DO_ICALL" | "DO_UCALL")
            && let Some(init) = parent
            && declarations.call_is_never(arena, pool, &code[init])?
        {
            continue;
        }
        let next = enclosing_call_after(op.name, pc, parent, &parents);
        for operand in [op.left, op.right] {
            if let Operand::Target(target) = operand {
                work.push((target, next));
            }
        }
        if let Some(target) = op.extended_target {
            work.push((target, next));
        }
        if op.uses_jump_table()
            && let Operand::Literal(index) = op.right
        {
            let [table] = fields(arena, pool[index], "LITERAL")?;
            let [entries, _, _, _, _] = fields(arena, table, "ARRAY")?;
            for &entry in get::list(arena, &entries)? {
                let [_, value] = fields(arena, entry, "ENTRY")?;
                let [target] = fields(arena, value, "INT")?;
                work.push((super::index(arena, target)?, next));
            }
        }
        if !matches!(op.name, "JMP" | "MATCH") && pc + 1 < code.len() {
            work.push((pc + 1, next));
        }
    }

    // Zend associates unreachable sends and completions with the preceding lexical call.
    let mut lexical = None;
    let mut mutable_by_call = vec![0; code.len()];
    for (pc, op) in code.iter().enumerate() {
        let parent = *parents[pc].get_or_insert(lexical);
        if let Some(init) = parent {
            if init < pc {
                mutable_by_call[init] |= sends[pc];
            }
            if mutable_by_call[init] & mask(op) != 0 {
                return Err(Error::new("a constant-bound array must be immutable"));
            }
        }
        lexical = enclosing_call_after(op.name, pc, parent, &parents);
    }
    Ok(())
}
