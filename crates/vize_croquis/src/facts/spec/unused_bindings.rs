//! TS-34: declarative unused binding rules and an independent naive evaluator.
//!
//! ```text
//! candidate(n, s) :- setup_decl(n, s), runtime_symbol(n), valid_script
//! read(n)         :- resolved_script_reference(n, read_or_type)
//! read(n)         :- template_reference(n), not template_shadow(n)
//! read(n)         :- component_or_directive(n)
//! read(n)         :- free_style_v_bind_reference(n)
//! unused(n, s)    :- candidate(n, s), not read(n), known_inline_blocks
//! ```
//!
//! This evaluator reparses only in the assurance harness. For every input
//! declaration it rescans every semantic reference and every traced expression;
//! no optimized producer, candidate storage or cached identifier set is read.

use std::collections::BTreeSet;

mod refs;

use oxc_allocator::Allocator;
use oxc_ast::AstKind;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_syntax::symbol::SymbolFlags;
use vize_carton::{CompactString, cstr};

use super::{agreement::Agreement, bindings_extract, trace::CheckedExpression};
use crate::Croquis;
use crate::drawer::{extract_identifiers_checked, extract_identifiers_oxc};
use crate::facts::{Bindings, BindingsTable, FactTable, UnusedBindingFact, UnusedBindings};
use crate::sfc::SfcDescriptor;

/// Rules over independent declarations, resolved reads and source blocks.
pub fn evaluate(
    descriptor: &SfcDescriptor<'_>,
    bindings: &FactTable<Bindings>,
    checked: &[CheckedExpression],
    croquis: &Croquis,
) -> Result<FactTable<UnusedBindings>, &'static str> {
    if descriptor.script.is_some() {
        return Err("plain-script");
    }
    let Some(setup) = descriptor.script_setup.as_ref() else {
        return Ok(FactTable::default());
    };
    if setup.src.is_some()
        || setup
            .lang
            .as_deref()
            .is_some_and(|lang| !matches!(lang, "js" | "ts" | "jsx" | "tsx"))
        || descriptor.template.as_ref().is_some_and(|block| {
            block.src.is_some() || block.lang.as_deref().is_some_and(|lang| lang != "html")
        })
        || descriptor.styles.iter().any(|block| block.src.is_some())
    {
        return Ok(FactTable::default());
    }
    let jsx = matches!(setup.lang.as_deref(), Some("jsx" | "tsx"));
    let decls = bindings_extract::extract(&setup.content, jsx)?;
    // The naive evaluator resolves the complete authored script and type
    // parameters together; it does not read production's free-reference set.
    let generic_scope = setup
        .attrs
        .get("generic")
        .map(|generic| cstr!("{}\n;(function<{generic}>() {{}});", setup.content));
    let semantic_source = generic_scope.as_deref().unwrap_or(&setup.content);
    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        semantic_source,
        if jsx {
            SourceType::tsx()
        } else {
            SourceType::ts()
        },
    )
    .parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return Err("script-parse-error");
    }
    let built = SemanticBuilder::new()
        .with_check_syntax_error(true)
        .with_build_nodes(true)
        .build(&parsed.program);
    let semantic = built.semantic;
    let scoping = semantic.scoping();
    if !built.diagnostics.is_empty()
        || scoping
            .scope_descendants_from_root()
            .any(|scope| scoping.scope_flags(scope).contains_direct_eval())
    {
        return Ok(FactTable::default());
    }
    let mut reads = BTreeSet::new();
    if let Some(template) = descriptor.template.as_ref() {
        reads.extend(refs::evaluate(&template.content));
    }
    for expression in checked {
        if extract_identifiers_checked(&expression.content).is_none() {
            return Ok(FactTable::default());
        }
        for name in extract_identifiers_oxc(&expression.content) {
            if !expression.scope_vars.contains(&name) {
                reads.insert(name);
            }
        }
    }
    for style in &descriptor.styles {
        let Some(ranges) = crate::sfc::__internal::checked_v_bind_expression_ranges(&style.content)
        else {
            return Ok(FactTable::default());
        };
        for range in ranges {
            if let Some(expression) = style.content.get(range) {
                let Some(style_reads) = extract_identifiers_checked(expression) else {
                    return Ok(FactTable::default());
                };
                reads.extend(style_reads);
            }
        }
    }
    let mut result = Vec::new();
    for decl in decls {
        let Some(symbol) = scoping.get_binding(scoping.root_scope_id(), decl.name.as_str().into())
        else {
            continue;
        };
        let flags = scoping.symbol_flags(symbol);
        if flags.intersects(SymbolFlags::TypeImport | SymbolFlags::Ambient)
            || (!flags.is_value() && !flags.contains(SymbolFlags::Import))
            || !bindings.contains_binding(&decl.name)
        {
            continue;
        }
        let script_read = semantic.nodes().iter().any(|node| {
            let AstKind::IdentifierReference(id) = node.kind() else {
                return false;
            };
            let Some(reference) = id.reference_id.get().map(|id| scoping.get_reference(id)) else {
                return false;
            };
            let flags = reference.flags();
            reference.symbol_id() == Some(symbol)
                && (flags.is_read() || flags.is_type() || flags.is_value_as_type())
        });
        let template_tag_read = croquis.component_usages.iter().any(|usage| {
            let authored = descriptor
                .template
                .as_ref()
                .and_then(|block| block.content.get(usage.start as usize..));
            let Some(authored) = authored.and_then(|text| text.strip_prefix('<')) else {
                return false;
            };
            let tag = authored
                .split(|ch: char| ch.is_whitespace() || matches!(ch, '/' | '>'))
                .next()
                .unwrap_or_default();
            normalized(tag.split('.').next().unwrap_or(tag)) == normalized(&decl.name)
        }) || croquis
            .used_directives
            .iter()
            .any(|directive| normalized(&cstr!("v-{directive}")) == normalized(&decl.name));
        if !script_read
            && !reads.contains(&decl.name)
            && !template_tag_read
            && let Some(span) = decl.span
        {
            result.push((decl.name.as_str().into(), UnusedBindingFact { span }));
        }
    }
    Ok(result.into_iter().collect())
}

fn normalized(name: &str) -> CompactString {
    name.chars()
        .filter(|ch| *ch != '-')
        .flat_map(char::to_lowercase)
        .collect()
}

/// Compare exact tables, including name identity and definition spans.
pub fn compare(
    name: &str,
    descriptor: &SfcDescriptor<'_>,
    bindings: &FactTable<Bindings>,
    checked: Option<&[CheckedExpression]>,
    croquis: &Croquis,
    production: &FactTable<UnusedBindings>,
    agreement: &mut Agreement,
) {
    let Some(checked) = checked else {
        agreement.skip("release-build");
        return;
    };
    match evaluate(descriptor, bindings, checked, croquis) {
        Err(reason) => agreement.skip(reason),
        Ok(spec) => agreement.compare(
            production.len(),
            (spec != *production)
                .then(|| cstr!("{name}: unused production {production:?} != spec {spec:?}")),
        ),
    }
}
