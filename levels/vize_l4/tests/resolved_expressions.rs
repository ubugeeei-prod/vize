use vize_l0::{Allocator, Span};
use vize_l2::expr::JsExpr;
use vize_l2::resolution::{BindingId, BindingLookup, resolve_expression};
use vize_l4::expr::{
    AccessError, AccessProvider, AccessSpelling, EmitErrorKind, ResolutionSetError,
    ResolvedExpressions,
};
use vize_l4::write::{NoLinks, Writer};

struct Facts;
impl BindingLookup for Facts {
    fn lookup(&self, _name: &str) -> Option<BindingId> {
        Some(BindingId::new(0))
    }
}
impl AccessProvider for Facts {
    fn spelling(
        &self,
        _: &vize_l2::resolution::Occurrence<'_>,
    ) -> Result<AccessSpelling<'_>, AccessError> {
        Ok(AccessSpelling::Rewrite {
            prefix: "_ctx.",
            replacement: None,
            suffix: "",
            helper: None,
        })
    }
}

#[test]
fn exact_retained_ast_reuses_its_resolution_without_rebinding() {
    let arena = Allocator::default();
    let file = "first second";
    let first = JsExpr::parse_in(&arena, "first", Span::new(0, 5)).unwrap();
    let second = JsExpr::parse_in(&arena, "second", Span::new(6, 12)).unwrap();
    let tables = [
        resolve_expression(first, &Facts).unwrap(),
        resolve_expression(second, &Facts).unwrap(),
    ];
    let expressions = ResolvedExpressions::checked(&tables).unwrap();
    let mut writer = Writer::<NoLinks>::default();
    expressions
        .write_retained(&mut writer, file, second, &Facts)
        .unwrap();
    writer.push(" + ");
    expressions
        .write_retained(&mut writer, file, first, &Facts)
        .unwrap();
    assert_eq!(writer.as_str(), "_ctx.second + _ctx.first");
}

#[test]
fn same_text_from_another_parse_cannot_reuse_the_table() {
    let arena = Allocator::default();
    let original = JsExpr::parse_in(&arena, "value", Span::new(0, 5)).unwrap();
    let foreign = JsExpr::parse_in(&arena, "value", Span::new(0, 5)).unwrap();
    let tables = [resolve_expression(original, &Facts).unwrap()];
    let expressions = ResolvedExpressions::checked(&tables).unwrap();
    let mut writer = Writer::<NoLinks>::default();
    assert_eq!(
        expressions
            .write_retained(&mut writer, "value", foreign, &Facts)
            .unwrap_err()
            .kind,
        EmitErrorKind::SourceMismatch
    );
    assert!(writer.is_empty());
    let missing = ResolvedExpressions::checked(&[]).unwrap();
    assert_eq!(
        missing
            .write_retained(&mut writer, "value", original, &Facts)
            .unwrap_err()
            .kind,
        EmitErrorKind::MissingResolution
    );
    assert!(writer.is_empty());
}

#[test]
fn duplicate_or_unordered_expression_ownership_is_rejected() {
    let arena = Allocator::default();
    let first = JsExpr::parse_in(&arena, "first", Span::new(0, 5)).unwrap();
    let second = JsExpr::parse_in(&arena, "second", Span::new(6, 12)).unwrap();
    let reversed = [
        resolve_expression(second, &Facts).unwrap(),
        resolve_expression(first, &Facts).unwrap(),
    ];
    assert_eq!(
        ResolvedExpressions::checked(&reversed).unwrap_err(),
        ResolutionSetError::UnorderedOrDuplicateSpan
    );
    let duplicate = [
        resolve_expression(first, &Facts).unwrap(),
        resolve_expression(first, &Facts).unwrap(),
    ];
    assert_eq!(
        ResolvedExpressions::checked(&duplicate).unwrap_err(),
        ResolutionSetError::UnorderedOrDuplicateSpan
    );
}
