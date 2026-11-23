use num_bigint::{BigInt, ToBigInt};
use p4spectec::{
    lang::{
        common::prim::num,
        data::{
            arena::Arena,
            typ,
            value::flat::{Value, get, make},
        },
        il::ast::Typ,
    },
    runner::{Eval, Extern, ExternError, Interface, Interpreter, RunnerContext, SideEffect},
};

pub(super) struct Binary64;

impl Extern for Binary64 {
    fn eval_func<I, F>(
        ctx: &mut RunnerContext<'_, I, F, Self>,
        name: &str,
        _types: &[Typ],
        values: &[Value],
    ) -> Eval<Value, ExternError>
    where
        F: Interface,
        I: Interpreter<F, Self>,
    {
        Eval::new(call(ctx.arena_mut(), name, values), SideEffect::Pure)
    }

    fn eval_rel<I, F>(
        _ctx: &mut RunnerContext<'_, I, F, Self>,
        _name: &str,
        _values: &[Value],
    ) -> Eval<Vec<Value>, ExternError>
    where
        F: Interface,
        I: Interpreter<F, Self>,
    {
        Eval::new(
            Err(ExternError::diagnostic_unconfigured()),
            SideEffect::Pure,
        )
    }

    fn clear(&mut self) {}
}

fn integer<'a>(arena: &'a Arena, value: &Value) -> Result<&'a BigInt, ExternError> {
    Ok(num::to_int(get::num(arena, value)?))
}

fn float(arena: &Arena, value: &Value) -> Result<f64, ExternError> {
    Ok(f64::from_bits(u64::try_from(integer(arena, value)?)?))
}

fn call(arena: &mut Arena, name: &str, args: &[Value]) -> Result<Value, ExternError> {
    let result = match (name, args) {
        ("f64_from_int", [value]) => i64::try_from(integer(arena, value)?)? as f64,
        ("f64_binary", [op, left, right]) => {
            let a = float(arena, left)?;
            let b = float(arena, right)?;
            get::matches_string! { arena, op,
                "ADD" => |_args| a + b,
                "SUB" => |_args| a - b,
                "MUL" => |_args| a * b,
                "DIV" => |_args| a / b,
                "POW" => |_args| a.powf(b),
                _ => return Err(ExternError::diagnostic_message("unknown binary64 operation")),
            }
        }
        ("f64_trunc", [value]) => {
            let n = float(arena, value)?.to_bigint().ok_or_else(|| {
                ExternError::diagnostic_message("cannot truncate a nonfinite binary64 value")
            })?;
            return Ok(make::int(arena, n, Default::default())?);
        }
        ("f64_parse", [value]) => {
            let bytes = super::bytes(arena, value)
                .map_err(|e| ExternError::diagnostic_message(e.to_string()))?;
            let token = std::str::from_utf8(&bytes)
                .map_err(|e| ExternError::diagnostic_message(e.to_string()))?;
            parse_decimal(token).map_err(|e| ExternError::diagnostic_message(e.to_string()))?
        }
        ("f64_format", [value, precision]) => {
            let bytes = format_float(
                float(arena, value)?,
                i32::try_from(integer(arena, precision)?)?,
            )?;
            let values = bytes
                .bytes()
                .map(|byte| make::nat(arena, u64::from(byte).into(), Default::default()))
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(make::list(
                arena,
                typ::make::list(typ::make::nat()).node.into(),
                values,
                Default::default(),
            )?);
        }
        _ => {
            return Err(ExternError::diagnostic_message(format!(
                "unknown binary64 call {name}"
            )));
        }
    };
    Ok(make::nat(
        arena,
        result.to_bits().into(),
        Default::default(),
    )?)
}

fn parse_decimal(token: &str) -> Result<f64, std::num::ParseFloatError> {
    // Zend bounds the explicit exponent before combining it with the position of the point.
    if let Some((mantissa, exponent)) = token.split_once(['e', 'E']) {
        let digits = exponent
            .trim_start_matches(['-', '+'])
            .trim_start_matches('0');
        if digits.len() > 5 || digits.len() == 5 && digits > "19999" {
            let bound = if exponent.starts_with('-') {
                "-19999"
            } else {
                "19999"
            };
            return format!("{mantissa}e{bound}").parse();
        }
    }
    token.parse()
}

fn format_float(value: f64, precision: i32) -> Result<String, ExternError> {
    if value.is_nan() {
        return Ok("NAN".into());
    }
    let sign = if value.is_sign_negative() { "-" } else { "" };
    if value.is_infinite() {
        return Ok(format!("{sign}INF"));
    }
    if value == 0.0 {
        return Ok(format!("{sign}0"));
    }

    let magnitude = value.abs();
    let mut shortest = zmij::Buffer::new();
    let rounded;
    let (raw, limit) = match precision {
        -1 => (shortest.format_finite(magnitude), 17),
        14 => {
            rounded = format!("{magnitude:.13e}");
            (rounded.as_str(), 14)
        }
        _ => {
            return Err(ExternError::diagnostic_message(
                "unknown binary64 formatting precision",
            ));
        }
    };
    let (mantissa, exponent) = match raw.split_once('e') {
        Some((digits, exponent)) => (digits, exponent.parse::<i32>().expect("decimal exponent")),
        None => (raw, 0),
    };
    let mantissa_point = mantissa.find('.').unwrap_or(mantissa.len());
    let digits = mantissa.replace('.', "");
    let trimmed = digits.trim_start_matches('0');
    let point = mantissa_point as i32 + exponent - (digits.len() - trimmed.len()) as i32;
    // Zend retains all 14 digits for small integer midpoints rounded down to a trailing zero.
    let padded_midpoint =
        precision == 14 && (1e14..1e15).contains(&magnitude) && magnitude % 100.0 == 5.0;
    let digits = if padded_midpoint {
        trimmed
    } else {
        trimmed.trim_end_matches('0')
    };
    let output = if point > limit || point < -3 {
        let (first, rest) = digits.split_at(1);
        let fraction = if rest.is_empty() { "0" } else { rest };
        format!("{sign}{first}.{fraction}E{:+}", point - 1)
    } else if point <= 0 {
        format!("{sign}0.{}{digits}", "0".repeat((-point) as usize))
    } else if point as usize >= digits.len() {
        format!(
            "{sign}{digits}{}",
            "0".repeat(point as usize - digits.len())
        )
    } else {
        let (whole, fraction) = digits.split_at(point as usize);
        format!("{sign}{whole}.{fraction}")
    };
    Ok(output)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use p4spectec::runner::{Config, build_sl};

    use super::*;

    #[test]
    fn spec_primitives_preserve_machine_rounding_and_truncate_exactly() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("spec");
        let paths = [
            dir.join("00-syntax.watsup"),
            dir.join("01-values/00-binary64.watsup"),
        ];
        let spec = p4spectec::structure(&paths, true).unwrap();
        let mut arena = Arena::new();
        let mut runner = build_sl(
            &mut arena,
            spec,
            Config::new(false, true, true, true),
            Binary64,
        )
        .unwrap();

        let max = make::int(&mut arena, i64::MAX.into(), Default::default()).unwrap();
        let as_float = runner
            .eval_func(&mut arena, "f64_from_int", &[], &[max])
            .result
            .unwrap();
        assert_eq!(
            float(&arena, &as_float).unwrap().to_bits(),
            0x43e0_0000_0000_0000
        );
        let as_int = runner
            .eval_func(&mut arena, "f64_trunc", &[], &[as_float])
            .result
            .unwrap();
        assert_eq!(integer(&arena, &as_int).unwrap(), &(BigInt::from(1) << 63));

        for (op, left, right, expected) in [
            (
                "ADD",
                9_007_199_254_740_992.0_f64,
                1.0_f64,
                9_007_199_254_740_992.0_f64,
            ),
            ("SUB", -0.0, 0.0, -0.0),
            ("MUL", f64::MAX, 2.0, f64::INFINITY),
            ("DIV", f64::from_bits(3), 2.0, f64::from_bits(2)),
            ("POW", -0.0, -3.0, f64::NEG_INFINITY),
        ] {
            let opcode = make::case_string! {
                arena: &mut arena, shape: op, args: [], typ: "opcode", span: Default::default()
            }
            .unwrap();
            let a = make::nat(&mut arena, left.to_bits().into(), Default::default()).unwrap();
            let b = make::nat(&mut arena, right.to_bits().into(), Default::default()).unwrap();
            let result = runner
                .eval_func(&mut arena, "f64_binary", &[], &[opcode, a, b])
                .result
                .unwrap();
            assert_eq!(
                float(&arena, &result).unwrap().to_bits(),
                expected.to_bits()
            );
        }
    }

    #[test]
    fn decimal_input_rounds_once_and_bounds_the_explicit_exponent() {
        let positive = format!("0.{}1E+00020000", "0".repeat(19_999));
        let negative = format!("1{}e-20000", "0".repeat(20_000));
        for (token, expected) in [
            (positive.as_str(), 0.1_f64),
            (negative.as_str(), 10.0),
            ("9007199254740993", 9_007_199_254_740_992.0),
            ("9007199254740995", 9_007_199_254_740_996.0),
            (
                "1.00000000000000011102230246251565404236316680908203125",
                1.0,
            ),
            (
                "1.00000000000000011102230246251565404236316680908203126",
                f64::from_bits(1.0_f64.to_bits() + 1),
            ),
            ("-0", -0.0),
            ("-1.0e-4000", -0.0),
            ("1e400", f64::INFINITY),
        ] {
            assert_eq!(parse_decimal(token).unwrap().to_bits(), expected.to_bits());
        }
    }

    #[test]
    fn decimal_rounding_and_notation() {
        for (value, precision, expected) in [
            (
                f64::from_bits(0x4197_0d97_2932_0000),
                -1,
                "96691658.29882812",
            ),
            (i64::MAX as f64, 14, "9.2233720368548E+18"),
            (i64::MAX as f64, -1, "9.223372036854776E+18"),
            (1e14, 14, "1.0E+14"),
            (1e14, -1, "100000000000000"),
            (100_000_000_000_005.0, 14, "1.0000000000000E+14"),
            (-123_456_789_000_005.0, 14, "-1.2345678900000E+14"),
            (1_000_000_000_000_050.0, 14, "1.0E+15"),
            (1e-4, 14, "0.0001"),
            (1e-5, 14, "1.0E-5"),
            (f64::from_bits(1), 14, "4.9406564584125E-324"),
            (f64::from_bits(1), -1, "5.0E-324"),
            (-0.0, 14, "-0"),
            (f64::NEG_INFINITY, 14, "-INF"),
            (f64::NAN, 14, "NAN"),
        ] {
            assert_eq!(format_float(value, precision).unwrap(), expected);
        }
    }
}
