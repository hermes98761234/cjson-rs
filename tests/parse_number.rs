use cjson::parse;

fn assert_number(input: &str, expected_int: i64, expected_double: f64) {
    let v = parse(input).unwrap();
    assert!(v.is_number());
    if expected_double.fract() == 0.0
        && expected_double >= i64::MIN as f64
        && expected_double <= i64::MAX as f64
    {
        assert_eq!(v.as_i64(), Some(expected_int));
    }
    assert!((v.as_f64().unwrap() - expected_double).abs() < 1e-10);
}

#[test]
fn parse_zero() {
    assert_number("0", 0, 0.0);
    assert_number("0.0", 0, 0.0);
    assert_number("-0", 0, 0.0); // -0 == 0 for our purposes
}

#[test]
fn parse_negative_integers() {
    assert_number("-1", -1, -1.0);
    assert_number("-32768", -32768, -32768.0);
    assert_number("-2147483648", -2147483648, -2147483648.0);
}

#[test]
fn parse_positive_integers() {
    assert_number("1", 1, 1.0);
    assert_number("32767", 32767, 32767.0);
    assert_number("2147483647", 2147483647, 2147483647.0);
}

#[test]
fn parse_positive_reals() {
    assert_number("0.001", 0, 0.001);
    assert_number("10e-10", 0, 10e-10);
    assert_number("10E-10", 0, 10e-10);
    assert_number("10e10", 100000000000, 10e10);
    assert_number("123e-128", 0, 123e-128);
}

#[test]
fn parse_negative_reals() {
    assert_number("-0.001", 0, -0.001);
    assert_number("-10e-10", 0, -10e-10);
    assert_number("-10E-10", 0, -10e-10);
    // -10e20 = -1e21, which overflows i64 — just check the double
    let v = parse("-10e20").unwrap();
    assert!((v.as_f64().unwrap() - (-10e20)).abs() < 1e10);
    assert_number("-123e-128", 0, -123e-128);
}

#[test]
fn parse_big_numbers() {
    // Very large numbers that overflow i64 but parse as f64
    let v = parse("9999999999999999999999999999999999999999999999912345678901234567").unwrap();
    assert!(v.is_number());
    assert!(v.as_f64().unwrap().is_finite());

    let v = parse("9999999999999999999999999999999999999999999999912345678901234567E10").unwrap();
    assert!(v.is_number());
    assert!(v.as_f64().unwrap().is_finite());

    let v = parse("999999999999999999999999999999999999999999999991234567890.1234567").unwrap();
    assert!(v.is_number());
    assert!(v.as_f64().unwrap().is_finite());
}

#[test]
fn parse_number_pi() {
    let v = parse("3.141592653589793").unwrap();
    assert!((v.as_f64().unwrap() - std::f64::consts::PI).abs() < 1e-15);
}

#[test]
fn parse_number_scientific_notation() {
    let v = parse("123e+127").unwrap();
    assert!(v.is_number());
    assert!(v.as_f64().unwrap().is_finite());
}
