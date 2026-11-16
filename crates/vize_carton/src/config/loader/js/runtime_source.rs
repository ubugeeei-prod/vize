//! Assemble the exact source ESM modules into the private native Node module.
//!
//! The validator shares the runtime's module scope: owned relative imports and
//! owned re-export lists are removed and every other relative import is
//! refused. The plugin host is bundled the same way into one module that the
//! runtime imports lazily from a `data:` URL, so a config without plugins never
//! parses or evaluates it. Built-in imports stay.

const VALIDATION: &str = include_str!("../vite-runtime-validation.js");
const REGISTRY: &str = include_str!("../plugin-registry.json");

/// Statically spliced into the runtime scope.
const OWNED: [(&str, &str); 1] = [("./vite-runtime-validation.js", VALIDATION)];

/// The lazily imported plugin host, in dependency order. Together these export
/// every name `plugin-host.js` re-exports.
const PLUGIN_HOST: [(&str, &str); 4] = [
    (
        "./plugin-canonical.js",
        include_str!("../plugin-canonical.js"),
    ),
    (
        "./plugin-capabilities.js",
        include_str!("../plugin-capabilities.js"),
    ),
    ("./plugin-plan.js", include_str!("../plugin-plan.js")),
    ("./plugin-inspect.js", include_str!("../plugin-inspect.js")),
];
const PLUGIN_HOST_IMPORT: &str = "import(\"./plugin-host.js\")";

pub(in crate::config::loader) fn vite_runtime() -> Result<vize_l0::String, &'static str> {
    assemble(super::VITE_RUNTIME)
}

fn assemble(runtime: &str) -> Result<vize_l0::String, &'static str> {
    let mut source = vize_l0::String::with_capacity(VALIDATION.len() + runtime.len() + 1);
    for (_, module) in OWNED {
        append_module(module, &OWNED, true, &mut source)?;
        source.push('\n');
    }
    let mut body = vize_l0::String::with_capacity(runtime.len());
    append_module(runtime, &OWNED, false, &mut body)?;
    if body.contains(PLUGIN_HOST_IMPORT) {
        let mut bundle = vize_l0::String::default();
        for (_, module) in PLUGIN_HOST {
            append_module(module, &PLUGIN_HOST, true, &mut bundle)?;
            bundle.push('\n');
        }
        let mut url = vize_l0::String::from("import(\"data:text/javascript;base64,");
        base64(bundle.as_bytes(), &mut url);
        url.push_str("\")");
        body = body.replace(PLUGIN_HOST_IMPORT, &url).into();
    }
    source.push_str(&body);
    Ok(source)
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

/// Standard padded base64, as `Buffer.prototype.toString("base64")` writes it.
fn base64(bytes: &[u8], output: &mut vize_l0::String) {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let symbol =
        |index: u32| char::from(ALPHABET.get(index as usize & 63).copied().unwrap_or(b'='));
    for chunk in bytes.chunks(3) {
        let [first, second, third] = [0, 1, 2].map(|index| chunk.get(index).copied());
        let value = (u32::from(first.unwrap_or(0)) << 16)
            | (u32::from(second.unwrap_or(0)) << 8)
            | u32::from(third.unwrap_or(0));
        output.push(symbol(value >> 18));
        output.push(symbol(value >> 12));
        output.push(if second.is_some() {
            symbol(value >> 6)
        } else {
            '='
        });
        output.push(if third.is_some() { symbol(value) } else { '=' });
    }
}

#[cfg(test)]
mod tests;
