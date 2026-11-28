//! Whole original source comparisons and typed refusal attribution by producer family.
use super::*;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use vize_l1_to_l2::native_file::NativeSfcIssueKind;

type Producer = for<'a> fn(&'a Allocator, &'a str, DescriptorOptions) -> NativeSfcObservation<'a>;

#[derive(Default)]
struct Counts {
    admitted: usize,
    refused: usize,
    operands: usize,
    script_bytes: usize,
    marginal: Vec<String>,
    reasons: BTreeMap<String, (usize, Vec<String>)>,
}

fn tag(debug: &str) -> &str {
    debug.split(['(', '{']).next().unwrap().trim()
}

fn refusal_reasons(owner: &NativeSfcObservation<'_>) -> BTreeSet<String> {
    let mut reasons = BTreeSet::new();
    for issue in owner.issues() {
        let reason = match issue.kind {
            NativeSfcIssueKind::VueScript(inner) | NativeSfcIssueKind::TemplateFactory(inner) => {
                format!("vue.{:?}", inner.kind)
            }
            kind => format!("native.{}", tag(&format!("{kind:?}"))),
        };
        reasons.insert(reason);
    }
    if let Some(rejected) = owner.rejected_file() {
        for issue in rejected.issues() {
            reasons.insert(format!("vue.{:?}", issue.kind));
        }
        if let Some(file) = rejected.rejected_file() {
            for issue in file.issues() {
                reasons.insert(format!("l2.{:?}", issue.kind));
            }
            for issue in file.template_issues() {
                reasons.insert(format!("template.{:?}", issue.kind));
            }
        }
    }
    if let Some(template) = owner.template() {
        for hole in template.holes() {
            reasons.insert(format!("hole.{}", tag(&format!("{:?}", hole.kind))));
        }
    }
    reasons
}

#[test]
#[ignore = "compare all tracked original products across native producer capabilities"]
fn original_template_admission_families() {
    let families: &[(&str, Producer)] = &[
        ("base", lower_sfc_native),
        (
            "listeners",
            vize_l1_to_l2::native_file::lower_sfc_with_listener_references,
        ),
        #[cfg(feature = "experimental-lexical-arrow-queries")]
        (
            "lexical",
            vize_l1_to_l2::native_file::lower_sfc_with_lexical_initializers,
        ),
    ];
    let manifest =
        std::fs::read(std::env::var_os("VIZE_TEMPLATE_PRODUCT_INPUTS").unwrap()).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut counts: Vec<Counts> = families.iter().map(|_| Counts::default()).collect();
    let (mut inputs, mut invalid) = (0, 0);
    for path in manifest.split(|&b| b == 0).filter(|path| !path.is_empty()) {
        inputs += 1;
        let name = std::str::from_utf8(path).unwrap();
        let bytes = std::fs::read(root.join(name)).unwrap();
        let Ok(source) = std::str::from_utf8(&bytes) else {
            invalid += 1;
            continue;
        };
        let mut previous = false;
        for ((family, produce), count) in families.iter().zip(&mut counts) {
            let arena = Allocator::new();
            let owner = produce(
                &arena,
                source,
                DescriptorOptions {
                    version: VueVersion::V3,
                    dialect: VueDialect::Vue,
                    template: Default::default(),
                },
            );
            let admitted = owner.admitted();
            assert!(
                !previous || admitted.is_some(),
                "{family} lost previously admitted original: {name}"
            );
            if let Some(native) = admitted {
                let descriptor = fixture_descriptor(source);
                let output = compare(&native, &descriptor, name);
                count.admitted += 1;
                count.operands += output.expressions.len();
                count.script_bytes += output
                    .script_setup
                    .as_ref()
                    .map_or(0, |script| script.content.len());
                if *family != "base" && !previous {
                    count.marginal.push(name.to_owned());
                }
                previous = true;
            } else {
                count.refused += 1;
                let reasons = refusal_reasons(&owner);
                assert!(
                    !reasons.is_empty(),
                    "every refusal needs typed source evidence: {name}"
                );
                for reason in reasons {
                    let row = count.reasons.entry(reason).or_default();
                    row.0 += 1;
                    if row.1.len() < 8 {
                        row.1.push(name.to_owned());
                    }
                }
                previous = false;
            }
        }
    }
    let mut report = BTreeMap::new();
    for ((family, _), count) in families.iter().zip(counts) {
        assert!(count.admitted > 0 && count.operands > 0 && count.script_bytes > 0);
        assert_eq!(count.admitted + count.refused + invalid, inputs);
        eprintln!(
            "native_admission family={family} inputs={inputs} invalid={invalid} admitted={} refused={} marginal={} operands={} script_bytes={}",
            count.admitted,
            count.refused,
            count.marginal.len(),
            count.operands,
            count.script_bytes
        );
        for (reason, (files, witnesses)) in &count.reasons {
            eprintln!(
                "native_refusal family={family} files={files} reason={reason} first={}",
                witnesses[0]
            );
        }
        report.insert(*family, json!({ "admitted": count.admitted, "refused": count.refused, "operands": count.operands, "scriptBytes": count.script_bytes, "newlyAdmitted": count.marginal, "reasons": count.reasons }));
    }
    if let Some(path) = std::env::var_os("VIZE_TEMPLATE_ADMISSION_REPORT") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(
                &json!({ "inputs": inputs, "invalid": invalid, "families": report }),
            )
            .unwrap(),
        )
        .unwrap();
    }
}
