//! Original native source laws for exceptional binding declarations.

use super::{ForName, ForParts};
use crate::lower::{Lowered, lower, lower_source_block};
use vize_l0::{Allocator, SourceRoot, Span, cstr};
use vize_l2::op::{ForOp, Op};
use vize_l2::scope::{ScopeFacts, ScopeOrigin};

fn with_for(source: &str, check: impl FnOnce(&Lowered<'_>, &ForOp<'_>, &ScopeFacts, &ForParts)) {
    let arena = Allocator::new();
    let (tree, errors) = vize_l1::markup::parse(&arena, source);
    assert!(errors.is_empty(), "{errors:?}");
    let lowered = lower(&arena, &tree, &errors);
    let Some(Op::For(for_op)) = lowered.root.ops.first() else {
        panic!("original source owns a native ForOp");
    };
    let (id, scope) = lowered.scopes.iter().next().expect("iteration scope");
    let parts = lowered.for_facts.get(id).expect("matching ForParts");
    assert_eq!(parts.tag, scope.tag);
    check(&lowered, for_op, scope, parts);
}

#[cfg(test)]
fn declarations<'a>(source: &'a str, scope: &'a ScopeFacts) -> alloc::vec::Vec<(&'a str, &'a str)> {
    scope
        .bindings
        .iter()
        .map(|binding| {
            let ScopeOrigin::Authored { span } = binding.origin else {
                panic!("original parameter declarations are authored");
            };
            (
                binding.name.as_str(),
                source
                    .get(span.start as usize..span.end as usize)
                    .expect("complete authored declaration token"),
            )
        })
        .collect()
}

#[test]
fn original_simple_escapes_and_unicode_keep_cooked_names_and_complete_tokens() {
    for (alias, cooked) in [
        (r"\u005frow", "_row"),
        (r"\u{5f}row", "_row"),
        ("π", "π"),
        ("𠮷", "𠮷"),
        ("&#92;u005frow", "_row"),
    ] {
        let source = cstr!("<li v-for=\"{alias} in rows\">{{{{ value }}}}</li>");
        with_for(&source, |_, for_op, scope, parts| {
            assert_eq!(parts.value, ForName::Named(cooked.into()), "{alias}");
            assert_eq!(parts.key, ForName::Absent);
            assert_eq!(parts.index, ForName::Absent);
            assert_eq!(declarations(&source, scope), [(cooked, alias)]);
            let ScopeOrigin::Authored { span } = scope.bindings.first().unwrap().origin else {
                panic!("authored declaration")
            };
            assert_eq!(span, for_op.binding.value.span());
        });
    }
}

#[test]
fn destructuring_key_index_and_crlf_use_only_real_binding_declarations() {
    let source = concat!(
        "<li\r\n v-for=\"({ label: ",
        r"\u005frow = seed, [lookup]: { name: α },",
        "\r\n items: [first = fallback, ...",
        r"\u0072est], ...extra }, \u006bey, \u{69}ndex) in rows",
        "\">{{ _row }}</li>"
    );
    with_for(source, |_, for_op, scope, parts| {
        assert_eq!(parts.value, ForName::Pending);
        assert_eq!(parts.key, ForName::Named("key".into()));
        assert_eq!(parts.index, ForName::Named("index".into()));
        assert_eq!(
            declarations(source, scope),
            [
                ("_row", r"\u005frow"),
                ("α", "α"),
                ("first", "first"),
                ("rest", r"\u0072est"),
                ("extra", "extra"),
                ("key", r"\u006bey"),
                ("index", r"\u{69}ndex"),
            ]
        );
        for declaration in &scope.bindings {
            let ScopeOrigin::Authored { span } = declaration.origin else {
                panic!("authored declaration")
            };
            assert!(
                [
                    Some(&for_op.binding.value),
                    for_op.binding.key.as_ref(),
                    for_op.binding.index.as_ref(),
                ]
                .into_iter()
                .flatten()
                .any(|alias| alias.span().start <= span.start && span.end <= alias.span().end)
            );
        }
    });
}

#[test]
fn decoded_pattern_tokens_project_to_the_full_authored_entity_spelling() {
    let source = r#"<li v-for="{ name: &#92;u005frow, &#x3b1;: value } in rows">x</li>"#;
    with_for(source, |_, _, scope, parts| {
        assert_eq!(parts.value, ForName::Pending);
        assert_eq!(
            declarations(source, scope),
            [("_row", "&#92;u005frow"), ("value", "value")]
        );
    });
}

#[test]
fn rejected_binding_goals_never_fall_back_to_expression_names_or_partial_patterns() {
    for alias in [
        r"\u0061 + other",
        r"{ good: \u0061, bad: obj.field }",
        r"{ good: \u0061, broken: }",
        r"[\u0061, obj.field]",
        r"\u00zz",
        r"\u0069f",
    ] {
        let source = cstr!("<li v-for=\"({alias}, key) in rows\">x</li>");
        with_for(&source, |_, _, scope, parts| {
            assert_eq!(parts.value, ForName::Pending, "{alias}");
            assert_eq!(parts.key, ForName::Named("key".into()), "{alias}");
            assert_eq!(declarations(&source, scope), [("key", "key")], "{alias}");
        });
    }
}

#[test]
fn guarded_deep_exceptional_pattern_publishes_no_partial_bindings() {
    let alias = cstr!("{}\\u0061{}", "[".repeat(256), "]".repeat(256));
    let source = cstr!("<li v-for=\"{alias} in rows\">x</li>");
    with_for(&source, |_, _, scope, parts| {
        assert_eq!(parts.value, ForName::Pending);
        assert!(scope.bindings.is_empty());
    });
}

#[test]
fn ordinary_ascii_positions_keep_existing_scope_and_pending_pattern_contract() {
    let source = r#"<li v-for="({ label: row }, key, index) in rows">x</li>"#;
    with_for(source, |_, _, scope, parts| {
        assert_eq!(parts.value, ForName::Pending);
        assert_eq!(parts.key, ForName::Named("key".into()));
        assert_eq!(parts.index, ForName::Named("index".into()));
        assert_eq!(
            declarations(source, scope),
            [("key", "key"), ("index", "index")]
        );
    });
}

#[test]
fn source_block_origins_stay_file_absolute_after_unicode_and_crlf() {
    let arena = Allocator::new();
    let prefix = "前置\r\n";
    let template = r#"<li v-for="\u005frow in rows">x</li>"#;
    let source = cstr!("{prefix}{template}");
    let root = SourceRoot::new(&source).unwrap();
    let block = root.block(template, prefix.len() as u32);
    // The parser and lowerer receive the actual file subslice, not the
    // separately allocated, byte-equal template used to spell the fixture.
    assert!(block.is_err());
    let actual = source.get(prefix.len()..).unwrap();
    let block = root.block(actual, prefix.len() as u32).unwrap();
    let (tree, errors) = vize_l1::markup::parse(&arena, actual);
    assert!(errors.is_empty());
    let lowered = lower_source_block(&arena, &tree, &errors, block);
    let (id, scope) = lowered.scopes.iter().next().unwrap();
    assert_eq!(declarations(&source, scope), [("_row", r"\u005frow")]);
    assert_eq!(
        lowered.for_facts.get(id).unwrap().value,
        ForName::Named("_row".into())
    );
    let start = (prefix.len() + template.find(r"\u005frow").unwrap()) as u32;
    assert_eq!(
        scope.bindings.first().unwrap().origin,
        ScopeOrigin::Authored {
            span: Span::new(start, start + 9)
        }
    );
}
