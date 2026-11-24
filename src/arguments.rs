use std::{collections::BTreeMap, ffi::OsString, path::PathBuf};

use clap::{Arg, ArgAction, Command, error::ErrorKind, value_parser};
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

fn flag_name(name: &str) -> String {
    name.to_ascii_lowercase().replace('_', "-")
}

impl InstallationDefaults {
    fn command(&self) -> Command {
        let mut cmd = Command::new("zend-opcodes")
            .no_binary_name(true)
            .about("Execute a Zend opcode listing using the specification")
            .arg(
                Arg::new("file")
                    .value_name("FILE.zop")
                    .required(true)
                    .value_parser(value_parser!(PathBuf))
                    .help("Listing to execute. PHP input is read from stdin"),
            )
            .arg(
                Arg::new("json")
                    .long("json")
                    .action(ArgAction::SetTrue)
                    .overrides_with("json")
                    .help("Write the result as JSON and exit with status 0"),
            );
        for (name, default) in &self.defaults {
            let aliases = self
                .aliases
                .iter()
                .filter(|(_, target)| *target == name)
                .map(|(alias, _)| flag_name(alias));
            cmd = cmd.arg(
                Arg::new(name)
                    .long(flag_name(name))
                    .visible_aliases(aliases)
                    .help_heading("Installation constants")
                    .value_name("VALUE")
                    .default_value(default)
                    .action(ArgAction::Append)
                    .allow_hyphen_values(true)
                    .value_parser(value_parser!(OsString)),
            );
        }
        cmd
    }
}

fn prepare_args(
    cmd: &Command,
    mut args: impl Iterator<Item = OsString>,
    json: &mut bool,
) -> Vec<OsString> {
    let value_flags: Vec<_> = cmd
        .get_arguments()
        .filter(|arg| arg.get_action().takes_values())
        .filter_map(Arg::get_long_and_visible_aliases)
        .flatten()
        .map(|name| OsString::from(format!("--{name}")))
        .collect();
    let mut options = Vec::new();
    let mut paths = Vec::new();
    // Select JSON even after a usage error. Keep option-like values with their flag
    // and pass filenames literally, including single-dash names visible in PHP's argv.
    while let Some(arg) = args.next() {
        if arg == "--" {
            paths.extend(args);
            break;
        }
        if value_flags.contains(&arg) {
            options.push(arg);
            let Some(value) = args.next() else {
                return options;
            };
            options.push(value);
        } else if arg == "-h" || arg.as_encoded_bytes().starts_with(b"--") {
            *json |= arg == "--json";
            options.push(arg);
        } else {
            paths.push(arg);
        }
    }
    if !paths.is_empty() {
        options.push("--".into());
        options.extend(paths);
    }
    options
}

pub(super) fn parse(
    args: impl Iterator<Item = OsString>,
    json: &mut bool,
) -> Result<Arguments, clap::Error> {
    let defaults: InstallationDefaults =
        serde_json::from_str(include_str!("../installation-defaults.json"))
            .map_err(|err| clap::Error::raw(ErrorKind::Io, err))?;
    let mut cmd = defaults.command();
    let args = prepare_args(&cmd, args, json);
    let mut matches = cmd.try_get_matches_from_mut(args)?;
    let file = matches.remove_one("file").expect("required listing path");
    let mut installation = Installation::new();
    for name in defaults.defaults.into_keys() {
        let mut values = matches
            .remove_many::<OsString>(&name)
            .expect("option has a default");
        let value = values.next().expect("option takes one value");
        if values.any(|other| other != value) {
            return Err(cmd.error(
                ErrorKind::ArgumentConflict,
                format!("conflicting values for '--{}' ({name})", flag_name(&name)),
            ));
        }
        installation.insert(name, value.into_encoded_bytes());
    }
    Ok(Arguments { file, installation })
}
