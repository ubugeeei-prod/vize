use super::tests::options;
use super::*;
use serde_json::json;
use std::{io::Write, path::Path};
use vize_l0::{SourceBlock, SourceRoot};
use vize_l1::container::Vue;

#[derive(Default, Debug)]
struct Counts {
    files: usize,
    scripts: usize,
    refused: usize,
    bindings: usize,
    mismatches: usize,
    stamps: usize,
    lint: usize,
}

fn check_script(
    name: &str,
    arena: &Allocator,
    block: SourceBlock<'_>,
    ty: SourceType,
    owner: Option<&DescriptorObservation<'_>>,
    output: &mut impl Write,
    counts: &mut Counts,
) {
    let syntax = AuthoredProgram::parse(arena, block, ty);
    let Ok(query) = LiteralAssignments::new(&syntax) else {
        counts.refused += 1;
        return;
    };
    let rows = query.prove();
    let checked = query
        .check(&rows)
        .unwrap_or_else(|e| panic!("{name}: {e:?}"));
    let relative = |span: Span| [span.start - block.start(), span.end - block.start()];
    let mut claims = Vec::new();
    let mut originals = checked.assignments();
    visit_binding_types(&checked, |binding, diagnostic| {
        let original = originals.next().unwrap();
        assert_eq!(original.binding, binding.binding);
        let message = diagnostic.map(|d| {
            assert_eq!(
                (d.start, d.end, d.code.as_deref()),
                (binding.binding.start, binding.binding.end, Some("TS2322"))
            );
            counts.mismatches += 1;
            d.message
        });
        claims.push(json!({"binding":relative(binding.binding),"annotation":relative(original.annotation),"initializer":relative(binding.initializer),"expected":binding.declared.keyword(),"actual":binding.literal.keyword(),"message":message}));
    });
    assert!(originals.next().is_none());
    counts.bindings += claims.len();
    counts.scripts += 1;
    let input = json!({
        "name": name, "source": block.source(), "tsx": ty.is_jsx(),
        "complete": checked.is_complete(), "claims": claims
    });
    serde_json::to_writer(&mut *output, &input).unwrap();
    writeln!(output).unwrap();
    if let Some(owner) = owner {
        let mut proposed = Vec::new();
        if default_prefer_const(owner, &checked, |d| proposed.push(d)).is_ok() {
            use vize_patina::{LintPreset, Linter, checked_prefer_const::RULE};
            let expected = Linter::with_preset(LintPreset::Ecosystem)
                .with_enabled_rules(Some(vec![RULE.into()]))
                .lint_sfc(owner.source(), "Input.vue");
            let shape = |d: &LintDiagnostic| {
                (
                    d.rule_name,
                    d.start,
                    d.end,
                    d.severity,
                    d.message.clone(),
                    d.help.clone(),
                    d.labels.len(),
                    d.fix.is_some(),
                )
            };
            assert_eq!(
                proposed.iter().map(shape).collect::<Vec<_>>(),
                expected
                    .diagnostics
                    .iter()
                    .filter(|d| d.rule_name == RULE)
                    .map(shape)
                    .collect::<Vec<_>>(),
                "{name}"
            );
            counts.stamps += 1;
            counts.lint += proposed.len();
        }
    }
}
fn file(name: &str, source: &str, output: &mut impl Write, counts: &mut Counts) {
    counts.files += 1;
    let arena = Allocator::new();
    if name.ends_with(".vue") || name.ends_with(".vue.txt") {
        let owner = Vue.observe_descriptor(&arena, source, options());
        if let Ok(view) = owner.admitted() {
            for script in [view.ordinary(), view.setup()]
                .into_iter()
                .flatten()
                .filter(|s| s.lang() == Lang::Ts)
            {
                check_script(
                    name,
                    &arena,
                    script.block(),
                    SourceType::ts(),
                    Some(&owner),
                    output,
                    counts,
                );
            }
        }
    } else if ![".d.ts", ".d.cts", ".d.mts"]
        .iter()
        .any(|suffix| name.ends_with(suffix))
        && let Ok(ty) = SourceType::from_path(Path::new(name))
        && ty.is_typescript()
    {
        check_script(
            name,
            &arena,
            SourceRoot::new(source).unwrap().whole_block(),
            ty,
            None,
            output,
            counts,
        );
    }
}
#[test]
#[ignore = "writes original programs for the pinned TypeScript differential"]
fn original_typescript_and_stock_prefer_const() {
    let mut counts = Counts::default();
    let mut out = std::io::BufWriter::new(
        std::fs::File::create(std::env::var_os("VIZE_LITERAL_OUTPUT").unwrap()).unwrap(),
    );
    for path in std::fs::read(std::env::var_os("VIZE_LITERAL_INPUTS").unwrap())
        .unwrap()
        .split(|&b| b == 0)
        .filter(|p| !p.is_empty())
    {
        let path = std::str::from_utf8(path).unwrap();
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let bytes = std::fs::read(root.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"));
        if let Ok(text) = std::str::from_utf8(&bytes) {
            file(path, text, &mut out, &mut counts);
        }
    }
    eprintln!("tracked literal products {counts:?}");
    let before = counts.bindings;
    for (i, ty) in ["number", "boolean", "string"].into_iter().enumerate() {
        for (j, value) in ["0x2a", "false", "'🦀'"].into_iter().enumerate() {
            for (k, kind) in ["const", "let", "var"].into_iter().enumerate() {
                let script = format!(
                    "\r\n{kind} 雪{i}{j}{k}: {ty} = {value}; export const second: number = 4;\r\n"
                );
                file(
                    "Checked.vue",
                    &format!("<script lang=ts>{script}</script>"),
                    &mut out,
                    &mut counts,
                );
                file("Checked.tsx", &script, &mut out, &mut counts);
            }
        }
    }
    assert_eq!(counts.bindings - before, 108);
    eprintln!("with generated controls {counts:?}");
    out.flush().unwrap();
}
