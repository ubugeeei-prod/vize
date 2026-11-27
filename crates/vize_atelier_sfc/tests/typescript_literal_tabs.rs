//! The pinned `compiler/sfc/typescript-literal-tabs` differential case: erasing
//! TypeScript keeps every authored tab. The erasure re-prints the script with
//! oxc, which indents by two spaces itself, so a tab inside a string, template
//! or regular expression literal, or a comment, survives byte for byte instead
//! of being rewritten as two spaces with the indentation.
//! `VIZE_BLESS_TYPESCRIPT_LITERAL_TABS=1` rewrites the pinned SSR module.

use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode};
use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, TemplateCompileOptions, compile_sfc,
    compile_sfc_with_template_syntax_and_codegen_options, parse_sfc,
};

fn fixture(file: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/compiler/typescript-literal-tabs")
        .join(file)
}

fn source() -> String {
    std::fs::read_to_string(fixture("Layout.vue.txt")).expect("input")
}

fn options(ssr: bool) -> SfcCompileOptions {
    SfcCompileOptions {
        parse: SfcParseOptions {
            filename: "Layout.vue".into(),
            ..Default::default()
        },
        template: TemplateCompileOptions {
            ssr,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn compile(ssr: bool) -> String {
    let source = source();
    let descriptor = parse_sfc(&source, SfcParseOptions::default()).expect("SFC");
    let result = compile_sfc(&descriptor, options(ssr)).expect("compile");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    result.code.as_str().to_owned()
}

/// The authored literals and comment, each holding a raw tab.
const LITERALS: &[&str] = &[
    "const tab = \"\t\";",
    "const raw = \"a\tb\";",
    "const tpl = `x\ty\n\tz`;",
    "const re = /\t/;",
    "/* keep\ta comment tab */",
    "  return `\n\t${tab}`;",
];

#[test]
fn the_ssr_module_matches_its_pinned_reference() {
    let code = compile(true);
    let path = fixture("module.expected.txt");
    if std::env::var_os("VIZE_BLESS_TYPESCRIPT_LITERAL_TABS").is_some() {
        std::fs::write(&path, &code).expect("bless module");
        return;
    }
    let expected = std::fs::read_to_string(&path).expect("pinned module");
    assert_eq!(code, expected);
}

#[test]
fn every_lane_keeps_the_authored_tabs() {
    for ssr in [false, true] {
        let code = compile(ssr);
        for literal in LITERALS {
            assert!(code.contains(literal), "ssr={ssr}: {literal:?} in\n{code}");
        }
        // Seven tabs are authored inside literals and the comment; the
        // indentation the erasure prints adds none.
        assert_eq!(code.matches('\t').count(), 7, "ssr={ssr}:\n{code}");
    }
}

#[test]
fn the_source_map_still_anchors_the_erased_script() {
    let source = source();
    let descriptor = parse_sfc(&source, SfcParseOptions::default()).expect("SFC");
    let result = compile_sfc_with_template_syntax_and_codegen_options(
        &descriptor,
        options(false),
        TemplateSyntaxMode::Standard,
        CodegenOptions {
            source_map: true,
            ..Default::default()
        },
    )
    .expect("compile");
    let code = result.code.as_str();
    let mappings = result.map.expect("source map")["mappings"]
        .as_str()
        .expect("mappings")
        .to_owned();
    // Each declaration after a literal with a tab still maps to its authored
    // line (zero-based): the erasure no longer widens what precedes it.
    for (generated, authored) in [("const re = ", 5), ("function pad() {", 7)] {
        let line = code
            .lines()
            .position(|line| line.starts_with(generated))
            .unwrap_or_else(|| panic!("{generated:?} in\n{code}"));
        let sources = source_lines(&mappings, line);
        assert!(
            sources.first() == Some(&authored),
            "{generated:?} maps to {sources:?}"
        );
    }
}

/// The authored (zero-based) source lines the segments on generated `line`
/// point at, decoded from the Source Map v3 `mappings`.
fn source_lines(mappings: &str, line: usize) -> Vec<i64> {
    const BASE64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut source_line = 0i64;
    let mut found = Vec::new();
    for (index, group) in mappings.split(';').enumerate() {
        for field in group.split(',').filter(|field| !field.is_empty()) {
            let mut values = Vec::new();
            let (mut value, mut shift) = (0i64, 0);
            for byte in field.bytes() {
                let digit = BASE64.iter().position(|b| *b == byte).expect("base64") as i64;
                value += (digit & 31) << shift;
                if digit & 32 == 0 {
                    values.push(if value & 1 == 1 {
                        -(value >> 1)
                    } else {
                        value >> 1
                    });
                    (value, shift) = (0, 0);
                } else {
                    shift += 5;
                }
            }
            if let Some(delta) = values.get(2) {
                source_line += delta;
                if index == line {
                    found.push(source_line);
                }
            }
        }
    }
    found
}
