use vize_l0::{Allocator, Span};
use vize_l2::expr::JsExpr;
use vize_l2::resolution::{BindingId, BindingLookup, resolve_expression};
use vize_l4::expr::vue::{Access, AccessStyle, Binding, BindingError, VueAccess};
use vize_l4::expr::{AccessError, EmitErrorKind, write_expression};
use vize_l4::runtime::{HelperModule, Vocabulary};
use vize_l4::write::{NoLinks, Recorded, Writer};

struct Names<'a>(&'a [&'a str]);
impl BindingLookup for Names<'_> {
    fn lookup(&self, name: &str) -> Option<BindingId> {
        self.0
            .iter()
            .position(|candidate| *candidate == name)
            .and_then(|index| u32::try_from(index).ok())
            .map(BindingId::new)
    }
}

const VOCABULARY: Vocabulary = Vocabulary {
    modules: &[HelperModule {
        module: "vue",
        names: &["unref"],
    }],
};

#[test]
fn one_ast_splices_unicode_shorthand_aliases_and_named_spans() {
    let arena = Allocator::default();
    let source = "({ 作者, alias, local, Math })";
    let file = vize_l0::cstr!("前{source}後");
    let expression =
        JsExpr::parse_in(&arena, source, Span::new(3, 3 + source.len() as u32)).unwrap();
    let table =
        resolve_expression(expression, &Names(&["作者", "alias", "local", "Math"])).unwrap();
    let bindings = [
        Binding {
            id: BindingId::new(0),
            access: Access::Context,
        },
        Binding {
            id: BindingId::new(1),
            access: Access::PropsAliased {
                property: "actual-\"name",
            },
        },
        Binding {
            id: BindingId::new(2),
            access: Access::Local,
        },
        Binding {
            id: BindingId::new(3),
            access: Access::Global,
        },
    ];
    let access = VueAccess::checked(&bindings, AccessStyle::Function, &VOCABULARY).unwrap();
    let mut recorded = Writer::<Recorded>::default();
    recorded.push("out=");
    write_expression(&mut recorded, &file, &table, &access).unwrap();
    let output = recorded.finish();
    assert_eq!(
        output.text.as_str(),
        "out=({ 作者: _ctx.作者, alias: $props[\"actual-\\\"name\"], local, Math })"
    );
    assert!(core::ptr::eq(expression.ast, table.expression().ast));
    let links: Vec<_> = output
        .links
        .links()
        .iter()
        .filter(|link| link.name.is_some())
        .collect();
    assert_eq!(links.len(), 6); // two expanded keys/values and two kept references
    assert_eq!(links[0].authored, Span::new(6, 12));
    assert_eq!(links[1].authored, Span::new(6, 12));
    assert_eq!(links[1].generated, Span::new(15, 26));
    assert_eq!(links[1].name.as_ref().unwrap().as_str(), "作者");
    let mut plain = Writer::<NoLinks>::default();
    plain.push("out=");
    write_expression(&mut plain, &file, &table, &access).unwrap();
    assert_eq!(plain.as_str(), output.text.as_str());
}

#[test]
fn ref_reads_updates_and_maybe_ref_writes_keep_operator_semantics() {
    let arena = Allocator::default();
    for (source, kind, expected, helpers) in [
        ("refName + 1", Access::SetupRef, "refName.value + 1", false),
        ("++refName", Access::SetupRef, "++refName.value", false),
        (
            "refName += 1",
            Access::SetupMaybeRef,
            "refName.value += 1",
            false,
        ),
        (
            "refName + 1",
            Access::SetupMaybeRef,
            "_unref(refName) + 1",
            true,
        ),
        (
            "new refName(1)",
            Access::SetupMaybeRef,
            "new (_unref(refName))(1)",
            true,
        ),
    ] {
        let expression =
            JsExpr::parse_in(&arena, source, Span::new(0, source.len() as u32)).unwrap();
        let table = resolve_expression(expression, &Names(&["refName"])).unwrap();
        let access = VueAccess::checked(
            &[Binding {
                id: BindingId::new(0),
                access: kind,
            }],
            AccessStyle::Inline,
            &VOCABULARY,
        )
        .unwrap();
        let mut writer = Writer::<Recorded>::default();
        write_expression(&mut writer, source, &table, &access).unwrap();
        assert_eq!(writer.as_str(), expected);
        assert_eq!(!writer.helpers().is_empty(), helpers);
    }
}

#[test]
fn failures_preserve_existing_text_links_helpers_and_indentation() {
    let arena = Allocator::default();
    let source = "first + maybe++";
    let expression = JsExpr::parse_in(&arena, source, Span::new(0, source.len() as u32)).unwrap();
    let table = resolve_expression(expression, &Names(&["first", "maybe"])).unwrap();
    let access = VueAccess::checked(
        &[
            Binding {
                id: BindingId::new(0),
                access: Access::SetupMaybeRef,
            },
            Binding {
                id: BindingId::new(1),
                access: Access::SetupLet,
            },
        ],
        AccessStyle::Inline,
        &VOCABULARY,
    )
    .unwrap();
    let mut writer = Writer::<Recorded>::default();
    writer.push_named("existing", Span::new(100, 108), "existing");
    writer.indent();
    let error = write_expression(&mut writer, source, &table, &access).unwrap_err();
    assert_eq!(
        error.kind,
        EmitErrorKind::Access(AccessError::UnsupportedUsage)
    );
    assert_eq!(error.span, Span::new(8, 13));
    assert_eq!(writer.as_str(), "existing");
    assert!(writer.helpers().is_empty());
    writer.newline();
    let output = writer.finish();
    assert_eq!(output.text.as_str(), "existing\n  ");
    assert_eq!(output.links.links().len(), 1);
}

#[test]
fn missing_framework_bindings_and_helpers_are_checked_before_append() {
    let arena = Allocator::default();
    let expression = JsExpr::parse_in(&arena, "value", Span::new(0, 5)).unwrap();
    let table = resolve_expression(expression, &Names(&["value"])).unwrap();
    let missing = VueAccess::checked(&[], AccessStyle::Function, &VOCABULARY).unwrap();
    let mut writer = Writer::<NoLinks>::default();
    assert_eq!(
        write_expression(&mut writer, "value", &table, &missing)
            .unwrap_err()
            .kind,
        EmitErrorKind::Access(AccessError::MissingBinding)
    );
    let empty = Vocabulary { modules: &[] };
    let access = VueAccess::checked(
        &[Binding {
            id: BindingId::new(0),
            access: Access::SetupMaybeRef,
        }],
        AccessStyle::Inline,
        &empty,
    )
    .unwrap();
    assert_eq!(
        write_expression(&mut writer, "value", &table, &access)
            .unwrap_err()
            .kind,
        EmitErrorKind::Access(AccessError::MissingHelper)
    );
    assert!(writer.is_empty());
    assert!(matches!(
        VueAccess::checked(
            &[
                Binding {
                    id: BindingId::new(1),
                    access: Access::Context
                },
                Binding {
                    id: BindingId::new(1),
                    access: Access::Local
                },
            ],
            AccessStyle::Function,
            &VOCABULARY
        ),
        Err(BindingError::UnorderedOrDuplicateIdentity)
    ));
}

#[test]
fn identity_source_mismatch_and_read_only_writes_are_refused() {
    let arena = Allocator::default();
    let expression = JsExpr::parse_in(&arena, "value", Span::new(0, 5)).unwrap();
    let table = resolve_expression(expression, &Names(&["value"])).unwrap();
    let access = VueAccess::checked(
        &[Binding {
            id: BindingId::new(0),
            access: Access::Context,
        }],
        AccessStyle::Server,
        &VOCABULARY,
    )
    .unwrap();
    let mut writer = Writer::<NoLinks>::default();
    assert_eq!(
        write_expression(&mut writer, "other", &table, &access)
            .unwrap_err()
            .kind,
        EmitErrorKind::SourceMismatch
    );
    for kind in [
        Access::SetupConst,
        Access::Props,
        Access::PropsAliased { property: "key" },
        Access::Global,
    ] {
        let expression = JsExpr::parse_in(&arena, "value = 1", Span::new(0, 9)).unwrap();
        let table = resolve_expression(expression, &Names(&["value"])).unwrap();
        let access = VueAccess::checked(
            &[Binding {
                id: BindingId::new(0),
                access: kind,
            }],
            AccessStyle::Function,
            &VOCABULARY,
        )
        .unwrap();
        assert_eq!(
            write_expression(&mut writer, "value = 1", &table, &access)
                .unwrap_err()
                .kind,
            EmitErrorKind::Access(AccessError::UnsupportedUsage)
        );
    }
    assert!(writer.is_empty());
}
