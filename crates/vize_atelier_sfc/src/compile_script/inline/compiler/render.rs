use vize_carton::String;

use crate::script::ScriptCompileContext;

use super::super::super::{import_utils::extract_import_identifiers, TemplateParts};

const VAPOR_RENDER_ALIAS_BASE: &str = "__vaporRender";
const VAPOR_TEMPLATE_REF_SETTER: &str = "vaporTemplateRefSetter";

/// Emit the render function return statement or setup binding return.
pub(super) fn emit_render_return(
    output: &mut vize_carton::Vec<u8>,
    template: &TemplateParts<'_>,
    imports: &[String],
    is_ts: bool,
    is_vapor: bool,
    vapor_render_alias: Option<&str>,
    ctx: &ScriptCompileContext,
) {
    if !template.render_body.is_empty() {
        if is_ts {
            output.extend_from_slice(b"return (_ctx: any,_cache: any) => {\n");
        } else {
            output.extend_from_slice(b"return (_ctx, _cache) => {\n");
        }

        // Output component/directive resolution statements (preamble)
        for line in template.preamble.lines() {
            if !line.trim().is_empty() {
                output.extend_from_slice(b"  ");
                output.extend_from_slice(line.as_bytes());
                output.push(b'\n');
            }
        }
        if !template.preamble.is_empty() {
            output.push(b'\n');
        }

        if template.render_is_block {
            for line in template.render_body.lines() {
                if line.trim().is_empty() {
                    output.push(b'\n');
                    continue;
                }

                output.extend_from_slice(b"  ");
                output.extend_from_slice(line.as_bytes());
                output.push(b'\n');
            }
        } else {
            // Indent the render body properly
            let mut first_line = true;
            for line in template.render_body.lines() {
                if first_line {
                    output.extend_from_slice(b"  return ");
                    output.extend_from_slice(line.as_bytes());
                    first_line = false;
                } else {
                    output.push(b'\n');
                    // Preserve existing indentation by adding 2 spaces (setup indent)
                    if !line.trim().is_empty() {
                        output.extend_from_slice(b"  ");
                    }
                    output.extend_from_slice(line.as_bytes());
                }
            }
            if first_line {
                output.extend_from_slice(b"  return null");
            }
            output.push(b'\n');
        }
        output.extend_from_slice(b"}\n");
    } else {
        let setup_bindings = collect_setup_bindings(ctx, imports, template);
        if is_vapor && !template.render_fn.is_empty() {
            let needs_template_ref_setter = template.render_fn.contains("_createTemplateRefSetter");
            if needs_template_ref_setter {
                output.extend_from_slice(b"const ");
                output.extend_from_slice(VAPOR_TEMPLATE_REF_SETTER.as_bytes());
                output.extend_from_slice(b" = _createTemplateRefSetter()\n");
            }
            output.extend_from_slice(b"const __returned__ = { ");
            let mut binding_index = 0usize;
            if needs_template_ref_setter {
                output.extend_from_slice(VAPOR_TEMPLATE_REF_SETTER.as_bytes());
                binding_index += 1;
            }
            for name in setup_bindings.iter() {
                if binding_index > 0 {
                    output.extend_from_slice(b", ");
                }
                binding_index += 1;
                output.extend_from_slice(name.as_bytes());
            }
            output.extend_from_slice(b" }\n");
            output.extend_from_slice(b"Object.defineProperty(__returned__, '__isScriptSetup', { enumerable: false, value: true })\n");
            output.extend_from_slice(b"const __instance = _getCurrentInstance()\n");
            output.extend_from_slice(b"const __ctx = _proxyRefs(__returned__)\n");
            output.extend_from_slice(b"if (__instance) __instance.setupState = __ctx\n");
            output.extend_from_slice(b"return ");
            output.extend_from_slice(vapor_render_alias.unwrap_or("render").as_bytes());
            output.extend_from_slice(b"(__ctx, __props, __emit, __attrs, __slots)\n");
        } else if !setup_bindings.is_empty() {
            // No template (e.g., Musea art files) -- return setup bindings as an object
            // so they're accessible for runtime template compilation (compileToFunction).
            output.extend_from_slice(b"return { ");
            for (i, name) in setup_bindings.iter().enumerate() {
                if i > 0 {
                    output.extend_from_slice(b", ");
                }
                output.extend_from_slice(name.as_bytes());
            }
            output.extend_from_slice(b" }\n");
        } else if !template.render_fn.is_empty() {
            output.extend_from_slice(b"return {}\n");
        }
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
    let mut buffer = [0u8; 20];
    let mut index = buffer.len();
    let mut remaining = value;

    loop {
        index -= 1;
        buffer[index] = b'0' + (remaining % 10) as u8;
        remaining /= 10;
        if remaining == 0 {
            break;
        }
    }

    let digits = std::str::from_utf8(&buffer[index..]).expect("usize digits should be ASCII");
    target.push_str(digits);
}

fn collect_setup_bindings(
    ctx: &ScriptCompileContext,
    imports: &[String],
    template: &TemplateParts<'_>,
) -> Vec<String> {
    use crate::types::BindingType;

    let mut bindings: Vec<String> = ctx
        .bindings
        .bindings
        .iter()
        .filter(|(_, bt)| {
            matches!(
                bt,
                BindingType::SetupLet
                    | BindingType::SetupMaybeRef
                    | BindingType::SetupRef
                    | BindingType::SetupReactiveConst
                    | BindingType::SetupConst
                    | BindingType::LiteralConst
            )
        })
        .map(|(name, _)| String::from(name.as_str()))
        .collect();

    let has_template = !template.render_fn.is_empty() || !template.render_body.is_empty();
    let mut template_code =
        String::with_capacity(template.render_fn.len() + template.render_body.len());
    template_code.push_str(template.render_fn);
    template_code.push_str(template.render_body);

    for import in imports {
        for name in extract_import_identifiers(import) {
            let should_return =
                !has_template || template_references_setup_binding(&template_code, name.as_str());
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
    if template_code.contains(setup_access.as_str()) {
        return true;
    }

    let mut ctx_access = String::with_capacity(name.len() + 5);
    ctx_access.push_str("_ctx.");
    ctx_access.push_str(name);
    template_code.contains(ctx_access.as_str())
}
