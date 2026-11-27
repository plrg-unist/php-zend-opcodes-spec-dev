use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use clap::{Arg, ArgAction, Command, error::ErrorKind, value_parser};

use crate::constants::{ConstantValue, Constants, REFERENCE};

pub(super) struct Arguments {
    pub file: PathBuf,
    pub constants: Constants,
}

// These existing options accept literal OS bytes. All other names use --constant.
const INSTALLATION_OPTIONS: &[&str] = &[
    "DEFAULT_INCLUDE_PATH",
    "PEAR_EXTENSION_DIR",
    "PEAR_INSTALL_DIR",
    "PHP_BINARY",
    "PHP_BINDIR",
    "PHP_BUILD_DATE",
    "PHP_CONFIG_FILE_PATH",
    "PHP_CONFIG_FILE_SCAN_DIR",
    "PHP_DATADIR",
    "PHP_EXTENSION_DIR",
    "PHP_LIBDIR",
    "PHP_LOCALSTATEDIR",
    "PHP_MANDIR",
    "PHP_PREFIX",
    "PHP_SBINDIR",
    "PHP_SYSCONFDIR",
];

fn command(reference: Option<&Constants>) -> Command {
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
        );
    for &name in INSTALLATION_OPTIONS {
        let mut help = format!("Set {name} as a PHP byte string");
        if let Some(bytes) = reference.and_then(|table| table.default_bytes(name)) {
            help.push_str(&format!(" [default: {}]", String::from_utf8_lossy(bytes)));
        }
        cmd = cmd.arg(
            Arg::new(name)
                .long(name.to_ascii_lowercase().replace('_', "-"))
                .help_heading("Installation constants")
                .value_name("VALUE")
                .action(ArgAction::Append)
                .allow_hyphen_values(true)
                .value_parser(value_parser!(OsString))
                .help(help),
        );
    }
    cmd
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
    let mut cmd = command(None);
    let args = prepare_args(&cmd, args, json);
    let mut matches = match cmd.try_get_matches_from_mut(&args) {
        Err(error) if error.kind() == ErrorKind::DisplayHelp => {
            let reference = Constants::read(Path::new(REFERENCE)).ok();
            return Err(command(reference.as_ref())
                .try_get_matches_from(args)
                .unwrap_err());
        }
        result => result?,
    };
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
    for &name in INSTALLATION_OPTIONS {
        for raw in matches.remove_many::<OsString>(name).into_iter().flatten() {
            overrides.push((
                name.to_owned(),
                ConstantValue::String(raw.into_encoded_bytes()),
            ));
        }
    }
    constants
        .apply(overrides)
        .map_err(|err| cmd.error(ErrorKind::ArgumentConflict, err.to_string()))?;
    Ok(Arguments { file, constants })
}
