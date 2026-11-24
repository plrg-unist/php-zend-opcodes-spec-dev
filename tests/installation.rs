use std::{ffi::OsStr, io, path::Path, process::Command};

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

#[test]
fn fetch_constant_prints_the_cli_value() -> io::Result<()> {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/project/installation_php_binary.zop");
    for value in [
        OsStr::new("--json"),
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
