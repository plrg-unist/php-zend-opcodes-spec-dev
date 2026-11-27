use std::{ffi::OsString, path::PathBuf};

use clap::{Arg, ArgAction, Command, error::ErrorKind, value_parser};

use crate::constants::{ConstantValue, Constants, REFERENCE};

pub(super) struct Arguments {
    pub file: PathBuf,
    pub constants: Constants,
}

fn command() -> Command {
    Command::new("zend-opcodes")
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
        )
        .arg(
            Arg::new("constants")
                .long("constants")
                .value_name("FILE")
                .value_parser(value_parser!(PathBuf))
                .allow_hyphen_values(true)
                .help(
                    "Complete runtime constant table (default: repository runtime-constants.json)",
                ),
        )
        .arg(
            Arg::new("constant")
                .long("constant")
                .value_name("NAME=JSON")
                .action(ArgAction::Append)
                .allow_hyphen_values(true)
                .help("Add or override a runtime constant after loading the table (repeatable)"),
        )
}

fn prepare_args(mut args: impl Iterator<Item = OsString>, json: &mut bool) -> Vec<OsString> {
    let mut options = Vec::new();
    let mut paths = Vec::new();
    // Select JSON even after a usage error. Keep option-like values with their flag
    // and pass filenames literally, including single-dash names visible in PHP's argv.
    while let Some(arg) = args.next() {
        if arg == "--" {
            paths.extend(args);
            break;
        }
        if arg == "--constants" || arg == "--constant" {
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
    let mut cmd = command();
    let mut matches = cmd.try_get_matches_from_mut(prepare_args(args, json))?;
    let file = matches.remove_one("file").expect("required listing path");
    let path = matches
        .remove_one::<PathBuf>("constants")
        .unwrap_or_else(|| PathBuf::from(REFERENCE));
    let mut constants = Constants::read(&path)
        .map_err(|err| cmd.error(ErrorKind::ValueValidation, err.to_string()))?;

    let mut overrides = Vec::new();
    for option in matches
        .remove_many::<String>("constant")
        .into_iter()
        .flatten()
    {
        let (name, text) = option
            .split_once('=')
            .ok_or_else(|| cmd.error(ErrorKind::ValueValidation, "--constant expects NAME=JSON"))?;
        let value = ConstantValue::from_json(text).map_err(|err| {
            cmd.error(
                ErrorKind::ValueValidation,
                format!("constant {name:?}: {err}"),
            )
        })?;
        overrides.push((name.to_owned(), value));
    }
    constants
        .apply(overrides)
        .map_err(|err| cmd.error(ErrorKind::ArgumentConflict, err.to_string()))?;
    Ok(Arguments { file, constants })
}
