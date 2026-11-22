use super::*;
use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_l1::css::lang::StyleLanguage;

fn examine(name: &str, block: SourceBlock<'_>, rows: &mut Vec<Value>) -> bool {
    let products = ScssProducts::read(block);
    let historical = StyleLanguage::Scss.authored_classes(block.source());
    assert_eq!(products.classes(), historical.as_ref(), "{name}");
    let Ok(css) = products.css() else {
        return false;
    };
    let program = products.program().unwrap();
    for use_ in program.variable_uses() {
        let decl = program.definition_at(use_.span.start).unwrap();
        assert_eq!(
            decl.span.slice(block.root_source()).replace('_', "-"),
            use_.span.slice(block.root_source()).replace('_', "-"),
            "{name}"
        );
        assert!(decl.span.end <= use_.span.start);
    }
    let mut samples = Vec::with_capacity(31);
    for i in 0..35 {
        let start = std::time::Instant::now();
        for _ in 0..8 {
            let product = ScssProducts::read(std::hint::black_box(block));
            std::hint::black_box(product.css().unwrap());
            std::hint::black_box(product.classes());
            let program = product.program().unwrap();
            for reference in program.variable_uses() {
                std::hint::black_box(program.definition_at(reference.span.start));
            }
        }
        if i >= 4 {
            samples.push(start.elapsed().as_nanos() as u64 / 8);
        }
    }
    samples.sort_unstable();
    rows.push(json!({"name": name, "source": block.source(), "css": css.as_str(), "native_ns": samples[15]}));
    true
}

#[test]
#[ignore = "all tracked SCSS inputs and the actual project Dart Sass preprocessor"]
fn every_admitted_scss_input_matches_the_project_preprocessor() {
    mark_installed();
    let manifest = std::env::var("VIZE_SCSS_MANIFEST").unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let mut rows = Vec::new();
    let mut selected = 0;
    let mut files = 0;
    let mut originals = 0;
    for name in std::fs::read_to_string(manifest)
        .unwrap()
        .split_terminator('\0')
    {
        let path = root.join(name);
        let bytes = std::fs::read(path).unwrap();
        let Ok(source) = std::str::from_utf8(&bytes) else {
            continue;
        };
        files += 1;
        if name.ends_with(".scss") {
            selected += 1;
            examine(name, whole(source), &mut rows);
            continue;
        }
        if !source.to_ascii_lowercase().contains("scss") {
            continue;
        }
        let Ok(desc) = vize_atelier_sfc::parse_sfc(source, Default::default()) else {
            continue;
        };
        let arena = Allocator::default();
        let native = Vue.observe_descriptor(&arena, source, options());
        for style in &desc.styles {
            if style.src.is_some()
                || vize_l1::container::lang::style_lang(style.lang.as_deref())
                    != Ok(StyleLanguage::Scss)
            {
                continue;
            }
            selected += 1;
            let text = source.get(style.loc.start..style.loc.end).unwrap();
            assert_eq!(text, style.content.as_ref());
            let block = SourceRoot::new(source)
                .unwrap()
                .block(text, style.loc.start as u32)
                .unwrap();
            if examine(name, block, &mut rows)
                && let Ok(view) = native.admitted()
                && let Some(actual) = view.styles().find(|s| s.block().span() == block.span())
            {
                assert_eq!(
                    ScssProducts::from_style(actual)
                        .unwrap()
                        .css()
                        .unwrap()
                        .as_str(),
                    rows.last().unwrap()["css"].as_str().unwrap()
                );
                originals += 1;
            }
        }
    }
    let real = rows.len();
    for (label, src) in [
        ("empty", ""),
        ("root binding", "$x:4px;"),
        ("mixed nesting", ".a {a:b; .b {} x:y}"),
        ("reassignment", "$x:red;.a{x:$x}$x:blue;.b{x:$x}"),
        ("property list", ".a{x:1px solid #a0b1c2}"),
        ("three levels", ".a { .b { &__child { color: red } } }"),
    ] {
        assert!(examine(label, whole(src), &mut rows));
    }
    if std::env::var_os("VIZE_REQUIRE_REPOSITORY_SCSS").is_some() {
        assert!(
            real >= 19 && originals >= 11,
            "real {real} original Vue views {originals}"
        );
    }
    let mut child = Command::new("node")
        .current_dir(&root)
        .arg(root.join("crates/vize_atelier_sfc/tests/observed_scss/oracle.mjs"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&rows).unwrap())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "Dart Sass failed");
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["checked"], rows.len());
    for (input, ns) in rows.iter().zip(result["medians"].as_array().unwrap()) {
        let native = input["native_ns"].as_u64().unwrap();
        let dart = ns.as_u64().unwrap();
        eprintln!("scss_compile case={} ns={dart}/{native}", input["name"]);
        assert!(native <= dart, "{}", input["name"]);
    }
    eprintln!(
        "scss_differential files={files} selected_blocks={selected} admitted={real} original_vue_views={originals} dart_sass={}",
        result["version"]
    );
}
