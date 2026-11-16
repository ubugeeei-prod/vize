//! Assemble the exact source ESM modules into the private native Node module.
//!
//! The validator shares the runtime's module scope: owned relative imports and
//! owned re-export lists are removed and every other relative import is
//! refused. Plugin modules are bundled the same way into modules the runtime
//! imports lazily from `data:` URLs, so a config without plugins never parses
//! or evaluates them. The config runtime carries only the plan host; the
//! tooling runtime (`vize plugin inspect`) also carries the plugin tools.
//! Built-in imports stay.

const VALIDATION: &str = include_str!("../vite-runtime-validation.js");
const REGISTRY: &str = include_str!("../plugin-registry.json");

/// Statically spliced into the runtime scope.
const OWNED: [(&str, &str); 1] = [("./vite-runtime-validation.js", VALIDATION)];

const CANONICAL: (&str, &str) = (
    "./plugin-canonical.js",
    include_str!("../plugin-canonical.js"),
);
const CAPABILITIES: (&str, &str) = (
    "./plugin-capabilities.js",
    include_str!("../plugin-capabilities.js"),
);
const PLAN: (&str, &str) = ("./plugin-plan.js", include_str!("../plugin-plan.js"));
const INSPECT: (&str, &str) = ("./plugin-inspect.js", include_str!("../plugin-inspect.js"));
const LINT: (&str, &str) = ("./plugin-lint.js", include_str!("../plugin-lint.js"));

/// `plugin-host.js`: everything config evaluation needs, in dependency order.
const PLUGIN_HOST: [(&str, &str); 3] = [CANONICAL, CAPABILITIES, PLAN];
/// `plugin-tools.js`: the host plus inspection and lint jobs, in dependency order.
const PLUGIN_TOOLS: [(&str, &str); 5] = [CANONICAL, CAPABILITIES, PLAN, INSPECT, LINT];

const PLUGIN_HOST_IMPORT: &str = "import(\"./plugin-host.js\")";
const PLUGIN_TOOLS_IMPORT: &str = "import(\"./plugin-tools.js\")";
const TOOLS_UNAVAILABLE: &str =
    "Promise.reject(new Error(\"plugin tools are not part of the config runtime\"))";

/// The config evaluation runtime.
pub(in crate::config::loader) fn vite_runtime() -> Result<vize_l0::String, &'static str> {
    assemble(super::VITE_RUNTIME, false)
}

/// The runtime of `vize plugin inspect` and the lint sidecar, which also carry the plugin tools.
pub(in crate::config::loader) fn vite_tools_runtime() -> Result<vize_l0::String, &'static str> {
    assemble(super::VITE_RUNTIME, true)
}

fn assemble(runtime: &str, tools: bool) -> Result<vize_l0::String, &'static str> {
    let mut source = vize_l0::String::with_capacity(VALIDATION.len() + runtime.len() + 1);
    for (_, module) in OWNED {
        append_module(module, &OWNED, true, &mut source)?;
        source.push('\n');
    }
    let mut body = vize_l0::String::with_capacity(runtime.len());
    append_module(runtime, &OWNED, false, &mut body)?;
    if body.contains(PLUGIN_HOST_IMPORT) {
        body = body
            .replace(PLUGIN_HOST_IMPORT, &lazy_import(&PLUGIN_HOST)?)
            .into();
    }
    if body.contains(PLUGIN_TOOLS_IMPORT) {
        let replacement = if tools {
            lazy_import(&PLUGIN_TOOLS)?
        } else {
            TOOLS_UNAVAILABLE.into()
        };
        body = body.replace(PLUGIN_TOOLS_IMPORT, &replacement).into();
    }
    source.push_str(&body);
    Ok(source)
}

/// A dynamic import of one bundled module. The source travels as a JSON string
/// literal and is URL-encoded only when the import actually runs.
fn lazy_import(modules: &[(&str, &str)]) -> Result<vize_l0::String, &'static str> {
    let mut bundle = vize_l0::String::default();
    for (_, module) in modules {
        append_module(module, modules, true, &mut bundle)?;
        bundle.push('\n');
    }
    let literal = serde_json::to_string(bundle.as_str())
        .map_err(|_| "unencodable plugin module in the product runtime")?;
    let mut import = vize_l0::String::from(
        "import(\"data:text/javascript;charset=utf-8,\" + encodeURIComponent(",
    );
    import.push_str(&literal);
    import.push_str("))");
    Ok(import)
}

/// Copy one module, resolving its relative imports against `owned`.
fn append_module(
    module: &str,
    owned: &[(&str, &str)],
    is_owned: bool,
    output: &mut vize_l0::String,
) -> Result<(), &'static str> {
    let mut rest = module;
    while !rest.is_empty() {
        let line_end = rest.find('\n').map_or(rest.len(), |index| index + 1);
        let Some(statement_end) = rest.starts_with("import ").then(|| import_end(rest)) else {
            let (line, tail) = rest.split_at(line_end);
            // A re-export list would export a shared binding twice.
            if !(is_owned && line.starts_with("export {") && line.trim_end().ends_with("};")) {
                output.push_str(line);
            }
            rest = tail;
            continue;
        };
        let statement_end = statement_end.ok_or("unterminated import in the product runtime")?;
        let (statement, tail) = rest.split_at(statement_end);
        rest = tail;
        let Some(specifier) = specifier(statement) else {
            return Err("unreadable import specifier in the product runtime");
        };
        if !specifier.starts_with("./") {
            output.push_str(statement);
        } else if specifier == "./plugin-registry.json" {
            let binding = statement
                .strip_prefix("import ")
                .and_then(|head| head.split_once(" from "))
                .map(|(binding, _)| binding.trim())
                .filter(|binding| {
                    binding
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_')
                })
                .ok_or("the plugin registry import must bind one default name")?;
            output.push_str("const ");
            output.push_str(binding);
            output.push_str(" = ");
            output.push_str(REGISTRY.trim_end());
            output.push_str(";\n");
        } else if !owned.iter().any(|(name, _)| *name == specifier) {
            return Err("unowned relative import in the product runtime");
        }
    }
    Ok(())
}

/// The end (after the newline) of the import statement at the start of `text`.
fn import_end(text: &str) -> Option<usize> {
    text.find(";\n").map(|index| index + 2).or_else(|| {
        text.strip_suffix(';')
            .map(|_| text.len())
            .filter(|_| !text.contains('\n'))
    })
}

fn specifier(statement: &str) -> Option<&str> {
    let (_, after) = statement.rsplit_once(" from \"")?;
    after.split_once('"').map(|(specifier, _)| specifier)
}

#[cfg(test)]
mod tests;
