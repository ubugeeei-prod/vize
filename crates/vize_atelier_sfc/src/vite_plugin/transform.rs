use std::path::{Component, Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, ArrayExpressionElement, CallExpression, Expression, StringLiteral};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{SmallVec, String};

use super::js_string::push_js_string_literal;

/// Alias rewrite rule used by Vite integration helpers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DynamicImportAliasRule {
    /// Prefix to match in compiled code.
    pub from_prefix: String,
    /// Prefix to write back for browser/runtime imports.
    pub to_prefix: String,
}

/// Vite define replacement entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DefineReplacement {
    /// Define key to replace.
    pub key: String,
    /// Stringified replacement value.
    pub value: String,
}

struct StringLiteralReplacement {
    start: usize,
    end: usize,
    value: String,
}

const BUILTIN_DEFINE_PREFIXES: [&str; 10] = [
    "import.meta.server",
    "import.meta.client",
    "import.meta.dev",
    "import.meta.test",
    "import.meta.prerender",
    "import.meta.env",
    "import.meta.hot",
    "__VUE_",
    "__NUXT_",
    "process.env",
];

const VIRTUAL_MODULE_DEFINE_KEYS: [&str; 5] = [
    "import.meta.server",
    "import.meta.client",
    "import.meta.dev",
    "import.meta.test",
    "import.meta.prerender",
];

/// Rewrite relative `import.meta.glob` literals in Vize virtual modules.
///
/// Vite resolves relative glob patterns against the importer ID. Vize virtual
/// modules are `\0` IDs, so Vite requires globs to be rooted before it sees
/// the module. This keeps the original SFC path as the base while preserving
/// Vite's own import-glob transform for the rest of the work.
pub fn rewrite_import_meta_glob_base(code: &str, importer: &str, root: &str) -> String {
    if !code.contains("import.meta.glob") {
        return String::from(code);
    }

    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, code, SourceType::tsx().with_module(true)).parse();
    if !parsed.errors.is_empty() {
        return String::from(code);
    }

    let mut collector = ImportMetaGlobCollector {
        importer,
        root,
        replacements: Vec::new(),
    };
    collector.visit_program(&parsed.program);
    apply_string_literal_replacements(code, collector.replacements)
}

/// Rewrite static asset `src` values in compiled render output into imports.
pub fn rewrite_static_asset_urls(code: &str, alias_rules: &[DynamicImportAliasRule]) -> String {
    if alias_rules.is_empty() {
        return String::from(code);
    }

    let bytes = code.as_bytes();
    let mut output = String::with_capacity(code.len());
    let mut imports: SmallVec<[String; 4]> = SmallVec::new();
    let mut counter = 0usize;
    let mut last = 0usize;
    let mut cursor = 0usize;

    while cursor < bytes.len() {
        let Some(candidate) = parse_src_candidate(code, cursor) else {
            cursor += 1;
            continue;
        };

        let full_path = &code[candidate.value_start..candidate.value_end];
        if !is_script_asset(full_path)
            && alias_rules
                .iter()
                .any(|rule| full_path.starts_with(rule.from_prefix.as_str()))
        {
            output.push_str(&code[last..candidate.prefix_start]);
            output.push_str(&code[candidate.prefix_start..candidate.value_prefix_end]);
            let var_name = push_static_import(&mut imports, counter, full_path);
            output.push_str(var_name.as_str());
            counter += 1;
            cursor = candidate.value_end + 1;
            last = cursor;
        } else {
            cursor = candidate.value_end + 1;
        }
    }

    if counter == 0 {
        return String::from(code);
    }

    output.push_str(&code[last..]);
    let import_bytes = imports
        .iter()
        .fold(0usize, |acc, import| acc + import.len() + 1);
    let mut rewritten = String::with_capacity(import_bytes + output.len());
    for import in imports {
        rewritten.push_str(import.as_str());
        rewritten.push('\n');
    }
    rewritten.push_str(output.as_str());
    rewritten
}

struct ImportMetaGlobCollector<'a> {
    importer: &'a str,
    root: &'a str,
    replacements: Vec<StringLiteralReplacement>,
}

impl ImportMetaGlobCollector<'_> {
    fn collect_argument<'a>(&mut self, argument: &Argument<'a>) {
        match argument {
            Argument::StringLiteral(literal) => self.collect_string_literal(literal),
            Argument::ArrayExpression(array) => {
                for element in array.elements.iter() {
                    if let ArrayExpressionElement::StringLiteral(literal) = element {
                        self.collect_string_literal(literal);
                    }
                }
            }
            _ => {}
        }
    }

    fn collect_string_literal(&mut self, literal: &StringLiteral<'_>) {
        let value = literal.value.as_str();
        let Some(rewritten) = normalize_import_meta_glob_pattern(value, self.importer, self.root)
        else {
            return;
        };

        self.replacements.push(StringLiteralReplacement {
            start: literal.span.start as usize,
            end: literal.span.end as usize,
            value: rewritten,
        });
    }
}

impl<'a> Visit<'a> for ImportMetaGlobCollector<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if is_import_meta_glob_call(call)
            && let Some(argument) = call.arguments.first()
        {
            self.collect_argument(argument);
        }
        walk::walk_call_expression(self, call);
    }
}

fn is_import_meta_glob_call(call: &CallExpression<'_>) -> bool {
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return false;
    };
    member.property.name.as_str() == "glob" && is_import_meta_expression(&member.object)
}

fn is_import_meta_expression(expression: &Expression<'_>) -> bool {
    let Expression::MetaProperty(meta) = expression else {
        return false;
    };
    meta.meta.name.as_str() == "import" && meta.property.name.as_str() == "meta"
}

fn normalize_import_meta_glob_pattern(pattern: &str, importer: &str, root: &str) -> Option<String> {
    let (negated, pattern) = pattern
        .strip_prefix('!')
        .map_or((false, pattern), |pattern| (true, pattern));
    if !pattern.starts_with("./") && !pattern.starts_with("../") {
        return None;
    }

    let importer_parent = Path::new(importer)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let absolute_pattern = lexical_normalize(&importer_parent.join(pattern));
    let absolute_root = lexical_normalize(Path::new(if root.is_empty() { "." } else { root }));

    let mut rewritten = String::with_capacity(pattern.len() + 2);
    if negated {
        rewritten.push('!');
    }

    if let Ok(relative) = absolute_pattern.strip_prefix(&absolute_root) {
        rewritten.push('/');
        let relative = normalize_path_for_import(relative);
        rewritten.push_str(relative.as_str());
    } else {
        rewritten.push_str(normalize_path_for_import(&absolute_pattern).as_str());
    }

    Some(rewritten)
}

fn normalize_path_for_import(path: &Path) -> String {
    normalize_slashes(path.to_string_lossy().as_ref())
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    let mut normal_count = 0usize;

    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if normal_count > 0 {
                    normal_count -= 1;
                    normalized.pop();
                } else if !normalized.is_absolute() {
                    normalized.push(component.as_os_str());
                }
            }
            Component::Normal(part) => {
                normal_count += 1;
                normalized.push(part);
            }
        }
    }

    normalized
}

fn apply_string_literal_replacements(
    code: &str,
    mut replacements: Vec<StringLiteralReplacement>,
) -> String {
    if replacements.is_empty() {
        return String::from(code);
    }

    replacements.sort_by_key(|replacement| replacement.start);
    let mut output = String::with_capacity(code.len());
    let mut last = 0usize;
    let mut changed = false;

    for replacement in replacements {
        if replacement.start < last || replacement.end > code.len() {
            continue;
        }
        output.push_str(&code[last..replacement.start]);
        push_js_string_literal(&mut output, replacement.value.as_str());
        last = replacement.end;
        changed = true;
    }

    if !changed {
        return String::from(code);
    }

    output.push_str(&code[last..]);
    output
}

/// Rewrite dynamic template imports so Vite leaves runtime expressions alone.
pub fn rewrite_dynamic_template_imports(
    code: &str,
    alias_rules: &[DynamicImportAliasRule],
) -> String {
    let bytes = code.as_bytes();
    let mut output = String::with_capacity(code.len());
    let mut last = 0usize;
    let mut cursor = 0usize;

    while cursor < bytes.len() {
        if !is_import_boundary(bytes, cursor) {
            cursor += 1;
            continue;
        }

        let Some(open_paren) = skip_ascii_ws(bytes, cursor + "import".len())
            .filter(|idx| bytes.get(*idx) == Some(&b'('))
        else {
            cursor += 1;
            continue;
        };
        let Some(backtick) =
            skip_ascii_ws(bytes, open_paren + 1).filter(|idx| bytes.get(*idx) == Some(&b'`'))
        else {
            cursor += 1;
            continue;
        };

        output.push_str(&code[last..cursor]);
        output.push_str("import(/* @vite-ignore */ `");

        let template_start = backtick + 1;
        if let Some(rule) = alias_rules
            .iter()
            .find(|rule| code[template_start..].starts_with(rule.from_prefix.as_str()))
        {
            output.push_str(rule.to_prefix.as_str());
            cursor = template_start + rule.from_prefix.len();
        } else {
            cursor = template_start;
        }
        last = cursor;
    }

    if last == 0 {
        return String::from(code);
    }

    output.push_str(&code[last..]);
    output
}

/// Returns whether a define key is normally provided by Vite/Vue/Nuxt.
pub fn is_builtin_define(key: &str) -> bool {
    BUILTIN_DEFINE_PREFIXES
        .iter()
        .any(|prefix| key == *prefix || has_prefixed_define_key(key, prefix))
}

/// Returns whether a define should be applied inside Vize virtual modules.
pub fn should_apply_define_in_virtual_module(key: &str) -> bool {
    VIRTUAL_MODULE_DEFINE_KEYS.contains(&key) || !is_builtin_define(key)
}

/// Apply Vite define replacements to generated virtual module code.
pub fn apply_define_replacements(code: &str, defines: &[DefineReplacement]) -> String {
    if defines.is_empty() {
        return String::from(code);
    }

    let mut sorted: SmallVec<[&DefineReplacement; 8]> = defines.iter().collect();
    sorted.sort_by_key(|define| std::cmp::Reverse(define.key.len()));

    let mut current = String::from(code);
    for define in sorted {
        if define.key.is_empty() || !current.contains(define.key.as_str()) {
            continue;
        }
        current = replace_define_key(current.as_str(), define.key.as_str(), define.value.as_str());
    }

    current
}

/// Normalize an alias target for browser-side Vite imports.
pub fn to_browser_import_prefix(replacement: &str) -> String {
    let normalized = normalize_slashes(replacement);
    if normalized.starts_with("/@fs/") {
        return normalized;
    }
    let path = std::path::Path::new(replacement);
    if path.is_absolute() && path.exists() {
        let mut output = String::with_capacity(normalized.len() + "/@fs".len());
        output.push_str("/@fs");
        output.push_str(normalized.as_str());
        return output;
    }
    normalized
}

struct SrcCandidate {
    prefix_start: usize,
    value_prefix_end: usize,
    value_start: usize,
    value_end: usize,
}

fn parse_src_candidate(code: &str, cursor: usize) -> Option<SrcCandidate> {
    let bytes = code.as_bytes();
    let key_end =
        if bytes.get(cursor) == Some(&b's') && bytes.get(cursor..cursor + 3) == Some(&b"src"[..]) {
            cursor + 3
        } else if bytes.get(cursor) == Some(&b'"')
            && bytes.get(cursor + 1..cursor + 4) == Some(&b"src"[..])
            && bytes.get(cursor + 4) == Some(&b'"')
        {
            cursor + 5
        } else {
            return None;
        };

    let colon = skip_ascii_ws(bytes, key_end)?;
    if bytes.get(colon) != Some(&b':') {
        return None;
    }
    let quote_index = skip_ascii_ws(bytes, colon + 1)?;
    let quote = *bytes.get(quote_index)?;
    if quote != b'\'' && quote != b'"' {
        return None;
    }

    let value_start = quote_index + 1;
    let mut value_end = value_start;
    while value_end < bytes.len() && bytes[value_end] != quote {
        value_end += 1;
    }
    (value_end < bytes.len()).then_some(SrcCandidate {
        prefix_start: cursor,
        value_prefix_end: quote_index,
        value_start,
        value_end,
    })
}

fn push_static_import(imports: &mut SmallVec<[String; 4]>, counter: usize, path: &str) -> String {
    let mut var_name = String::with_capacity("__vize_static_".len() + 20);
    var_name.push_str("__vize_static_");
    push_usize(&mut var_name, counter);

    let mut import = String::with_capacity("import  from ;".len() + var_name.len() + path.len());
    import.push_str("import ");
    import.push_str(var_name.as_str());
    import.push_str(" from ");
    push_js_string_literal(&mut import, path);
    import.push(';');
    imports.push(import);
    var_name
}

fn push_usize(output: &mut String, mut value: usize) {
    let mut buffer = [0u8; 20];
    let mut cursor = buffer.len();
    loop {
        cursor -= 1;
        buffer[cursor] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    output.push_str(std::str::from_utf8(&buffer[cursor..]).unwrap_or(""));
}

fn normalize_slashes(value: &str) -> String {
    if !value.as_bytes().contains(&b'\\') {
        return String::from(value);
    }

    let mut normalized = String::with_capacity(value.len());
    for char in value.chars() {
        normalized.push(if char == '\\' { '/' } else { char });
    }
    normalized
}

fn is_script_asset(path: &str) -> bool {
    const SCRIPT_EXTENSIONS: [&[u8]; 8] = [
        b".js", b".mjs", b".cjs", b".ts", b".mts", b".cts", b".jsx", b".tsx",
    ];
    let bytes = path.as_bytes();
    SCRIPT_EXTENSIONS
        .iter()
        .any(|suffix| ends_with_ignore_ascii_case(bytes, suffix))
}

fn ends_with_ignore_ascii_case(value: &[u8], suffix: &[u8]) -> bool {
    value.len() >= suffix.len()
        && value[value.len() - suffix.len()..]
            .iter()
            .zip(suffix.iter())
            .all(|(left, right)| left.eq_ignore_ascii_case(right))
}

fn is_import_boundary(bytes: &[u8], cursor: usize) -> bool {
    bytes.get(cursor..cursor + "import".len()) == Some(&b"import"[..])
        && cursor
            .checked_sub(1)
            .and_then(|idx| bytes.get(idx))
            .is_none_or(|byte| !is_identifier_byte(*byte))
}

fn skip_ascii_ws(bytes: &[u8], mut cursor: usize) -> Option<usize> {
    while matches!(bytes.get(cursor), Some(b' ' | b'\t' | b'\n' | b'\r')) {
        cursor += 1;
    }
    (cursor < bytes.len()).then_some(cursor)
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

fn has_prefixed_define_key(key: &str, prefix: &str) -> bool {
    key.strip_prefix(prefix)
        .and_then(|suffix| suffix.as_bytes().first().copied())
        .is_some_and(|byte| byte == b'.' || byte == b'_')
}

fn replace_define_key(input: &str, key: &str, value: &str) -> String {
    let input_bytes = input.as_bytes();
    let key_bytes = key.as_bytes();
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0usize;
    let mut last = 0usize;
    let mut changed = false;

    while cursor + key_bytes.len() <= input_bytes.len() {
        if &input_bytes[cursor..cursor + key_bytes.len()] == key_bytes
            && input_bytes
                .get(cursor + key_bytes.len())
                .is_none_or(|byte| !is_define_tail_byte(*byte))
        {
            output.push_str(&input[last..cursor]);
            output.push_str(value);
            cursor += key_bytes.len();
            last = cursor;
            changed = true;
        } else {
            cursor += 1;
        }
    }

    if !changed {
        return String::from(input);
    }

    output.push_str(&input[last..]);
    output
}

fn is_define_tail_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$' || byte == b'.'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_static_asset_src_values() {
        let rules = [DynamicImportAliasRule {
            from_prefix: "@/".into(),
            to_prefix: "/src/".into(),
        }];
        let code = r#"const node = { src: "@/assets/logo.svg", other: true };"#;

        insta::assert_snapshot!(rewrite_static_asset_urls(code, &rules), @r###"
        import __vize_static_0 from "@/assets/logo.svg";
        const node = { src: __vize_static_0, other: true };
        "###);
    }

    #[test]
    fn skips_script_asset_src_values() {
        let rules = [DynamicImportAliasRule {
            from_prefix: "@/".into(),
            to_prefix: "/src/".into(),
        }];
        let code = r#"const node = { src: "@/entry.ts" };"#;

        assert_eq!(rewrite_static_asset_urls(code, &rules), code);
    }

    #[test]
    fn rewrites_dynamic_template_imports() {
        let rules = [DynamicImportAliasRule {
            from_prefix: "@/".into(),
            to_prefix: "/src/".into(),
        }];
        let code = "const image = import(`@/assets/${name}.svg`);";

        assert_eq!(
            rewrite_dynamic_template_imports(code, &rules).as_str(),
            "const image = import(/* @vite-ignore */ `/src/assets/${name}.svg`);"
        );
    }

    #[test]
    fn rewrites_import_meta_glob_relative_patterns() {
        let code = r#"const modules = import.meta.glob("./demos/*.vue", { eager: true });"#;

        assert_eq!(
            rewrite_import_meta_glob_base(code, "/project/src/App.vue", "/project").as_str(),
            r#"const modules = import.meta.glob("/src/demos/*.vue", { eager: true });"#
        );
    }

    #[test]
    fn rewrites_import_meta_glob_array_and_negated_patterns() {
        let code = r#"const modules = import.meta.glob<{ default: unknown }>(["./demos/*.vue", "!../legacy/*.vue", "/src/stable/*.vue"]);"#;

        assert_eq!(
            rewrite_import_meta_glob_base(code, "/project/src/App.vue", "/project").as_str(),
            r#"const modules = import.meta.glob<{ default: unknown }>(["/src/demos/*.vue", "!/legacy/*.vue", "/src/stable/*.vue"]);"#
        );
    }

    #[test]
    fn skips_non_calls_and_non_relative_import_meta_globs() {
        let code = r#"const text = "import.meta.glob('./demos/*.vue')"; const modules = import.meta.glob("/src/demos/*.vue");"#;

        assert_eq!(
            rewrite_import_meta_glob_base(code, "/project/src/App.vue", "/project").as_str(),
            code
        );
    }

    #[test]
    fn applies_define_replacements_longest_first() {
        let defines = [
            DefineReplacement {
                key: "import.meta.env".into(),
                value: "{}".into(),
            },
            DefineReplacement {
                key: "import.meta.env.MODE".into(),
                value: "\"test\"".into(),
            },
        ];

        assert_eq!(
            apply_define_replacements("const mode = import.meta.env.MODE;", &defines).as_str(),
            "const mode = \"test\";"
        );
    }
}
