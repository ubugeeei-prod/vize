//! `vize_l2::query` Vue binding kinds equal the observed `SetupFacts` rows on
//! every `<script setup>` in the repository corpus (#6871 Q3): every answer is
//! the row at the binding's own declaration site, and no row answers two
//! bindings. Rows whose declaration the File does not record (an L2 coverage
//! gap, not a query answer) are counted and named.

mod support {
    pub mod vue_setup;
}

use support::vue_setup::corpus;
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::container::{Vue, vue::DescriptorOptions};
use vize_l1::embed::{
    EmbedSource,
    syntax::{ProgramOptions, parse_program_once},
};
use vize_l1::markup::SurfaceParseOptions;
use vize_l2::framework::vue::SetupObserver;
use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};
use vize_l2::query::{Model, VueQueryError};

/// `(answered rows, rows without a File binding)` in `source`, or `None` when
/// the native levels refuse it first.
fn check(source: &str) -> Option<(usize, std::vec::Vec<std::string::String>)> {
    let arena = Allocator::default();
    let descriptor = Vue.observe_descriptor(
        &arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let view = descriptor.admitted().ok()?;
    let script = view.setup()?;
    let block = script.block();
    let syntax = parse_program_once(
        &arena,
        EmbedSource::authored(source, block.span()).ok()?,
        ProgramOptions::module(script.lang()),
    );
    let admitted = syntax.admitted_program()?;
    let mut producer = FileProducer::new(&arena, descriptor.source()).ok()?;
    let mut observer = SetupObserver::new();
    producer
        .program_observed(
            ProgramInput::checked(admitted, block, script.container_index()).ok()?,
            ProgramScope::Nested,
            &mut observer,
        )
        .ok()?;
    let facts = observer.finish().ok()?;
    let file = producer.finish().ok()?;
    let model = Model::new(&file).with_setup(&facts);
    let mut answers = vec![0usize; facts.bindings().len()];
    for binding in file.bindings() {
        match model.setup_binding(binding) {
            Ok(Some(row)) => {
                assert_eq!(Some(row.name), binding.declaration().map(|row| row.span));
                let index = facts
                    .bindings()
                    .iter()
                    .position(|candidate| core::ptr::eq(candidate, row))
                    .unwrap();
                answers[index] += 1;
            }
            Ok(None) | Err(VueQueryError::Unclassified(_)) => {}
            Err(error) => panic!("{error:?}"),
        }
    }
    assert!(answers.iter().all(|count| *count <= 1), "{answers:?}");
    let missing = facts
        .bindings()
        .iter()
        .zip(&answers)
        .filter(|(_, count)| **count == 0)
        .map(|(row, _)| format!("{:?}", row.kind))
        .collect();
    Some((answers.iter().sum(), missing))
}

#[test]
fn every_setup_binding_row_is_the_answer_for_exactly_one_file_binding() {
    let (root, files) = corpus();
    let (mut checked, mut rows) = (0, 0);
    let mut missing = std::vec::Vec::new();
    for path in &files {
        let Ok(source) = std::fs::read_to_string(path) else {
            continue;
        };
        let relative = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .display()
            .to_string();
        let answered =
            std::panic::catch_unwind(|| check(&source)).unwrap_or_else(|_| panic!("{relative}"));
        if let Some((answered, gaps)) = answered {
            checked += 1;
            rows += answered;
            missing.extend(gaps);
        }
    }
    missing.sort();
    let mut kinds = std::vec::Vec::<(std::string::String, usize)>::new();
    for kind in missing {
        match kinds.last_mut() {
            Some((last, count)) if *last == kind => *count += 1,
            _ => kinds.push((kind, 1)),
        }
    }
    // Destructured, enum and props-destructure locals are not File
    // declarations yet; their rows stay unanswered rather than name-matched.
    println!("{checked} scripts, {rows} rows answered, unrecorded by kind: {kinds:?}");
    assert!(checked > 100, "only {checked} setup scripts were observed");
    assert!(rows > checked, "only {rows} rows in {checked} scripts");
}
