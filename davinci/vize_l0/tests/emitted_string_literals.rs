use std::{
    fs::File,
    io::{BufWriter, Write},
};

use vize_l0::{String, cstr, js_string::push_js_string_literal};

#[test]
fn all_scalar_values_and_escape_boundaries() {
    let mut packet = std::env::var_os("VIZE_LITERAL_PROOF_ROWS")
        .map(|path| BufWriter::new(File::create(path).unwrap()));
    let mut written = 0;
    let mut check = |input: &str| {
        let mut output = String::from("existing:");
        push_js_string_literal(&mut output, input);
        let literal = output.strip_prefix("existing:").unwrap();
        assert_eq!(literal, serde_json::to_string(input).unwrap());
        assert_eq!(
            serde_json::from_str::<Box<str>>(literal).unwrap().as_ref(),
            input
        );
        if let Some(writer) = &mut packet {
            serde_json::to_writer(&mut *writer, &(input, literal)).unwrap();
            writer.write_all(b"\n").unwrap();
        }
        written += 1;
    };

    check("");
    let boundaries = (0..32).map(char::from).chain([
        '"', '\\', '/', '0', 'u', '\u{7f}', 'é', '\u{2028}', '\u{2029}', '🦀',
    ]);
    for first in boundaries.clone() {
        for second in boundaries.clone() {
            check(&cstr!("a{first}{second}雪"));
        }
    }

    let mut scalars = 0usize;
    let mut plain = String::default();
    let mut separated = String::default();
    for c in (0..=char::MAX as u32).filter_map(char::from_u32) {
        plain.push(c);
        separated.extend(['"', c, '\\']);
        scalars += 1;
        if scalars.is_multiple_of(512) {
            check(&plain);
            check(&separated);
            plain.clear();
            separated.clear();
        }
    }
    assert_eq!(scalars, 1_112_064);
    assert!(plain.is_empty() && separated.is_empty());
    assert_eq!(written, 1 + 42 * 42 + 2 * (scalars / 512));
    if let Some(mut writer) = packet {
        writer.flush().unwrap();
    }
}
