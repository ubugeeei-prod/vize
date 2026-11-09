use super::super::super::InferredCall;
use super::super::super::TemplateRefKeyChecks;
use crate::virtual_ts::generator::script_module::collect_script_module_plan;
use crate::virtual_ts::{VizeMapping, VizeSubSpan};
use std::borrow::Cow;
use vize_carton::{String, cstr};

#[test]
fn inserted_comma_has_no_authored_subspan_and_original_bytes_stay_exact() {
    let source = "const 身分 = <名 /* , > */>(value: 名): 名 => value;";
    let plan = collect_script_module_plan(source, false);
    assert_eq!(plan.tsx_arrows.len(), 1);
    let position = plan.tsx_arrows[0].position;
    let checks = TemplateRefKeyChecks {
        calls: &[],
        helper: "",
        next: 0,
        tsx_arrows: &plan.tsx_arrows,
    };
    let mut code = crate::virtual_ts::TextBuf::from_text("header\n");
    let mut mappings = Vec::new();
    checks.emit_line(
        &mut code,
        &mut mappings,
        Cow::Borrowed(source),
        (0, source.len()),
        &|n| 300 + n,
    );
    let expected = cstr!("header\n{},{}\n", &source[..position], &source[position..]);
    assert_eq!(code, expected);
    assert_eq!(
        mappings,
        vec![VizeMapping {
            gen_range: 7..7 + source.len() + 1,
            src_range: 300..300 + source.len(),
            sub_spans: vec![
                VizeSubSpan {
                    gen_range: 7..7 + position,
                    src_range: 300..300 + position
                },
                VizeSubSpan {
                    gen_range: 8 + position..8 + source.len(),
                    src_range: 300 + position..300 + source.len()
                },
            ],
        }]
    );
}

#[test]
fn template_ref_callee_and_arrow_share_original_coordinates() {
    let source = "const item = useTemplateRef('x'); const identity = <T>(value: T): T => value;";
    let plan = collect_script_module_plan(source, false);
    assert_eq!(plan.tsx_arrows.len(), 1);
    let position = plan.tsx_arrows[0].position;
    let start = source.find("useTemplateRef").unwrap();
    let end = start + "useTemplateRef".len();
    for imported in [true, false] {
        let calls = [InferredCall {
            callee: start..end,
            name: "useTemplateRef".into(),
            imported,
            statement_start: 0,
            literal: None,
        }];
        let helper = "__privateRef";
        let checks = TemplateRefKeyChecks {
            calls: &calls,
            helper,
            next: 0,
            tsx_arrows: &plan.tsx_arrows,
        };
        let mut code = crate::virtual_ts::TextBuf::default();
        let mut mappings = Vec::new();
        checks.emit_line(
            &mut code,
            &mut mappings,
            Cow::Borrowed(source),
            (0, source.len()),
            &|n| 500 + n,
        );
        let replacement = if imported {
            String::from(helper)
        } else {
            cstr!("(useTemplateRef as {helper}Global)")
        };
        let expected = cstr!(
            "{}{}{},{}\n",
            &source[..start],
            replacement,
            &source[end..position],
            &source[position..]
        );
        assert_eq!(code, expected);
        let after_call = start + replacement.len();
        let comma = after_call + position - end;
        let callee_start = start + usize::from(!imported);
        let callee_len = if imported {
            helper.len()
        } else {
            "useTemplateRef".len()
        };
        assert_eq!(
            mappings,
            vec![VizeMapping {
                gen_range: 0..source.len() + replacement.len() - (end - start) + 1,
                src_range: 500..500 + source.len(),
                sub_spans: vec![
                    VizeSubSpan {
                        gen_range: 0..start,
                        src_range: 500..500 + start
                    },
                    VizeSubSpan {
                        gen_range: callee_start..callee_start + callee_len,
                        src_range: 500 + start..500 + end
                    },
                    VizeSubSpan {
                        gen_range: after_call..comma,
                        src_range: 500 + end..500 + position
                    },
                    VizeSubSpan {
                        gen_range: comma + 1..expected.len() - 1,
                        src_range: 500 + position..500 + source.len()
                    },
                ],
            }]
        );
    }
}

#[test]
fn already_unambiguous_or_recovered_source_keeps_its_whole_line() {
    for source in [
        "const f = <T,>(value: T) => value;",
        "const f = <T extends unknown>(value: T) => value;",
        "const f = <T = unknown>(value: T) => value;",
        "const f = <const T>(value: T) => value;",
        "const f = <T, U>(value: T, other: U) => [value, other];",
        "const f = <div />;",
        "const f = <T>(value: T) => ;",
        "const text = '<T>(value) => value';",
    ] {
        let plan = collect_script_module_plan(source, false);
        assert_eq!(plan.tsx_arrows.len(), 0, "{source}");
        let checks = TemplateRefKeyChecks {
            calls: &[],
            helper: "",
            next: 0,
            tsx_arrows: &plan.tsx_arrows,
        };
        let mut code = crate::virtual_ts::TextBuf::default();
        let mut mappings = Vec::new();
        checks.emit_line(
            &mut code,
            &mut mappings,
            Cow::Borrowed(source),
            (0, source.len()),
            &|n| n,
        );
        assert_eq!(code, cstr!("{source}\n"));
        assert_eq!(
            mappings,
            vec![VizeMapping {
                gen_range: 0..source.len(),
                src_range: 0..source.len(),
                sub_spans: vec![]
            }]
        );
    }
}
