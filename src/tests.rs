use super::{Arena, parse_zop, print_zop, validate_unit};
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
        assert!(validate_unit(&arena, &program).is_err());
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
fn execution_checks_slots_and_smart_branch_pairs() {
    let listing = BRANCH_AND_ARRAY
        .replace("frame 0", "frame 2")
        .replace(
            "ASSIGN CV0, lit[0] | line=7",
            "T1 = IS_SMALLER CV0, lit[0] | smart=JMPZ",
        )
        .replace("JMPZ CV0", "JMPZ T1");
    let mut arena = Arena::new();
    let valid = parse_zop(&mut arena, &listing).unwrap();
    validate_unit(&arena, &valid).unwrap();
    for (original, invalid) in [
        ("frame 2", "frame 0"),
        ("T1 = IS_SMALLER", "T0 = IS_SMALLER"),
        ("IS_SMALLER CV0", "IS_SMALLER CV1"),
        ("JMPZ T1", "JMPZ V1"),
        ("JMPZ T1", "JMPZ T2"),
        ("smart=JMPZ", "smart=JMPNZ"),
        ("T1 = IS_SMALLER CV0", "T1 = ASSIGN CV0"),
        ("JMPZ T1, @3", "JMPZ T1, CV0"),
        ("RETURN lit[1]", "ECHO lit[1]"),
    ] {
        let program = parse_zop(&mut arena, &listing.replace(original, invalid)).unwrap();
        assert!(validate_unit(&arena, &program).is_err(), "{invalid}");
    }
}

#[test]
fn execution_checks_scalar_instruction_forms() {
    let template = BRANCH_AND_ARRAY
        .replace("frame 0", "frame 3")
        .replace("float(-0.0)", r#""s""#);
    let mut arena = Arena::new();
    let mut validate = |row: &str| {
        let listing = template.replace("ASSIGN CV0, lit[0] | line=7", row);
        let unit = parse_zop(&mut arena, &listing).unwrap();
        validate_unit(&arena, &unit)
    };
    for row in [
        "T1 = CHECK_VAR CV0",
        "T1 = BOOL V2",
        "T1 = BOOL_NOT CV0, CV0",
        "T1 = BOOL_XOR lit[0]",
        "TYPE_CHECK CV0 | ext=16",
        "T1 = GET_TYPE CV0 | cache=0",
        "T1 = STRLEN V2",
        "T1 = CAST CV0 | ext=3",
        "ASSIGN_OP CV0, lit[0] | ext=13",
        "T1 = QM_ASSIGN",
        "T1 = COPY_TMP CV0",
        "FREE CV0",
        "FREE T1 | ext=1",
        "FREE T1, 0",
        "T1 = CONCAT lit[0], lit[0]",
        "T1 = CONCAT CV0, lit[1]",
        "T1 = FAST_CONCAT lit[1], CV0",
        "T1 = ROPE_INIT _, lit[1] | ext=3",
        "T1 = ROPE_INIT _, CV0 | ext=0",
        "T2 = ROPE_INIT _, CV0 | ext=5",
        "T2 = ROPE_ADD T1, CV0 | ext=1",
        "T3 = ROPE_ADD T3, CV0 | ext=2",
        "T2 = ROPE_END T1, CV0 | ext=2",
        "T1 = ROPE_END T3, CV0 | ext=2",
    ] {
        assert!(validate(row).is_err(), "{row}");
    }
    for row in [
        "T1 = BOOL lit[1]",
        "CHECK_VAR CV0",
        "T1 = TYPE_CHECK CV0 | ext=4294967295",
        "T1 = CAST CV0 | ext=7",
        "ASSIGN_OP V1, lit[1] | ext=1",
        "T1 = QM_ASSIGN V2",
        "T1 = COPY_TMP V2",
        "FREE V2, 0 | ext=1",
        "T1 = CONCAT CV0, lit[0]",
        "T1 = FAST_CONCAT lit[0], lit[0]",
        "T2 = ROPE_INIT _, CV0 | ext=4",
        "T2 = ROPE_ADD T2, lit[0] | ext=3",
        "T1 = ROPE_END T2, CV0 | ext=3",
        "T3 = ROPE_END T1, CV0 | ext=3",
    ] {
        validate(row).unwrap_or_else(|error| panic!("{row}: {error}"));
    }
}

#[test]
fn execution_checks_numeric_instruction_forms() {
    let template = BRANCH_AND_ARRAY.replace("frame 0", "frame 3");
    let mut arena = Arena::new();
    let mut validate = |row: &str| {
        let listing = template.replace("ASSIGN CV0, lit[0] | line=7", row);
        let unit = parse_zop(&mut arena, &listing).unwrap();
        validate_unit(&arena, &unit)
    };
    for name in [
        "DIV",
        "POW",
        "IS_EQUAL",
        "IS_NOT_EQUAL",
        "IS_IDENTICAL",
        "IS_NOT_IDENTICAL",
        "SPACESHIP",
    ] {
        validate(&format!("T1 = {name} lit[0], lit[1]")).unwrap();
        for operands in ["V2, CV0", "CV0, V2"] {
            let row = format!("T1 = {name} {operands}");
            assert!(validate(&row).is_err(), "{row}");
        }
    }
    for name in [
        "ADD",
        "SUB",
        "MUL",
        "MOD",
        "SL",
        "SR",
        "BW_OR",
        "BW_AND",
        "BW_XOR",
        "IS_SMALLER",
        "IS_SMALLER_OR_EQUAL",
    ] {
        validate(&format!("T1 = {name} V2, V3")).unwrap();
        validate(&format!("T1 = {name} lit[0], lit[1]")).unwrap();
    }
    for row in [
        "ADD CV0, lit[0]",
        "T1 = MOD CV0",
        "T1 = MUL CV0, lit[1] | cache=0",
        "T1 = BW_NOT V2, CV0",
        "BW_NOT CV0",
        "T1 = POST_DEC T2",
        "POST_INC CV0",
        "T1 = PRE_DEC lit[0]",
        "T1 = POST_INC CV0, lit[0]",
    ] {
        assert!(validate(row).is_err(), "{row}");
    }
    for row in [
        "T1 = BW_NOT V2",
        "PRE_DEC V2",
        "T1 = POST_INC CV0",
        "CV0 = POST_DEC V2",
    ] {
        validate(row).unwrap();
    }
}

#[test]
fn execution_checks_logical_filename() {
    let mut arena = Arena::new();
    for (filename, valid) in [
        (r#""""#, false),
        (r#""ab\x00c""#, false),
        (r#""\xff\x0a""#, true),
    ] {
        let unit = parse_zop(
            &mut arena,
            &BRANCH_AND_ARRAY.replace(r#"file "c""#, &format!("file {filename}")),
        )
        .unwrap();
        assert_eq!(validate_unit(&arena, &unit).is_ok(), valid, "{filename}");
    }
}

#[test]
fn trailing_switch_cleanup_must_be_unreachable() {
    let listing = BRANCH_AND_ARRAY
        .replace("frame 0", "frame 1")
        .replace("JMPZ CV0, @3", "JMPZ CV0, @2")
        .replace("RETURN lit[1] | line=9", "FREE T1 | ext=2");
    let mut arena = Arena::new();
    let mut validate = |source: &str| {
        let unit = parse_zop(&mut arena, source).unwrap();
        validate_unit(&arena, &unit)
    };
    validate(&listing).unwrap();
    let paired = listing
        .replace("ASSIGN CV0, lit[0]", "ASSIGN_DIM CV0, lit[0]")
        .replace("JMPZ CV0, @2", "OP_DATA lit[0]");
    validate(&paired).unwrap();

    let table = listing
        .replace(
            "array(\"0\" => float(nan:0xfff8000000000042), -3 => true; next=0; cursor=1)",
            "array(0 => int(2))",
        )
        .replace("JMPZ CV0, @2 | line=7", "MATCH CV0, lit[1] | ext=@2");
    validate(&table).unwrap();
    for invalid in [
        listing.replace("JMPZ CV0, @2", "ASSIGN_DIM CV0, lit[0]"),
        listing.replace("ASSIGN CV0, lit[0]", "OP_DATA lit[0]"),
        listing.replace("FREE T1 | ext=2", "FREE T1"),
        listing.replace("RETURN lit[0]", "ECHO lit[0]"),
        listing.replace("RETURN lit[0]", "DO_UCALL"),
        listing.replace("JMPZ CV0, @2", "JMPZ CV0, @3"),
        listing.replace(
            "  code",
            "  tries 1\n  try @0 catch @3 finally @0 end @0\n  code",
        ),
        listing.replace(
            "  code",
            "  tries 1\n  try @0 catch @0 finally @3 end @3\n  code",
        ),
        table.replace("ext=@2", "ext=@3"),
        table.replace("0 => int(2)", "0 => int(3)"),
    ] {
        assert!(validate(&invalid).is_err(), "{invalid}");
    }
}

#[test]
fn execution_validates_bodies_inside_deferred_attribute_arguments() {
    let closure = "ast(closure(0 2\nbody \"f\" {\nlines 2 2\nframe 0 cache 0\n\
        size 1 literals 1 children 0\nlit[0] = null\ncode\n0: RETURN lit[0]\n}\n))";
    let listing = PROPERTY_AND_ATTRIBUTE.replace("int(-7)", closure);
    let mut arena = Arena::new();
    let valid = parse_zop(&mut arena, &listing).unwrap();
    validate_unit(&arena, &valid).unwrap();
    let broken = listing.replace("0: RETURN lit[0]\n", "0: RETURN lit[1]\n");
    let program = parse_zop(&mut arena, &broken).unwrap();
    assert!(validate_unit(&arena, &program).is_err());
}

#[test]
fn execution_rejects_attribute_tables_as_value_operands() {
    let listing = include_str!("../tests/project/handwritten.zop").replace(
        "int(41)",
        "attributes {\nattribute \"Example\" 0 1 0 0 none\n}",
    );
    let mut arena = Arena::new();
    let parsed = parse_zop(&mut arena, &listing).unwrap();
    assert!(validate_unit(&arena, &parsed).is_err());
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
