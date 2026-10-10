//! Complete observation and law helpers for the serialized legacy-slot judge.
mod structure;

use serde_json::{Value, json};
use vize_armature::legacy::LegacyDialectCapabilities;
use vize_atelier_core::{
    CodegenMode, CodegenOptions, CompilerError, TransformOptions, codegen, lane, parser,
};
use vize_l0::{Allocator, config::VueVersion, profiler::global_profiler};
use vize_relief::{ExpressionNode, PropNode, RootNode, TemplateChildNode};

fn window<R>(action: impl FnOnce() -> R) -> (R, Value) {
    let profiler = global_profiler();
    profiler.clear();
    profiler.enable();
    let before = vize_atelier_core::expr_parse_probe::expr_parse_count();
    let result = action();
    profiler.disable();
    let summary = profiler.counter_summary();
    let entries: Vec<_> = summary
        .entries
        .iter()
        .map(|entry| {
            json!({
                "name": entry.name, "samples": entry.samples, "total": entry.total,
                "average": entry.average, "min": entry.min, "max": entry.max
            })
        })
        .collect();
    (
        result,
        json!({"entries": entries, "attempts": summary.total("davinci.expr.parses"),
        "oldExprReparseDelta": vize_atelier_core::expr_parse_probe::expr_parse_count() - before}),
    )
}

fn errors(errors: &[CompilerError]) -> Value {
    json!({"debug": format!("{errors:#?}"), "items": errors.iter().map(|error| json!({
        "code": format!("{:?}", error.code), "message": error.message.as_str(), "loc": error.loc
    })).collect::<Vec<_>>()})
}

fn expression(expression: &ExpressionNode<'_>) -> Value {
    let ExpressionNode::Simple(node) = expression else {
        return json!({"compoundDebug": format!("{expression:#?}")});
    };
    let mut result = json!({"content": node.content, "contentPointer": node.content.as_ptr() as usize,
        "loc": node.loc, "role": "absent", "raw": null, "astPointer": null});
    if let Some(retained) = node.js_ast {
        result["raw"] = json!(retained.raw);
        result["rawPointer"] = json!(retained.raw.as_ptr() as usize);
        result["astPointer"] = json!(core::ptr::from_ref(retained.ast) as usize);
        result["retainedDebug"] = json!(format!("{:?}", retained.ast));
        match retained.as_slot_parameters() {
            Some(Ok(parameters)) => {
                result["role"] = json!("ok");
                result["parametersDebug"] = json!(format!("{parameters:#?}"));
            }
            Some(Err(diagnostics)) => {
                result["role"] = json!("error");
                result["diagnosticCount"] = json!(diagnostics.len());
                result["diagnosticsDebug"] = json!(format!("{diagnostics:#?}"));
            }
            None => result["role"] = json!("expression"),
        }
        result["ordinaryViewPresent"] = json!(retained.as_expression().is_some());
    }
    result
}

fn visit(
    children: &[TemplateChildNode<'_>],
    source: &str,
    slots: &mut Vec<Value>,
    attrs: &mut Vec<Value>,
    ordinary: &mut Vec<Value>,
) {
    for child in children {
        match child {
            TemplateChildNode::Element(element) => {
                for prop in &element.props {
                    match prop {
                        PropNode::Attribute(attr) => attrs.push(json!({"name": attr.name, "loc": attr.loc,
                            "value": attr.value.as_ref().map(|value| json!({"content": value.content,
                                "pointer": value.content.as_ptr() as usize, "loc": value.loc}))})),
                        PropNode::Directive(dir) if dir.name == "slot" => {
                            let mut slot = dir.exp.as_ref().map(expression).unwrap_or_else(|| json!({"role": "none"}));
                            slot["directiveLoc"] = json!(dir.loc);
                            slot["declarations"] = json!(vize_atelier_core::steps::v_slot::get_slot_prop_names(dir, source)
                                .iter().map(|name| name.as_str()).collect::<Vec<_>>());
                            slots.push(slot);
                        }
                        PropNode::Directive(dir) => {
                            if let Some(exp) = &dir.exp { ordinary.push(expression(exp)); }
                        }
                    }
                }
                visit(&element.children, source, slots, attrs, ordinary);
            }
            TemplateChildNode::Interpolation(node) => ordinary.push(expression(&node.content)),
            _ => {}
        }
    }
}

fn snapshot(root: &RootNode<'_>) -> Value {
    let (mut slots, mut attrs, mut ordinary) = (Vec::new(), Vec::new(), Vec::new());
    visit(
        &root.children,
        root.source,
        &mut slots,
        &mut attrs,
        &mut ordinary,
    );
    json!({"debug": format!("{root:#?}"), "slots": slots, "attributes": attrs, "ordinary": ordinary})
}

pub(crate) fn compile(
    case: &Value,
    variant: &str,
    dialect: VueVersion,
    mode: CodegenMode,
) -> Value {
    let source = case[variant].as_str().expect("authored whole source");
    let allocator = Allocator::new();
    let ((mut root, parse_errors), parse_counters) = window(|| parser::parse(&allocator, source));
    let original = snapshot(&root);
    let caps = LegacyDialectCapabilities::for_dialect(dialect);
    let (_, first_counters) = window(|| {
        vize_atelier_core::steps::legacy::desugar_legacy_template(&allocator, &mut root, caps)
    });
    let first = snapshot(&root);
    let (_, second_counters) = window(|| {
        vize_atelier_core::steps::legacy::desugar_legacy_template(&allocator, &mut root, caps)
    });
    let second = snapshot(&root);
    let prefix = mode == CodegenMode::Module;
    let is_ts = case["isTs"].as_bool().expect("authored TS policy");
    let transform_options = TransformOptions {
        prefix_identifiers: prefix,
        dialect,
        is_ts,
        ..Default::default()
    };
    let (transform_errors, transform_counters) =
        window(|| lane::transform(&allocator, &mut root, transform_options, None));
    let transformed = format!("{root:#?}");
    let codegen_options = CodegenOptions {
        mode,
        prefix_identifiers: prefix,
        is_ts,
        ..Default::default()
    };
    let (generated, codegen_counters) = window(|| codegen::generate(&root, codegen_options));
    // Independent observer is outside every product parse/counter window.
    let module_structure = (mode == CodegenMode::Module)
        .then(|| structure::observe(&format!("{}\n{}", generated.preamble, generated.code)));
    json!({"id": case["id"], "variant": variant, "dialect": format!("{dialect:?}"), "mode": format!("{mode:?}"),
        "source": source, "parse": {"root": original, "errors": errors(&parse_errors), "counters": parse_counters},
        "first": {"root": first, "counters": first_counters}, "second": {"root": second, "counters": second_counters},
        "transform": {"rootDebug": transformed, "errors": errors(&transform_errors), "counters": transform_counters},
        "generated": {"code": generated.code.as_str(), "preamble": generated.preamble.as_str(), "map": generated.map.as_ref().map(|map| map.as_str())},
        "codegenCounters": codegen_counters, "moduleStructure": module_structure})
}

pub(crate) fn judge(case: &Value, packet: &Value, structures: &Value) {
    let id = case["id"].as_str().expect("case ID");
    let eq = |a: &Value, b: &Value, law: &str| assert_eq!(a, b, "{id}: {law}");
    let parse = &packet["parse"];
    let original = &parse["root"];
    let first = &packet["first"];
    let root = &first["root"];
    let second = &packet["second"];
    let variant = packet["variant"].as_str().expect("variant");
    let v3 = packet["dialect"] == "V3";
    let legacy = variant == "legacy";
    let converts = legacy && !v3 && case["kind"] == "legacy";
    let attempts = if converts {
        case["conversionAttempts"].as_u64().expect("count")
    } else {
        0
    };
    eq(
        &first["counters"]["attempts"],
        &json!(attempts),
        "first admission",
    );
    let counter = first["counters"]["entries"]
        .as_array()
        .expect("counters")
        .iter()
        .find(|entry| entry["name"] == "davinci.expr.parses");
    if attempts == 0 {
        assert!(counter.is_none(), "{id}: no parameter goal entered");
    } else {
        let counter = counter.expect("original parameter goal");
        for field in ["samples", "total", "min", "max"] {
            eq(&counter[field], &json!(1), "one goal");
        }
    }
    for stage in [first, second] {
        eq(
            &stage["counters"]["oldExprReparseDelta"],
            &json!(0),
            "no wrapper parse",
        );
    }
    eq(
        &second["counters"]["attempts"],
        &json!(0),
        "idempotent admission",
    );
    eq(root, &second["root"], "whole converted tree identity");
    eq(
        &parse["errors"]["items"],
        &json!([]),
        "valid authored template",
    );
    eq(
        &original["ordinary"],
        &root["ordinary"],
        "ordinary carrier identity",
    );
    if v3 || case["kind"] == "mixed-modern" {
        eq(original, root, "inert dialect/mixed gate");
    }
    if legacy && case["kind"] == "legacy" {
        eq(&original["slots"], &json!([]), "no attribute AST storage");
    }
    if legacy && v3 && case["kind"] != "mixed-modern" {
        return;
    }
    let slots = root["slots"].as_array().expect("slots");
    if case["kind"] == "v3-only" {
        assert!(slots.is_empty(), "{id}: native scope");
        return;
    }
    assert_eq!(slots.len(), 1, "{id}: one slot");
    let slot = &slots[0];
    eq(&slot["role"], &case["role"], "retained role");
    eq(
        &slot["declarations"],
        &case["declarations"],
        "authored declarations",
    );
    if case["role"] == "none" {
        return;
    }
    eq(
        &slot["ordinaryViewPresent"],
        &json!(false),
        "no fake expression",
    );
    eq(&slot["raw"], &case["decodedHeader"], "frozen decoded raw");
    eq(
        &slot["loc"]["span"],
        &case["headerSpans"][variant],
        "physical header span",
    );
    if converts {
        eq(
            &slot["rawPointer"],
            &slot["contentPointer"],
            "no wrapper source copy",
        );
        let attr = original["attributes"]
            .as_array()
            .expect("attributes")
            .iter()
            .find(|attr| attr["name"] == "slot-scope" || attr["name"] == "scope")
            .expect("legacy value");
        eq(
            &slot["rawPointer"],
            &attr["value"]["pointer"],
            "original value identity",
        );
    } else {
        eq(
            &original["slots"][0]["rawPointer"],
            &slot["rawPointer"],
            "modern raw identity",
        );
        eq(
            &original["slots"][0]["astPointer"],
            &slot["astPointer"],
            "modern identity",
        );
    }
    if case["role"] == "error" {
        let count = slot["diagnosticCount"].as_u64().expect("diagnostics");
        if let Some(expected) = case["errorCount"].as_u64() {
            assert_eq!(count, expected, "{id}: error custody");
        } else {
            assert!(count > 0, "{id}: malformed errors retained");
        }
    } else {
        eq(
            &packet["transform"]["errors"]["items"],
            &json!([]),
            "valid transform",
        );
    }
    if !v3 && packet["mode"] == "Module" && case["role"] == "ok" {
        let expected = structures["cases"]
            .as_array()
            .expect("whole structures")
            .iter()
            .find(|expected| expected["id"] == case["id"])
            .expect("authored whole structure");
        eq(
            &packet["moduleStructure"]["parse"],
            &expected["expectedModuleParse"],
            "whole module parser",
        );
        eq(
            &packet["moduleStructure"]["structure"],
            &expected["expectedWholeStructure"],
            "complete ordered lexical structure",
        );
    }
}
