//! Exact typed-producer inputs; signatures are explicit, never inferred.
#![expect(clippy::unwrap_used, reason = "fixture byte offsets fit u32")]

use vize_extension_contract::contract::Span;
use vize_extension_contract::typed_expression::{
    Demand, TypedBinding, TypedExpression, TypedExpressionBatch,
};

pub fn binding(name: &str, signature: &str) -> TypedBinding {
    TypedBinding {
        name: name.into(),
        kind: "setup".into(),
        signature: signature.into(),
    }
}

pub fn expression(id: u32, source: &str, expected: Demand) -> TypedExpression {
    let start = 100 + id * 100;
    TypedExpression {
        id,
        source: source.into(),
        span: Span {
            start,
            end: start + u32::try_from(source.len()).unwrap(),
        },
        locals: Vec::new(),
        expected,
    }
}

pub fn batch() -> TypedExpressionBatch {
    let mut local = expression(7, "item + title", Demand::Show);
    local.locals = vec![binding("item", "String")];
    let mut shadow = expression(8, "title + index", Demand::Show);
    shadow.locals = vec![binding("title", "Int"), binding("index", "Int")];
    TypedExpressionBatch {
        environment: vec![
            binding("title", "String"),
            binding("count", "Ref[Int]"),
            binding("visible", "Bool"),
            binding("event_name", "String"),
            binding("save", "() -> Unit"),
            binding("use_label", "() -> String"),
            binding("items", "Array[String]"),
        ],
        expressions: vec![
            expression(0, "title", Demand::Show),
            expression(1, "count.val", Demand::Show),
            expression(2, "visible", Demand::Condition),
            expression(3, "event_name", Demand::Name),
            expression(4, "save", Demand::Handler),
            expression(5, "count.val += 1", Demand::Statement),
            expression(6, "use_label()", Demand::Value),
            local,
            shadow,
            expression(9, "items[0]", Demand::Show),
            expression(10, "fn() { save() }", Demand::Handler),
            expression(11, "\"😀\" + title", Demand::Show),
        ],
    }
}

pub fn typo_batch() -> TypedExpressionBatch {
    let mut batch = batch();
    batch.expressions = vec![
        expression(0, "count.missing", Demand::Show),
        expression(1, "count.val", Demand::Condition),
        expression(2, "count.val", Demand::Name),
        expression(3, "use_label", Demand::Handler),
        expression(4, "use_label()", Demand::Statement),
        expression(5, "use_label(1)", Demand::Value),
        expression(6, "item", Demand::Show),
    ];
    batch
}
