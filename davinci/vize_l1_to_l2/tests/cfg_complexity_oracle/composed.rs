//! Cross-check the physical shared walk against the two product analyses and
//! the independent page-order visitor. The parent runner also builds the naive CFG.

mod profile;

use std::path::{Path, PathBuf};

use vize_l0::Span;
use vize_l0::pass::{Remark, RemarkKind, RemarkSink, observer::RemarkArgValue};
use vize_l1_to_l2::Lowered;
use vize_l2::analysis::{static_facts, template, template_complexity::ComplexityFacts};
use vize_l2::walk::{NodeRef, PageWalk, visit_nodes};

type Row = (
    RemarkKind,
    &'static str,
    Span,
    Vec<(&'static str, RemarkArgValue)>,
);

#[derive(Default)]
struct Rows(Vec<Row>);

impl RemarkSink for Rows {
    const ENABLED: bool = true;

    fn emit(&mut self, remark: &Remark<'_>) {
        let args = remark
            .args
            .iter()
            .map(|arg| (arg.key, arg.value.into()))
            .collect();
        self.0.push((remark.kind, remark.name, remark.span, args));
    }
}

pub(super) fn check(lowered: &Lowered<'_>, complexity: &ComplexityFacts) {
    let root = &lowered.artifact.root;
    let total = lowered.op_count;
    let mut original_trace = lowered.artifact.provenance.clone();
    let mut shared_trace = original_trace.clone();
    let (mut original_remarks, mut shared_remarks) = (Rows::default(), Rows::default());
    let expected =
        static_facts::run_remarked(root, total, &mut original_trace, &mut original_remarks);
    let joint = template::run(root, total, &mut shared_trace, &mut shared_remarks);
    assert_eq!(
        expected.sorted_entries(),
        joint.static_facts.sorted_entries()
    );
    assert_eq!(complexity, &joint.complexity);
    assert_eq!(original_trace, shared_trace);
    assert_eq!(original_remarks.0, shared_remarks.0);
    let mut walk = PageWalk::new();
    let mut visits = 0;
    visit_nodes(&mut walk, &root.ops, &mut |_, node| {
        visits += u32::from(matches!(node, NodeRef::Op(_)));
    })
    .expect("bounded fixture");
    assert_eq!(walk.minted(), total);
    assert_eq!(visits, joint.op_visits);
}

#[test]
fn root_text_and_comments_keep_the_next_owner_ids_and_exact_observer_order() {
    for template in [
        "",
        "text",
        "text<!--a-->tail",
        "text<!--a--><i id='x'>child</i><!--z-->",
        "<!--before--><Card :p='q'/><b v-if='valid'>ok</b>",
    ] {
        let (production, independent) = super::runner::both_template(template);
        assert_eq!(production, independent, "{template}");
    }
}

#[test]
#[ignore = "NUL-delimited list of original Vue files"]
fn original_templates() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let data = std::fs::read_to_string(
        std::env::var_os("VIZE_COMPOSED_TEMPLATE_FILES").expect("file list"),
    )
    .expect("file list");
    let files: Vec<PathBuf> = data
        .split_terminator('\0')
        .map(|path| directory.join(path))
        .collect();
    let report = super::runner::run_files(&files);
    eprintln!("{}", report.scope_line("paired"));
    assert_eq!(report.files as usize, files.len());
    assert_eq!(report.unreadable, 0);
    assert!(report.templates > 0);
    // The composed and independent Davinci results are asserted for every admitted template.
    // The archived naive CFG oracle is also run, with its own aggregate verdict.
    report.verdict("paired").expect("naive oracle");
}
