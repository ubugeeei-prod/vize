use std::io::Write;
use std::process::{Command, Stdio};

use vize_l0::{Allocator, Span};
use vize_l2::expr::JsExpr;
use vize_l2::resolution::{BindingId, BindingLookup, resolve_expression};
use vize_l4::expr::vue::{Access, AccessStyle, Binding, VueAccess};
use vize_l4::expr::{AccessError, EmitErrorKind, write_expression};
use vize_l4::runtime::Vocabulary;
use vize_l4::write::{NoLinks, Recorded, Writer};

struct Names;
impl BindingLookup for Names {
    fn lookup(&self, name: &str) -> Option<BindingId> {
        match name {
            "__proto__" => Some(BindingId::new(0)),
            "other" => Some(BindingId::new(1)),
            _ => None,
        }
    }
}

const VOCABULARY: Vocabulary = Vocabulary { modules: &[] };

#[test]
fn actual_shorthand_output_keeps_own_properties_and_object_prototype_in_js() {
    let arena = Allocator::default();
    let access = VueAccess::checked(
        &[
            Binding {
                id: BindingId::new(0),
                access: Access::Context,
            },
            Binding {
                id: BindingId::new(1),
                access: Access::Context,
            },
        ],
        AccessStyle::Function,
        &VOCABULARY,
    )
    .unwrap();
    let mut fixtures = Vec::new();
    for (source, expected) in [
        (
            "({ __proto__, other })",
            "({ [\"__proto__\"]: _ctx.__proto__, other: _ctx.other })",
        ),
        (
            "({ \\u005f_proto__, other })",
            "({ [\"__proto__\"]: _ctx.\\u005f_proto__, other: _ctx.other })",
        ),
    ] {
        let file = vize_l0::cstr!("前{source}後");
        let expression =
            JsExpr::parse_in(&arena, source, Span::new(3, 3 + source.len() as u32)).unwrap();
        let table = resolve_expression(expression, &Names).unwrap();
        assert!(core::ptr::eq(expression.ast, table.expression().ast));
        let occurrence = table.occurrences().first().unwrap();
        assert_eq!(occurrence.name, "__proto__");
        assert!(occurrence.shorthand);
        let authored = expression.authored_span(occurrence.span).unwrap();
        let mut writer = Writer::<Recorded>::default();
        write_expression(&mut writer, &file, &table, &access).unwrap();
        let output = writer.finish();
        let prefix = "({ [\"__proto__\"]: _ctx.";
        assert_eq!(output.text.as_str(), expected);
        let named: Vec<_> = output
            .links
            .links()
            .iter()
            .filter(|link| link.name.is_some())
            .collect();
        assert_eq!(named.len(), 4);
        assert_eq!(named[0].generated, Span::new(3, 16));
        assert_eq!(named[0].authored, authored);
        assert_eq!(named[1].authored, authored);
        assert_eq!(named[0].name.as_ref().unwrap().as_str(), "__proto__");
        assert_eq!(named[1].generated.start, prefix.len() as u32 - 5);
        assert_eq!(
            named[1].generated.end,
            (prefix.len() + occurrence.span.len() as usize) as u32
        );
        let mut plain = Writer::<NoLinks>::default();
        write_expression(&mut plain, &file, &table, &access).unwrap();
        assert_eq!(plain.as_str(), output.text.as_str());
        fixtures.push(serde_json::json!({ "original": source, "emitted": output.text.as_str() }));
    }
    let mut child = Command::new("node")
        .args([
            "--input-type=module",
            "-e",
            include_str!("../../../tests/tooling/support/native-expression-prototype.ts"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Node is required for actual emitted JavaScript semantics");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&fixtures).unwrap())
        .unwrap();
    let result = child.wait_with_output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        core::str::from_utf8(&result.stderr).unwrap_or("non-UTF8 stderr")
    );
    let receipt: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(receipt["fixtures"], 2);
    assert_eq!(receipt["executions"], 6);
}

#[test]
fn later_missing_access_preserves_writer_after_computed_key_preflight() {
    let arena = Allocator::default();
    let source = "({ __proto__, other })";
    let expression = JsExpr::parse_in(&arena, source, Span::new(0, source.len() as u32)).unwrap();
    let table = resolve_expression(expression, &Names).unwrap();
    let access = VueAccess::checked(
        &[Binding {
            id: BindingId::new(0),
            access: Access::Context,
        }],
        AccessStyle::Function,
        &VOCABULARY,
    )
    .unwrap();
    let mut writer = Writer::<Recorded>::default();
    writer.push_named("before", Span::new(50, 56), "before");
    writer.indent();
    let error = write_expression(&mut writer, source, &table, &access).unwrap_err();
    assert_eq!(
        error.kind,
        EmitErrorKind::Access(AccessError::MissingBinding)
    );
    assert_eq!(error.span, Span::new(14, 19));
    assert_eq!(writer.as_str(), "before");
    assert!(writer.helpers().is_empty());
    writer.newline();
    let output = writer.finish();
    assert_eq!(output.text.as_str(), "before\n  ");
    assert_eq!(output.links.links().len(), 1);
}
