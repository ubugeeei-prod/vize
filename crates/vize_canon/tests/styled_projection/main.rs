//! Qualify complete original styled components against the existing emitter.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "standalone checker fixtures assert complete oracle inputs and use owned UTF-8 controls"
)]
use std::{fs, path::Path};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_canon::{
    ImportRewriter, batch::generate_vue_document_virtual_ts, virtual_ts::VirtualTsOptions,
};

use vize_l0::{Allocator, Span, line_index::utf16_len};
use vize_l1_to_l2::native_file::{NativeSfcObservation, lower_sfc_native};
use vize_l4::targets::ts::vue::{
    StyledVueInput, VueProjection, VueProjectionError, project_vue, project_vue_no_links,
};

mod checker;
mod input;
use input::options;

#[derive(Default, Debug)]
struct Counts {
    sources: usize,
    invalid: usize,
    native: usize,
    previous: usize,
    styled: usize,
    styles: usize,
    recorded_bytes: usize,
    links: usize,
    ranges: usize,
    retained_errors: usize,
    canon_bytes: usize,
    style_bodies: usize,
    bindings: usize,
    incomplete: usize,
}

// Preserve byte offsets, CR/LF positions, and every authored script and template
// byte for the original projection. Only the independent control is rewritten.
fn control_source(source: &str, observed: &NativeSfcObservation<'_>) -> String {
    let mut replacement = source.as_bytes().to_vec();
    for style in observed.descriptor().admitted().unwrap().styles() {
        let span = style.original_block().authored_span();
        for byte in replacement
            .get_mut(span.start as usize..span.end as usize)
            .expect("authored style block")
        {
            if !matches!(*byte, b'\n' | b'\r') {
                *byte = b' ';
            }
        }
    }
    String::from_utf8(replacement).unwrap()
}

fn compare(current: &VueProjection<'_, '_>, original: &VueProjection<'_, '_>, counts: &mut Counts) {
    assert_eq!(current.source_kind(), original.source_kind());
    assert_eq!(current.document(), original.document());
    let doc = original.document();
    assert_eq!(
        current.file().artifact().source(),
        current.original().observation().descriptor().source()
    );
    let mut probes = vec![0, 1, doc.len() / 2, doc.len().saturating_sub(1), doc.len()];
    for link in doc.links() {
        probes.extend([link.generated.start as usize, link.generated.end as usize]);
    }
    probes.sort_unstable();
    probes.dedup();
    for byte in probes {
        let Ok(start) = u32::try_from(byte) else {
            continue;
        };
        for width in [0, 1, 2, 9] {
            if let Some(end) = start.checked_add(width) {
                let span = Span::new(start, end);
                assert_eq!(current.map_span(span), original.map_span(span));
                counts.ranges += 1;
            }
        }
        if let Some(prefix) = doc.as_str().get(..byte) {
            let at = u32::try_from(utf16_len(prefix)).unwrap();
            for width in [0, 1, 2, u32::MAX] {
                assert_eq!(current.map_utf16(at, width), original.map_utf16(at, width));
                counts.ranges += 1;
            }
        }
    }
    if doc.is_recording() {
        counts.recorded_bytes += doc.len();
        counts.links += doc.links().len();
    }
}

// Ask the independent original CSS lowerer whether each possible opener
// belongs to code. Complete its argument on a copy of the original prefix so
// an unfinished call is observable without rewriting comments or earlier strings.
fn first_original_binding(css: &str) -> Option<usize> {
    use vize_croquis::sfc::__internal::v_bind_expression_ranges;
    css.match_indices("v-bind").find_map(|(start, spelling)| {
        let end = start + spelling.len();
        let first = css
            .get(end..)?
            .chars()
            .find(|&c| c != '\u{feff}' && (c == '\u{85}' || !c.is_whitespace()));
        if first != Some('(') {
            return None;
        }
        let mut probe = css.get(..end)?.to_owned();
        probe.push_str("(x)");
        let marker = end + 1..end + 2;
        v_bind_expression_ranges(&probe)
            .contains(&marker)
            .then_some(start)
    })
}

fn qualify(source: &str, path: &Path, counts: &mut Counts) {
    // Exercise the lexical boundary even on components the later IR cannot admit.
    if let Ok(descriptor) = parse_sfc(source, SfcParseOptions::default()) {
        for style in &descriptor.styles {
            use vize_croquis::sfc::__internal::{
                checked_v_bind_expression_ranges, v_bind_expression_ranges,
            };
            use vize_l1::css::bind::{first_v_bind_start, v_bind_calls};
            let css = style.content.as_ref();
            let checked = checked_v_bind_expression_ranges(css);
            let complete = checked.is_some();
            let expected = checked.unwrap_or_else(|| v_bind_expression_ranges(css));
            let calls: Vec<_> = v_bind_calls(css).collect();
            let actual: Vec<_> = calls.iter().map(|call| call.expression_range()).collect();
            assert_eq!(actual, expected);
            assert_eq!(first_v_bind_start(css), first_original_binding(css));
            counts.style_bodies += 1;
            counts.bindings += calls.len();
            counts.incomplete += usize::from(!complete);
        }
    }
    let arena = Allocator::default();
    let obs = lower_sfc_native(&arena, source, options());
    let Some(owner) = obs.admitted() else { return };
    counts.native += 1;
    let styles = obs.descriptor().admitted().unwrap().styles().len();
    let strict = project_vue(owner);
    let prepared = StyledVueInput::prepare(obs.admitted().unwrap());
    let projected = prepared
        .as_ref()
        .map_err(|e| *e)
        .and_then(StyledVueInput::project);
    let silent = prepared
        .as_ref()
        .map_err(|e| *e)
        .and_then(StyledVueInput::project_no_links);
    let result = projected.as_ref().map(|_| ()).map_err(|e| *e);
    assert_eq!(result, silent.as_ref().map(|_| ()).map_err(|e| *e));
    if styles == 0 {
        assert_eq!(result, strict.as_ref().map(|_| ()).map_err(|e| *e));
        match (projected, strict, silent) {
            (Ok(on), Ok(control), Ok(off)) => {
                counts.previous += 1;
                compare(&on, &control, counts);
                compare(
                    &off,
                    &project_vue_no_links(obs.admitted().unwrap()).unwrap(),
                    counts,
                );
            }
            _ => counts.retained_errors += 1,
        }
        return;
    }
    assert_eq!(strict.err(), Some(VueProjectionError::UnsupportedStyles));
    let (Ok(on), Ok(off)) = (projected, silent) else {
        counts.retained_errors += 1;
        return;
    };
    let masked = control_source(source, &obs);
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("original SFC parser");
    assert!(descriptor.css_vars.is_empty());
    assert!(
        descriptor
            .styles
            .iter()
            .all(|style| style.module.is_none() && style.src.is_none())
    );
    let rewriter = ImportRewriter::new();
    let config = VirtualTsOptions::default();
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path);
    let canon = generate_vue_document_virtual_ts(&path, source, &config, &rewriter, false)
        .expect("original styled checker");
    let canon_control = generate_vue_document_virtual_ts(&path, &masked, &config, &rewriter, false)
        .expect("control checker");
    assert_eq!(canon.code, canon_control.code);
    assert_eq!(canon.pre_rewrite_code, canon_control.pre_rewrite_code);
    assert_eq!(canon.mapping, canon_control.mapping);
    assert_eq!(
        canon.import_source_map.is_empty(),
        canon_control.import_source_map.is_empty()
    );
    if !canon.import_source_map.is_empty() {
        for offset in 0..=u32::try_from(canon.code.len().max(canon.pre_rewrite_code.len())).unwrap()
        {
            assert_eq!(
                canon.import_source_map.get_original_offset(offset),
                canon_control.import_source_map.get_original_offset(offset)
            );
            assert_eq!(
                canon.import_source_map.get_virtual_offset(offset),
                canon_control.import_source_map.get_virtual_offset(offset)
            );
        }
    }
    assert_eq!(canon.source_type, canon_control.source_type);
    assert_eq!(canon.virtual_suffix, canon_control.virtual_suffix);
    counts.canon_bytes += canon.code.len();
    let comparison_arena = Allocator::default();
    let control = lower_sfc_native(&comparison_arena, &masked, options());
    let original = project_vue(control.admitted().unwrap()).unwrap();
    let original_silent = project_vue_no_links(control.admitted().unwrap()).unwrap();
    compare(&on, &original, counts);
    compare(&off, &original_silent, counts);
    counts.styled += 1;
    counts.styles += styles;
}

#[test]
fn original_ranges_survive_leading_trailing_and_multiple_styles() {
    let cases = [
        "<style>.é { color: red }</style>\r\n<script setup>const 雪 = '🎨';</script><template><p>{{ 雪 }}</p></template>",
        "<template><p>{{ 'a&amp;b' }}</p></template><style scoped lang='css'>p { color: blue }</style>",
        "<style>/* v-bind(missing) v-bind (unused) */</style><template>text</template><style>.a::after {content:'v-bind (hidden)'} .x {--x: my-v-bind (foo)}</style>",
        "<script setup>const green = true;</script><style>.a { color: green }</style>",
    ];
    let mut counts = Counts::default();
    for source in cases {
        let before = counts.styled;
        qualify(source, Path::new("StyledChecker.vue"), &mut counts);
        assert_eq!(counts.styled, before + 1, "{source}");
    }
    assert_eq!(counts.styled, cases.len(), "{counts:?}");
    assert!(counts.styles > counts.styled && counts.links > 0 && counts.ranges > 0);
}

#[test]
fn authored_bindings_incomplete_calls_modules_and_preprocessors_stay_unsupported() {
    for style in [
        "<style>p { color: v-bind(color) }</style>",
        "<style>p { color: v-bind(unclosed }</style>",
        "<style>p { color: v-bind (missing) }</style>",
        "<style>p { color: v-bind\t\r\n(unfinished }</style>",
        "<style>p { color: v-bind\u{feff}(missing) }</style>",
        "<style module>.a{}</style>",
        "<style module='theme'>.a{}</style>",
        "<style lang='scss'>.a{}</style>",
        "<style scoped='false'>.a{}</style>",
        "<style>.a{}</style><style>p{x:v-bind(x)}</style>",
    ] {
        let source = format!("<template>text</template>{style}");
        let arena = Allocator::default();
        let observation = lower_sfc_native(&arena, &source, options());
        let owner = observation.admitted().unwrap();
        assert_eq!(
            StyledVueInput::prepare(owner).err(),
            Some(VueProjectionError::UnsupportedStyles),
            "{style}"
        );
    }
}

#[test]
fn an_external_style_or_a_style_only_document_keeps_the_original_admission() {
    for source in [
        "<template>text</template><style src='theme.css'></style>",
        "<style>p { color: red }</style>",
    ] {
        let arena = Allocator::default();
        let observation = lower_sfc_native(&arena, source, options());
        assert!(observation.admitted().is_none(), "{source}");
    }
}

#[test]
#[ignore = "requires a NUL-delimited manifest of original SFCs"]
fn original_styled_files() {
    let manifest = fs::read(std::env::var_os("VIZE_STYLED_CHECKER_INPUTS").unwrap()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut counts = Counts::default();
    let mut checker_inputs = Vec::new();
    for path in manifest.split(|&b| b == 0).filter(|p| !p.is_empty()) {
        let file = fs::read(root.join(std::str::from_utf8(path).unwrap())).unwrap();
        counts.sources += 1;
        let Ok(source) = std::str::from_utf8(&file) else {
            counts.invalid += 1;
            continue;
        };
        let previous = counts.styled;
        qualify(
            source,
            Path::new(std::str::from_utf8(path).unwrap()),
            &mut counts,
        );
        if counts.styled > previous {
            checker_inputs.push((std::str::from_utf8(path).unwrap(), source.to_owned()));
        }
    }
    let sources: Vec<_> = checker_inputs
        .iter()
        .map(|(path, source)| (*path, source.as_str()))
        .collect();
    let reports = checker::compare_programs(&sources);
    eprintln!(
        "strict checker documents={} diagnostics={}",
        reports.len(),
        reports.iter().map(Vec::len).sum::<usize>()
    );
    assert!(counts.previous > 0 && counts.styled > 0 && counts.retained_errors > 0);
    eprintln!("{counts:?}");
}
