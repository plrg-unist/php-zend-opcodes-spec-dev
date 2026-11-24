use std::{collections::BTreeMap, ffi::OsString, path::PathBuf};

use serde::Deserialize;

pub(super) type Installation = BTreeMap<String, Vec<u8>>;

#[derive(Deserialize)]
struct InstallationDefaults {
    defaults: BTreeMap<String, String>,
    aliases: BTreeMap<String, String>,
}

pub(super) struct Arguments {
    pub file: PathBuf,
    pub installation: Installation,
}

pub(super) fn parse(
    mut args: impl Iterator<Item = OsString>,
    json: &mut bool,
) -> Result<Arguments, String> {
    const USAGE: &str = "usage: zend-opcodes [--json] [--constant-name VALUE ...] FILE.zop (e.g. --php-binary /example/php)";
    let InstallationDefaults { defaults, aliases } =
        serde_json::from_str(include_str!("../installation-defaults.json"))
            .map_err(|err| err.to_string())?;
    let flags: BTreeMap<_, _> = defaults
        .keys()
        .map(|name| (name, name))
        .chain(&aliases)
        .map(|(name, canonical)| {
            let flag = format!("--{}", name.to_ascii_lowercase().replace('_', "-"));
            (flag, canonical)
        })
        .collect();

    let mut overrides = BTreeMap::new();
    let mut file = None;
    let mut positional = false;
    let mut error = None;
    while let Some(arg) = args.next() {
        if !positional && arg == "--" {
            positional = true;
        } else if !positional && arg == "--json" {
            *json = true;
        } else if !positional && arg.as_encoded_bytes().starts_with(b"--") {
            let mut parts = arg.as_encoded_bytes().splitn(2, |&byte| byte == b'=');
            let flag = String::from_utf8_lossy(parts.next().unwrap());
            let Some(&name) = flags.get(flag.as_ref()) else {
                error.get_or_insert_with(|| format!("unknown option {flag}"));
                continue;
            };
            let value = parts
                .next()
                .map(<[u8]>::to_vec)
                .or_else(|| args.next().map(OsString::into_encoded_bytes));
            let Some(value) = value else {
                error.get_or_insert_with(|| format!("missing value for {flag}"));
                break;
            };
            if overrides
                .get(name)
                .is_some_and(|previous| previous != &value)
            {
                error.get_or_insert_with(|| format!("conflicting values for {name}"));
            }
            overrides.insert(name.clone(), value);
        } else if file.replace(PathBuf::from(arg)).is_some() {
            error.get_or_insert_with(|| USAGE.to_owned());
        }
    }
    if let Some(error) = error {
        return Err(error);
    }

    let installation = defaults
        .into_iter()
        .map(|(name, default)| {
            let value = overrides
                .remove(&name)
                .unwrap_or_else(|| default.into_bytes());
            (name, value)
        })
        .collect();
    Ok(Arguments {
        file: file.ok_or(USAGE)?,
        installation,
    })
}
