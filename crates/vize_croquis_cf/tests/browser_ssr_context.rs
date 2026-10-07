//! Entire authored #7908 corpus through public Croquis and cross-file APIs.
use serde_json::{Value, json};
use std::{fs, path::Path};
use vize_armature::Parser;
use vize_croquis::{Analyzer, AnalyzerOptions, Croquis};
use vize_croquis_cf::{
    CrossFileAnalyzer, CrossFileDiagnosticKind, CrossFileOptions, DiagnosticSource,
};
use vize_l0::{Allocator, String, cstr};

struct Script<'a> {
    content: &'a str,
    start: u32,
    setup: bool,
}

fn scripts(source: &str) -> Result<Vec<Script<'_>>, &'static str> {
    source
        .match_indices("<script")
        .map(|(at, _)| {
            let tail = source.get(at..).ok_or("invalid script opening offset")?;
            let content_at = at + tail.find('>').ok_or("missing script opening delimiter")? + 1;
            let content_tail = source
                .get(content_at..)
                .ok_or("invalid script content offset")?;
            let end = content_at
                + content_tail
                    .find("</script>")
                    .ok_or("missing script closing tag")?;
            Ok(Script {
                content: source
                    .get(content_at..end)
                    .ok_or("invalid script content range")?,
                start: content_at as u32,
                setup: source
                    .get(at..content_at)
                    .ok_or("invalid script opening range")?
                    .contains("setup"),
            })
        })
        .collect()
}

fn draw(script: &Script<'_>) -> Croquis {
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    if script.setup {
        analyzer.analyze_script_setup(script.content);
    } else {
        analyzer.analyze_script_plain(script.content);
    }
    analyzer.finish()
}

fn analyze(source: &str, filename: &str) -> Result<Vec<Value>, String> {
    let blocks = scripts(source)?;
    let setup = blocks.iter().find(|block| block.setup);
    let plain = blocks.iter().find(|block| !block.setup);
    let analysis = match (plain, setup) {
        (Some(plain), Some(setup)) => {
            let mut joined = draw(setup);
            joined.shift_script_offsets(plain.content.len() as u32 + 1);
            joined.merge_plain_script(draw(plain));
            joined
        }
        (_, Some(setup)) => draw(setup),
        (Some(plain), None) => draw(plain),
        _ => return Err("whole fixture requires a script".into()),
    };
    let start = source
        .find("<template>")
        .ok_or("missing template opening tag")?
        + "<template>".len();
    let tail = source
        .get(start..)
        .ok_or("invalid template content offset")?;
    let end = start
        + tail
            .find("</template>")
            .ok_or("missing template closing tag")?;
    let template = source
        .get(start..end)
        .ok_or("invalid template content range")?;
    let allocator = Allocator::default();
    let (root, errors) = Parser::new(&allocator, template).parse();
    if !errors.is_empty() {
        return Err(cstr!("{filename}: {errors:?}"));
    }
    let mut drawer = Analyzer::with_summary(AnalyzerOptions::full(), analysis, true);
    drawer.analyze_template(&root);
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::minimal().with_server_client_boundary(true));
    analyzer.add_file_with_analysis(Path::new(filename), source, drawer.finish());
    analyzer.analyze().diagnostics.into_iter().map(|diagnostic| {
        let CrossFileDiagnosticKind::BrowserApiInSsr { api, context } = diagnostic.kind else {
            return Err(cstr!("unexpected full diagnostic: {diagnostic:?}"));
        };
        Ok(json!({
            "kind": "BrowserApiInSsr", "api": api.as_str(), "context": context.as_str(),
            "severity": diagnostic.severity.display_name(), "file": diagnostic.primary_file.as_u32(),
            "source": match diagnostic.primary_source {
                DiagnosticSource::Script => "script",
                DiagnosticSource::Template => "template",
                DiagnosticSource::Unspecified => "unspecified",
            },
            "start": diagnostic.primary_offset, "end": diagnostic.primary_end_offset,
            "related": diagnostic.related_files.iter().map(|(file, at, text)| {
                json!([file.as_u32(), at, text.as_str()])
            }).collect::<Vec<_>>(),
            "message": diagnostic.message.as_str(),
            "suggestion": diagnostic.suggestion.as_ref().map(|text| text.as_str()),
        }))
    }).collect()
}

#[test]
fn complete_originals_and_guarded_handler_watch_controls_preserve_all_producer_fields() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../vize/tests/fixtures/issue-7908");
    let source: Value =
        serde_json::from_slice(&fs::read(directory.join("source.json")).unwrap()).unwrap();
    let cases = source["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 38);
    let mut observations = Vec::new();
    for case in cases {
        let filename = case["path"].as_str().unwrap();
        let bytes = fs::read(directory.join(case["file"].as_str().unwrap())).unwrap();
        assert_eq!(bytes.len() as u64, case["bytes"].as_u64().unwrap());
        let input = std::str::from_utf8(&bytes).unwrap();
        let blocks = scripts(input).unwrap();
        let plain = blocks.iter().find(|block| !block.setup);
        let setup = blocks.iter().find(|block| block.setup);
        let expected: Vec<_> = case["doctorLocations"].as_array().unwrap().iter().map(|location| {
            let authored = location["start"].as_u64().unwrap() as u32;
            let (block, prefix) = if let Some(setup) = setup
                && setup.start <= authored && authored < setup.start + setup.content.len() as u32 {
                (setup, plain.map_or(0, |plain| plain.content.len() as u32 + 1))
            } else { (plain.unwrap(), 0) };
            let relative = prefix + authored - block.start;
            let tail = input.get(authored as usize..).unwrap();
            let api = if tail.starts_with("window") { "window" } else {
                assert!(tail.starts_with("document")); "document"
            };
            json!({
                "kind":"BrowserApiInSsr", "api":api, "context":if api == "document" { "DOM API" } else { "Browser global" },
                "severity":"warning", "file":0, "source":"script", "start":relative, "end":relative,
                "related":[], "message":"Browser API used in potentially SSR context",
                "suggestion":source["defaultHelp"]["suggestion"],
            })
        }).collect();
        let actual = analyze(input, filename).unwrap();
        observations.push(json!({"input":case, "expected":expected, "actual":actual}));
        assert_eq!(actual, expected, "{filename}");
    }
    assert_eq!(observations.len(), 38);
    let output = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/differential/browser-ssr-producer.json");
    fs::create_dir_all(output.parent().unwrap()).unwrap();
    fs::write(
        output,
        serde_json::to_vec_pretty(&json!({
            "schema":"vize.browser-ssr.producer", "version":1,
            "sourceRevision":std::env::var("SOURCE_SHA").ok(), "complete":true,
            "inputs":observations.len(), "observations":observations
        }))
        .unwrap(),
    )
    .unwrap();
}
