use std::{ffi::OsString, path::PathBuf};

use clap::{Arg, ArgAction, Command, error::ErrorKind, value_parser};

use crate::inputs::{REFERENCE, RuntimeInputs};

pub(super) struct Arguments {
    pub file: PathBuf,
    pub inputs: RuntimeInputs,
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
            Arg::new("inputs")
                .long("inputs")
                .value_name("FILE")
                .value_parser(value_parser!(PathBuf))
                .allow_hyphen_values(true)
                .help("Complete runtime inputs (default: repository runtime-inputs.json)"),
        )
        .arg(
            Arg::new("input")
                .long("input")
                .value_name("KEY=JSON")
                .action(ArgAction::Append)
                .allow_hyphen_values(true)
                .help(
                    "Set opline_size_bytes or constants.NAME after loading the inputs (repeatable)",
                ),
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
        if arg == "--inputs" || arg == "--input" {
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
        .remove_one::<PathBuf>("inputs")
        .unwrap_or_else(|| PathBuf::from(REFERENCE));
    let mut inputs = RuntimeInputs::read(&path)
        .map_err(|err| cmd.error(ErrorKind::ValueValidation, err.to_string()))?;

    let overrides = matches.get_many::<String>("input").into_iter().flatten();
    inputs
        .apply(overrides.map(String::as_str))
        .map_err(|err| cmd.error(ErrorKind::ValueValidation, err.to_string()))?;
    Ok(Arguments { file, inputs })
}
