//! Helper functions for SFC compilation.

use oxc_allocator::Allocator as OxcAllocator;
use oxc_ast::ast::{BindingPattern, Expression, Statement, VariableDeclarationKind};
use oxc_parser::Parser as OxcParser;
use oxc_span::SourceType;
use vize_atelier_core::Allocator as TemplateAllocator;
use vize_carton::{String, ToCompactString};

use crate::script::{resolve_template_v_model_identifiers, ScriptCompileContext};
use crate::types::{BindingMetadata, BindingType};

/// Generate scope ID from filename
pub(super) fn generate_scope_id(filename: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    filename.hash(&mut hasher);
    let value = hasher.finish() & 0xFFFFFFFF;
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(8);
    for shift in (0..32).step_by(4).rev() {
        let digit = ((value >> shift) & 0xF) as usize;
        out.push(HEX[digit] as char);
    }
    out
}

/// Extract component name from filename
pub(super) fn extract_component_name(filename: &str) -> String {
    std::path::Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("anonymous")
        .to_compact_string()
}

pub(super) fn demote_v_model_reactive_const_bindings(
    template_content: &str,
    script_lang: Option<&str>,
    script_content: &mut String,
    ctx: &mut ScriptCompileContext,
    script_bindings: &mut BindingMetadata,
    croquis: &mut vize_croquis::analysis::Croquis,
) -> Vec<String> {
    let template_allocator = TemplateAllocator::default();
    let (root, _) = vize_atelier_core::parse(&template_allocator, template_content);
    let v_model_ids = resolve_template_v_model_identifiers(&root);

    if v_model_ids.is_empty() {
        return Vec::new();
    }

    let source_type = source_type_for_script_lang(script_lang);
    let allocator = OxcAllocator::default();
    let ret = OxcParser::new(&allocator, script_content, source_type).parse();

    if ret.panicked {
        return Vec::new();
    }

    let mut rewritten = String::default();
    let mut last_end = 0usize;
    let mut demoted_ids = Vec::new();

    for stmt in ret.program.body.iter() {
        let Statement::VariableDeclaration(var_decl) = stmt else {
            continue;
        };

        if var_decl.kind != VariableDeclarationKind::Const {
            continue;
        }

        let mut should_demote = false;

        for declarator in &var_decl.declarations {
            let BindingPattern::BindingIdentifier(id) = &declarator.id else {
                continue;
            };

            let binding_name = id.name.as_str();
            if !v_model_ids.contains(binding_name) {
                continue;
            }

            if !matches!(
                script_bindings.bindings.get(binding_name),
                Some(BindingType::SetupReactiveConst)
            ) {
                continue;
            }

            if !is_demotable_reactive_initializer(declarator.init.as_ref()) {
                continue;
            }

            should_demote = true;
            demoted_ids.push(binding_name.to_compact_string());
            update_binding_to_setup_let(binding_name, ctx, script_bindings, croquis);
        }

        if !should_demote {
            continue;
        }

        let decl_start = var_decl.span.start as usize;
        let decl_end = var_decl.span.end as usize;
        let after_const = decl_start + "const".len();

        rewritten.push_str(&script_content[last_end..decl_start]);
        rewritten.push_str("let");
        rewritten.push_str(&script_content[after_const..decl_end]);
        last_end = decl_end;
    }

    if demoted_ids.is_empty() {
        return demoted_ids;
    }

    rewritten.push_str(&script_content[last_end..]);
    *script_content = rewritten;

    demoted_ids
}

fn source_type_for_script_lang(lang: Option<&str>) -> SourceType {
    match lang {
        Some("ts") => SourceType::ts(),
        Some("tsx") => SourceType::tsx(),
        Some("jsx") => SourceType::jsx(),
        _ => SourceType::mjs(),
    }
}

fn is_demotable_reactive_initializer(init: Option<&Expression<'_>>) -> bool {
    let Some(Expression::CallExpression(call)) = init else {
        return false;
    };

    let Expression::Identifier(callee) = &call.callee else {
        return false;
    };

    matches!(callee.name.as_str(), "reactive" | "shallowReactive")
}

fn update_binding_to_setup_let(
    binding_name: &str,
    ctx: &mut ScriptCompileContext,
    script_bindings: &mut BindingMetadata,
    croquis: &mut vize_croquis::analysis::Croquis,
) {
    script_bindings
        .bindings
        .insert(binding_name.to_compact_string(), BindingType::SetupLet);
    ctx.bindings
        .bindings
        .insert(binding_name.to_compact_string(), BindingType::SetupLet);
    croquis
        .bindings
        .bindings
        .insert(binding_name.to_compact_string(), BindingType::SetupLet);
}
