use super::*;
use vize_atelier_sfc::NativeScssSession;

const REAL: &str = include_str!(
    "../../../../tests/_fixtures/differential/linter/patina-rule-fixes-9872-9873/ScssBox.vue.txt"
);

fn row(name: &str, source: &str, trim: bool) -> Result<Value, NativeSfcCompileError> {
    let arena = Allocator::default();
    let options = NativeSfcCompileOptions {
        filename: FILENAME,
        scope_id: Some("data-v-oracle"),
        source_map: true,
        style_trim: trim,
        ..Default::default()
    };
    let old = NativeScssSession::new(&arena, source, options);
    assert!(
        old.emit([]).is_err(),
        "{name}: previous scoped SCSS decision"
    );
    let mapped = old.with_scoped_styles().emit([])?;
    let plain = NativeScssSession::new(
        &arena,
        source,
        NativeSfcCompileOptions {
            source_map: false,
            ..options
        },
    )
    .with_scoped_styles()
    .emit([])
    .unwrap();
    assert_eq!(mapped.scope_id(), Some("data-v-oracle"));
    assert_eq!((mapped.code(), mapped.css()), (plain.code(), plain.css()));
    assert!(plain.source_map().is_none() && plain.css_source_map().is_none());
    assert!(
        plain.document().links().is_empty()
            && plain
                .css_document()
                .is_none_or(|doc| doc.links().is_empty())
    );
    let map: Value = serde_json::from_str(mapped.source_map().unwrap()).unwrap();
    let css_map: Option<Value> = mapped
        .css_source_map()
        .map(|map| serde_json::from_str(map).unwrap());
    Ok(
        json!({"name":name,"filename":FILENAME,"source":source,"trim":trim,"scopedScss":true,
        "code":mapped.code(),"map":map,"css":mapped.css(),"cssMap":css_map}),
    )
}

#[test]
fn original_project_component_compiles_without_retagging_or_preprocessing_its_source() {
    let product = row("ScssBox.vue", REAL, false).unwrap();
    assert_eq!(
        product["css"],
        ".box .inner[data-v-oracle] {\n  margin: 4px;\n}"
    );
    assert_eq!(product["cssMap"]["sourcesContent"], json!([REAL]));
    assert_eq!(product["map"]["sourcesContent"], json!([REAL]));
    assert!(
        product["code"]
            .as_str()
            .unwrap()
            .contains("_sfc_main.__scopeId = \"data-v-oracle\"")
    );
}

#[test]
fn unsupported_selectors_options_and_broken_styles_keep_original_typed_refusals() {
    for tag in [
        "<style lang=scss scoped>.a::before{color:red}</style>",
        "<style scoped lang=scss>.a,.b{color:red}</style>",
        "<style lang=scss scoped>.a{color:$missing}</style>",
        "<style scoped lang=scss>.a{color:</style>",
        "<style scoped lang=scss>.a{color:v-bind(color)}</style>",
        "<style scoped module lang=scss>.a{color:red}</style>",
        "<style src='other.scss' scoped lang=scss/>",
    ] {
        let source = format!("<template><p/></template>{tag}");
        let arena = Allocator::default();
        let session =
            NativeScssSession::new(&arena, &source, Default::default()).with_scoped_styles();
        assert!(core::ptr::eq(
            session.observation().descriptor().source(),
            source.as_str()
        ));
        assert!(matches!(
            session.emit([]),
            Err(NativeSfcCompileError::Scss { .. }
                | NativeSfcCompileError::ExternalBlock { .. }
                | NativeSfcCompileError::StyleCompilationUnavailable { .. })
        ));
    }
}

#[test]
#[ignore = "requires the tracked Vue manifest, pinned Sass and Vue compiler"]
fn observed_scoped_components_match_published_style_and_render_products() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest =
        std::fs::read_to_string(std::env::var_os("VIZE_SCSS_MANIFEST").unwrap()).unwrap();
    let mut rows = Vec::new();
    let (mut candidates, mut originals, mut borrowed) = (0, 0, 0);
    let frozen: Value = serde_json::from_str(FIXTURES).unwrap();
    let setup = frozen["fixtures"][0]["source"].as_str().unwrap();
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
        let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&source, Default::default()) else {
            continue;
        };
        let styles: Vec<_> = descriptor
            .styles
            .iter()
            .filter(|s| {
                s.scoped
                    && s.lang
                        .as_deref()
                        .is_some_and(|l| l.eq_ignore_ascii_case("scss"))
            })
            .collect();
        if styles.is_empty() {
            continue;
        }
        candidates += 1;
        if let Ok(r) = row(name, &source, false) {
            rows.push(r);
            originals += 1;
        }
        for style in styles {
            let tag = &source[style.loc.tag_start..style.loc.tag_end];
            if let Ok(r) = row(name, &format!("{setup}{tag}"), false) {
                rows.push(r);
                borrowed += 1;
            }
        }
    }
    for fixture in frozen["fixtures"].as_array().unwrap() {
        for trim in [false, true] {
            for tag in [
                "<style lang=scss scoped>\r\n$v:red;\n.a{color:$v;.child{padding:2px} &.on{margin:$v} height:0}.b{width:1px}</style>",
                "<style scoped lang='scss'>$n:1px;.a{&-name{width:$n}.b.c{.d{padding:$n}}}</style><style>.plain{color:blue}</style>",
                "<style scoped lang='scss'>$empty:red;</style>",
            ] {
                let text = format!(
                    "<!-- 雪🦀 -->\r\n{}{tag}",
                    fixture["source"].as_str().unwrap()
                );
                rows.push(row("Scoped.vue", &text, trim).unwrap());
            }
        }
    }
    if std::env::var_os("VIZE_REQUIRE_REPOSITORY_SCSS").is_some() {
        assert!(
            originals > 0
                && borrowed > 0
                && rows.iter().any(|r| r["source"].as_str() == Some(REAL))
        );
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
        "scoped SCSS candidates={candidates} complete_originals={originals} original_tags_with_setup={borrowed} checked={}",
        rows.len()
    );
}
