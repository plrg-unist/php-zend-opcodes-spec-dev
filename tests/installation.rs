use std::{ffi::OsStr, io, path::Path, process::Command};

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

#[test]
fn fetch_constant_prints_the_cli_value() -> io::Result<()> {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/project/installation_php_binary.zop");
    for value in [
        OsStr::new("--json"),
        OsStr::new("--help"),
        OsStr::new(""),
        #[cfg(unix)]
        OsStr::from_bytes(b"/example/\xff"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_zend-opcodes"))
            .arg("--php-binary")
            .arg(value)
            .arg(&path)
            .output()?;
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty());
        assert_eq!(output.stdout, value.as_encoded_bytes());
    }
    Ok(())
}

#[test]
fn json_reports_invalid_arguments() -> io::Result<()> {
    for args in [
        &["--unknown", "--json", "input.zop"][..],
        &["--php-binary", "--", "--unknown", "--json"],
        &["first.zop", "second.zop", "--json"],
        &[
            "--php-extension-dir=A",
            "--pear-extension-dir=B",
            "--json",
            "input.zop",
        ],
        &["--json", "input.zop", "--php-binary"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_zend-opcodes"))
            .args(args)
            .output()?;
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty());
        let outcome: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        assert_eq!(outcome["kind"], "tool_error");
        assert!(!outcome["message"].as_str().unwrap().is_empty());
    }
    Ok(())
}

#[test]
fn invalid_runtime_constants_fail_before_program_execution() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    let table = dir.path().join("constants.json");
    let listing = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/project/handwritten.zop");
    for (contents, overrides) in [
        (r#"{"TrUe":false}"#, &[][..]),
        (r#"{"A":1,"A":2}"#, &[]),
        (r#"{"A":{"alias":"B"}}"#, &[]),
        (r#"{"A":{"value":2,"deprecated":{"since":null}}}"#, &[]),
        (r#"{}"#, &["--constant", "NuLl=0"]),
        (r#"{}"#, &["--constant", "A=9223372036854775808"]),
        (r#"{}"#, &["--constant", r#"A={"float_bits":"xyz"}"#]),
        (
            r#"{"A":1,"B":{"alias":"A"}}"#,
            &["--constant", "A=1", "--constant", "B=true"],
        ),
    ] {
        std::fs::write(&table, contents)?;
        let output = Command::new(env!("CARGO_BIN_EXE_zend-opcodes"))
            .args(["--json", "--constants"])
            .arg(&table)
            .args(overrides)
            .arg(&listing)
            .output()?;
        assert!(
            output.status.success() && output.stderr.is_empty(),
            "{output:?}"
        );
        let outcome: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        assert_eq!(
            outcome["kind"], "tool_error",
            "{contents} {overrides:?}: {outcome}"
        );
    }
    Ok(())
}

#[test]
fn help_lists_the_constant_inputs() -> io::Result<()> {
    let catalog: serde_json::Value =
        serde_json::from_str(include_str!("../runtime-constants.json"))?;
    for args in [&["-h"][..], &["--help"], &["--json", "--help"]] {
        let output = Command::new(env!("CARGO_BIN_EXE_zend-opcodes"))
            .args(args)
            .output()?;
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty());
        let help = String::from_utf8(output.stdout).unwrap();
        assert!(help.contains("<FILE.zop>") && help.contains("--json"));
        for flag in [
            "--constants",
            "--constant",
            "--php-binary",
            "--default-include-path",
            "--php-extension-dir",
            "--pear-extension-dir",
        ] {
            assert!(help.contains(flag), "help is missing {flag}");
        }
        assert!(help.contains(catalog["PHP_BINARY"].as_str().unwrap()));
    }
    Ok(())
}

#[test]
fn usage_errors_go_to_stderr() -> io::Result<()> {
    for (args, detail) in [
        (&[][..], "<FILE.zop>"),
        (&[""], "<FILE.zop>"),
        (&["--php-binary"], "--php-binary"),
        (&["--unknown", "--php-binary", "--json"], "--unknown"),
        (&["--unknown", "--", "--json"], "--unknown"),
        (&["one.zop", "two.zop"], "two.zop"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_zend-opcodes"))
            .args(args)
            .output()?;
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        assert!(output.stdout.is_empty());
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(
            error.contains(detail) && error.contains("--help"),
            "{error}"
        );
    }
    Ok(())
}

#[test]
fn option_like_listing_paths_keep_their_spelling() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/project/array_cli_server_keys.zop");
    for (name, args) in [
        (
            "-help.zop",
            &[
                "-help.zop",
                "--php-extension-dir=A",
                "--pear-extension-dir=A",
            ][..],
        ),
        ("--json", &["--", "--json"][..]),
    ] {
        std::fs::copy(&fixture, dir.path().join(name))?;
        let output = Command::new(env!("CARGO_BIN_EXE_zend-opcodes"))
            .current_dir(dir.path())
            .args(args)
            .output()?;
        assert!(output.status.success(), "{output:?}");
        assert!(
            output.stdout.starts_with(format!("{name}\n").as_bytes()),
            "{output:?}"
        );
    }
    Ok(())
}
