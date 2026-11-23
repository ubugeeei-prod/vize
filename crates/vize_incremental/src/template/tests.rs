#![expect(
    clippy::unwrap_used,
    reason = "directed original frames and corpus paths"
)]
use super::*;
use std::{fs, path::Path, sync::Arc};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
    pass::{NoObserver, PassEvent, PassObserver},
};
use vize_l1::container::{Vue, vue::DescriptorOptions};
use vize_l1_to_l2::pass::template::OriginalTemplate;
type Counts = (usize, usize, usize);

#[derive(Default)]
struct Walks(usize);
impl PassObserver for Walks {
    fn before_pass(&mut self, event: &PassEvent<'_>) {
        self.0 += usize::from(event.is_group_entry());
    }
}
fn compare(
    document: &mut TemplateDocument,
    snapshot: &Arc<String>,
    observer: &mut impl PassObserver,
) -> Option<Counts> {
    let source = snapshot.as_str();
    let arena = Allocator::default();
    let descriptor = Vue.observe_descriptor(
        &arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: Default::default(),
        },
    );
    let expected = descriptor.admitted_template();
    let actual = document.read(snapshot, observer).map(|view| {
        view.map(|read| {
            let selected = expected.as_ref().unwrap().unwrap();
            let expected = OriginalTemplate::analyze(&arena, selected, &mut NoObserver).unwrap();
            assert_eq!(read.block().span(), expected.block().span());
            assert_eq!(read.block().root_source(), source);
            assert_eq!(read.block().source(), expected.block().source());
            assert_eq!(read.opening(), expected.open_tag());
            assert_eq!(
                read.relative().static_facts.sorted_entries(),
                expected.analysis().static_facts.sorted_entries()
            );
            assert_eq!(read.relative().op_visits, expected.analysis().op_visits);
            let rows = read
                .authored_contributions()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            assert_eq!(rows, expected.analysis().complexity.contributions);
            let report = read.report().unwrap();
            assert_eq!(report.opening, expected.open_tag());
            assert_eq!(report.content, expected.block().span());
            assert_eq!(report.complexity, expected.analysis().complexity);
            assert_eq!(report.diagnostics, expected.l2ir().diagnostics);
            (
                rows.len(),
                expected.l2ir().diagnostics.len(),
                expected.analysis().static_facts.len(),
            )
        })
    });
    match expected {
        Err(refusal) => {
            let Err(TemplateReadError::Descriptor(e)) = actual else {
                panic!("expected descriptor refusal")
            };
            assert_eq!(e.issues.as_ref(), refusal.issues());
            assert_eq!(e.errors.as_ref(), refusal.errors());
            None
        }
        Ok(None) => {
            assert!(actual.unwrap().is_none());
            None
        }
        Ok(Some(_)) => Some(actual.unwrap().unwrap()),
    }
}

#[test]
fn changes_to_source_frames_reuse_the_pass_and_changes_to_the_body_recompute_it() {
    let mut document = TemplateDocument::default();
    let first = "<template><p v-if='a && b'>雪</p></template>";
    let shifted = "<script lang='coffee'>x = -> 1</script>\r\n<!--🧭--><template><p v-if='a && b'>雪</p></template>";
    let same_frame = "<script lang='coffee'>x = -> 2</script>\r\n<!--🧭--><template><p v-if='a && b'>雪</p></template>";
    for (source, runs, admitted) in [
        (first, 1, true),
        (first, 0, true),
        (shifted, 0, true),
        (same_frame, 0, true),
        ("<template><p :x=>雪</p></template>", 1, true),
        ("<template lang='pug'>p Hello</template>", 0, false),
        ("<template>x</template><template>y</template>", 0, false),
        ("<style src='else.css'></style>", 0, false),
        (first, 1, true),
    ] {
        let mut walks = Walks::default();
        let answer = compare(&mut document, &Arc::new(source.to_owned()), &mut walks);
        assert_eq!(answer.is_some(), admitted);
        assert_eq!(walks.0, runs, "{source}");
    }
    let first = Arc::new(first.to_owned());
    let read = document.read(&first, &mut NoObserver).unwrap().unwrap();
    {
        let byte = u32::try_from(read.block().source().find('雪').unwrap()).unwrap();
        assert_eq!(read.authored_span(Span::new(byte + 1, byte + 2)), None);
        assert_eq!(read.authored_span(Span::new(3, 2)), None);
        assert_eq!(read.authored_span(Span::new(u32::MAX, u32::MAX)), None);
        assert_eq!(read.authored_span(Span::new(0, 0)), Some(Span::new(10, 10)));
    }
}

#[test]
fn source_owners_preserve_identity_and_release_edited_text() {
    let mut document = TemplateDocument::default();
    let source = Arc::new("<template><p v-if='a'>雪</p></template>".to_owned());
    let previous = Arc::downgrade(&source);
    let mut visits = Walks::default();
    for snapshot in [&source, &Arc::clone(&source), &Arc::new((*source).clone())] {
        let view = document.read(snapshot, &mut visits).unwrap().unwrap();
        assert_eq!(view.relative().complexity.cyclomatic, 2);
        assert_eq!(visits.0, 1);
    }
    drop(source);
    assert_eq!(previous.strong_count(), 0);
    let changed = Arc::new("<template><span/></template>".to_owned());
    let view = document.read(&changed, &mut visits).unwrap().unwrap();
    assert_eq!(view.relative().complexity.cyclomatic, 1);
    assert_eq!(visits.0, 2);
    let current = Arc::downgrade(&changed);
    drop(changed);
    assert_eq!(current.strong_count(), 0);
}

#[test]
fn diagnostic_labels_and_each_proof_link_follow_the_current_body() {
    use vize_l0::diag::{DiagnosticPart, PartKind, Stage, WitnessChain, WitnessKey, WitnessLink};
    use vize_l0::pass::AnalysisId;

    let diagnostic = |offset, last| {
        let span = |start, end| Span::new(offset + start, offset + end);
        let first = WitnessLink::new(AnalysisId::new(2), span(0, 3), WitnessKey::Index(9));
        let last = WitnessLink::new(
            AnalysisId::new(7),
            span(last, 4),
            WitnessKey::Name("x".into()),
        );
        Diagnostic::proven(
            Stage::Semantic,
            span(3, 4),
            "check",
            WitnessChain::new(first).then(last),
        )
        .with_part(DiagnosticPart::new(
            PartKind::Secondary,
            span(0, 3),
            "origin",
        ))
    };
    let mut document = TemplateDocument::default();
    for prefix in ["", "<!--🧭-->\r\n"] {
        let source = Arc::new(format!("{prefix}<template>雪x</template>"));
        let read = document.read(&source, &mut NoObserver).unwrap().unwrap();
        let project = |span| read.authored_span(span).ok_or(span);
        let base = u32::try_from(prefix.len() + "<template>".len()).unwrap();
        assert_eq!(
            diagnostic(0, 3).try_map_spans(project),
            Ok(diagnostic(base, 3))
        );
        assert_eq!(
            diagnostic(0, 1).try_map_spans(project),
            Err(Span::new(1, 4))
        );
    }
}

#[test]
#[ignore = "compares all source frames in a tracked or upstream input manifest"]
fn original_files_match_clean_semantic_rows_and_complete_diagnostics() {
    let paths = fs::read(std::env::var_os("VIZE_TEMPLATE_PASS_INPUTS").unwrap()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut document = TemplateDocument::default();
    let (mut inputs, mut invalid, mut accepted, mut rows, mut diagnostics, mut owners) =
        (0, 0, 0, 0, 0, 0);
    for path in paths.split(|&b| b == 0).filter(|p| !p.is_empty()) {
        let path = std::str::from_utf8(path).unwrap();
        let bytes = fs::read(root.join(path)).unwrap();
        let Ok(source) = std::str::from_utf8(&bytes) else {
            invalid += 1;
            continue;
        };
        if let Some((n, d, o)) =
            compare(&mut document, &Arc::new(source.to_owned()), &mut NoObserver)
        {
            accepted += 1;
            rows += n;
            diagnostics += d;
            owners += o;
        }
        inputs += 1;
    }
    assert!(accepted > 0 && rows > 0 && owners > 0);
    println!(
        "resident_templates utf8={inputs} non_utf8={invalid} admitted={accepted} decisions={rows} diagnostics={diagnostics} owners={owners}"
    );
}
