//! TS-42 type edits use OXC member/type spans from the parsed setup block.

use oxc_ast::ast::TSPropertySignature;
use oxc_ast_visit::{Visit, walk};
use oxc_span::{GetSpan, SourceType, Span};

struct Member<'s> {
    name: &'s str,
    span: Option<Span>,
}

impl<'a> Visit<'a> for Member<'_> {
    fn visit_ts_property_signature(&mut self, member: &TSPropertySignature<'a>) {
        if member.key.static_name().as_deref() == Some(self.name) {
            assert!(self.span.is_none(), "edit member must be unique");
            self.span = member
                .type_annotation
                .as_ref()
                .map(|annotation| annotation.type_annotation.span());
        }
        walk::walk_ts_property_signature(self, member);
    }
}

pub(super) fn replace_member_type(source: &str, name: &str, replacement: &str) -> String {
    let descriptor = vize_incremental::parse_descriptor("Button.vue", source).unwrap();
    let script = descriptor.script_setup.as_ref().unwrap();
    let allocator = oxc_allocator::Allocator::new();
    let parsed = oxc_parser::Parser::new(&allocator, &script.content, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let mut member = Member { name, span: None };
    member.visit_program(&parsed.program);
    let span = member.span.expect("typed member must exist");
    let start = script.loc.start + span.start as usize;
    let end = script.loc.start + span.end as usize;
    let mut edited = String::from(source);
    edited.replace_range(start..end, replacement);
    let descriptor = vize_incremental::parse_descriptor("Button.vue", &edited).unwrap();
    let script = descriptor.script_setup.as_ref().unwrap();
    assert!(
        oxc_parser::Parser::new(&allocator, &script.content, SourceType::ts())
            .parse()
            .diagnostics
            .is_empty()
    );
    edited
}
