use super::*;
use vize_atelier_sfc::{NativeScssSession, compile_native_styles, scss::ScssProducts};

fn row(name: &str, source: &str, trim: bool) -> Result<(Value, bool), NativeSfcCompileError> {
    let arena = Allocator::default();
    let options = NativeSfcCompileOptions {
        filename: name,
        scope_id: Some("data-v-oracle"),
        style_trim: trim,
        source_map: true,
        ..Default::default()
    };
    let descriptor = Vue.observe_descriptor(&arena, source, options.descriptor);
    let view = descriptor
        .admitted_styles()
        .map_err(|_| NativeSfcCompileError::Descriptor)?;
    let prepared: Vec<_> = view
        .styles()
        .filter_map(|style| ScssProducts::from_style(style).ok())
        .collect();
    let direct = compile_native_styles(&descriptor, options.into(), []);
    let shared = compile_native_styles(&descriptor, options.into(), &prepared);
    assert_eq!(direct, shared, "{name}");
    let session = NativeScssSession::new_scoped(&arena, source, options);
    assert_eq!(session.emit_styles(&prepared), direct, "{name}");
    let mapped = direct?;
    let plain = compile_native_styles(
        &descriptor,
        NativeSfcCompileOptions {
            source_map: false,
            ..options
        }
        .into(),
        &prepared,
    )?;
    assert_eq!(plain.css_source_map(), None);
    assert!(
        plain
            .css_document()
            .is_none_or(|doc| doc.links().is_empty())
    );
    let css = mapped.css();
    assert_eq!(css, plain.css(), "{name}");
    assert_eq!(mapped.scope_id(), plain.scope_id(), "{name}");
    let module = session.emit(&prepared);
    if let Ok(whole) = &module {
        assert_eq!(whole.css_document(), mapped.css_document(), "{name}");
        assert_eq!(whole.css_source_map(), mapped.css_source_map(), "{name}");
        assert_eq!(whole.scope_id(), mapped.scope_id(), "{name}");
    }
    let map: Option<Value> = mapped
        .css_source_map()
        .map(|json| serde_json::from_str(json).unwrap());
    Ok((
        json!({"name":name,"filename":name,"source":source,"stylesOnly":true,"scopedScss":true,"trim":trim,"css":css,"cssMap":map}),
        module.is_ok(),
    ))
}

const STYLE: &str = "<style lang=scss scoped>$ink:red;.badge{color:$ink}</style>";

#[test]
fn script_and_template_refusals_leave_original_style_text_and_mappings_available() {
    for other in [
        "<script setup lang='ts'>const broken: =</script><template><div/></template>",
        "<template><div>{{(}}</div></template>",
        "<script src='./external.js'></script><template><div/></template>",
        "<template lang='pug'>p hello</template>",
        "",
    ] {
        let source = format!("<!-- 雪🦀 -->\r\n{other}\r\n{STYLE}");
        let (result, whole) = row(FILENAME, &source, true).unwrap();
        assert!(!whole, "{other}");
        assert_eq!(result["css"], ".badge[data-v-oracle] {\n  color: red;\n}");
        assert_eq!(result["cssMap"]["sourcesContent"], json!([source]));
    }
    let arena = Allocator::default();
    let source = "<template><p/></template><style scoped lang=scss src='external.scss'/>";
    let descriptor = Vue.observe_descriptor(
        &arena,
        source,
        NativeSfcCompileOptions::default().descriptor,
    );
    let output = compile_native_styles(&descriptor, NativeSfcCompileOptions::default().into(), []);
    assert!(
        matches!(output, Err(NativeSfcCompileError::ExternalBlock { container_index:1, span }) if span.slice(source) == "src='external.scss'")
    );
}

#[test]
#[ignore = "uses the original components and pinned Vue and Sass preprocessors"]
fn every_admitted_stylesheet_on_original_whole_documents_matches_the_published_compilers() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest =
        std::fs::read_to_string(std::env::var_os("VIZE_SCSS_MANIFEST").unwrap()).unwrap();
    const ORIGINALS: [&str; 7] = [
        "crates/vize_patina/tests/fixtures/deprecated-diagnostic-spans/originals/src/views/profile/components/UserCard.vue",
        "tests/_fixtures/_projects/style-preprocessors/src/ScopedScss.vue",
        "tests/_fixtures/differential/linter/component-positive-template-corpus/tiny-rdm--frontend--src--components--content_value--FormatSelector.vue.txt",
        "tests/_fixtures/differential/linter/component-positive-template-corpus/tiny-rdm--frontend--src--components--dialogs--ConnectionDialog.vue.txt",
        "tests/_fixtures/differential/linter/component-positive-template-corpus/tiny-rdm--frontend--src--components--dialogs--PreferencesDialog.vue.txt",
        "tests/_fixtures/differential/linter/no-dupe-keys-447/original-sources/tiny-rdm/frontend/src/components/common/TtlInput.vue.txt",
        "tests/_fixtures/differential/linter/patina-rule-fixes-9872-9873/ScssBox.vue.txt",
    ];
    let mut checked = [false; ORIGINALS.len()];
    let mut rows = Vec::new();
    let (mut candidates, mut full, mut partial) = (0, 0, 0);
    for name in manifest
        .split_terminator('\0')
        .filter(|s| !s.ends_with(".scss"))
    {
        let Ok(source) = std::fs::read_to_string(root.join(name)) else {
            continue;
        };
        if !source.to_ascii_lowercase().contains("scss") {
            continue;
        }
        let arena = Allocator::default();
        let descriptor = Vue.observe_descriptor(
            &arena,
            &source,
            NativeSfcCompileOptions::default().descriptor,
        );
        let Ok(view) = descriptor.admitted_styles() else {
            continue;
        };
        if !view.styles().any(|s| {
            s.attrs().iter().any(|a| a.name == "scoped")
                && s.attrs().iter().any(|a| {
                    a.name == "lang" && a.value.is_some_and(|v| v.eq_ignore_ascii_case("scss"))
                })
        }) {
            continue;
        }
        candidates += 1;
        match (row(name, &source, false), row(name, &source, true)) {
            (Ok((raw, complete)), Ok((trimmed, trimmed_complete))) => {
                assert_eq!(complete, trimmed_complete, "{name}");
                if complete {
                    full += 1
                } else {
                    partial += 1
                }
                if let Some(i) = ORIGINALS.iter().position(|original| *original == name) {
                    checked[i] = true;
                }
                println!("original stylesheet {name} whole_component={complete}");
                rows.extend([raw, trimmed]);
            }
            (Err(raw), Err(trimmed)) => assert_eq!(raw, trimmed, "{name}"),
            outputs => panic!("trim changed original selection for {name}: {outputs:?}"),
        }
    }
    let original_rows = rows.len();
    assert!(partial > 0);
    if std::env::var_os("VIZE_REQUIRE_REPOSITORY_SCSS").is_some() {
        assert_eq!(checked, [true; ORIGINALS.len()], "{ORIGINALS:?}");
    }
    assert_eq!(original_rows, 2 * (full + partial));
    for before in [
        "<script setup>const =</script>",
        "<template><div>{{(}}</div></template>",
        "<script src='external.js'></script><template lang=pug>p hello</template>",
        "<script setup>let a=1</script><script setup>let b=2</script>",
    ] {
        let source = format!("<!-- 雪🦀 -->\r\n{before}{STYLE}");
        rows.push(row(FILENAME, &source, false).unwrap().0);
    }
    let mut oracle = Command::new("node")
        .arg(root.join("crates/vize_atelier_sfc/tests/component_scss/oracle.ts"))
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    oracle
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&rows).unwrap())
        .unwrap();
    assert!(oracle.wait().unwrap().success());
    println!(
        "original scoped styles candidates={candidates} whole_components={full} independent_stylesheets={partial} original_mapped_rows={original_rows}"
    );
}

#[test]
#[ignore = "paired CSS-only requests on original components with increasing template sizes"]
fn style_requests_only_run_the_passes_needed_for_css() {
    mark_installed();
    for elements in [1, 16, 64] {
        let source = format!(
            "<template><main>{}</main></template>{STYLE}",
            "<p>a&#32; b</p>".repeat(elements)
        );
        for mapped in [false, true] {
            let options = NativeSfcCompileOptions {
                filename: FILENAME,
                scope_id: Some("data-v-oracle"),
                source_map: mapped,
                ..Default::default()
            };
            let arena = Allocator::default();
            let full = NativeScssSession::new_scoped(&arena, &source, options)
                .emit([])
                .unwrap();
            let descriptor = Vue.observe_descriptor(&arena, &source, options.descriptor);
            let css = compile_native_styles(&descriptor, options.into(), []).unwrap();
            assert_eq!(full.css_document(), css.css_document());
            assert_eq!(full.css_source_map(), css.css_source_map());
            assert_eq!(full.scope_id(), css.scope_id());
            paired(
                &format!("style-request elements={elements} mapped={mapped}"),
                |only_css| {
                    let arena = Allocator::default();
                    if only_css {
                        let descriptor =
                            Vue.observe_descriptor(&arena, black_box(&source), options.descriptor);
                        drop(black_box(
                            compile_native_styles(&descriptor, options.into(), []).unwrap(),
                        ));
                    } else {
                        let session =
                            NativeScssSession::new_scoped(&arena, black_box(&source), options);
                        drop(black_box(session.emit([]).unwrap()));
                    }
                },
            );
        }
    }
}

#[test]
fn mixed_styles_match_the_authored_order_and_reject_same_text_from_another_owner() {
    let source = format!(
        "<!-- 🦀 -->\r\n<template lang=pug>p hello</template>{STYLE}\r\n<style> .plain {{color:green}} </style><style scoped>.last {{color:blue}}</style>"
    );
    let foreign = source.clone();
    let arena = Allocator::default();
    let options = NativeSfcCompileOptions {
        filename: FILENAME,
        scope_id: Some("data-v-oracle"),
        source_map: true,
        style_trim: true,
        ..Default::default()
    };
    let owner = Vue.observe_descriptor(&arena, &source, options.descriptor);
    let second = Vue.observe_descriptor(&arena, &foreign, options.descriptor);
    let prepared =
        ScssProducts::from_style(owner.admitted_styles().unwrap().styles().next().unwrap())
            .unwrap();
    let direct = compile_native_styles(&owner, options.into(), []).unwrap();
    let shared = compile_native_styles(&owner, options.into(), [&prepared]).unwrap();
    assert_eq!(shared, direct);
    assert_eq!(
        shared.css_document().unwrap().as_str(),
        ".badge[data-v-oracle] {\n  color: red;\n}\n.plain {color:green}\n.last[data-v-oracle] {color:blue}"
    );
    assert_eq!(
        serde_json::from_str::<Value>(shared.css_source_map().unwrap()).unwrap()["sourcesContent"],
        json!([source])
    );
    assert_eq!(
        compile_native_styles(&second, options.into(), [&prepared]),
        Err(NativeSfcCompileError::UnusedPreparedScss {
            span: prepared.block().span()
        })
    );
    assert_eq!(
        compile_native_styles(&owner, options.into(), [&prepared, &prepared]),
        Err(NativeSfcCompileError::UnusedPreparedScss {
            span: prepared.block().span()
        })
    );
}
