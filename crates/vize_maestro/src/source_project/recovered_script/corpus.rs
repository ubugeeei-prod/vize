//! Compare the original local evidence to the tolerant TypeScript parser on
//! real edits of the admitted scripts supplied by the type and lint workflow.
use super::*;
use serde_json::{Value, json};
use std::io::{BufRead, Write};
use vize_l2::query::script::assignments::prefix::LiteralPrefixIndex;

#[derive(Default, Debug)]
struct Counts {
    originals: usize,
    retained: Coverage,
    reopened: Coverage,
}

#[derive(Default, Debug)]
struct Coverage {
    edits: usize,
    bindings: usize,
    mismatches: usize,
    empty: usize,
    directives: usize,
}

impl Coverage {
    fn open(
        &mut self,
        entry: &Value,
        source: &str,
        ty: SourceType,
        mode: &str,
        out: &mut impl Write,
    ) {
        let block = SourceRoot::new(source).unwrap().whole_block();
        match LiteralPrefixIndex::open(block, ty) {
            Ok(index) => match index.revalidate(block, ty) {
                Ok(view) => self.record(&view, entry, source, mode, out),
                Err(PrefixRefusal::DiagnosticDirective) => self.directives += 1,
                Err(e) => panic!("{e:?}"),
            },
            Err(PrefixOpenError::Syntax(ScriptRefusal::DiagnosticDirective)) => {
                self.directives += 1
            }
            Err(e) => panic!("{e:?}"),
        }
    }

    fn record(
        &mut self,
        view: &UnchangedLiterals<'_, '_>,
        entry: &Value,
        current: &str,
        mode: &str,
        output: &mut impl Write,
    ) {
        let mut claims = Vec::new();
        for row in view.assignments() {
            assert_eq!(view.binding_at(row.binding.start), Some(row));
            self.mismatches += usize::from(row.verdict == Verdict::Mismatch);
            claims.push(json!({
                "binding":[row.binding.start,row.binding.end],"annotation":[row.annotation.start,row.annotation.end],
                "initializer":[row.initializer.start,row.initializer.end],"actual":row.actual.keyword(),"expected":row.expected.keyword(),"assignable":row.verdict==Verdict::Assignable
            }));
        }
        self.edits += 1;
        self.bindings += claims.len();
        self.empty += usize::from(claims.is_empty());
        let result = json!({"name":entry["name"],"recovery":mode,"source":current,"tsx":entry["tsx"],"covered":view.covered().end,"claims":claims});
        serde_json::to_writer(&mut *output, &result).unwrap();
        writeln!(output).unwrap();
    }
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
        for suffix in [
            "\nconst __vize_incomplete: = ;\n",
            "\nconst __vize_string = 'unfinished",
            "\nfunction __vize_block() { const buried: number=1;",
        ] {
            let current = format!("{}{suffix}", &text[..at]);
            let frame = SourceRoot::new(&current).unwrap().whole_block();
            match idx.revalidate(frame, ty) {
                Ok(view) => counts
                    .retained
                    .record(&view, entry, &current, "retained", output),
                Err(PrefixRefusal::DiagnosticDirective) => counts.retained.directives += 1,
                Err(e) => panic!("{e:?}"),
            }
            counts
                .reopened
                .open(entry, &current, ty, "first-open", output);
        }
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
    assert!(counts.originals > 0);
    for sample in [&counts.retained, &counts.reopened] {
        assert!(
            sample.edits > counts.originals
                && sample.bindings > 0
                && sample.empty > 0
                && sample.mismatches > 0,
            "{counts:?}"
        );
    }
    eprintln!("recovered edit corpus {counts:?}");
    let inputs = std::fs::File::open(std::env::var_os("VIZE_FIRST_OPEN_INPUT").unwrap()).unwrap();
    let mut first = Coverage::default();
    let mut sources = 0;
    for line in std::io::BufReader::new(inputs).lines() {
        let entry: Value = serde_json::from_str(&line.unwrap()).unwrap();
        let ty = if entry["tsx"] == true {
            SourceType::tsx()
        } else {
            SourceType::ts()
        };
        first.open(
            &entry,
            entry["source"].as_str().unwrap(),
            ty,
            "upstream-original",
            &mut out,
        );
        sources += 1;
    }
    assert_eq!(first.edits + first.directives, sources);
    assert!(first.edits > 0);
    eprintln!("unaltered damaged TypeScript sources {sources}: {first:?}");
    out.flush().unwrap();
}
