//! Complete output and ordered diagnostic parity for one authenticated original.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
#[path = "official_diagnosed.rs"]
mod official;
use vize_atelier_core::CompilerError;
use vize_l0::{
    Allocator, Span,
    diag::{Diagnostic, Stage},
    pass::BudgetObserver,
};
use vize_l1_to_l2::{DomEmitOptions, LegacyCaps, emit_dom_with_options, lower_with_caps};

const ORIGINAL: &str = include_str!(concat!(
    "../../../../tests/_fixtures/differential/compiler/n8n-if-key-regression/",
    "InstanceAiConfirmationPanel.vue.txt"
));
const EXPECTED: &str = include_str!(
    "../../../../tests/_fixtures/differential/compiler/n8n-if-key-regression/diagnostics.json"
);
const PROVENANCE: &str = include_str!(
    "../../../../tests/_fixtures/differential/compiler/n8n-if-key-regression/provenance.json"
);
const OFFICIAL: &str = include_str!(
    "../../../../tests/_fixtures/differential/compiler/n8n-if-key-regression/official/record.json"
);

/// `Some(Ok)` earns comparison credit only after full code and every diagnostic
/// match. Ordinary product admission remains strict; this is a test-only oracle.
pub(super) fn compare(
    source: &str,
    template: &str,
    legacy_errors: &[CompilerError],
    legacy_module: &str,
) -> Option<Result<(), String>> {
    if source != ORIGINAL {
        return None;
    }
    Some(compare_authenticated(
        template,
        legacy_errors,
        legacy_module,
    ))
}

fn compare_authenticated(
    template: &str,
    legacy_errors: &[CompilerError],
    legacy_module: &str,
) -> Result<(), String> {
    let expected: Vec<Value> = serde_json::from_str(EXPECTED).map_err(|e| e.to_string())?;
    if expected.len() != 3 {
        return Err("exact three independently pinned diagnostics required".into());
    }
    let provenance: Value = serde_json::from_str(PROVENANCE).map_err(|e| e.to_string())?;
    let official: Value = serde_json::from_str(OFFICIAL).map_err(|e| e.to_string())?;
    let sha = format!("{:x}", Sha256::digest(ORIGINAL.as_bytes()));
    if provenance["sha256"] != sha || official["record"]["sha256"] != sha {
        return Err("original compiler oracle hash mismatch".into());
    }
    official::authenticate(template, &provenance, &official, &expected)?;
    let canonical = expected
        .iter()
        .map(|e| {
            json!({"code":e["code"],
        "message":e["message"],"span":e["span"]})
        })
        .collect::<Vec<_>>();
    let legacy = legacy_errors
        .iter()
        .map(|e| {
            json!({"code":format!("{:?}",e.code),
        "message":e.message,"span":e.loc.as_ref().map(|loc|
            json!({"start":loc.span.start,"end":loc.span.end}))})
        })
        .collect::<Vec<_>>();
    if legacy != canonical {
        return Err(format!(
            "complete ordered legacy diagnostics differ: expected={canonical:?} actual={legacy:?}"
        ));
    }
    for value in &expected {
        let loc = &value["officialLocation"];
        let start = official::byte_offset(
            template,
            loc["start"]["offset"].as_u64().ok_or("start offset")? as usize,
        )?;
        let end = official::byte_offset(
            template,
            loc["end"]["offset"].as_u64().ok_or("end offset")? as usize,
        )?;
        if value["officialCode"] != 29
            || value["span"] != json!({"start":start,"end":end})
            || template.get(start..end) != loc["source"].as_str()
        {
            return Err("original diagnostic code/source coordinates differ from official".into());
        }
        for (side, offset) in [("start", start), ("end", end)] {
            let prefix = template.get(..offset).ok_or("original diagnostic offset")?;
            let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
            let column = prefix
                .rsplit('\n')
                .next()
                .unwrap_or_default()
                .encode_utf16()
                .count()
                + 1;
            if loc[side]["line"] != line || loc[side]["column"] != column {
                return Err(
                    "original diagnostic line/column differs from no-map official oracle".into(),
                );
            }
        }
    }
    let allocator = Allocator::new();
    let (tree, errors) = vize_l1::parse(&allocator, template);
    let mut lowered = lower_with_caps(&allocator, &tree, &errors, LegacyCaps::VUE3);
    let facts = vize_l1_to_l2::pass::run_dom_transform_with_profile(
        &mut lowered,
        &mut BudgetObserver::new(),
        vize_l1_to_l2::pass::TransformProfile::DEFAULT,
    );
    let native = lowered.diagnostics.iter().map(|e| json!({
        "code": if e.message == vize_l1_to_l2::lower::SAME_KEY_MESSAGE {"VIfSameKey"} else {"unexpected"},
        "message":e.message,"span":{"start":e.span.start,"end":e.span.end}
    })).collect::<Vec<_>>();
    let expected_native = expected
        .iter()
        .map(|e| {
            let start = e["span"]["start"].as_u64().ok_or("expected native start")? as u32;
            let end = e["span"]["end"].as_u64().ok_or("expected native end")? as u32;
            Ok(Diagnostic::legacy_error(
                &vize_l1_to_l2::exemptions::LOWERING,
                Stage::Semantic,
                Span::new(start, end),
                e["message"].as_str().ok_or("expected message")?,
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    if native != canonical || lowered.diagnostics != expected_native {
        return Err(format!(
            "complete ordered native diagnostics differ: expected={expected_native:#?} actual={:#?}",
            lowered.diagnostics
        ));
    }
    if emit_dom_with_options(&lowered, &facts, &DomEmitOptions::DEFAULT).is_ok() {
        return Err("ordinary diagnosed product input must still reject".into());
    }
    // Retain the diagnosed artifact's real code in test space. The complete
    // diagnostic vector has already been independently qualified above.
    let retained = std::mem::take(&mut lowered.diagnostics);
    let native = emit_dom_with_options(&lowered, &facts, &DomEmitOptions::DEFAULT)
        .map_err(|e| format!("complete diagnosed emission refused: {e:?}"))?
        .assembled();
    lowered.diagnostics = retained;
    if native != legacy_module {
        return Err(format!(
            "complete diagnosed modules differ:\nlegacy={legacy_module}\nnative={native}"
        ));
    }
    Ok(())
}
