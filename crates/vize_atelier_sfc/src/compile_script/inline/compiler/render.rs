use oxc_syntax::identifier::is_identifier_part;
use vize_carton::string_builder::TextBuf;
use vize_carton::{FxHashSet, String};

use crate::module_map::Runs;
use crate::script::ScriptCompileContext;

use super::super::super::{TemplateParts, import_utils::extract_import_identifiers};
use super::trace::TemplateTrace;

mod template_refs;

const VAPOR_RENDER_ALIAS_BASE: &str = "__vaporRender";
const VAPOR_TEMPLATE_REF_SETTER: &str = "vaporTemplateRefSetter";

pub(super) struct SetupBindingInputs<'a> {
    pub(super) imports: &'a [String],
    pub(super) runtime_used_identifiers: Option<&'a FxHashSet<String>>,
}

/// Emit the render function return statement or setup binding return.
#[expect(clippy::too_many_arguments, reason = "independent compile inputs")]
pub(super) fn emit_render_return(
    output: &mut TextBuf,
    template: &TemplateParts<'_>,
    setup_bindings: SetupBindingInputs<'_>,
    is_ts: bool,
    is_vapor: bool,
    is_prod: bool,
    vapor_render_alias: Option<&str>,
    ctx: &ScriptCompileContext,
) -> InlineRenderLayout {
    let mut layout = InlineRenderLayout::default();
    if !template.render_body.is_empty() {
        if is_ts {
            output.push_str("return (_ctx: any,_cache: any) => {\n");
        } else {
            output.push_str("return (_ctx, _cache) => {\n");
        }

        // Output component/directive resolution statements (preamble)
        layout.assets = output.len();
        for line in template.preamble.lines() {
            if !line.trim().is_empty() {
                output.push_str("  ");
                output.push_str(line);
                output.push('\n');
            }
        }
        if !template.preamble.is_empty() {
            output.push('\n');
        }

        if template.render_is_block {
            for line in template.render_body.lines() {
                if line.trim().is_empty() {
                    output.push('\n');
                    continue;
                }

                output.push_str("  ");
                output.push_str(line);
                output.push('\n');
            }
        } else {
            // Indent the render body properly
            layout.body = output.len() + "  return ".len();
            let mut first_line = true;
            for line in template.render_body.lines() {
                if first_line {
                    output.push_str("  return ");
                    output.push_str(line);
                    first_line = false;
                } else {
                    output.push('\n');
                    output.push_str(line);
                }
            }
            if first_line {
                output.push_str("  return null");
            }
            output.push('\n');
        }
        output.push_str("}\n");
    } else {
        let setup_bindings = collect_setup_bindings(ctx, setup_bindings, template);
        if is_vapor && !template.render_fn.is_empty() {
            let needs_template_ref_setter = template.render_fn.contains("_createTemplateRefSetter");
            if needs_template_ref_setter {
                template_refs::emit_setter(output, &setup_bindings, ctx);
            }
            output.push_str("const __returned__ = { ");
            let mut binding_index = 0usize;
            if needs_template_ref_setter {
                output.push_str(VAPOR_TEMPLATE_REF_SETTER);
                binding_index += 1;
            }
            for name in setup_bindings.iter() {
                if binding_index > 0 {
                    output.push_str(", ");
                }
                binding_index += 1;
                output.push_str(name);
            }
            output.push_str(" }\n");
            if needs_template_ref_setter {
                output.push_str(";({}).constructor.defineProperty(__returned__, '__isScriptSetup', { enumerable: false, value: true })\n");
            } else {
                output.push_str("Object.defineProperty(__returned__, '__isScriptSetup', { enumerable: false, value: true })\n");
            }
            if is_prod {
                // Production does not proxy returned setup bindings. Preserve
                // ref unwrapping and writes for the separate Vapor render.
                output.push_str("const __instance = _getCurrentInstance()\n");
                output.push_str("const __ctx = _proxyRefs(__returned__)\n");
                output.push_str("if (__instance) __instance.setupState = __ctx\n");
                output.push_str("return ");
                output.push_str(vapor_render_alias.unwrap_or("render"));
                output.push_str("(__ctx, __props, __emit, __attrs, __slots)\n");
            } else {
                // Development owns the setup-state proxy and render scope so
                // child-triggered HMR can rerender with the parent's bindings.
                output.push_str("return __returned__\n");
            }
        } else if !setup_bindings.is_empty() {
            if !template.render_fn.is_empty() {
                output.push_str("const __returned__ = { ");
            } else {
                // No template (e.g., Musea art files) -- return setup bindings as an object
                // so they're accessible for runtime template compilation (compileToFunction).
                output.push_str("return { ");
            }
            for (i, name) in setup_bindings.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }
                output.push_str(name);
            }
            output.push_str(" }\n");
            if !template.render_fn.is_empty() {
                output.push_str("Object.defineProperty(__returned__, '__isScriptSetup', { enumerable: false, value: true })\n");
                output.push_str("return __returned__\n");
            }
        } else if !template.render_fn.is_empty() {
            output.push_str("return {}\n");
        }
    }
    layout
}

/// Where [`emit_render_return`] wrote the inline render's template parts,
/// as offsets into the text it emitted.
#[derive(Clone, Copy, Default)]
pub(super) struct InlineRenderLayout {
    /// The first asset-resolution statement's two-space indent.
    assets: usize,
    /// The return expression, after `return `.
    body: usize,
}

impl InlineRenderLayout {
    /// The `.vue` provenance of `render`, the text [`emit_render_return`]
    /// emitted from `template`, given the provenance of the parts it copied.
    ///
    /// Each asset statement follows a two-space indent and each line of the
    /// return expression follows the previous line's `\n`. The copies are
    /// verified against `render` before `trace` is read through them.
    pub(super) fn provenance(
        self,
        render: &str,
        template: &TemplateParts<'_>,
        trace: &TemplateTrace,
    ) -> Runs {
        let statements = template
            .preamble
            .lines()
            .filter(|line| !line.trim().is_empty());
        let mut assets = Runs::default();
        assets.copy_lines(self.assets, template.preamble, statements, 2);
        let mut body = Runs::default();
        body.copy_lines(
            self.body,
            template.render_body,
            template.render_body.lines(),
            0,
        );
        let mut placed = assets
            .verified(render, template.preamble)
            .compose(&trace.preamble);
        let body = body.verified(render, template.render_body);
        placed.append(0, &body.compose(&trace.render_body));
        placed
    }
}

pub(super) fn build_vapor_render_alias(
    content: &str,
    normal_script_content: Option<&str>,
    template_render_fn: &str,
) -> String {
    let mut suffix = 0usize;
    loop {
        let candidate = build_vapor_render_alias_candidate(suffix);
        let candidate_str = candidate.as_str();
        if !content.contains(candidate_str)
            && normal_script_content.is_none_or(|script| !script.contains(candidate_str))
            && !template_render_fn.contains(candidate_str)
        {
            return candidate;
        }
        suffix += 1;
    }
}

fn build_vapor_render_alias_candidate(suffix: usize) -> String {
    let mut candidate = String::from(VAPOR_RENDER_ALIAS_BASE);
    if suffix == 0 {
        return candidate;
    }

    candidate.push('_');
    append_usize(&mut candidate, suffix);
    candidate
}

fn append_usize(target: &mut String, value: usize) {
    use std::fmt::Write as _;
    let _ = write!(target, "{value}");
}

fn collect_setup_bindings(
    ctx: &ScriptCompileContext,
    setup_bindings: SetupBindingInputs<'_>,
    template: &TemplateParts<'_>,
) -> Vec<String> {
    use crate::types::BindingType;

    let has_template = !template.render_fn.is_empty() || !template.render_body.is_empty();
    let mut template_code =
        String::with_capacity(template.render_fn.len() + template.render_body.len());
    template_code.push_str(template.render_fn);
    template_code.push_str(template.render_body);
    let imported_identifiers: FxHashSet<String> = setup_bindings
        .imports
        .iter()
        .flat_map(|import| extract_import_identifiers(import).into_iter())
        .collect();

    let mut bindings: Vec<String> = ctx
        .bindings
        .bindings
        .iter()
        .filter(|(name, bt)| {
            let is_setup_binding = matches!(
                bt,
                BindingType::SetupLet
                    | BindingType::SetupMaybeRef
                    | BindingType::SetupRef
                    | BindingType::SetupReactiveConst
                    | BindingType::SetupConst
                    | BindingType::LiteralConst
            );
            let is_runtime_import = !imported_identifiers.contains(name.as_str())
                || !has_template
                || setup_bindings
                    .runtime_used_identifiers
                    .is_some_and(|ids| ids.contains(name.as_str()))
                || template_references_setup_binding(&template_code, name.as_str());
            is_setup_binding && is_runtime_import
        })
        .map(|(name, _)| String::from(name.as_str()))
        .collect();

    for import in setup_bindings.imports {
        for name in extract_import_identifiers(import) {
            let should_return = !has_template
                || setup_bindings
                    .runtime_used_identifiers
                    .is_some_and(|ids| ids.contains(name.as_str()))
                || template_references_setup_binding(&template_code, name.as_str());
            if should_return && !bindings.iter().any(|binding| binding == name.as_str()) {
                bindings.push(name);
            }
        }
    }

    bindings
}

fn template_references_setup_binding(template_code: &str, name: &str) -> bool {
    let mut setup_access = String::with_capacity(name.len() + 8);
    setup_access.push_str("$setup.");
    setup_access.push_str(name);
    if contains_member_access(template_code, &setup_access) {
        return true;
    }

    let mut setup_bracket_access = String::with_capacity(name.len() + 11);
    setup_bracket_access.push_str("$setup[\"");
    setup_bracket_access.push_str(name);
    setup_bracket_access.push_str("\"]");
    if template_code.contains(setup_bracket_access.as_str()) {
        return true;
    }

    let mut ctx_access = String::with_capacity(name.len() + 5);
    ctx_access.push_str("_ctx.");
    ctx_access.push_str(name);
    contains_member_access(template_code, &ctx_access)
}

/// A generated member read must end at the binding name, not at a prefix of a
/// different binding (for example, `$setup.ItemTab` is not a read of `Item`).
fn contains_member_access(template_code: &str, access: &str) -> bool {
    template_code.match_indices(access).any(|(start, _)| {
        template_code
            .get(start + access.len()..)
            .is_some_and(|suffix| !suffix.chars().next().is_some_and(is_identifier_part))
    })
}

#[cfg(test)]
mod tests {
    use super::template_references_setup_binding;

    #[test]
    fn generated_member_reads_match_complete_binding_names() {
        for access in [
            "$setup.ItemTab",
            "_ctx.ItemTab",
            "$setup.Item_",
            "_ctx.Item2",
            "const π = 1; $setup.Item済",
        ] {
            assert!(
                !template_references_setup_binding(access, "Item"),
                "{access}"
            );
        }
        for access in [
            "$setup.Item",
            "$setup.Item.label",
            "$setup.Item[0]",
            "_ctx.Item",
            "_ctx.Item.label",
            "$setup[\"Item\"]",
            "const π = 1; $setup.Item",
        ] {
            assert!(
                template_references_setup_binding(access, "Item"),
                "{access}"
            );
        }
    }
}
