use std::path::Path;

use p4spectec::{
    SpecLang,
    lang::data::{arena::Arena, value::flat::Value},
    runner::{Config, NullExtern, NullInterface, Runner},
    runner_spec_with_warnings,
};

pub(super) fn runner(arena: &mut Arena) -> Runner<NullInterface, NullExtern> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let paths = [
        root.join("spec/syntax.watsup"),
        root.join("src/tests.watsup"),
    ];
    let (spec, _) = runner_spec_with_warnings(SpecLang::Sl, &paths);
    let spec = spec.unwrap_or_else(|error| panic!("{error}"));
    let config = Config::new(false, true, true, true);
    Runner::new(arena, spec, config, NullInterface, NullExtern)
        .unwrap_or_else(|error| panic!("{error}"))
}

pub(super) fn check_unit(
    runner: &mut Runner<NullInterface, NullExtern>,
    arena: &mut Arena,
    value: Value,
) -> Result<(), String> {
    // SpecTec's recursive type guard can visit nested bodies, expressions and arrays on one stack.
    const GUARD_STACK: usize = 32 * 1024 * 1024;
    stacker::maybe_grow(GUARD_STACK, GUARD_STACK, || {
        runner
            .eval_func(arena, "accept_unit", &[], &[value])
            .result
            .map(|_| ())
            .map_err(|error| error.into_report().to_string())
    })
}
