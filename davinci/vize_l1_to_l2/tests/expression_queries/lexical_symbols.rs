//! Independent OXC symbol declarations and resolved reference targets.
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceBlock};
use vize_l2::{
    file::{FileArtifact, ReferenceTarget},
    resolution::Usage,
};

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Symbols {
    pub declarations: Vec<(u32, u32, String)>,
    pub references: Vec<(u32, u32, u32, bool, bool, bool)>,
}

impl Symbols {
    fn sorted(mut self) -> Self {
        self.declarations.sort_unstable();
        self.references.sort_unstable();
        self
    }
}

pub(super) fn oxc(block: SourceBlock<'_>, ty: SourceType) -> Symbols {
    with_oxc(block, ty, |_| {})
}

pub(super) fn with_oxc(
    block: SourceBlock<'_>,
    ty: SourceType,
    queries: impl FnOnce(&oxc_semantic::Scoping),
) -> Symbols {
    let arena = Allocator::new();
    let parsed = Parser::new(arena.as_oxc(), block.source(), ty).parse();
    assert!(parsed.diagnostics.is_empty());
    let built = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program);
    assert!(built.diagnostics.is_empty(), "{:?}", built.diagnostics);
    let semantic = built.semantic;
    let scopes = semantic.scoping();
    let base = block.start();
    let mut rows = Symbols::default();
    assert!(scopes.root_unresolved_references().is_empty());
    for id in scopes.symbol_ids() {
        let decl = scopes.symbol_span(id);
        rows.declarations.push((
            base + decl.start,
            base + decl.end,
            scopes.symbol_name(id).to_owned(),
        ));
        for reference in semantic.symbol_references(id) {
            let span = semantic.reference_span(reference);
            rows.references.push((
                base + span.start,
                base + span.end,
                base + decl.start,
                reference.is_read(),
                reference.is_write(),
                reference.scope_id() != scopes.symbol_scope_id(id),
            ));
        }
    }
    queries(scopes);
    rows.sorted()
}

pub(super) fn file(file: &FileArtifact<'_>) -> Symbols {
    let mut rows = Symbols::default();
    for binding in file.bindings() {
        let decl = binding.declaration().unwrap();
        rows.declarations
            .push((decl.span.start, decl.span.end, decl.name.to_string()));
    }
    for row in file.references() {
        let ReferenceTarget::Resolved(id) = row.target else {
            panic!("unresolved {}", row.name)
        };
        let decl = file.binding(id).unwrap().declaration().unwrap();
        rows.references.push((
            row.span.start,
            row.span.end,
            decl.span.start,
            row.usage != Usage::Write,
            row.usage != Usage::Read,
            row.scope != decl.scope,
        ));
    }
    rows.sorted()
}
