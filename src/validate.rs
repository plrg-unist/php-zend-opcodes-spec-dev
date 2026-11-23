use std::collections::{HashMap, HashSet};

use p4spectec::lang::{
    common::prim::num,
    data::value::flat::{ValueKind, get},
};

use crate::{Arena, Error, Result, Value, ast::fields, case_fields};

/// Checks the body-relative references of a parsed unit and supported opcode forms before execution.
pub fn validate_unit(arena: &Arena, unit: &Value) -> Result<()> {
    let [filename, _, _, main, functions, classes] = fields(arena, *unit, "UNIT")?;
    let filename = get::list(arena, &filename)?;
    if filename.is_empty() {
        return Err(Error::new("empty unit filename"));
    }
    for &byte in filename {
        if !(1..=255).contains(&index(arena, byte)?) {
            return Err(Error::new("invalid byte in unit filename"));
        }
    }
    let mut pending = vec![classes, functions, main];
    let mut shared_arrays = HashMap::new();
    let mut constant_literals = HashSet::new();
    while let Some(value) = pending.pop() {
        match arena.value.kind(value.node) {
            ValueKind::Case(_) => match case_fields(arena, value)? {
                ("BODY", &[decl, frame, pool, tries, ranges, children, code]) => {
                    let [temps, _, locals, statics] = fields(arena, frame, "FRAME")?;
                    if let Some(table) = get::opt(arena, &statics)? {
                        fields::<3>(arena, table, "ARRAY")?;
                    }
                    let locals = get::list(arena, &locals)?.len();
                    let slots = locals + index(arena, temps)?;
                    // On the pinned 64-bit ABI, offsets use 16-byte zvals after five call-frame slots.
                    if slots > (u32::MAX / 16 - 5) as usize {
                        return Err(Error::new("frame exceeds Zend's slot range"));
                    }
                    let layout = Layout {
                        locals,
                        slots,
                        pool: get::list(arena, &pool)?,
                    };
                    let code = get::list(arena, &code)?;
                    let executable = layout.validate_code(arena, code)?;
                    layout.validate_ranges(arena, tries, ranges, code.len(), executable)?;
                    // Deferred expressions anywhere in declarations or values can contain another body.
                    pending.extend([decl, frame, pool, children]);
                }
                ("BODY", _) => return Err(Error::new("invalid body fields")),
                ("VALUE", &[_, _, value]) => {
                    check_constant_literal(arena, value, &mut constant_literals)?;
                    pending.push(value);
                }
                ("SHARED_ARRAY", &[id, array]) => {
                    let id = u32::try_from(index(arena, id)?)
                        .map_err(|_| Error::new("shared array id exceeds u32"))?;
                    let [entries, _, _] = fields(arena, array, "ARRAY")?;
                    get::list(arena, &entries)?;
                    let payload = arena.value.canon_id(array.node);
                    match shared_arrays.insert(id, payload) {
                        None => pending.push(array),
                        Some(known) if known == payload => {}
                        Some(_) => {
                            return Err(Error::new(format!("conflicting shared array {id}")));
                        }
                    }
                }
                ("SHARED_ARRAY", _) => return Err(Error::new("invalid shared array fields")),
                // A deferred ARRAY expression starts with numeric flags; a zval starts with entries.
                ("ARRAY", &[entries, next, cursor])
                    if matches!(arena.value.kind(entries.node), ValueKind::List(_)) =>
                {
                    check_array(arena, entries, next, cursor)?;
                    pending.push(entries);
                }
                (_, args) => pending.extend_from_slice(args),
            },
            ValueKind::List(items) => {
                if let Some(first) = items.first()
                    && !matches!(arena.value.kind(first.node), ValueKind::Num(_))
                {
                    pending.extend_from_slice(items);
                }
            }
            ValueKind::Opt(Some(child)) => pending.push(*child),
            _ => {}
        }
    }
    Ok(())
}

// A zval inside a constant-expression VALUE cannot itself defer another expression.
fn check_constant_literal(arena: &Arena, value: Value, checked: &mut HashSet<Value>) -> Result<()> {
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        if !checked.insert(value) {
            continue;
        }
        match case_fields(arena, value)? {
            ("DEFERRED", _) => {
                return Err(Error::new(
                    "a constant-expression value cannot contain a deferred zval",
                ));
            }
            ("SHARED_ARRAY", &[_, array]) => pending.push(array),
            ("ARRAY", &[entries, _, _]) => {
                for &entry in get::list(arena, &entries)? {
                    let [_, item] = fields(arena, entry, "ENTRY")?;
                    pending.push(item);
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn index(arena: &Arena, value: Value) -> Result<usize> {
    usize::try_from(num::to_int(get::num(arena, &value)?))
        .map_err(|_| Error::new("expected a nonnegative index"))
}

fn signed_index(arena: &Arena, value: Value) -> Result<i64> {
    i64::try_from(num::to_int(get::num(arena, &value)?))
        .map_err(|_| Error::new("expected a signed array index"))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Operand {
    Unused,
    Immediate,
    Target(usize),
    Literal(usize),
    Cv(usize),
    Tmp(usize),
    Var(usize),
}

impl Operand {
    fn read(arena: &Arena, value: Value, layout: &Layout<'_>) -> Result<Self> {
        let operand = match case_fields(arena, value)? {
            ("UNUSED", []) => Self::Unused,
            ("IMM", [_]) => Self::Immediate,
            ("TARGET", &[n]) => Self::Target(index(arena, n)?),
            ("LIT", &[n]) => Self::Literal(index(arena, n)?),
            ("CV", &[n]) => Self::Cv(index(arena, n)?),
            ("TMP", &[n]) => Self::Tmp(index(arena, n)?),
            ("VAR", &[n]) => Self::Var(index(arena, n)?),
            _ => return Err(Error::new("invalid operand")),
        };
        match operand {
            Self::Literal(n) if n >= layout.pool.len() => {
                Err(Error::new(format!("missing literal lit[{n}]")))
            }
            Self::Cv(n) if n >= layout.locals => Err(Error::new(format!("invalid local CV{n}"))),
            Self::Tmp(n) | Self::Var(n) if !(layout.locals..layout.slots).contains(&n) => {
                Err(Error::new(format!("invalid temporary slot {n}")))
            }
            _ => Ok(operand),
        }
    }

    fn slot(self) -> bool {
        matches!(self, Self::Cv(_) | Self::Tmp(_) | Self::Var(_))
    }

    fn value(self) -> bool {
        self.slot() || matches!(self, Self::Literal(_))
    }

    fn rvalue(self) -> bool {
        matches!(self, Self::Literal(_) | Self::Tmp(_) | Self::Cv(_))
    }
}

struct Instruction<'a> {
    name: &'a str,
    result: Operand,
    left: Operand,
    right: Operand,
    smart: Option<&'a str>,
    extended_target: Option<usize>,
    extended_value: usize,
}

impl<'a> Instruction<'a> {
    fn uses_jump_table(&self) -> bool {
        matches!(self.name, "SWITCH_LONG" | "SWITCH_STRING" | "MATCH")
    }

    fn read(arena: &'a Arena, row: Value, layout: &Layout<'_>) -> Result<Self> {
        let [
            op,
            result,
            left,
            right,
            _,
            ext,
            cache,
            default_cache,
            mask,
            smart,
            numeric,
        ] = fields(arena, row, "OPLINE")?;
        let name = case_fields(arena, op)?.0;
        let result = Operand::read(arena, result, layout)?;
        let left = Operand::read(arena, left, layout)?;
        let right = Operand::read(arena, right, layout)?;
        let smart = get::opt(arena, &smart)?
            .map(|v| case_fields(arena, v))
            .transpose()?
            .map(|(tag, _)| tag);
        let ext = get::opt(arena, &ext)?
            .map(|v| case_fields(arena, v))
            .transpose()?;
        let extended_target = match ext {
            Some(("EXT_TARGET", &[n])) => Some(index(arena, n)?),
            _ => None,
        };
        let extended_value = match ext {
            Some(("EXT_VALUE", &[n])) => index(arena, n)?,
            _ => 0,
        };
        let row = Self {
            name,
            result,
            left,
            right,
            smart,
            extended_target,
            extended_value,
        };
        let unused = Operand::Unused;
        if result != unused && !result.slot() {
            return Err(Error::new("expected a result slot"));
        }
        let writable = matches!(left, Operand::Cv(_) | Operand::Var(_));
        let valid = match name {
            "NOP" => left == unused && right == unused && result == unused,
            "ASSIGN" | "ASSIGN_OP" => writable && right.rvalue(),
            "PRE_INC" | "PRE_DEC" => writable && right == unused,
            "POST_INC" | "POST_DEC" => writable && right == unused && result.slot(),
            "ADD"
            | "SUB"
            | "MUL"
            | "MOD"
            | "SL"
            | "SR"
            | "BW_OR"
            | "BW_AND"
            | "BW_XOR"
            | "IS_SMALLER"
            | "IS_SMALLER_OR_EQUAL" => left.value() && right.value() && result.slot(),
            "BW_NOT" => left.value() && right == unused && result.slot(),
            "BOOL" | "BOOL_NOT" | "CAST" | "GET_TYPE" | "STRLEN" | "TYPE_CHECK" | "COUNT" => {
                left.rvalue() && right == unused && result.slot()
            }
            "DIV"
            | "POW"
            | "IS_IDENTICAL"
            | "IS_NOT_IDENTICAL"
            | "IS_EQUAL"
            | "IS_NOT_EQUAL"
            | "SPACESHIP"
            | "BOOL_XOR"
            | "CONCAT"
            | "FAST_CONCAT"
            | "ISSET_ISEMPTY_DIM_OBJ"
            | "ARRAY_KEY_EXISTS" => left.rvalue() && right.rvalue() && result.slot(),
            "INIT_ARRAY" | "ADD_ARRAY_ELEMENT" => {
                (left.value() || (name == "INIT_ARRAY" && left == unused))
                    && (right.rvalue() || right == unused)
                    && result.slot()
            }
            "ADD_ARRAY_UNPACK" => left.value() && right == unused && result.slot(),
            "FETCH_DIM_R" | "FETCH_DIM_IS" => left.value() && right.rvalue() && result.slot(),
            "IN_ARRAY" => left.rvalue() && matches!(right, Operand::Literal(_)) && result.slot(),
            "CHECK_VAR" => matches!(left, Operand::Cv(_)) && right == unused && result == unused,
            "ISSET_ISEMPTY_CV" => {
                matches!(left, Operand::Cv(_)) && right == unused && result.slot()
            }
            "QM_ASSIGN" => left.value() && right == unused && result.slot(),
            "COPY_TMP" => {
                matches!(left, Operand::Tmp(_) | Operand::Var(_))
                    && right == unused
                    && result.slot()
            }
            "FREE" => {
                let expected = if extended_value & 1 != 0 {
                    Operand::Immediate
                } else {
                    unused
                };
                matches!(left, Operand::Tmp(_) | Operand::Var(_))
                    && right == expected
                    && result == unused
            }
            "ROPE_INIT" => left == unused && right.rvalue() && matches!(result, Operand::Tmp(_)),
            "ROPE_ADD" => matches!(left, Operand::Tmp(_)) && right.rvalue() && result == left,
            "ROPE_END" => matches!(left, Operand::Tmp(_)) && right.rvalue() && result.slot(),
            "JMP" => matches!(left, Operand::Target(_)) && right == unused && result == unused,
            "JMPZ" | "JMPNZ" => {
                left.rvalue() && matches!(right, Operand::Target(_)) && result == unused
            }
            "JMPZ_EX" | "JMPNZ_EX" | "JMP_SET" | "COALESCE" | "JMP_NULL" => {
                left.rvalue() && matches!(right, Operand::Target(_)) && result.slot()
            }
            "CASE" | "CASE_STRICT" => {
                matches!(left, Operand::Tmp(_)) && right.rvalue() && result.slot()
            }
            "SWITCH_LONG" | "SWITCH_STRING" | "MATCH" => {
                left.value() && matches!(right, Operand::Literal(_)) && result == unused
            }
            "ECHO" | "RETURN" => left.rvalue() && right == unused && result == unused,
            _ => return Ok(row),
        };
        if !valid {
            return Err(Error::new(format!("wrong operands for {name}")));
        }
        if name == "CONCAT" && matches!((left, right), (Operand::Literal(_), Operand::Literal(_))) {
            return Err(Error::new("CONCAT has no two-literal handler"));
        }
        for operand in [left, right] {
            if let Operand::Literal(n) = operand {
                let ("LITERAL", &[value]) = case_fields(arena, layout.pool[n])? else {
                    return Err(Error::new(format!(
                        "expected a value in lit[{n}] for {name}"
                    )));
                };
                if matches!(
                    name,
                    "CONCAT" | "FAST_CONCAT" | "ROPE_INIT" | "ROPE_ADD" | "ROPE_END"
                ) && case_fields(arena, value)?.0 != "STRING"
                {
                    return Err(Error::new(format!(
                        "expected a string in lit[{n}] for {name}"
                    )));
                }
            }
        }
        let valid_ext = if row.uses_jump_table() {
            extended_target.is_some()
        } else {
            matches!(ext, None | Some(("EXT_VALUE", [_])))
        };
        if !valid_ext
            || get::opt(arena, &cache)?.is_some()
            || get::opt(arena, &default_cache)?.is_some()
            || get::opt(arena, &mask)?.is_some()
        {
            return Err(Error::new(format!("unexpected metadata for {name}")));
        }
        if get::bool(arena, &numeric)? {
            let Operand::Literal(n) = right else {
                return Err(Error::new(
                    "numeric-string-key requires a dimension literal",
                ));
            };
            if !matches!(
                name,
                "FETCH_DIM_R" | "FETCH_DIM_IS" | "ISSET_ISEMPTY_DIM_OBJ"
            ) || layout.literal_tag(arena, n)? != "INT"
                || layout.literal_tag(arena, n + 1)? != "STRING"
            {
                return Err(Error::new("invalid numeric-string-key literal pair"));
            }
        }
        if let ("IN_ARRAY", Operand::Literal(n)) = (name, right) {
            let [value] = fields(arena, layout.pool[n], "LITERAL")?;
            let array = match case_fields(arena, value)? {
                ("SHARED_ARRAY", &[_, array]) => array,
                _ => value,
            };
            let [entries, _, _] = fields(arena, array, "ARRAY")?;
            if extended_value == 0 {
                for &entry in get::list(arena, &entries)? {
                    let [key, _] = fields(arena, entry, "ENTRY")?;
                    if case_fields(arena, key)?.0 != "KEY" {
                        return Err(Error::new("non-strict IN_ARRAY requires string keys"));
                    }
                }
            }
        }
        match (name, result, left) {
            ("JMP_NULL", _, _) if extended_value & 3 == 3 => {
                return Err(Error::new("invalid nullsafe chain mode"));
            }
            ("CAST", _, _) if !(4..=8).contains(&extended_value) => {
                return Err(Error::new("invalid cast type"));
            }
            ("ASSIGN_OP", _, _) if !(1..=12).contains(&extended_value) => {
                return Err(Error::new("unknown compound assignment operator"));
            }
            // The pinned ABI stores two zend_string pointers per temporary zval slot.
            ("ROPE_INIT", Operand::Tmp(base), _)
                if extended_value == 0 || extended_value.div_ceil(2) > layout.slots - base =>
            {
                return Err(Error::new("invalid rope storage"));
            }
            ("ROPE_ADD" | "ROPE_END", _, Operand::Tmp(base))
                if extended_value / 2 >= layout.slots - base =>
            {
                return Err(Error::new("invalid rope storage"));
            }
            ("ROPE_END", Operand::Tmp(dst) | Operand::Var(dst), Operand::Tmp(base))
                if dst >= base && dst - base <= extended_value / 2 =>
            {
                return Err(Error::new("rope result overlaps its storage"));
            }
            _ => {}
        }
        Ok(row)
    }
}

struct Layout<'a> {
    locals: usize,
    slots: usize,
    pool: &'a [Value],
}

impl Layout<'_> {
    fn literal_tag<'a>(&self, arena: &'a Arena, n: usize) -> Result<&'a str> {
        let literal = self
            .pool
            .get(n)
            .ok_or_else(|| Error::new(format!("missing literal lit[{n}]")))?;
        let [value] = fields(arena, *literal, "LITERAL")?;
        Ok(case_fields(arena, value)?.0)
    }

    fn validate_ranges(
        &self,
        arena: &Arena,
        tries: Value,
        ranges: Value,
        count: usize,
        executable: usize,
    ) -> Result<()> {
        let mut previous = 0;
        for &region in get::list(arena, &tries)? {
            let [start, catch, finally, end] = fields(arena, region, "TRY")?;
            let (start, catch, finally, end) = (
                index(arena, start)?,
                index(arena, catch)?,
                index(arena, finally)?,
                index(arena, end)?,
            );
            if [start, catch, finally, end].into_iter().any(|n| n >= count)
                || catch >= executable
                || finally >= executable
                || start < previous
                || (catch != 0 && catch < start)
                || (finally != 0 && finally < start.max(catch))
                || (finally == 0) != (end == 0)
                || end < finally
            {
                return Err(Error::new("invalid try region"));
            }
            previous = start;
        }
        previous = 0;
        for &range in get::list(arena, &ranges)? {
            let [slot, kind, start, end] = fields(arena, range, "LIVE")?;
            let (start, end) = (index(arena, start)?, index(arena, end)?);
            if !(self.locals..self.slots).contains(&index(arena, slot)?)
                || index(arena, kind)? > 4
                || start < previous
                || start >= end
                || end > count
            {
                return Err(Error::new("invalid live range"));
            }
            previous = start;
        }
        Ok(())
    }

    fn validate_code(&self, arena: &Arena, rows: &[Value]) -> Result<usize> {
        let code: Vec<_> = rows
            .iter()
            .map(|&row| Instruction::read(arena, row, self))
            .collect::<Result<_>>()?;
        // Nested switches can leave several unreachable cleanup instructions after a return.
        let mut executable = code.as_slice();
        while let Some((last, before)) = executable.split_last()
            && last.name == "FREE"
            && last.extended_value == 2
        {
            executable = before;
        }
        let Some(last) = executable.last() else {
            return Err(Error::new("body has no instructions"));
        };
        if !matches!(
            last.name,
            "RETURN"
                | "RETURN_BY_REF"
                | "GENERATOR_RETURN"
                | "THROW"
                | "FAST_RET"
                | "JMP"
                | "MATCH"
                | "MATCH_ERROR"
                | "VERIFY_NEVER_TYPE"
        ) && !(executable.len() == code.len() && matches!(last.name, "DO_ICALL" | "DO_UCALL"))
        {
            return Err(Error::new("body can fall past its final instruction"));
        }
        let mut needs_data = false;
        let mut tables = HashSet::new();
        for (i, op) in code.iter().enumerate() {
            if (op.name == "OP_DATA") != needs_data {
                return Err(Error::new(format!("unpaired OP_DATA at instruction {i}")));
            }
            needs_data = matches!(
                op.name,
                "ASSIGN_DIM"
                    | "ASSIGN_DIM_OP"
                    | "ASSIGN_OBJ"
                    | "ASSIGN_OBJ_OP"
                    | "ASSIGN_OBJ_REF"
                    | "ASSIGN_STATIC_PROP"
                    | "ASSIGN_STATIC_PROP_OP"
                    | "ASSIGN_STATIC_PROP_REF"
                    | "DECLARE_ATTRIBUTED_CONST"
                    | "FRAMELESS_ICALL_3"
            );
            for operand in [op.left, op.right] {
                if let Operand::Target(target) = operand {
                    check_target(executable, target)?;
                }
            }
            if let Some(target) = op.extended_target {
                check_target(executable, target)?;
            }
            if let Some(smart) = op.smart {
                let jump = match smart {
                    "SMART_JMPZ" => "JMPZ",
                    "SMART_JMPNZ" => "JMPNZ",
                    _ => return Err(Error::new("unknown smart branch")),
                };
                if !smart_producer(op.name)
                    || !matches!(op.result, Operand::Tmp(_))
                    || !code
                        .get(i + 1)
                        .is_some_and(|next| next.name == jump && next.left == op.result)
                {
                    return Err(Error::new(format!(
                        "mismatched smart branch at instruction {i}"
                    )));
                }
            }
            if op.uses_jump_table() {
                let Operand::Literal(n) = op.right else {
                    return Err(Error::new("expected jump-table literal"));
                };
                if !tables.insert(n) {
                    return Err(Error::new("shared jump-table literal"));
                }
                check_jump_table(arena, self.pool[n], op.name, executable)?;
            }
        }
        Ok(executable.len())
    }
}

fn check_array(arena: &Arena, entries: Value, next: Value, cursor: Value) -> Result<()> {
    let entries = get::list(arena, &entries)?;
    if let Some(cursor) = get::opt(arena, &cursor)?
        && index(arena, cursor)? > entries.len()
    {
        return Err(Error::new("array cursor exceeds entry count"));
    }
    let mut keys = HashSet::new();
    let mut next_index = i64::MIN;
    for &entry in entries {
        let [key, value] = fields(arena, entry, "ENTRY")?;
        match case_fields(arena, key)? {
            ("INDEX", &[n]) => {
                next_index = next_index.max(signed_index(arena, n)?.saturating_add(1));
            }
            ("KEY", [_]) => {}
            _ => return Err(Error::new("invalid array key")),
        }
        if !keys.insert(arena.value.canon_id(key.node)) {
            return Err(Error::new("duplicate array key"));
        }
        if case_fields(arena, value)?.0 == "UNDEF" {
            return Err(Error::new("undef cannot be an array element"));
        }
    }
    if let Some(next) = get::opt(arena, &next)?
        && signed_index(arena, next)? < next_index
    {
        return Err(Error::new(
            "next array index precedes an existing integer key",
        ));
    }
    Ok(())
}

fn check_jump_table(
    arena: &Arena,
    literal: Value,
    name: &str,
    code: &[Instruction<'_>],
) -> Result<()> {
    let [array] = fields(arena, literal, "LITERAL")?;
    let [entries, _, _] = fields(arena, array, "ARRAY")?;
    for &entry in get::list(arena, &entries)? {
        let [key, target] = fields(arena, entry, "ENTRY")?;
        let kind = case_fields(arena, key)?.0;
        if (name == "SWITCH_STRING" && kind != "KEY") || (name == "SWITCH_LONG" && kind != "INDEX")
        {
            return Err(Error::new(format!("invalid jump-table key for {name}")));
        }
        let [target] = fields(arena, target, "INT")?;
        check_target(code, index(arena, target)?)?;
    }
    Ok(())
}

fn check_target(code: &[Instruction<'_>], target: usize) -> Result<()> {
    if code.get(target).is_some_and(|op| op.name != "OP_DATA") {
        Ok(())
    } else {
        Err(Error::new(format!("invalid jump target @{target}")))
    }
}

fn smart_producer(name: &str) -> bool {
    matches!(
        name,
        "IS_IDENTICAL"
            | "IS_NOT_IDENTICAL"
            | "IS_EQUAL"
            | "IS_NOT_EQUAL"
            | "IS_SMALLER"
            | "IS_SMALLER_OR_EQUAL"
            | "CASE"
            | "CASE_STRICT"
            | "ISSET_ISEMPTY_CV"
            | "ISSET_ISEMPTY_VAR"
            | "ISSET_ISEMPTY_DIM_OBJ"
            | "ISSET_ISEMPTY_PROP_OBJ"
            | "ISSET_ISEMPTY_STATIC_PROP"
            | "INSTANCEOF"
            | "TYPE_CHECK"
            | "DEFINED"
            | "IN_ARRAY"
            | "ARRAY_KEY_EXISTS"
    )
}
