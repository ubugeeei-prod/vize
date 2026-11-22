//! Compare the original local evidence to the tolerant TypeScript parser on
//! real edits of the admitted scripts supplied by the type and lint workflow.
use super::*;
use serde_json::{Value, json};
use std::io::{BufRead, Write};
use vize_l2::query::script::assignments::prefix::LiteralPrefixIndex;

#[derive(Default, Debug)]
struct Counts {
    originals: usize,
    edits: usize,
    bindings: usize,
    empty: usize,
    directives: usize,
}

fn examine(entry: &Value, output: &mut impl Write, counts: &mut Counts) {
    if entry["claims"].as_array().unwrap().is_empty() {
        return;
    }
    let text = entry["source"].as_str().unwrap();
    let ty = if entry["tsx"] == true {
        SourceType::tsx()
    } else {
        SourceType::ts()
    };
    let block = SourceRoot::new(text).unwrap().whole_block();
    let arena = Allocator::new();
    let old = AuthoredProgram::parse(&arena, block, ty);
    let q = LiteralAssignments::new(&old).unwrap();
    let rows = q.prove();
    let checked = q.check(&rows).unwrap();
    let idx = LiteralPrefixIndex::collect(block, &checked).unwrap();
    counts.originals += 1;
    let mut offsets = vec![0, text.len()];
    for row in checked.assignments() {
        offsets.push(row.binding.start as usize);
        offsets.push(row.initializer.end as usize);
        // Cut just after a complete statement, preserving an explicit semicolon.
        if text.as_bytes().get(row.initializer.end as usize) == Some(&b';') {
            offsets.push(row.initializer.end as usize + 1);
        }
    }
    offsets.sort_unstable();
    offsets.dedup();
    for at in offsets {
        let current = format!("{}\nconst __vize_incomplete: = ;\n", &text[..at]);
        let frame = SourceRoot::new(&current).unwrap().whole_block();
        let view = match idx.revalidate(frame, ty) {
            Ok(view) => view,
            Err(PrefixRefusal::DiagnosticDirective) => {
                counts.directives += 1;
                continue;
            }
            Err(e) => panic!("{e:?}"),
        };
        let mut evidence = Vec::new();
        for row in view.assignments() {
            assert_eq!(view.binding_at(row.binding.start), Some(row));
            evidence.push(json!({
                "binding":[row.binding.start,row.binding.end],"annotation":[row.annotation.start,row.annotation.end],
                "initializer":[row.initializer.start,row.initializer.end],"actual":row.actual.keyword(),"expected":row.expected.keyword(),"assignable":row.verdict==Verdict::Assignable
            }));
        }
        counts.edits += 1;
        counts.bindings += evidence.len();
        counts.empty += usize::from(evidence.is_empty());
        let result = json!({"name":entry["name"],"source":current,"tsx":entry["tsx"],"covered":view.covered().end,"claims":evidence});
        serde_json::to_writer(&mut *output, &result).unwrap();
        writeln!(output).unwrap();
    }
}
#[test]
#[ignore = "requires the actual TypeScript source corpus and a reference-checker output path"]
fn changed_suffixes_on_original_programs() {
    let originals = std::fs::File::open(std::env::var_os("VIZE_PREFIX_INPUT").unwrap()).unwrap();
    let output = std::fs::File::create(std::env::var_os("VIZE_PREFIX_OUTPUT").unwrap()).unwrap();
    let mut out = std::io::BufWriter::new(output);
    let mut counts = Counts::default();
    for line in std::io::BufReader::new(originals).lines() {
        examine(
            &serde_json::from_str(&line.unwrap()).unwrap(),
            &mut out,
            &mut counts,
        );
    }
    out.flush().unwrap();
    assert!(
        counts.originals > 0
            && counts.edits > counts.originals
            && counts.bindings > 0
            && counts.empty > 0,
        "{counts:?}"
    );
    eprintln!("recovered edit corpus {counts:?}");
}
