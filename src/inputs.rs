use std::{fs, path::Path};

use serde::Deserialize;
use serde_json::value::RawValue;

use crate::{
    Result,
    constants::{ConstantValue, Constants},
};

pub(super) const REFERENCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/runtime-inputs.json");

pub(super) struct RuntimeInputs {
    pub opline_size_bytes: i32,
    pub constants: Constants,
}

fn opline_size(raw: &RawValue) -> Result<i32> {
    raw.get()
        .parse::<i32>()
        .ok()
        .filter(|&bytes| bytes > 0)
        .ok_or_else(|| "opline_size_bytes must be a JSON integer from 1 to 2147483647".into())
}

impl RuntimeInputs {
    pub fn read(path: &Path) -> Result<Self> {
        let data = fs::read(path).map_err(|err| format!("{}: {err}", path.display()))?;
        Self::parse(&data).map_err(|err| format!("{}: {err}", path.display()).into())
    }

    fn parse(data: &[u8]) -> Result<Self> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct InputFile<'a> {
            #[serde(borrow)]
            opline_size_bytes: &'a RawValue,
            #[serde(borrow)]
            constants: &'a RawValue,
        }

        // Serde also accepts positional arrays when deserializing a struct.
        if !data.trim_ascii_start().starts_with(b"{") {
            return Err("runtime inputs must be a JSON object".into());
        }
        let file: InputFile = serde_json::from_slice(data)?;
        Ok(Self {
            opline_size_bytes: opline_size(file.opline_size_bytes)?,
            constants: Constants::parse(file.constants)?,
        })
    }

    pub fn apply<'a>(&mut self, overrides: impl Iterator<Item = &'a str>) -> Result<()> {
        let mut size = None;
        let mut constants = Vec::new();
        for option in overrides {
            let (key, text) = option.split_once('=').ok_or("--input expects KEY=JSON")?;
            if key == "opline_size_bytes" {
                let value = opline_size(serde_json::from_str(text)?)?;
                if size.as_ref().is_some_and(|previous| *previous != value) {
                    return Err("conflicting values for opline_size_bytes".into());
                }
                size = Some(value);
            } else if let Some(name) = key.strip_prefix("constants.") {
                let value = ConstantValue::from_json(text)
                    .map_err(|err| format!("constant {name:?}: {err}"))?;
                constants.push((name.to_owned(), value));
            } else {
                return Err(format!("unknown runtime input {key:?}").into());
            }
        }
        self.constants.apply(constants)?;
        if let Some(size) = size {
            self.opline_size_bytes = size;
        }
        Ok(())
    }
}
