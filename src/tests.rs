use super::{Arena, parse_zop, print_zop};
use p4spectec::lang::{
    data::value::flat::make,
    traits::{eq::SyntaxEq, print::Print},
};

#[path = "test_spec.rs"]
mod test_spec;

const BYTES_AND_LITERAL: &str = r#"zend-opcodes
file "f\xff"
halt 11
autoglobal "_ENV"
main
body - {
  lines 2 4
  frame 0 cache 0
  size 2 literals 2 children 0
  lit[0] = "A\x00\xff\\\""
  lit[1] = int(-7)
  code
    0: ECHO lit[0] | line=2
    1: RETURN lit[1] | line=4
}
end
"#;

const BRANCH_AND_ARRAY: &str = r#"zend-opcodes
file "c"
main
body - {
  lines 7 9
  frame 0 cache 0
  locals 1 ["x"]
  size 4 literals 2 children 0
  lit[0] = float(-0.0)
  lit[1] = array("0" => float(nan:0xfff8000000000042), -3 => true; next=0; cursor=1)
  code
    0: ASSIGN CV0, lit[0] | line=7
    1: JMPZ CV0, @3 | line=7
    2: RETURN lit[0] | line=8
    3: RETURN lit[1] | line=9
}
end
"#;

const PROPERTY_AND_ATTRIBUTE: &str = r#"zend-opcodes
file "d"
main
body - {
  lines 1 3
  frame 0 cache 0
  size 1 literals 1 children 0
  lit[0] = int(1)
  code
    0: RETURN lit[0] | line=3 ext=4294967295
}
class "a" "A" linked 512 2 2 - - {
attribute "A" 0 2 0 1 none
attrarg "k" int(-7)
property "p" 1 type(16) - = int(12)
}
end
"#;

#[test]
fn conversions_match_spec_terms() {
    let mut arena = Arena::new();
    let mut runner = test_spec::runner(&mut arena);
    for (name, listing) in [
        ("bytes_and_literal", BYTES_AND_LITERAL),
        ("branch_and_array", BRANCH_AND_ARRAY),
        ("property_and_attribute", PROPERTY_AND_ATTRIBUTE),
    ] {
        let expected = runner
            .eval_func(&mut arena, name, &[], &[])
            .result
            .unwrap_or_else(|error| panic!("{name}: {}", error.into_report()));
        let parsed = parse_zop(&mut arena, listing).unwrap();
        test_spec::check_unit(&mut runner, &mut arena, parsed).unwrap();
        assert!(
            arena
                .value
                .typ(parsed.note)
                .syntax_eq(arena.value.typ(expected.note)),
            "{name}"
        );
        assert!(
            parsed.view(&arena).syntax_eq(&expected.view(&arena)),
            "{name}\nparsed: {}\nexpected: {}",
            parsed.view(&arena).to_string(),
            expected.view(&arena).to_string()
        );
        let printed = print_zop(&arena, &expected).unwrap();
        let reparsed = parse_zop(&mut arena, &printed).unwrap();
        assert!(
            reparsed.view(&arena).syntax_eq(&expected.view(&arena)),
            "{name}\n{printed}"
        );
        assert_eq!(print_zop(&arena, &reparsed).unwrap(), printed, "{name}");
    }
}

#[test]
fn spec_accepts_arrays_at_zends_nesting_limit() {
    let literal = format!("{}int(9){}", "array(0 => ".repeat(256), ")".repeat(256));
    let listing = BYTES_AND_LITERAL.replace("int(-7)", &literal);
    let mut arena = Arena::new();
    let mut runner = test_spec::runner(&mut arena);
    let unit = parse_zop(&mut arena, &listing).unwrap();
    test_spec::check_unit(&mut runner, &mut arena, unit).unwrap();
}

#[test]
fn syntax_preserves_unresolved_references() {
    let mut arena = Arena::new();
    let mut runner = test_spec::runner(&mut arena);
    for (input, original, dangling) in [
        (BYTES_AND_LITERAL, "ECHO lit[0]", "ECHO lit[9]"),
        (BRANCH_AND_ARRAY, "JMPZ CV0, @3", "JMP @9"),
    ] {
        let program = parse_zop(&mut arena, &input.replace(original, dangling)).unwrap();
        test_spec::check_unit(&mut runner, &mut arena, program).unwrap();
        let printed = print_zop(&arena, &program).unwrap();
        assert!(printed.contains(dangling), "{printed}");
        let reparsed = parse_zop(&mut arena, &printed).unwrap();
        assert!(
            reparsed.view(&arena).syntax_eq(&program.view(&arena)),
            "{printed}"
        );
    }
}

#[test]
fn malformed_listings_return_errors() {
    let mut arena = Arena::new();
    for input in [
        BYTES_AND_LITERAL.replace("zend-opcodes\n", "zend-opcodes extra\n"),
        BYTES_AND_LITERAL.replace(r"\xff", r"\xz9"),
        BYTES_AND_LITERAL.replace("ECHO", "UNKNOWN_OPCODE"),
        BYTES_AND_LITERAL.replace("lit[0] |", "lit[-1] |"),
        BRANCH_AND_ARRAY.replace("nan:0xfff8000000000042", "nan:0x3ff0000000000000"),
        BRANCH_AND_ARRAY.replace("line=7", "line=4294967296"),
        format!("{PROPERTY_AND_ATTRIBUTE}garbage\n"),
    ] {
        assert!(parse_zop(&mut arena, &input).is_err(), "accepted:\n{input}");
    }
}

#[test]
fn values_outside_the_listing_format_are_rejected() {
    let mut arena = Arena::new();
    let mut runner = test_spec::runner(&mut arena);
    let boolean = make::bool(&mut arena, true, Default::default()).unwrap();
    assert!(test_spec::check_unit(&mut runner, &mut arena, boolean).is_err());
    assert!(print_zop(&arena, &boolean).is_err());
    let bad_byte = runner
        .eval_func(&mut arena, "nonbyte_filename", &[], &[])
        .result
        .unwrap();
    test_spec::check_unit(&mut runner, &mut arena, bad_byte).unwrap();
    assert!(print_zop(&arena, &bad_byte).is_err());
}
