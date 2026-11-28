//! Validate function declarations, call frames and array literals bound to partial calls.

use std::collections::{HashMap, HashSet};

use super::{Instruction, Operand, index};
use crate::{Arena, Error, Result, Value, ast::fields, case_fields};
use p4spectec::lang::data::value::flat::get;

#[derive(Clone, Copy)]
pub(super) struct Declarations<'a> {
    pub(super) functions: &'a HashMap<Vec<u8>, Value>,
    pub(super) classes: &'a [Value],
    pub(super) scope: Option<Value>,
}

fn bytes(arena: &Arena, value: Value) -> Result<Vec<u8>> {
    get::list(arena, &value)?
        .iter()
        .map(|&byte| u8::try_from(index(arena, byte)?).map_err(|_| Error::new("invalid name byte")))
        .collect()
}

pub(super) fn function_table(
    arena: &Arena,
    functions: &[Value],
) -> Result<HashMap<Vec<u8>, Value>> {
    let mut table = HashMap::new();
    for &item in functions {
        let [key, body] = fields(arena, item, "NAMEDBODY")?;
        let key = bytes(arena, key)?;
        let [decl, _, _, _, _, _, _] = fields(arena, body, "BODY")?;
        let [name, flags, _, scope, _, _, _] = fields(arena, decl, "BODYDECL")?;
        let Some(name) = get::opt(arena, &name)? else {
            return Err(Error::new("expected a named function"));
        };
        if get::opt(arena, &scope)?.is_some() || index(arena, flags)? & (1 << 22) != 0 {
            return Err(Error::new(
                "expected an ordinary named function in the unit table",
            ));
        }
        if key.is_empty()
            || key.contains(&0)
            || key != bytes(arena, name)?.to_ascii_lowercase()
            || matches!(key.as_slice(), b"strlen" | b"exit" | b"die")
            || table.insert(key, body).is_some()
        {
            return Err(Error::new("invalid or duplicate function key"));
        }
    }
    Ok(table)
}

pub(super) struct Parameters {
    fixed: usize,
    variadic: bool,
}

fn type_flags(arena: &Arena, ty: Value) -> Result<(usize, bool)> {
    let (kind, fields) = case_fields(arena, ty)?;
    let mask = index(
        arena,
        *fields
            .first()
            .ok_or_else(|| Error::new("missing type mask"))?,
    )?;
    // A named, union or intersection type has a type even when its printed value mask is zero.
    Ok((mask, kind != "MASK" || mask & ((1 << 25) - 1) != 0))
}

pub(super) fn parameters(arena: &Arena, decl: Value, locals: &[Value]) -> Result<Parameters> {
    let [name, flags, _, _, _, signature, _] = fields(arena, decl, "BODYDECL")?;
    let flags = index(arena, flags)?;
    let typed_args = flags & (1 << 8) != 0;
    let returns = flags & (1 << 13) != 0;
    let variadic = flags & (1 << 14) != 0;
    let Some(signature) = get::opt(arena, &signature)? else {
        if typed_args || returns || variadic {
            return Err(Error::new("missing argument information"));
        }
        return Ok(Parameters { fixed: 0, variadic });
    };
    let [declared, required, result, args] = fields(arena, signature, "ARGS")?;
    let fixed = index(arena, declared)?;
    let args = get::list(arena, &args)?;
    if fixed > (u32::MAX - 2) as usize
        || index(arena, required)? > fixed
        || args.len() != fixed + usize::from(variadic)
        || args.len() > locals.len()
    {
        return Err(Error::new("inconsistent function argument counts"));
    }
    let result = get::opt(arena, &result)?;
    if result.is_some() != returns
        || result
            .map(|ty| type_flags(arena, ty))
            .transpose()?
            .is_some_and(|(_, set)| !set)
    {
        return Err(Error::new("return type disagrees with function flags"));
    }
    if get::opt(arena, &name)?.is_none() && (!args.is_empty() || returns) {
        return Err(Error::new("main body cannot have parameters"));
    }
    let mut has_type = false;
    for (i, (&arg, &local)) in args.iter().zip(locals).enumerate() {
        let [name, ty, _] = fields(arena, arg, "ARG")?;
        let (mask, set) = type_flags(arena, ty)?;
        if arena.value.canon_id(name.node) != arena.value.canon_id(local.node)
            || (mask & (1 << 27) != 0) != (variadic && fixed == i)
            || (mask >> 25) & 3 > 2
        {
            return Err(Error::new(format!("inconsistent argument {}", i + 1)));
        }
        has_type |= set;
    }
    if has_type != typed_args {
        return Err(Error::new("type hints disagree with function flags"));
    }
    Ok(Parameters { fixed, variadic })
}

impl Parameters {
    pub(super) fn check_receives(&self, code: &[Instruction<'_>], executable: usize) -> Result<()> {
        let count = self.fixed + usize::from(self.variadic);
        if executable <= count {
            return Err(Error::new("missing code after parameters"));
        }
        for (i, op) in code.iter().enumerate() {
            if matches!(op.name, "RECV" | "RECV_INIT" | "RECV_VARIADIC") {
                if op.left != Operand::Immediate(i + 1)
                    || op.result != Operand::Cv(i)
                    || i >= count
                    || (op.name == "RECV_VARIADIC") != (self.variadic && i == self.fixed)
                {
                    return Err(Error::new(
                        "parameter position disagrees with the function declaration",
                    ));
                }
            } else if i < count {
                return Err(Error::new("expected parameter receive instruction"));
            }
        }
        Ok(())
    }
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
            return match self.functions.get(&name) {
                Some(&body) => returns_never(arena, body),
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

fn opens(name: &str) -> bool {
    matches!(
        name,
        "INIT_FCALL"
            | "INIT_FCALL_BY_NAME"
            | "INIT_NS_FCALL_BY_NAME"
            | "INIT_DYNAMIC_CALL"
            | "INIT_METHOD_CALL"
            | "INIT_STATIC_METHOD_CALL"
            | "INIT_PARENT_PROPERTY_HOOK_CALL"
            | "INIT_USER_CALL"
            | "NEW"
    )
}

fn closes(name: &str) -> bool {
    matches!(
        name,
        "DO_ICALL"
            | "DO_UCALL"
            | "DO_FCALL"
            | "DO_FCALL_BY_NAME"
            | "CALLABLE_CONVERT"
            | "CALLABLE_CONVERT_PARTIAL"
    )
}

fn sends(name: &str) -> bool {
    matches!(
        name,
        "SEND_VAL"
            | "SEND_VAL_EX"
            | "SEND_VAR"
            | "SEND_VAR_EX"
            | "SEND_VAR_NO_REF"
            | "SEND_VAR_NO_REF_EX"
            | "SEND_REF"
            | "SEND_FUNC_ARG"
            | "SEND_USER"
            | "SEND_PLACEHOLDER"
            | "CHECK_FUNC_ARG"
    )
}

fn needs_call(name: &str) -> bool {
    sends(name)
        || closes(name)
        || matches!(
            name,
            "SEND_ARRAY"
                | "SEND_UNPACK"
                | "CHECK_UNDEF_ARGS"
                | "FETCH_FUNC_ARG"
                | "FETCH_DIM_FUNC_ARG"
                | "FETCH_OBJ_FUNC_ARG"
                | "FETCH_STATIC_PROP_FUNC_ARG"
        )
}

fn enclosing_call_after(
    name: &str,
    pc: usize,
    parent: Option<usize>,
    parents: &[Option<Option<usize>>],
) -> Option<usize> {
    if opens(name) {
        Some(pc)
    } else if closes(name) {
        parent.and_then(|init| parents[init].flatten())
    } else {
        parent
    }
}

fn direct_targets(
    arena: &Arena,
    pool: &[Value],
    code: &[Instruction<'_>],
    declarations: &Declarations<'_>,
) -> Result<Vec<Option<Value>>> {
    let mut targets = vec![None; code.len()];
    for (pc, op) in code.iter().enumerate() {
        if !opens(op.name) {
            continue;
        }
        let target = match (op.name, op.right) {
            ("INIT_FCALL" | "INIT_FCALL_BY_NAME", Operand::Literal(n)) => {
                let key = n + usize::from(op.name == "INIT_FCALL_BY_NAME");
                let name = name_literal(arena, pool, key)?
                    .ok_or_else(|| Error::new("expected a function name"))?;
                // INIT_FCALL looks up an already registered function. INIT_FCALL_BY_NAME may fail at runtime.
                if op.name == "INIT_FCALL"
                    && (name.is_empty()
                        || name.contains(&0)
                        || name.iter().any(u8::is_ascii_uppercase))
                {
                    return Err(Error::new("invalid direct function lookup key"));
                }
                declarations.functions.get(&name).copied()
            }
            _ => None,
        };
        // For targets outside the unit, the header and arguments give the known minimum.
        let mut slots = 5 + op.extended_value;
        if let Some(body) = target {
            let [decl, frame, _, _, _, _, _] = fields(arena, body, "BODY")?;
            let [temps, _, locals, _] = fields(arena, frame, "FRAME")?;
            let [_, _, _, _, _, signature, _] = fields(arena, decl, "BODYDECL")?;
            let params = match get::opt(arena, &signature)? {
                Some(sig) => index(arena, fields::<4>(arena, sig, "ARGS")?[0])?,
                None => 0,
            };
            let locals = get::list(arena, &locals)?.len();
            let extra = locals
                .checked_sub(params.min(op.extended_value))
                .ok_or_else(|| Error::new("inconsistent function argument counts"))?;
            slots += index(arena, temps)? + extra;
        }
        if slots > u32::MAX as usize / 16 {
            return Err(Error::new("call frame size exceeds the Zend byte field"));
        }
        if op.name == "INIT_FCALL" {
            let Operand::Immediate(frame_bytes) = op.left else {
                return Err(Error::new("expected direct call frame byte size"));
            };
            if !frame_bytes.is_multiple_of(16) || slots > frame_bytes / 16 {
                return Err(Error::new("direct call frame is too small or misaligned"));
            }
            targets[pc] = target;
        }
    }
    Ok(targets)
}

fn visit_jump_targets(
    arena: &Arena,
    pool: &[Value],
    op: &Instruction<'_>,
    mut visit: impl FnMut(usize) -> Result<()>,
) -> Result<()> {
    for operand in [op.left, op.right] {
        if let Operand::Target(target) = operand {
            visit(target)?;
        }
    }
    if let Some(target) = op.extended_target {
        visit(target)?;
    }
    if op.uses_jump_table()
        && let Operand::Literal(n) = op.right
    {
        let [table] = fields(arena, pool[n], "LITERAL")?;
        let [entries, _, _, _, _] = fields(arena, table, "ARRAY")?;
        for &entry in get::list(arena, &entries)? {
            let [_, value] = fields(arena, entry, "ENTRY")?;
            let [target] = fields(arena, value, "INT")?;
            visit(index(arena, target)?)?;
        }
    }
    Ok(())
}

fn raises(name: &str) -> bool {
    matches!(name, "THROW" | "MATCH_ERROR" | "VERIFY_NEVER_TYPE")
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
    if !code.iter().any(|op| opens(op.name) || needs_call(op.name)) {
        return Ok(());
    }
    let targets = direct_targets(arena, pool, code, declarations)?;
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
        if let Some(previous) = parents[pc] {
            if previous != parent {
                return Err(Error::new("jump crosses an unfinished call"));
            }
            continue;
        }
        parents[pc] = Some(parent);
        let op = &code[pc];
        if raises(op.name) {
            continue;
        }
        if matches!(
            op.name,
            "RETURN" | "RETURN_BY_REF" | "GENERATOR_RETURN" | "FAST_RET"
        ) {
            if parent.is_some() {
                return Err(Error::new("unfinished call"));
            }
            continue;
        }
        if matches!(op.name, "DO_ICALL" | "DO_UCALL")
            && let Some(init) = parent
            && declarations.call_is_never(arena, pool, &code[init])?
        {
            continue;
        }
        let next = enclosing_call_after(op.name, pc, parent, &parents);
        visit_jump_targets(arena, pool, op, |target| {
            work.push((target, next));
            Ok(())
        })?;
        if !matches!(op.name, "JMP" | "MATCH") && pc + 1 < code.len() {
            work.push((pc + 1, next));
        }
    }

    // Zend associates unreachable sends and completions with the preceding lexical call.
    let mut lexical = None;
    let mut mutable_by_call = vec![0; code.len()];
    let mut placeholders = HashSet::new();
    for (pc, op) in code.iter().enumerate() {
        let parent = *parents[pc].get_or_insert(lexical);
        lexical = enclosing_call_after(op.name, pc, parent, &parents);
        if needs_call(op.name) {
            let init =
                parent.ok_or_else(|| Error::new(format!("{} requires an open call", op.name)))?;
            let open = &code[init];
            if sends(op.name)
                && let Operand::Immediate(pos) = op.right
            {
                if !(1..=open.extended_value).contains(&pos) {
                    return Err(Error::new("argument position exceeds the open call frame"));
                }
                // CHECK_FUNC_ARG reads only the number. Positional sends address a signed byte offset.
                if op.name != "CHECK_FUNC_ARG" && 5 + (pos - 1) > i32::MAX as usize / 16 {
                    return Err(Error::new(
                        "argument position exceeds Zend's signed byte offset",
                    ));
                }
            }
            if op.name == "SEND_PLACEHOLDER" {
                placeholders.insert(init);
            }
            if closes(op.name)
                && op.name != "CALLABLE_CONVERT_PARTIAL"
                && placeholders.contains(&init)
            {
                return Err(Error::new("placeholder sent to an ordinary call"));
            }
            if matches!(op.name, "DO_ICALL" | "DO_UCALL") {
                if open.name == "INIT_FCALL" {
                    if (op.name == "DO_UCALL") != targets[init].is_some() {
                        return Err(Error::new(
                            "call completion disagrees with its direct target kind",
                        ));
                    }
                } else if !matches!(
                    open.name,
                    "INIT_STATIC_METHOD_CALL"
                        | "INIT_METHOD_CALL"
                        | "INIT_PARENT_PROPERTY_HOOK_CALL"
                ) {
                    return Err(Error::new(
                        "specialized call completion needs a known target",
                    ));
                }
            }
            if closes(op.name) && pc + 1 == code.len() {
                if !matches!(op.name, "DO_ICALL" | "DO_UCALL")
                    || !declarations.call_is_never(arena, pool, open)?
                {
                    return Err(Error::new("body can fall past its final call"));
                }
                lexical = None;
            }
            if selected != 0 && init < pc {
                mutable_by_call[init] |= mutable_argument(arena, pool, op)?;
            }
            if mutable_by_call[init] & mask(op) != 0 {
                return Err(Error::new("a constant-bound array must be immutable"));
            }
        }
    }
    if lexical.is_some() && !raises(code.last().unwrap().name) {
        return Err(Error::new("unfinished call"));
    }
    // Dead branches are still subject to the same call-boundary checks as reachable ones.
    for (pc, op) in code.iter().enumerate() {
        visit_jump_targets(arena, pool, op, |target| {
            if parents[pc] == parents[target] {
                Ok(())
            } else {
                Err(Error::new("jump crosses an unfinished call"))
            }
        })?;
    }
    Ok(())
}
