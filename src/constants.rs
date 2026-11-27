use std::{
    collections::{BTreeMap, btree_map::Entry},
    fmt, fs,
    path::Path,
};

use p4spectec::{
    lang::data::{
        notation::flat::{Mixfix, parse::ToMixop},
        typ,
        value::flat::make,
    },
    phrase,
};
use serde::{Deserialize, Deserializer, de};
use serde_json::value::RawValue;
use zend_opcodes::{Arena, Value};

use crate::{Result, byte_list};

pub(super) const REFERENCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/runtime-constants.json");

#[derive(PartialEq, Eq)]
pub(super) enum ConstantValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(u64),
    String(Vec<u8>),
    Stream(i64),
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum TaggedValue {
    Bytes(Vec<u8>),
    FloatBits(String),
    Stream(i64),
}

impl ConstantValue {
    pub(super) fn from_json(text: &str) -> Result<Self> {
        Self::parse(serde_json::from_str(text)?)
    }

    fn parse(raw: &RawValue) -> Result<Self> {
        let text = raw.get();
        Ok(match text.as_bytes()[0] {
            b'n' => Self::Null,
            b't' => Self::Bool(true),
            b'f' => Self::Bool(false),
            b'"' => Self::String(serde_json::from_str::<String>(text)?.into_bytes()),
            b'-' | b'0'..=b'9' => {
                if text.contains(['.', 'e', 'E']) {
                    let value: f64 = text.parse()?;
                    if !value.is_finite() {
                        return Err("a non-finite float requires float_bits".into());
                    }
                    Self::Float(value.to_bits())
                } else {
                    Self::Int(text.parse()?)
                }
            }
            b'{' => match serde_json::from_str(text)? {
                TaggedValue::Bytes(bytes) => Self::String(bytes),
                TaggedValue::FloatBits(bits) => {
                    if bits.len() != 16 || !bits.bytes().all(|c| c.is_ascii_hexdigit()) {
                        return Err("float_bits requires 16 hexadecimal digits".into());
                    }
                    Self::Float(u64::from_str_radix(&bits, 16)?)
                }
                TaggedValue::Stream(id) if id > 0 => Self::Stream(id),
                TaggedValue::Stream(_) => {
                    return Err("stream requires a positive resource ID".into());
                }
            },
            _ => return Err("expected a PHP scalar, byte string, or stream".into()),
        })
    }

    fn spec_value(&self, arena: &mut Arena) -> Result<Value> {
        let (shape, args) = match self {
            Self::Null => ("NULL", vec![]),
            Self::Bool(false) => ("FALSE", vec![]),
            Self::Bool(true) => ("TRUE", vec![]),
            Self::Int(n) => (
                "INT int",
                vec![make::int(arena, (*n).into(), Default::default())?],
            ),
            Self::Float(bits) => (
                "FLOAT nat",
                vec![make::nat(arena, (*bits).into(), Default::default())?],
            ),
            Self::String(bytes) => ("STRING bytes", vec![byte_list(arena, bytes)?]),
            Self::Stream(id) => (
                "STREAM nat",
                vec![make::nat(arena, (*id as u64).into(), Default::default())?],
            ),
        };
        Ok(
            make::case_string! { arena: arena, shape: shape, args: args, typ: "slotValue", span: Default::default() }?,
        )
    }
}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Deprecation {
    since: String,
    message: String,
}

impl Deprecation {
    fn diagnostic(&self, name: &str) -> String {
        let mut warning = format!("Constant {name} is deprecated");
        if !self.since.is_empty() {
            warning.push_str(" since ");
            warning.push_str(&self.since);
        }
        if !self.message.is_empty() {
            warning.push_str(", ");
            warning.push_str(&self.message);
        }
        warning
    }
}

// Keep each value's original number spelling, including integers at the signed
// 64-bit limits, and report duplicate names before building the lookup table.
struct Fields<'a>(BTreeMap<String, &'a RawValue>);

impl<'de> Deserialize<'de> for Fields<'de> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Fields<'de>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a JSON object")
            }

            fn visit_map<A: de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> std::result::Result<Self::Value, A::Error> {
                let mut fields = BTreeMap::new();
                while let Some((name, value)) = map.next_entry::<String, &RawValue>()? {
                    if fields.contains_key(&name) {
                        return Err(de::Error::custom(format!("duplicate field {name:?}")));
                    }
                    fields.insert(name, value);
                }
                Ok(Fields(fields))
            }
        }
        deserializer.deserialize_map(Visitor)
    }
}

struct Constant {
    value: ConstantValue,
    deprecated: Option<Deprecation>,
}

enum Definition {
    Value(Constant),
    Alias(String),
}

impl Definition {
    fn parse(raw: &RawValue) -> Result<Self> {
        if raw.get().starts_with('{') {
            let Fields(fields) = serde_json::from_str(raw.get())?;
            if let Some(target) = fields.get("alias") {
                if fields.len() != 1 {
                    return Err("alias takes only a target name".into());
                }
                let name: String = serde_json::from_str(target.get())?;
                return Ok(Self::Alias(lookup_key(&name)));
            }
            if let Some(value) = fields.get("value") {
                let warning = fields
                    .get("deprecated")
                    .ok_or("expected deprecated metadata alongside value")?;
                if fields.len() != 2 {
                    return Err("unexpected field alongside value and deprecated".into());
                }
                return Ok(Self::Value(Constant {
                    value: ConstantValue::parse(value)?,
                    deprecated: Some(serde_json::from_str(warning.get())?),
                }));
            }
        }
        Ok(Self::Value(Constant {
            value: ConstantValue::parse(raw)?,
            deprecated: None,
        }))
    }
}

// Zend lowercases the namespace prefix up to the last separator before a NUL.
fn lookup_key(name: &str) -> String {
    let mut key = name.to_owned();
    if let Some(i) = name.split('\0').next().unwrap_or_default().rfind('\\') {
        key[..i].make_ascii_lowercase();
    }
    key
}

fn input_key(name: &str) -> Result<String> {
    if ["true", "false", "null"]
        .iter()
        .any(|core| name.eq_ignore_ascii_case(core))
    {
        return Err(format!("constant {name:?} is a core PHP literal").into());
    }
    Ok(lookup_key(name))
}

// Keep the declared spelling for diagnostics and the registered key for lookup.
struct Binding {
    name: String,
    definition: Definition,
}

pub(super) struct Constants(BTreeMap<String, Binding>);

impl Constants {
    pub(super) fn read(path: &Path) -> Result<Self> {
        let data = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Self::from_slice(&data).map_err(|e| format!("{}: {e}", path.display()).into())
    }

    fn from_slice(data: &[u8]) -> Result<Self> {
        let Fields(fields) = serde_json::from_slice(data)?;
        let mut entries = BTreeMap::new();
        for (name, raw) in fields {
            let key = input_key(&name)?;
            let definition =
                Definition::parse(raw).map_err(|e| format!("constant {name:?}: {e}"))?;
            if entries
                .insert(key.clone(), Binding { name, definition })
                .is_some()
            {
                return Err(format!("duplicate registered constant {key:?}").into());
            }
        }
        let table = Self(entries);
        for (key, binding) in &table.0 {
            if table.get(key).is_none() {
                return Err(format!(
                    "alias {:?} must refer directly to a value in this table",
                    binding.name
                )
                .into());
            }
        }
        Ok(table)
    }

    fn target<'a>(&'a self, key: &'a str) -> &'a str {
        match self.0.get(key).map(|b| &b.definition) {
            Some(Definition::Alias(target)) => target,
            _ => key,
        }
    }

    fn get(&self, key: &str) -> Option<&Constant> {
        match &self.0.get(self.target(key))?.definition {
            Definition::Value(entry) => Some(entry),
            Definition::Alias(_) => None,
        }
    }

    pub(super) fn default_bytes(&self, name: &str) -> Option<&[u8]> {
        match &self.get(name)?.value {
            ConstantValue::String(bytes) => Some(bytes),
            _ => None,
        }
    }

    pub(super) fn apply(&mut self, overrides: Vec<(String, ConstantValue)>) -> Result<()> {
        let mut changes = BTreeMap::new();
        for (name, value) in overrides {
            let key = input_key(&name)?;
            let target = self.target(&key).to_owned();
            match changes.entry(target) {
                Entry::Vacant(e) => {
                    e.insert((name, value));
                }
                Entry::Occupied(e) if e.get().1 == value => {}
                Entry::Occupied(_) => {
                    return Err(format!("conflicting values for constant {name:?}").into());
                }
            }
        }
        for (key, (name, value)) in changes {
            match self.0.entry(key) {
                Entry::Occupied(mut entry) => {
                    let Definition::Value(constant) = &mut entry.get_mut().definition else {
                        unreachable!()
                    };
                    constant.value = value;
                }
                Entry::Vacant(entry) => {
                    entry.insert(Binding {
                        name,
                        definition: Definition::Value(Constant {
                            value,
                            deprecated: None,
                        }),
                    });
                }
            }
        }
        Ok(())
    }

    pub(super) fn spec_value(&self, arena: &mut Arena) -> Result<Value> {
        let named = |name: &str, args| {
            typ::make::var(phrase!(node: name.into(), span: Default::default()), args)
        };
        let bytes_type = named("bytes", vec![]);
        let info_type = named("constantInfo", vec![]);
        let pair_type = named("pair", vec![bytes_type.clone(), info_type]);
        let notation = "k ':' v".to_mixop(&mut arena.mixop)?;
        let mut pairs = Vec::with_capacity(self.0.len());
        for (key, binding) in &self.0 {
            let constant = self.get(key).expect("validated alias");
            let name = byte_list(arena, key.as_bytes())?;
            let value = constant.value.spec_value(arena)?;
            let warning = constant
                .deprecated
                .as_ref()
                .map(|d| byte_list(arena, d.diagnostic(&binding.name).as_bytes()))
                .transpose()?;
            let warning = make::opt(
                arena,
                typ::make::opt(bytes_type.clone()).node.into(),
                warning,
                Default::default(),
            )?;
            let info = make::case_string! { arena: arena, shape: "CONST_INFO slotValue bytes", args: [value, warning], typ: "constantInfo", span: Default::default() }?;
            let pair = Mixfix::new(&arena.mixop, notation, vec![name, info])?;
            pairs.push(make::case_flat(
                arena,
                pair_type.node.clone().into(),
                pair,
                Default::default(),
            )?);
        }
        let pairs = make::list(
            arena,
            typ::make::list(pair_type).node.into(),
            pairs,
            Default::default(),
        )?;
        Ok(
            make::case_string! { arena: arena, shape: "`{ pairs `}", args: [pairs], typ: "predefinedConstants", span: Default::default() }?,
        )
    }
}
