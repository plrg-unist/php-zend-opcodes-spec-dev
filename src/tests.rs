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

const SHARED_ARRAYS: &str = r#"zend-opcodes
file "s"
main
body - {
  lines 1 1
  frame 0 cache 0
  size 1 literals 2 children 0
  lit[0] = array#42(0 => array#7(0 => int(9)), 1 => array-ref(7))
  lit[1] = array-ref(42)
  code
    0: RETURN lit[1] | line=1
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
        ("shared_arrays", SHARED_ARRAYS),
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
fn shared_arrays_require_a_completed_unique_definition() {
    let mut arena = Arena::new();
    for listing in [
        SHARED_ARRAYS.replace("array-ref(7)", "array-ref(42)"),
        SHARED_ARRAYS.replace("array-ref(7)", "array-ref(0)"),
        SHARED_ARRAYS.replace("array#7", "array#42"),
        SHARED_ARRAYS.replace("array-ref(42)", "array#7()"),
    ] {
        assert!(parse_zop(&mut arena, &listing).is_err(), "{listing}");
    }
    let mut runner = test_spec::runner(&mut arena);
    for name in ["conflicting_arrays", "bad_array_payload"] {
        let unit = runner.eval_func(&mut arena, name, &[], &[]).result.unwrap();
        assert!(validate_unit(&arena, &unit).is_err());
        assert!(print_zop(&arena, &unit).is_err());
    }
}

#[test]
fn shared_arrays_cross_body_and_class_metadata() {
    let listing = PROPERTY_AND_ATTRIBUTE
        .replace("lit[0] = int(1)", "lit[0] = array#17(0 => int(1))")
        .replace("int(-7)", "array-ref(17)")
        .replace("int(12)", "array-ref(17)");
    let mut arena = Arena::new();
    let unit = parse_zop(&mut arena, &listing).unwrap();
    validate_unit(&arena, &unit).unwrap();
    assert_eq!(print_zop(&arena, &unit).unwrap(), listing);

    let statics = SHARED_ARRAYS.replace("  size", "  statics array(0 => array#8())\n  size");
    let unit = parse_zop(&mut arena, &statics).unwrap();
    validate_unit(&arena, &unit).unwrap();
    let invalid = statics.replace("statics array(", "statics array#9(");
    let unit = parse_zop(&mut arena, &invalid).unwrap();
    assert!(validate_unit(&arena, &unit).is_err());
}

#[test]
fn in_array_checks_the_keys_of_a_shared_literal() {
    let listing = SHARED_ARRAYS
        .replace("frame 0", "frame 1")
        .replace("size 1", "size 2")
        .replace(
            "0: RETURN lit[1]",
            "0: T0 = IN_ARRAY lit[0], lit[1] | ext=1\n    1: RETURN T0",
        );
    let mut arena = Arena::new();
    let unit = parse_zop(&mut arena, &listing).unwrap();
    validate_unit(&arena, &unit).unwrap();
    let loose = listing.replace("ext=1", "ext=0");
    let unit = parse_zop(&mut arena, &loose).unwrap();
    assert!(validate_unit(&arena, &unit).is_err());
}

#[test]
fn constant_expression_literals_check_shared_array_contents() {
    let listing = SHARED_ARRAYS.replace(
        "lit[1] = array-ref(42)",
        "lit[1] = ast(value(0 1 array-ref(42)))",
    );
    let mut arena = Arena::new();
    let valid = parse_zop(&mut arena, &listing).unwrap();
    validate_unit(&arena, &valid).unwrap();
    let deferred = listing.replace("int(9)", r#"ast(constant(0 1 "NAME"))"#);
    let invalid = parse_zop(&mut arena, &deferred).unwrap();
    assert!(validate_unit(&arena, &invalid).is_err());
}

#[test]
fn shared_array_graphs_roundtrip_without_expanding_references() {
    let mut literals = vec!["  lit[0] = array#0()".to_owned()];
    for id in 1..64 {
        let prev = id - 1;
        literals.push(format!(
            "  lit[{id}] = array#{id}(0 => array-ref({prev}), 1 => array-ref({prev}))"
        ));
    }
    let listing = format!(
        "zend-opcodes\nfile \"g\"\nmain\nbody - {{\n  lines 1 1\n  frame 0 cache 0\n  size 1 literals 64 children 0\n{}\n  code\n    0: RETURN lit[63] | line=1\n}}\nend\n",
        literals.join("\n")
    );
    let mut arena = Arena::new();
    let unit = parse_zop(&mut arena, &listing).unwrap();
    validate_unit(&arena, &unit).unwrap();
    assert_eq!(print_zop(&arena, &unit).unwrap(), listing);
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
fn execution_checks_value_branches_and_case_comparisons() {
    let template = BRANCH_AND_ARRAY.replace("frame 0", "frame 3");
    let mut arena = Arena::new();
    let mut validate = |row: &str| {
        let listing = template.replace("ASSIGN CV0, lit[0] | line=7", row);
        let unit = parse_zop(&mut arena, &listing).unwrap();
        validate_unit(&arena, &unit)
    };
    for name in ["JMPZ_EX", "JMPNZ_EX", "JMP_SET", "COALESCE", "JMP_NULL"] {
        for (dst, input) in [("T1", "lit[0]"), ("V3", "T1"), ("CV0", "CV0")] {
            validate(&format!("{dst} = {name} {input}, @2")).unwrap();
        }
        for row in [
            format!("{name} CV0, @2"),
            format!("T1 = {name} V3, @2"),
            format!("T1 = {name} CV0, 2"),
            format!("T1 = {name} CV0, T2"),
            format!("T1 = {name} CV0, @4"),
            format!("T1 = {name} CV0, @2 | ext=@2"),
            format!("T1 = {name} CV0, @2 | cache=0"),
            format!("T1 = {name} CV0, @2 | smart=JMPZ"),
        ] {
            assert!(validate(&row).is_err(), "{row}");
        }
    }
    for flags in [0, 1, 2, 4, 5, 6, 8, u32::MAX - 1] {
        validate(&format!("T1 = JMP_NULL CV0, @2 | ext={flags}")).unwrap();
    }
    for flags in [3, 7, u32::MAX] {
        assert!(validate(&format!("T1 = JMP_NULL CV0, @2 | ext={flags}")).is_err());
    }
    for name in ["CASE", "CASE_STRICT"] {
        for (dst, rhs) in [("T2", "lit[0]"), ("CV0", "T2"), ("V3", "CV0")] {
            validate(&format!("{dst} = {name} T1, {rhs}")).unwrap();
        }
        for row in [
            format!("{name} T1, T2"),
            format!("T1 = {name} V2, CV0"),
            format!("T1 = {name} CV0, CV0"),
            format!("T1 = {name} T2, V3"),
            format!("T1 = {name} T2, CV0 | cache=0"),
        ] {
            assert!(validate(&row).is_err(), "{row}");
        }
    }
    for name in ["CASE", "CASE_STRICT"] {
        for jump in ["JMPZ", "JMPNZ"] {
            let paired = template
                .replace(
                    "ASSIGN CV0, lit[0] | line=7",
                    &format!("T1 = {name} T2, CV0 | smart={jump}"),
                )
                .replace("JMPZ CV0", &format!("{jump} T1"));
            let unit = parse_zop(&mut arena, &paired).unwrap();
            validate_unit(&arena, &unit).unwrap();
            let wrong_slot = parse_zop(
                &mut arena,
                &paired.replace(&format!("{jump} T1"), &format!("{jump} T2")),
            )
            .unwrap();
            assert!(validate_unit(&arena, &wrong_slot).is_err());
        }
    }
}

#[test]
fn execution_checks_jump_tables() {
    let template = BRANCH_AND_ARRAY.replace("frame 0", "frame 3");
    let mut arena = Arena::new();
    let mut validate = |code: &str, array: &str| {
        let listing = template
            .replace("ASSIGN CV0, lit[0] | line=7", code)
            .replace(
                r#"array("0" => float(nan:0xfff8000000000042), -3 => true; next=0; cursor=1)"#,
                array,
            );
        let unit = parse_zop(&mut arena, &listing).unwrap();
        validate_unit(&arena, &unit)
    };
    for name in ["SWITCH_LONG", "SWITCH_STRING", "MATCH"] {
        for subject in ["CV0", "T1", "V1", "lit[0]"] {
            validate(&format!("{name} {subject}, lit[1] | ext=@2"), "array()").unwrap();
        }
        for row in [
            format!("T1 = {name} CV0, lit[1] | ext=@2"),
            format!("{name} _, lit[1] | ext=@2"),
            format!("{name} CV0, T1 | ext=@2"),
            format!("{name} CV0, lit[0] | ext=@2"),
            format!("{name} CV0, lit[1]"),
            format!("{name} CV0, lit[1] | ext=2"),
            format!("{name} CV0, lit[1] | ext=@4"),
            format!("{name} CV0, lit[1] | ext=@2 cache=0"),
            format!("{name} CV0, lit[1] | ext=@2 numeric-string-key"),
        ] {
            assert!(validate(&row, "array()").is_err(), "{row}");
        }
    }
    let dispatch = "MATCH CV0, lit[1] | ext=@2";
    for valid in [
        r#"array(-1 => int(0), "-1" => int(2), "\x00\xff" => int(3); next=0; cursor=3)"#,
        "array(-4 => int(1); next=-3)",
        "array(9223372036854775807 => int(2); next=9223372036854775807)",
    ] {
        validate(dispatch, valid).unwrap();
    }
    for (name, table) in [
        ("SWITCH_LONG", r#"array("0" => int(0))"#),
        ("SWITCH_STRING", "array(0 => int(0))"),
        ("MATCH", "array#0(0 => int(2))"),
        ("MATCH", "array(1 => int(2), 1 => int(3))"),
        ("MATCH", r#"array("A" => int(2), "\x41" => int(3))"#),
        ("MATCH", "array(1 => true)"),
        ("MATCH", "array(1 => int(-1))"),
        ("MATCH", "array(1 => int(4))"),
        ("MATCH", "array(1 => int(2); next=1)"),
        ("MATCH", "array(1 => int(2); cursor=2)"),
    ] {
        assert!(
            validate(&format!("{name} CV0, lit[1] | ext=@2"), table).is_err(),
            "{table}"
        );
    }
    let shared = template
        .replace("ASSIGN CV0, lit[0] | line=7", dispatch)
        .replace("JMPZ CV0, @3 | line=7", dispatch)
        .replace(
            r#"array("0" => float(nan:0xfff8000000000042), -3 => true; next=0; cursor=1)"#,
            "array(0 => int(2))",
        );
    let unit = parse_zop(&mut arena, &shared).unwrap();
    assert!(validate_unit(&arena, &unit).is_err());
}

#[test]
fn execution_checks_array_operands_and_dimension_keys() {
    let template = BRANCH_AND_ARRAY
        .replace("frame 0", "frame 3")
        .replace("literals 2", "literals 4")
        .replace("  code", "  lit[2] = int(0)\n  lit[3] = \"0\"\n  code");
    let mut arena = Arena::new();
    let mut validate = |row: &str| {
        let text = template.replace("ASSIGN CV0, lit[0] | line=7", row);
        let unit = parse_zop(&mut arena, &text).unwrap();
        validate_unit(&arena, &unit)
    };

    for row in [
        "T1 = INIT_ARRAY | ext=2",
        "T1 = INIT_ARRAY V2, CV0 | ext=9",
        "T1 = ADD_ARRAY_ELEMENT CV0 | ext=1",
        "T1 = ADD_ARRAY_UNPACK V2",
        "V1 = FETCH_DIM_R V2, T3",
        "T1 = FETCH_DIM_IS V2, lit[2] | numeric-string-key",
        "T1 = ISSET_ISEMPTY_DIM_OBJ CV0, lit[2] | numeric-string-key ext=1",
        "T1 = ISSET_ISEMPTY_CV CV0 | ext=4294967295",
        "T1 = ARRAY_KEY_EXISTS T2, CV0",
        "T1 = COUNT lit[1] | ext=1",
        "T1 = IN_ARRAY CV0, lit[1] | ext=2",
    ] {
        validate(row).unwrap_or_else(|err| panic!("{row}: {err}"));
    }
    for row in [
        "T1 = INIT_ARRAY CV0, V2",
        "T1 = ADD_ARRAY_ELEMENT",
        "T1 = ADD_ARRAY_ELEMENT CV0, V2",
        "T1 = ADD_ARRAY_UNPACK",
        "T1 = ADD_ARRAY_UNPACK CV0, CV0",
        "T1 = FETCH_DIM_R CV0, V2",
        "T1 = FETCH_DIM_IS CV0",
        "T1 = ISSET_ISEMPTY_DIM_OBJ V2, CV0",
        "T1 = ISSET_ISEMPTY_CV T2",
        "ISSET_ISEMPTY_CV CV0",
        "T1 = ARRAY_KEY_EXISTS CV0, V2",
        "T1 = COUNT V2",
        "T1 = IN_ARRAY V2, lit[1]",
        "T1 = IN_ARRAY CV0, lit[2]",
        "T1 = IN_ARRAY CV0, lit[1]",
        "COUNT CV0",
        "T1 = FETCH_DIM_R CV0, CV0 | numeric-string-key",
        "T1 = FETCH_DIM_R CV0, lit[0] | numeric-string-key",
        "T1 = FETCH_DIM_IS CV0, lit[3] | numeric-string-key",
        "T1 = ARRAY_KEY_EXISTS CV0, lit[2] | numeric-string-key",
    ] {
        assert!(validate(row).is_err(), "{row}");
    }
}

#[test]
fn execution_checks_nested_array_literals() {
    let mut arena = Arena::new();
    for (nested, valid) in [
        (r#"array(0 => null, "0" => true; next=3; cursor=2)"#, true),
        ("array(-4 => false; next=-3)", true),
        (
            "array(9223372036854775807 => true; next=9223372036854775807)",
            true,
        ),
        ("array(1 => null, 1 => false)", false),
        (r#"array("A" => true, "\x41" => false)"#, false),
        ("array(0 => undef)", false),
        ("array(1 => true; next=1)", false),
        ("array(5 => true; cursor=2)", false),
    ] {
        let text = BRANCH_AND_ARRAY.replace("float(-0.0)", &format!("array(0 => {nested})"));
        let unit = parse_zop(&mut arena, &text).unwrap();
        assert_eq!(validate_unit(&arena, &unit).is_ok(), valid, "{nested}");
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

    let nested = table.replace("size 4", "size 6").replace(
        "3: FREE T1 | ext=2",
        "3: FREE T1 | ext=2\n    4: FREE T1 | ext=2\n    5: FREE T1 | ext=2",
    );
    validate(&nested).unwrap();
    validate(
        &nested
            .replace("ASSIGN CV0, lit[0] | line=7", "FREE T1 | ext=2")
            .replace("ext=@2", "ext=@0"),
    )
    .unwrap();
    for target in 3..=5 {
        for invalid in [
            nested.replace("ASSIGN CV0, lit[0] | line=7", &format!("JMP @{target}")),
            nested.replace(
                "ASSIGN CV0, lit[0] | line=7",
                &format!("JMPZ CV0, @{target}"),
            ),
            nested.replace("ext=@2", &format!("ext=@{target}")),
            nested.replace("0 => int(2)", &format!("0 => int({target})")),
            nested.replace(
                "  code",
                &format!("  tries 1\n  try @0 catch @{target} finally @0 end @0\n  code"),
            ),
            nested.replace(
                "  code",
                &format!("  tries 1\n  try @0 catch @0 finally @{target} end @{target}\n  code"),
            ),
        ] {
            assert!(validate(&invalid).is_err(), "{invalid}");
        }
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
