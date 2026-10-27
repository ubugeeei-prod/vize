//! Static reads of literal Vite configs, without starting Node.
//!
//! When no project Vite or Vite+ is installed, the Node loader imports the
//! config module directly. A module whose only statement exports a JSON
//! literal evaluates to exactly that literal, so it is read with oxc and
//! projected natively, with the loader's observations replayed in its order.
//! Anything else returns `None` and goes to Node unchanged.

use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Expression, NumericLiteral, ObjectPropertyKind, PropertyKey, PropertyKind, Statement,
    StringLiteral, UnaryOperator,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use serde_json::{Map, Value};

use super::{BeforeConfigRead, js_cache::CachedEvaluation};

/// Loader engines in the order `importConfigModuleWithDependencies` probes them.
const ENGINES: [&str; 2] = ["vite", "vite-plus"];
/// Inherited by the Node child; each changes how or what it loads.
const NODE_INPUTS: [&str; 3] = [
    "NODE_PATH",
    "VIZE_EXPLICIT_VITE_CONFIG",
    "VIZE_EXPLICIT_NATIVE_CARRIER",
];

/// Evaluate `config` (already canonical) like the Node loader would, or `None`.
pub(super) fn evaluate(
    config: &Path,
    explicit: bool,
    before_read: &mut BeforeConfigRead<'_>,
) -> Option<CachedEvaluation> {
    if !static_runtime() {
        return None;
    }
    let name = config.file_name()?.to_str()?;
    let selected = if explicit {
        // The archived native carrier keeps its own Node branch.
        name.ends_with(".mjs") && name != "serialized-config.mjs"
            || name.ends_with(".js") && name != "serialized-config.js"
    } else {
        name == "vite.config.mjs" || name == "vite.config.js"
    };
    if !selected || std::fs::canonicalize(config).ok()? != config {
        return None;
    }
    let candidates = engine_candidates(config.parent()?);
    match before_read.as_deref_mut() {
        // Probe once unobserved, so a declined config leaves no observation
        // behind for the Node evaluation to repeat.
        Some(observe) => {
            read(config, explicit, &candidates, &mut |_| {})?;
            read(config, explicit, &candidates, observe)
        }
        None => read(config, explicit, &candidates, &mut |_| {}),
    }
}

/// Whether the Node child would import the config the same way. Posix paths
/// only, and a missing runtime keeps reporting its own error.
pub(super) fn static_runtime() -> bool {
    cfg!(unix)
        && NODE_INPUTS
            .iter()
            .all(|name| std::env::var_os(name).is_none())
        && inert_node_options(std::env::var_os("NODE_OPTIONS").as_deref())
        && which::which("node").is_ok()
}

/// Options that only silence warnings or size the heap leave the import as is.
pub(super) fn inert_node_options(options: Option<&std::ffi::OsStr>) -> bool {
    let Some(options) = options else {
        return true;
    };
    // Quoted or escaped values would need Node's own tokenizer.
    options.to_str().is_some_and(|options| {
        !options.contains(['"', '\'', '\\'])
            && options.split_ascii_whitespace().all(|option| {
                matches!(option, "--no-warnings" | "--no-deprecation")
                    || ["--disable-warning=", "--max-old-space-size="]
                        .iter()
                        .any(|prefix| {
                            option
                                .strip_prefix(prefix)
                                .is_some_and(|value| !value.is_empty())
                        })
            })
    })
}

/// `require.resolve.paths(engine)` without global paths: every ancestor's
/// `node_modules`, skipping ancestors that are themselves `node_modules`.
fn engine_candidates(directory: &Path) -> Vec<PathBuf> {
    let mut roots = directory
        .ancestors()
        .filter(|ancestor| ancestor.parent().is_some())
        .filter(|ancestor| {
            ancestor
                .file_name()
                .is_none_or(|name| name != "node_modules")
        })
        .map(|ancestor| ancestor.join("node_modules"))
        .collect::<Vec<_>>();
    roots.push(PathBuf::from("/node_modules"));
    ENGINES
        .iter()
        .flat_map(|engine| roots.iter().map(move |root| root.join(engine)))
        .collect()
}

fn read(
    config: &Path,
    explicit: bool,
    candidates: &[PathBuf],
    observe: &mut dyn FnMut(&[PathBuf]),
) -> Option<CachedEvaluation> {
    for candidate in candidates {
        observe(std::slice::from_ref(candidate));
        // Only an absent engine makes Node import the file itself.
        match std::fs::symlink_metadata(candidate) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            _ => return None,
        }
    }
    let path = config.to_path_buf();
    observe(std::slice::from_ref(&path));
    let source = std::fs::read_to_string(config).ok()?;
    if config
        .extension()
        .is_some_and(|extension| extension == "js")
        && !module_scope(config)
    {
        return None;
    }
    let Value::Object(source) = default_export(&source)? else {
        return None;
    };
    let root = config.parent()?.to_str()?;
    let projected = super::vite_projection::project(&source, explicit, root)?;
    let mut watch_dependencies = Vec::with_capacity(candidates.len() + 1);
    watch_dependencies.push(path);
    watch_dependencies.extend_from_slice(candidates);
    Some(CachedEvaluation {
        config: projected,
        complete: true,
        watch_dependencies,
        dependencies: vec![config.to_str()?.into()],
    })
}

/// Node's package scope for a `.js` file: the nearest `package.json` below
/// any `node_modules` boundary must declare `"type": "module"`.
fn module_scope(config: &Path) -> bool {
    for directory in config.ancestors().skip(1) {
        if directory
            .file_name()
            .is_some_and(|name| name == "node_modules")
        {
            return false;
        }
        match std::fs::read(directory.join("package.json")) {
            Ok(bytes) => {
                return serde_json::from_slice::<Value>(&bytes)
                    .is_ok_and(|manifest| manifest.get("type") == Some(&Value::from("module")));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return false,
        }
    }
    false
}

/// The value of a module whose only statement is `export default <literal>`.
fn default_export(source: &str) -> Option<Value> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() || !parsed.program.directives.is_empty() {
        return None;
    }
    let mut exported = None;
    for statement in &parsed.program.body {
        match statement {
            Statement::EmptyStatement(_) => {}
            Statement::ExportDefaultDeclaration(declaration) if exported.is_none() => {
                exported = Some(declaration.declaration.as_expression()?);
            }
            _ => return None,
        }
    }
    literal(exported?)
}

/// JSON-representable literals only: no identifiers, calls, templates,
/// spreads, computed or shorthand keys, accessors or methods.
fn literal(expression: &Expression<'_>) -> Option<Value> {
    Some(match expression {
        Expression::ObjectExpression(object) => {
            let mut map = Map::new();
            for property in &object.properties {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    return None;
                };
                if property.kind != PropertyKind::Init
                    || property.method
                    || property.shorthand
                    || property.computed
                {
                    return None;
                }
                let key = match &property.key {
                    PropertyKey::StaticIdentifier(name) => name.name.as_str(),
                    PropertyKey::StringLiteral(key) => strict_string(key)?,
                    _ => return None,
                };
                // `__proto__:` replaces the prototype instead of adding a key.
                if key == "__proto__" {
                    return None;
                }
                // Repeated keys keep the last value, as in JS.
                map.insert(key.into(), literal(&property.value)?);
            }
            Value::Object(map)
        }
        Expression::ArrayExpression(array) => Value::Array(
            array
                .elements
                .iter()
                .map(|element| literal(element.as_expression()?))
                .collect::<Option<_>>()?,
        ),
        Expression::StringLiteral(string) => Value::String(strict_string(string)?.into()),
        Expression::BooleanLiteral(boolean) => Value::Bool(boolean.value),
        Expression::NullLiteral(_) => Value::Null,
        Expression::NumericLiteral(number) => integer(number, 1.0)?,
        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::UnaryNegation => {
            match &unary.argument {
                Expression::NumericLiteral(number) => integer(number, -1.0)?,
                _ => return None,
            }
        }
        Expression::ParenthesizedExpression(inner) => literal(&inner.expression)?,
        _ => return None,
    })
}

/// Module code is strict, so legacy octal escapes are syntax errors there.
fn strict_string<'a>(string: &'a StringLiteral<'_>) -> Option<&'a str> {
    let raw = string.raw?.as_str().as_bytes();
    let octal = raw
        .windows(2)
        .any(|pair| matches!(pair, [b'\\', digit] if digit.is_ascii_digit()));
    (!string.lone_surrogates && !octal).then_some(string.value.as_str())
}

/// Safe integers only: `JSON.stringify` prints them as plain digits, which is
/// how serde reads the Node payload. Other numbers stay with Node.
fn integer(number: &NumericLiteral<'_>, sign: f64) -> Option<Value> {
    let legacy_octal = number.raw.is_some_and(
        |raw| matches!(raw.as_str().as_bytes(), [b'0', digit, ..] if digit.is_ascii_digit()),
    );
    let value = number.value * sign;
    if legacy_octal || value.fract() != 0.0 || value.abs() > 9_007_199_254_740_991.0 {
        return None;
    }
    Some(if value < 0.0 {
        Value::from(value as i64)
    } else {
        Value::from(value as u64)
    })
}
