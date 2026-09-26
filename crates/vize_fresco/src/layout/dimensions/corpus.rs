//! Deterministic syntax and f32 rounding boundaries, independent of the candidate.

use vize_l0::{String, cstr};

pub(super) fn values() -> Vec<String> {
    let mut values: Vec<String> = [
        "",
        "auto",
        "AUTO",
        "0",
        "-0",
        "+0",
        "-0.0",
        "0e-9999",
        "-0e9999",
        "+1",
        "01",
        "1.",
        ".5",
        "-.5",
        "+.5",
        "1e2",
        "1E+2",
        "1e-2",
        "NaN",
        "nan",
        "nAn",
        "NAN",
        "+NaN",
        "-NaN",
        "-nAn",
        "inf",
        "INF",
        "+Inf",
        "-iNF",
        "Infinity",
        "iNfInItY",
        "+INFINITY",
        "-Infinity",
        "0.1",
        "1.000000059604644775390625",
        "1.0000000596046447753906249",
        "1.0000000596046447753906251",
        "1.000000178813934326171875",
        "16777217",
        "16777219",
        "33554434",
        "33554438",
        "3.4028234663852886e38",
        "3.40282356779733661637539395458142568448e38",
        "3.4028236e38",
        "1e99999",
        "-1e99999",
        "1e-99999",
        "-1e-99999",
        "1.1754943508222875e-38",
        "1.1754942106924411e-38",
        "1.401298464324817e-45",
        "7.0064923216240853546186479164495806564013097093825788587853414194489554e-46",
        "7.0064923216240853546186479164495806564013097093825788587853414194489555e-46",
        " 1",
        "1 ",
        "\t1",
        "1\n",
        "\u{00a0}1",
        "1_000",
        "0x1",
        "0x1p0",
        "1,25",
        "１",
        "1px",
        "--1",
        "++1",
        "+-1",
        ".",
        "-.",
        "1e",
        "1e+",
        "1e-",
        "nan(payload)",
        "infinite",
        "inf0",
        "1\0",
        "\0",
        "1%%",
        "50%",
        "50 %",
        "%",
        "-NaN%",
        "Infinity%",
        "NaNfoo",
        "0.000000000000000001",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    // Exhaust short ASCII inputs, including control bytes and every grammar token.
    for first in 0..=127_u8 {
        values.push(cstr!("{}", char::from(first)));
        for second in 0..=127_u8 {
            values.push(cstr!("{}{}", char::from(first), char::from(second)));
        }
    }

    // Independent std formatting produces exact-roundtrip and long decimal forms.
    let mut state = 0x39a2_89bc_u32;
    for _ in 0..20_000 {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        let number = f32::from_bits(state);
        values.push(cstr!("{number}"));
        values.push(cstr!("{number:.48}"));
        values.push(cstr!("{number:.20e}"));
    }
    for exponent in [-9999, -500, -46, -45, -38, -1, 0, 1, 38, 39, 500, 9999] {
        for mantissa in ["0", "-0", "1", "-1", ".5", "1.9999999999999999999"] {
            values.push(cstr!("{mantissa}e{exponent}"));
        }
    }
    for length in [32, 64, 768, 8000] {
        values.push(cstr!("{}e-{}", String::from("9").repeat(length), length));
        values.push(cstr!("1e+{}", String::from("0").repeat(length)));
    }
    values
}
