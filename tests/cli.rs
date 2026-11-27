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
            .arg("--input")
            .arg(format!("constants.PHP_BINARY={json}"))
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
        &["--input", "--", "--unknown", "--json", listing],
        &[listing, listing, "--json"],
        &["--json", listing, "--input", "opline_size_bytes=0"],
        &["--json", listing, "--input", "opline_size_bytes=-1"],
        &["--json", listing, "--input", "opline_size_bytes=1.5"],
        &["--json", listing, "--input", r#"opline_size_bytes="56""#],
        &["--json", listing, "--input", "opline_size_bytes=true"],
        &["--json", listing, "--input", "unknown=42"],
        &[
            "--json",
            listing,
            "--input",
            "opline_size_bytes=32",
            "--input",
            "opline_size_bytes=56",
        ],
        &[
            "--input",
            r#"constants.PHP_EXTENSION_DIR="A""#,
            "--input",
            r#"constants.PEAR_EXTENSION_DIR="B""#,
            "--json",
            listing,
        ],
        &["--json", listing, "--input"],
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
    let table = dir.path().join("inputs.json");
    let listing = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/project/handwritten.zop");
    for (contents, overrides) in [
        (r#"{"TrUe":false}"#, &[][..]),
        (r#"{"A":1,"A":2}"#, &[]),
        (r#"{"A":{"alias":"B"}}"#, &[]),
        (r#"{"A":{"value":2,"deprecated":{"since":null}}}"#, &[]),
        (r#"{}"#, &["--input", "constants.NuLl=0"]),
        (r#"{}"#, &["--input", "constants.A=9223372036854775808"]),
        (r#"{}"#, &["--input", r#"constants.A={"float_bits":"xyz"}"#]),
        (
            r#"{"A":1,"B":{"alias":"A"}}"#,
            &["--input", "constants.A=1", "--input", "constants.B=true"],
        ),
    ] {
        std::fs::write(
            &table,
            format!(r#"{{"opline_size_bytes":32,"constants":{contents}}}"#),
        )?;
        let output = Command::new(env!("CARGO_BIN_EXE_zend-opcodes"))
            .args(["--json", "--inputs"])
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
fn opline_size_comes_from_the_input_file_and_accepts_overrides() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    let file = dir.path().join("inputs.json");
    std::fs::write(
        &file,
        r#"{"opline_size_bytes":40,"constants":{"opline_size_bytes":9}}"#,
    )?;
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/project");
    let offsets = fixtures.join("array_dispatch_offsets.zop");
    let lookup = dir.path().join("lookup.zop");
    let template = std::fs::read_to_string(fixtures.join("installation_php_binary.zop"))?;
    std::fs::write(&lookup, template.replace("PHP_BINARY", "opline_size_bytes"))?;
    for (listing, overrides, expected) in [
        (&offsets, &[][..], "40,-40,0\n"),
        (&offsets, &["--input", "opline_size_bytes=28"], "28,-28,0\n"),
        (
            &offsets,
            &["--input", "opline_size_bytes=18446744073709551617"],
            "1,-1,0\n",
        ),
        (
            &lookup,
            &["--input", "opline_size_bytes=18446744073709551616"],
            "9",
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_zend-opcodes"))
            .arg("--inputs")
            .arg(&file)
            .args(overrides)
            .arg(listing)
            .output()?;
        assert!(
            output.status.success() && output.stderr.is_empty(),
            "{output:?}"
        );
        assert_eq!(output.stdout, expected.as_bytes());
    }
    Ok(())
}

#[test]
fn help_lists_the_runtime_inputs() -> io::Result<()> {
    for args in [&["-h"][..], &["--help"], &["--json", "--help"]] {
        let output = Command::new(env!("CARGO_BIN_EXE_zend-opcodes"))
            .args(args)
            .output()?;
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty());
        let help = String::from_utf8(output.stdout).unwrap();
        assert!(help.contains("<FILE.zop>") && help.contains("--json"));
        for flag in ["--inputs <FILE>", "--input <KEY=JSON>"] {
            assert!(help.contains(flag), "help is missing {flag}");
        }
        assert!(help.contains("runtime-inputs.json"));
        assert!(!help.contains("--constant"));
    }
    Ok(())
}

#[test]
fn usage_errors_go_to_stderr() -> io::Result<()> {
    for (args, detail) in [
        (&[][..], "<FILE.zop>"),
        (&[""], "<FILE.zop>"),
        (&["--input"], "--input"),
        (
            &[
                "--php-binary",
                "/example/php",
                "tests/project/handwritten.zop",
            ],
            "unexpected argument '--php-binary'",
        ),
        (&["--unknown", "--inputs", "--json"], "--unknown"),
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
                r#"--input=constants.PHP_EXTENSION_DIR="A""#,
                r#"--input=constants.PEAR_EXTENSION_DIR="A""#,
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
