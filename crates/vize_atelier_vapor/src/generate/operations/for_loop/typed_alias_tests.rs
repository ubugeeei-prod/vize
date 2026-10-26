use oxc_ast::ast::{Expression, StaticMemberExpression};
use oxc_ast_visit::{Visit, walk};
use vize_carton::{SmallVec, String, ToCompactString, cstr};

struct Reads<'code> {
    code: &'code str,
    paths: SmallVec<[String; 4]>,
    keys: SmallVec<[&'code str; 2]>,
}

fn member_path(member: &StaticMemberExpression<'_>) -> Option<String> {
    let base = match &member.object {
        Expression::Identifier(name) => name.name.as_str().to_compact_string(),
        Expression::StaticMemberExpression(parent) => member_path(parent)?,
        _ => return None,
    };
    Some(cstr!("{base}.{}", member.property.name))
}

impl<'ast> Visit<'ast> for Reads<'_> {
    fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'ast>) {
        if let Some(path) = member_path(member)
            && (path.starts_with("_ctx.") || path.starts_with("_for_"))
        {
            self.paths.push(path);
            return;
        }
        walk::walk_static_member_expression(self, member);
    }

    fn visit_call_expression(&mut self, call: &oxc_ast::ast::CallExpression<'ast>) {
        if matches!(&call.callee, Expression::Identifier(name) if name.name == "_createFor") {
            let Some(oxc_ast::ast::Argument::ArrowFunctionExpression(key)) = call.arguments.get(2)
            else {
                panic!(
                    "typed keyed fixture must emit a key callback: {}",
                    self.code
                );
            };
            self.keys.push(
                self.code
                    .get(key.span.start as usize..key.span.end as usize)
                    .expect("complete callback source span"),
            );
        }
        walk::walk_call_expression(self, call);
    }
}

#[test]
fn typed_escaped_loop_aliases_keep_exact_reactive_reads_and_javascript_keys() {
    let cases: [(&str, &[&str], &str); 3] = [
        (
            r#"<p v-for="\u005frow: Row of items" :key="_row.id">{{ _row.id }}</p>"#,
            &["_ctx.items", "_for_item0.value.id"],
            r#"(\u005frow) => (_row.id)"#,
        ),
        (
            r#"<p v-for="(\u005frow: Row, \u005fkey: number, \u005findex: number) of items" :key="_row.id + _key + _index">{{ _row.id }}{{ _key }}{{ _index }}</p>"#,
            &[
                "_ctx.items",
                "_for_item0.value.id",
                "_for_key0.value",
                "_for_index0.value",
            ],
            r#"(\u005frow, \u005fkey, \u005findex) => (_row.id + _key + _index)"#,
        ),
        (
            r#"<p v-for="{ id: \u005frow }: Row of items" :key="_row">{{ _row }}</p>"#,
            &["_ctx.items", "_for_item0.value.id"],
            r#"({ id: \u005frow }) => (_row)"#,
        ),
    ];
    for (source, expected_reads, expected_key) in cases {
        for retained in [false, true] {
            let allocator = vize_carton::Allocator::new();
            let result = crate::compile_vapor(
                &allocator,
                source,
                crate::VaporCompilerOptions {
                    davinci_retained_lane: retained,
                    ..Default::default()
                },
            );
            assert_eq!(result.error_messages.as_slice(), &[] as &[String]);
            let parsed = oxc_parser::Parser::new(
                allocator.as_oxc(),
                &result.code,
                oxc_span::SourceType::mjs(),
            )
            .parse();
            assert!(
                parsed.diagnostics.is_empty(),
                "{:?}\n{}",
                parsed.diagnostics,
                result.code
            );
            let mut reads = Reads {
                code: &result.code,
                paths: SmallVec::new(),
                keys: SmallVec::new(),
            };
            reads.visit_program(&parsed.program);
            let paths: SmallVec<[&str; 4]> = reads.paths.iter().map(String::as_str).collect();
            assert_eq!(
                paths.as_slice(),
                expected_reads,
                "{source}\n{}",
                result.code
            );
            assert_eq!(
                reads.keys.as_slice(),
                &[expected_key],
                "{source}\n{}",
                result.code
            );
        }
    }
}
