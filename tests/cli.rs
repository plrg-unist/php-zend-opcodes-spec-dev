use std::{io, path::Path, process::Command};

#[test]
fn fetch_constant_prints_the_cli_value() -> io::Result<()> {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/project/installation_php_binary.zop");
    for (json, expected) in [
        (r#""--json""#, b"--json".as_slice()),
        (r#""--help""#, b"--help"),
        (r#""""#, b""),
        (r#"{"bytes":[0,255]}"#, b"\0\xff"),
        (r#"{"stream":42}"#, b"Resource id #42"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_zend-opcodes"))
            .arg("--constant")
            .arg(format!("PHP_BINARY={json}"))
            .arg(&path)
            .output()?;
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty());
        assert_eq!(output.stdout, expected);
    }
    Ok(())
}

#[test]
fn json_reports_invalid_arguments() -> io::Result<()> {
    let listing = "tests/project/handwritten.zop";
    for args in [
        &["--unknown", "--json", listing][..],
        &["--constant", "--", "--unknown", "--json", listing],
        &[listing, listing, "--json"],
        &[
            "--constant",
            r#"PHP_EXTENSION_DIR="A""#,
            "--constant",
            r#"PEAR_EXTENSION_DIR="B""#,
            "--json",
            listing,
        ],
        &["--json", listing, "--constant"],
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
    for args in [&["-h"][..], &["--help"], &["--json", "--help"]] {
        let output = Command::new(env!("CARGO_BIN_EXE_zend-opcodes"))
            .args(args)
            .output()?;
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty());
        let help = String::from_utf8(output.stdout).unwrap();
        assert!(help.contains("<FILE.zop>") && help.contains("--json"));
        for flag in ["--constants <FILE>", "--constant <NAME=JSON>"] {
            assert!(help.contains(flag), "help is missing {flag}");
        }
        assert!(help.contains("runtime-constants.json"));
        assert!(!help.contains("Installation constants"));
    }
    Ok(())
}

#[test]
fn usage_errors_go_to_stderr() -> io::Result<()> {
    for (args, detail) in [
        (&[][..], "<FILE.zop>"),
        (&[""], "<FILE.zop>"),
        (&["--constant"], "--constant"),
        (
            &[
                "--php-binary",
                "/example/php",
                "tests/project/handwritten.zop",
            ],
            "unexpected argument '--php-binary'",
        ),
        (&["--unknown", "--constants", "--json"], "--unknown"),
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
                r#"--constant=PHP_EXTENSION_DIR="A""#,
                r#"--constant=PEAR_EXTENSION_DIR="A""#,
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
