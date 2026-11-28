use std::{error::Error, fs, path::Path, process::Command};

use p4spectec::lang::traits::eq::SyntaxEq;
use zend_opcodes::{Arena, parse_zop, print_zop, validate_unit};

#[path = "../src/test_spec.rs"]
mod test_spec;

fn zend_fixtures(operation: &str, directory: &Path) -> Result<(), Box<dyn Error>> {
    let helper = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/zend_fixtures.py");
    let output = Command::new(helper)
        .arg(operation)
        .arg(directory)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "zend_fixtures.py {operation} failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(())
}

#[test]
fn corpus_roundtrip() -> Result<(), Box<dyn Error>> {
    let temp = tempfile::tempdir()?;
    zend_fixtures("prepare", temp.path())?;
    let mut arena = Arena::new();
    let mut spec = test_spec::runner(&mut arena);
    let count = fs::read_dir(temp.path())?.count();
    assert!(count > 0, "the fixture helper prepared no cases");
    for index in 0..count {
        let case = temp.path().join(index.to_string());
        let label = fs::read(case.join("label"))?;
        let label = String::from_utf8_lossy(&label);
        let input = fs::read(case.join("original.listing"))?;
        let parsed = str::from_utf8(&input)
            .map_err(|err| err.to_string())
            .and_then(|text| parse_zop(&mut arena, text).map_err(|err| err.to_string()));
        let ast = match parsed {
            Ok(ast) => ast,
            Err(err) if case.join("expected-error").is_file() => {
                fs::write(case.join("parse-error"), err)?;
                continue;
            }
            Err(err) => return Err(format!("{label}: parsing input: {err}").into()),
        };
        if !case.join("expected-error").is_file() {
            validate_unit(&arena, &ast, 32)
                .map_err(|err| format!("{label}: invalid unit: {err}"))?;
        }
        test_spec::check_unit(&mut spec, &mut arena, ast)
            .map_err(|err| format!("{label}: input AST: {err}"))?;
        let printed = print_zop(&arena, &ast).map_err(|err| format!("{label}: printing: {err}"))?;
        let reparsed = parse_zop(&mut arena, &printed)
            .map_err(|err| format!("{label}: parsing printed listing: {err}"))?;
        test_spec::check_unit(&mut spec, &mut arena, reparsed)
            .map_err(|err| format!("{label}: reprinted AST: {err}"))?;
        assert!(
            ast.view(&arena).syntax_eq(&reparsed.view(&arena)),
            "{label}: AST changed on round trip"
        );
        fs::write(case.join("printed.listing"), printed)?;
    }
    zend_fixtures("verify", temp.path())
}
