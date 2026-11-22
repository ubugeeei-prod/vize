#![cfg(all(feature = "native-sfc-scss", not(target_arch = "wasm32")))]
#![expect(
    clippy::unwrap_used,
    reason = "frozen modules and independent byte oracles"
)]

use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_sfc::{
    NativeSfcCompileError, NativeSfcCompileOptions, compile_native_sfc,
    native::compile_native_sfc_with_scss,
};
use vize_l0::Allocator;

#[path = "observed_scss/component_perf.rs"]
mod profile;

const FILENAME: &str = "Setup雪🌸.vue";
const FIXTURES: &str = include_str!("fixtures/native_sfc_ts_annotation_setup_vue_3_5_35.json");
const STYLES: &str = concat!(
    "<style> /* 🦀 */\r\n.a{color:red} </style>\r\n",
    "<style lang='scss'>$v: #abc; .card{padding:2px;.child{color:$v} &.on{color:blue} margin:0 $v}</style>",
    "<style lang='scss'>$empty: 2px;</style>",
    "<style lang='scss'>$v:red;\r\n$w:$v;\n.a{color:$w; margin:$v $w; padding:\n 0 2px}</style>",
    "<style lang='scss'>\r\n.second{\r width:4px}\u{c}.third {height: 0} </style>",
    "<style></style>"
);

// Both monomorphizations observe the original whole file. Keep refused files
// in the census and require equality of the complete refusal or both outputs.
fn capture(name: &str, source: &str, trim: bool) -> Result<Value, NativeSfcCompileError> {
    let arena = Allocator::default();
    let options = NativeSfcCompileOptions {
        filename: FILENAME,
        style_trim: trim,
        ..Default::default()
    };
    let mapped = compile_native_sfc_with_scss(
        &arena,
        source,
        NativeSfcCompileOptions {
            source_map: true,
            ..options
        },
    );
    let plain = compile_native_sfc_with_scss(&arena, source, options);
    assert_eq!(mapped.result().err(), plain.result().err(), "{name}");
    let output = mapped.result()?;
    let unlinked = plain.result()?;
    assert_eq!(
        (output.code(), output.css()),
        (unlinked.code(), unlinked.css())
    );
    assert!(unlinked.source_map().is_none() && unlinked.css_source_map().is_none());
    assert!(unlinked.document().links().is_empty());
    assert!(
        unlinked
            .css_document()
            .is_none_or(|doc| doc.links().is_empty())
    );
    let file = mapped.observation().admitted().unwrap().file().file();
    assert!(file.is_complete() && core::ptr::eq(file.artifact().source(), source));
    let old = compile_native_sfc(&arena, source, options);
    assert!(old.result().is_err(), "{name}: existing SCSS admission");
    let map: Value = serde_json::from_str(output.source_map().unwrap()).unwrap();
    let css_map: Option<Value> = output
        .css_source_map()
        .map(|map| serde_json::from_str(map).unwrap());
    Ok(
        json!({"name":name,"filename":FILENAME,"source":source,"trim":trim,
        "code":output.code(),"map":map,"css":output.css(),"cssMap":css_map}),
    )
}

#[test]
fn annotated_components_keep_the_frozen_module_and_map_with_mixed_styles() {
    let pack: Value = serde_json::from_str(FIXTURES).unwrap();
    for fixture in pack["fixtures"].as_array().unwrap() {
        let source = format!("{}{STYLES}", fixture["source"].as_str().unwrap());
        for trim in [false, true] {
            let row = capture(fixture["id"].as_str().unwrap(), &source, trim).unwrap();
            let mut expected = fixture["nativeMap"].clone();
            expected["sourcesContent"] = json!([source]);
            assert_eq!(row["code"], fixture["code"]);
            assert_eq!(row["map"], expected);
            assert_eq!(row["cssMap"]["sourcesContent"], json!([source]));
        }
    }
}

#[test]
fn existing_decisions_and_links_match_for_ordinary_css_inputs() {
    let template = "<template><p/></template>";
    for styles in [
        "",
        "<style/>",
        "<style></style><style> .a{color:red} </style>",
        "<style scoped>.card{color:blue}</style>",
        "<style lang='css'>\r\n.a{content:'雪🌸'}\r\n</style>",
        "<style module>.a{color:blue}</style>",
        "<style lang='less'>.a{color:@v}</style>",
    ] {
        let source = format!("{template}{styles}");
        for source_map in [false, true] {
            for style_trim in [false, true] {
                let arena = Allocator::default();
                let options = NativeSfcCompileOptions {
                    source_map,
                    style_trim,
                    ..Default::default()
                };
                let before = compile_native_sfc(&arena, &source, options);
                let after = compile_native_sfc_with_scss(&arena, &source, options);
                assert_eq!(before.result().err(), after.result().err(), "{styles}");
                if let Ok(expected) = before.result() {
                    let actual = after.result().unwrap();
                    assert_eq!(actual.document(), expected.document());
                    assert_eq!(actual.css_document(), expected.css_document());
                    assert_eq!(actual.source_map(), expected.source_map());
                    assert_eq!(actual.css_source_map(), expected.css_source_map());
                    assert_eq!(actual.scope_id(), expected.scope_id());
                }
            }
        }
    }
}

#[test]
fn failure_in_any_style_script_or_template_preserves_the_complete_input() {
    let template = "<template><p/></template>";
    let good = "<style lang=scss>.good{color:red}</style>";
    for (body, attr) in [
        (".b{width:$missing}", ""),
        (".b{color:v-bind(color)}", ""),
        ("@import 'sheet';", ""),
        (".b{", ""),
        (".b{color:red}", " module"),
        (".b{color:red}", " scoped"),
        (".b{color:red}", " src='other.scss'"),
    ] {
        let source = format!("{template}{good}<style lang=scss{attr}>{body}</style>");
        let arena = Allocator::default();
        let result = compile_native_sfc_with_scss(&arena, &source, Default::default());
        assert!(
            matches!(
                result.result(),
                Err(NativeSfcCompileError::ExternalBlock {
                    container_index: 2,
                    ..
                } | NativeSfcCompileError::StyleCompilationUnavailable {
                    container_index: 2,
                    ..
                } | NativeSfcCompileError::Scss {
                    container_index: 2,
                    ..
                })
            ),
            "{source}"
        );
        assert!(core::ptr::eq(
            result.observation().descriptor().source(),
            source.as_str()
        ));
    }
    for source in [
        format!("{good}<script setup>const n=;</script>{template}"),
        good.to_owned(),
    ] {
        assert!(capture("Broken.vue", &source, false).is_err());
    }
}

#[test]
#[ignore = "requires the tracked manifest, Vue compiler and Dart Sass"]
fn complete_styles_and_authored_map_segments_match_upstream() {
    let mut rows = Vec::new();
    let mut candidates = 0;
    let mut originals = 0;
    let mut borrowed = 0;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest =
        std::fs::read_to_string(std::env::var_os("VIZE_SCSS_MANIFEST").unwrap()).unwrap();
    let pack: Value = serde_json::from_str(FIXTURES).unwrap();
    let setup = pack["fixtures"][0]["source"].as_str().unwrap();
    for name in manifest
        .split_terminator('\0')
        .filter(|name| !name.ends_with(".scss"))
    {
        let Ok(source) = std::fs::read_to_string(root.join(name)) else {
            continue;
        };
        if !source.to_ascii_lowercase().contains("scss") {
            continue;
        }
        let Ok(stock) = vize_atelier_sfc::parse_sfc(&source, Default::default()) else {
            continue;
        };
        let mut styles = stock
            .styles
            .iter()
            .filter(|s| {
                s.lang
                    .as_deref()
                    .is_some_and(|lang| lang.eq_ignore_ascii_case("scss"))
            })
            .peekable();
        if styles.peek().is_none() {
            continue;
        }
        candidates += 1;
        if let Ok(row) = capture(name, &source, false) {
            rows.push(row);
            originals += 1;
        }
        for style in styles.filter(|s| s.src.is_none() && !s.scoped && s.module.is_none()) {
            let tag = &source[style.loc.tag_start..style.loc.tag_end];
            let composed = format!("{setup}{tag}");
            if let Ok(row) = capture(name, &composed, false) {
                rows.push(row);
                borrowed += 1;
            }
        }
    }
    assert!(candidates > 0 && borrowed > 0);
    for fixture in pack["fixtures"].as_array().unwrap() {
        for suffix in [STYLES, "<style lang='scss'>$only:blue;</style>"] {
            for prefix in ["", "<!-- 🦀 -->\r\n", "<!-- 雪🌸 -->"] {
                for trim in [false, true] {
                    let source = format!("{prefix}{}{suffix}", fixture["source"].as_str().unwrap());
                    rows.push(capture("Composed.vue", &source, trim).unwrap());
                }
            }
        }
    }
    let mut child = Command::new("node")
        .arg(root.join("crates/vize_atelier_sfc/tests/observed_scss/component.ts"))
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&rows).unwrap())
        .unwrap();
    assert!(child.wait().unwrap().success());
    println!(
        "whole SCSS components candidates={candidates} original={originals} original_styles_with_frozen_setup={borrowed} checked={}",
        rows.len()
    );
}
