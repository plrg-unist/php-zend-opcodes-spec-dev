mod binary64;

use std::{
    env,
    error::Error,
    fs,
    io::{self, Read, Write},
    panic,
    path::{Path, PathBuf},
    process::ExitCode,
};

use p4spectec::{
    coverage::{Cover, Hits, InstrHandler, log},
    diagnostic::{Renderer, Report},
    lang::{
        common::prim::num,
        data::{
            typ,
            value::flat::{get, make},
        },
    },
    runner::{Config, build_sl},
};
use serde::Serialize;
use zend_opcodes::{Arena, Value, parse_zop, validate_unit};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn diagnostic(report: &Report) -> String {
    Renderer::new(Default::default())
        .render_to_string(report)
        .unwrap_or_else(|err| format!("{report}\ndiagnostic rendering failed: {err}"))
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Outcome {
    Php {
        stdout: Vec<u8>,
        stderr: Vec<u8>,
        exit_status: u8,
    },
    Unsupported {
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
    Malformed {
        message: String,
    },
    ToolError {
        message: String,
    },
}

fn byte(arena: &Arena, value: &Value) -> Result<u8> {
    Ok(u8::try_from(num::to_int(get::num(arena, value)?))?)
}

fn bytes(arena: &Arena, value: &Value) -> Result<Vec<u8>> {
    get::list(arena, value)?
        .iter()
        .map(|item| byte(arena, item))
        .collect()
}

fn outcome(arena: &mut Arena, result: Value) -> Result<Outcome> {
    get::matches_string! { arena, &result,
        "PHP bytes bytes nat" => |fields| Ok(Outcome::Php {
            stdout: bytes(arena, &fields[0])?,
            stderr: bytes(arena, &fields[1])?,
            exit_status: byte(arena, &fields[2])?,
        }),
        "UNSUPPORTED bytes bytes" => |fields| Ok(Outcome::Unsupported {
            stdout: bytes(arena, &fields[0])?,
            stderr: bytes(arena, &fields[1])?,
        }),
        _ => Err("unexpected result from the execution specification".into()),
    }
}

fn execute(arena: &mut Arena, unit: Value, input: &[u8]) -> Result<Outcome> {
    let values = input
        .iter()
        .map(|&byte| make::nat(arena, u64::from(byte).into(), Default::default()))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let stdin = make::list(
        arena,
        typ::make::list(typ::make::nat()).node.into(),
        values,
        Default::default(),
    )?;

    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("spec");
    let spec = p4spectec::structure(&[dir], true).map_err(|report| diagnostic(&report))?;
    let coverage_path = env::var_os("ZEND_SPEC_COVERAGE").map(PathBuf::from);
    let config = Config::new(false, true, false, coverage_path.is_none());
    let mut runner = build_sl(arena, spec.clone(), config, binary64::Binary64)
        .map_err(|report| diagnostic(&report))?;
    let coverage = coverage_path.map(|path| {
        let cover = Cover::new(&spec);
        let hits = Hits::new(cover.len());
        runner
            .hooks_mut()
            .register(Box::new(InstrHandler::new(hits.clone())));
        (path, cover, hits)
    });

    let result = runner
        .eval_func(arena, "execute", &[], &[unit, stdin])
        .result
        .map_err(|err| diagnostic(&err.into_report()))?;
    let result = outcome(arena, result)?;
    if let Some((path, mut cover, hits)) = coverage {
        cover.add_hits(&hits);
        if cover.measure().total != cover.len() {
            return Err("the execution specification contains a coverage-ignore hint".into());
        }
        log::write_report_to(Some(&path), &spec, &cover, &[])?;
    }
    Ok(result)
}

fn run(path: &Path) -> Result<Outcome> {
    let listing = match fs::read_to_string(path) {
        Ok(listing) => listing,
        Err(err) if err.kind() == io::ErrorKind::InvalidData => {
            return Ok(Outcome::Malformed {
                message: err.to_string(),
            });
        }
        Err(err) => return Err(format!("{}: {err}", path.display()).into()),
    };
    let mut arena = Arena::new();
    let unit = parse_zop(&mut arena, &listing)
        .and_then(|unit| validate_unit(&arena, &unit).map(|()| unit));
    let unit = match unit {
        Ok(unit) => unit,
        Err(err) => {
            return Ok(Outcome::Malformed {
                message: err.to_string(),
            });
        }
    };
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input)?;
    execute(&mut arena, unit, &input)
}

fn write_streams(stdout: &[u8], stderr: &[u8]) -> io::Result<()> {
    let mut out = io::stdout().lock();
    out.write_all(stdout)?;
    out.flush()?;
    let mut err = io::stderr().lock();
    err.write_all(stderr)?;
    err.flush()
}

fn main() -> Result<ExitCode> {
    let mut args = env::args_os().skip(1).peekable();
    let json = args.next_if(|arg| arg == "--json").is_some();
    let file = (args.next(), args.next());
    let invoke = || match &file {
        (Some(file), None) => run(Path::new(file)),
        _ => Err("usage: zend-opcodes [--json] FILE.zop".into()),
    };
    let result = if json {
        panic::set_hook(Box::new(|_| {}));
        panic::catch_unwind(invoke).unwrap_or_else(|cause| {
            let message = cause
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| cause.downcast_ref::<&str>().copied())
                .unwrap_or("unknown panic");
            Err(format!("execution tool panicked: {message}").into())
        })
    } else {
        invoke()
    };
    let result = result.unwrap_or_else(|err| Outcome::ToolError {
        message: err.to_string(),
    });
    if json {
        let mut out = io::stdout().lock();
        serde_json::to_writer(&mut out, &result)?;
        writeln!(out)?;
        return Ok(ExitCode::SUCCESS);
    }
    match result {
        Outcome::Php {
            stdout,
            stderr,
            exit_status,
        } => {
            write_streams(&stdout, &stderr)?;
            Ok(ExitCode::from(exit_status))
        }
        Outcome::Unsupported { stdout, stderr } => {
            write_streams(&stdout, &stderr)?;
            panic!("unsupported PHP operation")
        }
        Outcome::Malformed { message } => panic!("malformed listing: {message}"),
        Outcome::ToolError { message } => panic!("execution tool failed: {message}"),
    }
}
