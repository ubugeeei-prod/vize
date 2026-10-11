//! Hoist only the dependencies of Nuxt route metadata, independently of setup.

use oxc_ast::ast::{Program, Statement};
use oxc_semantic::SemanticBuilder;
use oxc_span::{GetSpan, Span};
use vize_carton::{FxHashSet, String};

use super::imports::collect_static_imports;

pub(super) fn collect_dependencies(
    program: &Program<'_>,
    content: &str,
    payload: Option<Span>,
    nuxt_page_meta: bool,
) -> String {
    let Some(payload) = payload else {
        return String::default();
    };
    // Resolved references distinguish imported aliases from callback-local
    // bindings and object keys. Reuse this extraction's parsed program.
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(program)
        .semantic;
    let scoping = semantic.scoping();
    let mut references = Vec::new();
    for symbol in scoping.get_bindings(scoping.root_scope_id()).values() {
        let declaration = scoping.symbol_span(*symbol);
        let Some(statement) = program.body.iter().position(|statement| {
            matches!(
                statement,
                Statement::ImportDeclaration(_)
                    | Statement::VariableDeclaration(_)
                    | Statement::FunctionDeclaration(_)
            ) && contains(statement.span(), declaration)
        }) else {
            continue;
        };
        for reference in semantic.symbol_references(*symbol) {
            let flags = reference.flags();
            if flags.is_type() || flags.is_value_as_type() {
                continue;
            }
            references.push((semantic.reference_span(reference), statement));
        }
    }
    references.sort_unstable_by_key(|(span, _)| span.start);
    let mut collector = Dependencies {
        program,
        references,
        seen: FxHashSet::default(),
        imports: Vec::new(),
        declarations: Vec::new(),
    };
    collector.visit(payload);

    // Nuxt keeps whole selected import declarations in first-reference order,
    // then emits hoisted local declarations in their original source order.
    // Bare side-effect imports and unrelated setup dependencies are not part
    // of the metadata module in the pinned PageMetaPlugin contract.
    let mut output = collect_static_imports(
        collector
            .imports
            .iter()
            .filter_map(|index| program.body.get(*index)),
        content,
        nuxt_page_meta,
    );
    collector.declarations.sort_unstable();
    for index in collector.declarations {
        let Some(statement) = program.body.get(index) else {
            continue;
        };
        let span = statement.span();
        if let Some(source) = content.get(span.start as usize..span.end as usize) {
            output.push_str(source.trim());
            output.push('\n');
        }
    }
    output
}

struct Dependencies<'p, 'a> {
    program: &'p Program<'a>,
    references: Vec<(Span, usize)>,
    seen: FxHashSet<usize>,
    imports: Vec<usize>,
    declarations: Vec<usize>,
}

impl Dependencies<'_, '_> {
    fn visit(&mut self, range: Span) {
        // A depth-first worklist preserves Nuxt's first-reference order
        // without making declaration-chain depth consume the Rust stack.
        let mut pending = self.in_range(range);
        while let Some(index) = pending.pop() {
            if !self.seen.insert(index) {
                continue;
            }
            let Some(statement) = self.program.body.get(index) else {
                continue;
            };
            match statement {
                Statement::ImportDeclaration(_) => self.imports.push(index),
                statement => {
                    self.declarations.push(index);
                    pending.extend(self.in_range(statement.span()));
                }
            }
        }
    }

    fn in_range(&self, range: Span) -> Vec<usize> {
        let start = self
            .references
            .partition_point(|(span, _)| span.start < range.start);
        let end = self
            .references
            .partition_point(|(span, _)| span.start < range.end);
        let Some(references) = self.references.get(start..end) else {
            return Vec::new();
        };
        references
            .iter()
            .rev()
            .filter(|(span, _)| contains(range, *span))
            .map(|(_, statement)| *statement)
            .collect()
    }
}

fn contains(outer: Span, inner: Span) -> bool {
    inner.start >= outer.start && inner.end <= outer.end
}
