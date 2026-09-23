//! TS-25: template-expression anchors from the S4 projection match the
//! current virtual-TS generator on the TS-40 matrix.
//!
//! Generated text is not compared. A run that compares nothing fails.
//! Template diagnostics from `type_check_sfc` (the `vize check` binding
//! check) on `tests/_fixtures` must sit inside both mappings. Script
//! diagnostics are outside this projection.

use std::ops::Range;
use std::path::{Path, PathBuf};

use vize_armature::parse;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_canon::projection::project_template_expressions;
use vize_canon::virtual_ts::generate_virtual_ts;
use vize_croquis::{Analyzer, AnalyzerOptions};
use vize_s0::Allocator;

/// Measured template-expression anchors on the committed TS-40 matrix.
const TEMPLATE_ANCHOR_COMPARISONS: usize = 22;

#[test]
fn template_expression_anchors_match_the_current_generator() {
    let mut compared = 0usize;
    let mut files = 0usize;
    for (id, path) in matrix_fixtures() {
        let source = std::fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!("read {} ({}): {error}", path.display(), id);
        });
        let Some(template) = template_body(&source) else {
            continue;
        };
        files += 1;
        compared += compare_template(&id, &template);
    }
    assert!(files > 0, "the TS-40 matrix contributed no template");
    assert_eq!(
        compared, TEMPLATE_ANCHOR_COMPARISONS,
        "template-expression anchor comparisons changed; a zero run fails"
    );
}

fn compare_template(id: &str, template: &str) -> usize {
    let allocator = Allocator::new();
    let (root, _errors) = parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let old = generate_virtual_ts(&summary, None, Some(&root), 0);
    let new = project_template_expressions(template);
    let mut compared = 0usize;
    for span in new.spans() {
        let authored = span.src_range.clone();
        let resolved = new.diagnostic_range_to_authored(span.gen_range.start, span.gen_range.end);
        assert_eq!(
            resolved,
            Some((authored.start, authored.end)),
            "{id}: new projection does not resolve its own anchor {authored:?}"
        );
        let old_ranges = old_authored_ranges(&old.mapping);
        let old_hit = old_ranges.iter().any(|range| range == &authored);
        let text = template.get(authored.clone()).unwrap_or("");
        let contained: Vec<_> = old_ranges
            .iter()
            .filter(|range| authored.start <= range.start && range.end <= authored.end)
            .collect();
        // A compound interpolation is one opaque expression. Its pessimal
        // answer is the whole authored span, which contains the generator's
        // finer identifier ranges.
        let pessimal = text.matches("{{").count() > 1
            && !contained.is_empty()
            && contained.iter().any(|range| *range != &authored);
        assert!(
            old_hit || pessimal,
            "{id}: S4 anchor {authored:?} ({text}) is not an authored range of the current generator"
        );
        compared += 1;
    }
    compared
}

/// Measured check diagnostics on `tests/_fixtures` that both mappings cover.
const CHECK_DIAGNOSTIC_COMPARISONS: usize = 64;

#[test]
fn check_diagnostics_land_inside_the_projection() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/_fixtures");
    let mut files = Vec::new();
    collect_vue(&root, &mut files);
    assert!(
        files.len() > 100,
        "the vize check fixture corpus shrank to {} files",
        files.len()
    );
    let mut compared = 0usize;
    let mut divergences = Vec::new();
    for path in &files {
        let Ok(source) = std::fs::read_to_string(path) else {
            continue;
        };
        let Ok(descriptor) = parse_sfc(&source, SfcParseOptions::default()) else {
            continue;
        };
        let Some(template) = descriptor.template.as_ref() else {
            continue;
        };
        let body: &str = template.content.as_ref();
        if body.is_empty() {
            continue;
        }
        let base = template.loc.start as usize;
        let options = vize_canon::SfcTypeCheckOptions::new(path.display().to_string());
        let checked = vize_canon::type_check_sfc(&source, &options);
        let (old, new) = expression_ranges(body);
        for diagnostic in &checked.diagnostics {
            let start = diagnostic.start as usize;
            let end = diagnostic.end as usize;
            if start < base || end > base + body.len() || start >= end {
                continue;
            }
            let span = (start - base)..(end - base);
            if !contains(&old, &span) {
                continue;
            }
            if contains(&new, &span) {
                compared += 1;
            } else if divergences.len() < 12 {
                divergences.push(format!(
                    "{} {:?} {:?}",
                    path.display(),
                    diagnostic.code,
                    body.get(span).unwrap_or("")
                ));
            }
        }
    }
    assert!(
        divergences.is_empty(),
        "check diagnostics missed by the projection:\n{}",
        divergences.join("\n")
    );
    eprintln!("check diagnostic comparisons={compared}");
    assert_eq!(
        compared, CHECK_DIAGNOSTIC_COMPARISONS,
        "vize check diagnostic comparisons changed; a zero run fails"
    );
}

fn expression_ranges(template: &str) -> (Vec<Range<usize>>, Vec<Range<usize>>) {
    let allocator = Allocator::new();
    let (root, _errors) = parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let old = generate_virtual_ts(&summary, None, Some(&root), 0);
    let new = project_template_expressions(template);
    (old_authored_ranges(&old.mapping), old_authored_ranges(&new))
}

fn contains(ranges: &[Range<usize>], span: &Range<usize>) -> bool {
    ranges
        .iter()
        .any(|range| range.start <= span.start && span.end <= range.end)
}

fn collect_vue(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            // `_git-worktrees` is a gitignored local checkout, not the committed corpus.
            if name == "node_modules"
                || name == "_git"
                || name == "_git-worktrees"
                || name == "target"
            {
                continue;
            }
            collect_vue(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "vue") {
            out.push(path);
        }
    }
}

fn old_authored_ranges(
    mapping: &vize_canon::virtual_ts::ProjectionMapping,
) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    for row in mapping.spans() {
        ranges.push(row.src_range.clone());
        for sub in &row.sub_spans {
            ranges.push(sub.src_range.clone());
        }
    }
    ranges
}

fn template_body(source: &str) -> Option<String> {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).ok()?;
    let template = descriptor.template.as_ref()?;
    Some(template.content.as_ref().to_string())
}

fn matrix_fixtures() -> Vec<(String, PathBuf)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let matrix =
        std::fs::read_to_string(root.join("tests/_fixtures/davinci-ts40-projection/matrix.json"))
            .expect("TS-40 matrix");
    let mut fixtures = Vec::new();
    for (id_at, _) in matrix.match_indices("\"id\":") {
        let id = quoted_after(&matrix, id_at + "\"id\":".len());
        let file_at = matrix[id_at..].find("\"file\":").expect("fixture file");
        let file = quoted_after(&matrix[id_at..], file_at + "\"file\":".len());
        fixtures.push((id, root.join(file)));
    }
    assert_eq!(fixtures.len(), 12, "the TS-40 matrix lists 12 fixtures");
    fixtures
}

fn quoted_after(source: &str, from: usize) -> String {
    let rest = source[from..].trim_start();
    let rest = rest.strip_prefix('"').expect("opening quote");
    let end = rest.find('"').expect("closing quote");
    rest[..end].to_string()
}
