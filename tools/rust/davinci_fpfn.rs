#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[path = "./common.rs"]
mod common;

pub const CLASS_A: &str = "undefined-template-ref";
pub const CLASS_B: &str = "unused-binding";
pub const CLASS_A_RULE: &str = "vue/no-undefined-refs";
pub const SEEDED_NAME_SUFFIX: &str = "__davinci_seeded";
pub const UNUSED_BINDING_NAME: &str = "__davinci_seeded_unused";
pub const UNUSED_BINDING_STATEMENT: &str = "const __davinci_seeded_unused = 0;\n";
pub const RULE_MAP_FIXTURE: &str = "tests/_fixtures/patina-eslint-vue-rule-map.json";
pub const CORPUS_SHARD: [&str; 3] = ["splitpanes", "layoutit-grid", "cssgridgenerator"];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceInfo {
    pub kind: String,
    pub label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeedScope {
    pub files_copied: usize,
    pub class_a_eligible: usize,
    pub class_a_injections: usize,
    pub class_b_injections: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeedFile {
    pub path: String,
    pub class_a: bool,
    pub class_b: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_a_reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Identifier {
    pub original: Option<String>,
    pub seeded: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpanDescription {
    pub span: [usize; 2],
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Injection {
    #[serde(rename = "class")]
    pub class_name: String,
    pub path: String,
    pub expected_rule: Option<String>,
    pub identifier: Identifier,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_rename_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_script_setup_block: Option<bool>,
    pub expected: SpanDescription,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditRecord {
    pub span: [usize; 2],
    pub delta: isize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeedManifest {
    pub schema_version: usize,
    pub tool: String,
    pub source: SourceInfo,
    pub scope: SeedScope,
    pub files: Vec<SeedFile>,
    pub injections: Vec<Injection>,
    pub edits: BTreeMap<String, Vec<EditRecord>>,
}

#[derive(Clone, Debug)]
pub struct SourceRoot {
    pub root: PathBuf,
    pub prefix: String,
}

#[derive(Clone, Debug)]
pub struct ResolvedSources {
    pub kind: String,
    pub label: String,
    pub roots: Vec<SourceRoot>,
}

#[derive(Clone, Debug)]
pub struct ClassAPlan {
    pub name: String,
    pub seeded_name: String,
    pub rename_spans: Vec<[usize; 2]>,
    pub template_ref: [usize; 2],
}

#[derive(Clone, Debug)]
pub struct ClassBPlan {
    pub insert_at: usize,
    pub insert_text: String,
    pub created_block: bool,
}

#[derive(Clone, Debug)]
pub struct AppliedSeed {
    pub seeded: String,
    pub edits: Vec<EditRecord>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticRow {
    pub path: String,
    pub rule_id: String,
    pub severity: i64,
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

impl Ord for DiagnosticRow {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path
            .cmp(&other.path)
            .then(self.line.cmp(&other.line))
            .then(self.column.cmp(&other.column))
            .then(self.end_line.cmp(&other.end_line))
            .then(self.end_column.cmp(&other.end_column))
            .then(self.rule_id.cmp(&other.rule_id))
            .then(self.severity.cmp(&other.severity))
    }
}

impl PartialOrd for DiagnosticRow {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassAMiss {
    pub path: String,
    pub rule_id: String,
    pub severity: i64,
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub identifier: String,
}

impl From<(&DiagnosticRow, String)> for ClassAMiss {
    fn from((row, identifier): (&DiagnosticRow, String)) -> Self {
        Self {
            path: row.path.clone(),
            rule_id: row.rule_id.clone(),
            severity: row.severity,
            line: row.line,
            column: row.column,
            end_line: row.end_line,
            end_column: row.end_column,
            identifier,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LintCounts {
    pub baseline_diagnostics: usize,
    pub seeded_diagnostics: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClassAReport {
    pub expected: usize,
    pub detected: usize,
    pub misses: Vec<ClassAMiss>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClassBReport {
    pub expected: usize,
    pub detected: usize,
    pub note: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BaselineShiftReport {
    pub mapped: usize,
    pub misses: Vec<DiagnosticRow>,
    pub unmappable: Vec<DiagnosticRow>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeedAssertReport {
    pub schema_version: usize,
    pub tool: String,
    pub source: SourceInfo,
    pub scope: SeedScope,
    pub lint: LintCounts,
    pub class_a: ClassAReport,
    pub class_b: ClassBReport,
    pub baseline_shift: BaselineShiftReport,
    pub unexpected: Vec<DiagnosticRow>,
    pub verdict: String,
}

#[derive(Clone, Debug)]
pub struct VizeCli {
    pub command: String,
    pub prefix: Vec<String>,
}

#[derive(Clone, Debug)]
struct SfcBlock {
    attrs: String,
    content: String,
    content_start: usize,
}

#[derive(Clone, Debug)]
struct SfcBlocks {
    script_setup: Option<SfcBlock>,
    template: Option<SfcBlock>,
}

#[derive(Clone, Debug)]
struct TemplateSegment {
    start: usize,
    text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleMapReport {
    pub fixture: String,
    pub mapped_rules: usize,
    pub core_sidecar_rules: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuppressionScope {
    pub files_scanned: usize,
    pub suppression_comments: usize,
    pub named_suppressions: usize,
    pub bare_suppressions: usize,
    pub rule_names_seen: usize,
    pub mapped_names_seen: usize,
    pub unmapped_names_seen: usize,
    pub defused_run_diagnostics: usize,
    pub diagnostics_on_bare_suppressed_lines: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnmappedSuppression {
    pub rule: String,
    pub occurrences: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuppressionCandidate {
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub severity: i64,
    pub vize_rule: String,
    pub eslint_rule: String,
    pub comment_line: usize,
    pub kind: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuppressionReport {
    pub schema_version: usize,
    pub tool: String,
    pub source: SourceInfo,
    pub rule_map: RuleMapReport,
    pub scope: SuppressionScope,
    pub unmapped: Vec<UnmappedSuppression>,
    pub candidates: Vec<SuppressionCandidate>,
}

#[derive(Clone, Debug)]
pub struct SuppressionComment {
    pub line: usize,
    pub kind: String,
    pub rules: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct SuppressionRange {
    pub rule: Option<String>,
    pub start_line: usize,
    pub end_line: Option<usize>,
    pub comment_line: usize,
    pub kind: String,
}

#[derive(Clone, Debug)]
pub struct SuppressionScan {
    pub comments: Vec<SuppressionComment>,
    pub ranges: Vec<SuppressionRange>,
}

#[derive(Clone, Debug)]
pub struct RuleMap {
    pub mapped: BTreeMap<String, String>,
    pub fixture_path: String,
    pub fixture_mapped_count: usize,
    pub core_sidecar_count: usize,
}

pub fn list_vue_files(dir: &Path) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    walk_vue_files(dir, dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn walk_vue_files(root: &Path, dir: &Path, files: &mut Vec<String>) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    let mut entries = fs::read_dir(dir)
        .map_err(|error| format!("cannot read {}: {error}", dir.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("cannot read {}: {error}", dir.display()))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "node_modules" || name.starts_with('.') {
            continue;
        }
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| format!("cannot stat {}: {error}", path.display()))?;
        if file_type.is_dir() {
            walk_vue_files(root, &path, files)?;
        } else if file_type.is_file()
            && path.extension().and_then(|ext| ext.to_str()) == Some("vue")
        {
            files.push(common::relative_path(root, &path));
        }
    }
    Ok(())
}

pub fn shard_project_dir(repo_root: &Path, id: &str) -> PathBuf {
    repo_root.join("tests/_fixtures/_git").join(id)
}

pub fn resolve_fixture_sources(
    repo_root: &Path,
    fixtures: &Path,
) -> Result<ResolvedSources, String> {
    if !fixtures.exists() {
        return Err(format!(
            "--fixtures directory not found: {}",
            fixtures.display()
        ));
    }
    let label = common::relative_path(repo_root, fixtures);
    Ok(ResolvedSources {
        kind: "fixtures".to_string(),
        label,
        roots: vec![SourceRoot {
            root: fixtures.to_path_buf(),
            prefix: String::new(),
        }],
    })
}

pub fn resolve_corpus_sources(repo_root: &Path) -> Result<ResolvedSources, String> {
    let mut roots = Vec::new();
    for id in CORPUS_SHARD {
        let root = shard_project_dir(repo_root, id);
        if !root.exists() || list_vue_files(&root)?.is_empty() {
            return Err(format!(
                "corpus shard project {id} is not hydrated. Run:\n  git submodule update --init --depth 1 -- tests/_fixtures/_git/{id}"
            ));
        }
        roots.push(SourceRoot {
            root,
            prefix: format!("{id}/"),
        });
    }
    Ok(ResolvedSources {
        kind: "corpus-shard".to_string(),
        label: CORPUS_SHARD.join("+"),
        roots,
    })
}

fn extract_blocks(source: &str) -> SfcBlocks {
    let mut script_setup = None;
    let mut template = None;
    let mut script_setup_count = 0usize;
    let mut template_count = 0usize;
    let bytes = source.as_bytes();
    let mut line_start = 0usize;
    while line_start < source.len() {
        let line_end = source[line_start..]
            .find('\n')
            .map(|offset| line_start + offset)
            .unwrap_or(source.len());
        let line = &source[line_start..line_end];
        for tag in ["script", "template"] {
            let prefix = format!("<{tag}");
            if !line.starts_with(&prefix) {
                continue;
            }
            let Some(close_offset) = source[line_start..].find('>') else {
                continue;
            };
            let open_end = line_start + close_offset + 1;
            let attrs = source[line_start + prefix.len()..open_end - 1].to_string();
            let closing = format!("</{tag}>");
            let mut search = open_end;
            let mut closing_start = None;
            while search < source.len() {
                if (search == 0 || bytes.get(search - 1) == Some(&b'\n'))
                    && source[search..].starts_with(&closing)
                {
                    closing_start = Some(search);
                    break;
                }
                search += 1;
            }
            let Some(content_end) = closing_start else {
                continue;
            };
            let record = SfcBlock {
                attrs: attrs.clone(),
                content: source[open_end..content_end].to_string(),
                content_start: open_end,
            };
            if tag == "script" && has_setup_attr(&attrs) {
                script_setup_count += 1;
                script_setup = Some(record);
            } else if tag == "template" && attrs.trim().is_empty() {
                template_count += 1;
                template = Some(record);
            }
        }
        line_start = (line_end + 1).min(source.len() + 1);
    }
    if script_setup_count != 1 {
        script_setup = None;
    }
    if template_count != 1 {
        template = None;
    }
    SfcBlocks {
        script_setup,
        template,
    }
}

fn has_setup_attr(attrs: &str) -> bool {
    let bytes = attrs.as_bytes();
    let needle = b"setup";
    let mut index = 0;
    while index + needle.len() <= bytes.len() {
        if &bytes[index..index + needle.len()] == needle {
            let before = if index == 0 { b' ' } else { bytes[index - 1] };
            let after = bytes.get(index + needle.len()).copied().unwrap_or(b' ');
            if before.is_ascii_whitespace()
                && (after.is_ascii_whitespace() || after == b'=' || after == b' ')
            {
                return true;
            }
        }
        index += 1;
    }
    false
}

pub fn plan_class_a(source: &str) -> (Option<ClassAPlan>, Option<String>) {
    let blocks = extract_blocks(source);
    let (Some(script_setup), Some(template)) = (blocks.script_setup, blocks.template) else {
        return (
            None,
            Some("no-single-script-setup-and-template".to_string()),
        );
    };
    for name in top_level_bindings(&script_setup.content) {
        let seeded_name = format!("{name}{SEEDED_NAME_SUFFIX}");
        if source.contains(&seeded_name) {
            continue;
        }
        if total_template_occurrences(&template.content, &name) != 1 {
            continue;
        }
        if is_shadowed_in_template(&template.content, &name) {
            continue;
        }
        let mut hits = Vec::new();
        for segment in template_expression_segments(&template.content) {
            for offset in identifier_occurrences(&segment.text, &name) {
                hits.push(template.content_start + segment.start + offset);
            }
        }
        if hits.len() != 1 {
            continue;
        }
        let (spans, unsure) = script_token_occurrences(&script_setup.content, &name);
        if unsure || spans.is_empty() {
            continue;
        }
        return (
            Some(ClassAPlan {
                name: name.clone(),
                seeded_name,
                rename_spans: spans
                    .into_iter()
                    .map(|offset| {
                        [
                            script_setup.content_start + offset,
                            script_setup.content_start + offset + name.len(),
                        ]
                    })
                    .collect(),
                template_ref: [hits[0], hits[0] + name.len()],
            }),
            None,
        );
    }
    (None, Some("no-eligible-binding".to_string()))
}

pub fn plan_class_b(source: &str) -> (Option<ClassBPlan>, Option<String>) {
    if source.contains(UNUSED_BINDING_NAME) {
        return (None, Some("already-seeded".to_string()));
    }
    let blocks = extract_blocks(source);
    if let Some(script_setup) = blocks.script_setup {
        let insert_at = script_setup.content_start
            + if script_setup.content.starts_with('\n') {
                1
            } else {
                0
            };
        return (
            Some(ClassBPlan {
                insert_at,
                insert_text: UNUSED_BINDING_STATEMENT.to_string(),
                created_block: false,
            }),
            None,
        );
    }
    (
        Some(ClassBPlan {
            insert_at: 0,
            insert_text: format!("<script setup>\n{UNUSED_BINDING_STATEMENT}</script>\n\n"),
            created_block: true,
        }),
        None,
    )
}

pub fn apply_seed(
    source: &str,
    class_a: Option<&ClassAPlan>,
    class_b: Option<&ClassBPlan>,
) -> AppliedSeed {
    #[derive(Clone)]
    struct PendingEdit {
        span: [usize; 2],
        delta: isize,
        insert: String,
    }
    let mut edits = Vec::new();
    if let Some(plan) = class_a {
        for span in &plan.rename_spans {
            edits.push(PendingEdit {
                span: *span,
                delta: plan.seeded_name.len() as isize - plan.name.len() as isize,
                insert: plan.seeded_name.clone(),
            });
        }
    }
    if let Some(plan) = class_b {
        edits.push(PendingEdit {
            span: [plan.insert_at, plan.insert_at],
            delta: plan.insert_text.len() as isize,
            insert: plan.insert_text.clone(),
        });
    }
    edits.sort_by(|a, b| a.span[0].cmp(&b.span[0]).then(a.span[1].cmp(&b.span[1])));
    let mut seeded = source.to_string();
    for edit in edits.iter().rev() {
        seeded.replace_range(edit.span[0]..edit.span[1], &edit.insert);
    }
    AppliedSeed {
        seeded,
        edits: edits
            .into_iter()
            .map(|edit| EditRecord {
                span: edit.span,
                delta: edit.delta,
            })
            .collect(),
    }
}

fn script_token_occurrences(script_content: &str, name: &str) -> (Vec<usize>, bool) {
    let blanked = blank_script_noise(script_content);
    let mut spans = Vec::new();
    let mut unsure = false;
    let mut from = 0usize;
    while let Some(relative) = blanked[from..].find(name) {
        let at = from + relative;
        from = at + 1;
        let before = if at == 0 {
            None
        } else {
            blanked.as_bytes().get(at - 1).copied()
        };
        let after = blanked.as_bytes().get(at + name.len()).copied();
        if before.is_some_and(is_js_ident) || after.is_some_and(is_js_ident) {
            continue;
        }
        if before == Some(b'.') || follows_object_key(&blanked[at + name.len()..]) {
            unsure = true;
            continue;
        }
        spans.push(at);
    }
    (spans, unsure)
}

pub fn top_level_bindings(script_content: &str) -> Vec<String> {
    let blanked = blank_script_noise(script_content);
    let mut bindings = Vec::new();
    let mut depth = 0isize;
    let mut line_start = 0usize;
    for index in 0..=blanked.len() {
        if index == blanked.len() || blanked.as_bytes()[index] == b'\n' {
            let line = &blanked[line_start..index];
            if depth == 0 {
                if let Some(binding) = parse_top_level_declaration(line.trim_start()) {
                    bindings.push(binding);
                }
            }
            for byte in line.bytes() {
                if matches!(byte, b'{' | b'(' | b'[') {
                    depth += 1;
                } else if matches!(byte, b'}' | b')' | b']') {
                    depth -= 1;
                }
            }
            line_start = index + 1;
        }
    }
    bindings
}

fn parse_top_level_declaration(line: &str) -> Option<String> {
    let mut rest = line;
    if let Some(after) = rest.strip_prefix("export") {
        if !after
            .as_bytes()
            .first()
            .is_some_and(|byte| byte.is_ascii_whitespace())
        {
            return None;
        }
        rest = after.trim_start();
    }
    for keyword in ["const", "let", "var", "function"] {
        if let Some(after) = rest.strip_prefix(keyword) {
            if !after
                .as_bytes()
                .first()
                .is_some_and(|byte| byte.is_ascii_whitespace())
            {
                continue;
            }
            let after = after.trim_start();
            let mut chars = after.char_indices();
            let Some((_, first)) = chars.next() else {
                return None;
            };
            if !(first.is_ascii_alphabetic() || first == '_' || first == '$') {
                return None;
            }
            let mut end = first.len_utf8();
            for (index, ch) in chars {
                if ch.is_ascii_alphanumeric() || ch == '_' || ch == '$' {
                    end = index + ch.len_utf8();
                } else {
                    break;
                }
            }
            return Some(after[..end].to_string());
        }
    }
    None
}

pub fn blank_script_noise(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut mode: Option<u8> = None;
    let mut index = 0usize;
    while index < bytes.len() {
        let byte = bytes[index];
        let next = bytes.get(index + 1).copied();
        if mode.is_none() {
            if matches!(byte, b'"' | b'\'' | b'`') {
                mode = Some(byte);
                out.push(byte as char);
            } else if byte == b'/' && next == Some(b'/') {
                mode = Some(b'L');
                out.push_str("  ");
                index += 1;
            } else if byte == b'/' && next == Some(b'*') {
                mode = Some(b'B');
                out.push_str("  ");
                index += 1;
            } else {
                out.push(byte as char);
            }
        } else if mode == Some(b'L') {
            if byte == b'\n' {
                mode = None;
                out.push('\n');
            } else {
                out.push(' ');
            }
        } else if mode == Some(b'B') {
            if byte == b'*' && next == Some(b'/') {
                mode = None;
                out.push_str("  ");
                index += 1;
            } else if byte == b'\n' {
                out.push('\n');
            } else {
                out.push(' ');
            }
        } else {
            let quote = mode.unwrap();
            if byte == b'\\' {
                out.push_str("  ");
                index += 1;
            } else if byte == quote {
                mode = None;
                out.push(byte as char);
            } else if byte == b'\n' {
                out.push('\n');
            } else {
                out.push(' ');
            }
        }
        index += 1;
    }
    out
}

fn template_expression_segments(template_content: &str) -> Vec<TemplateSegment> {
    let mut segments = Vec::new();
    let mut from = 0usize;
    while let Some(start_rel) = template_content[from..].find("{{") {
        let start = from + start_rel;
        let body_start = start + 2;
        let Some(end_rel) = template_content[body_start..].find("}}") else {
            break;
        };
        let body_end = body_start + end_rel;
        segments.push(TemplateSegment {
            start: body_start,
            text: template_content[body_start..body_end].to_string(),
        });
        from = body_end + 2;
    }

    let bytes = template_content.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        let attr_start = if index == 0 || bytes[index].is_ascii_whitespace() {
            let mut name_start = index;
            if bytes[index].is_ascii_whitespace() {
                name_start += 1;
            }
            name_start
        } else {
            index += 1;
            continue;
        };
        if !bytes
            .get(attr_start)
            .is_some_and(|byte| matches!(byte, b'v' | b':' | b'@' | b'#'))
        {
            index += 1;
            continue;
        }
        let mut name_end = attr_start;
        while name_end < bytes.len()
            && !bytes[name_end].is_ascii_whitespace()
            && !matches!(bytes[name_end], b'"' | b'\'' | b'<' | b'>' | b'/' | b'=')
        {
            name_end += 1;
        }
        let mut cursor = skip_ascii_ws(bytes, name_end);
        if bytes.get(cursor) != Some(&b'=') {
            index = name_end.max(index + 1);
            continue;
        }
        cursor = skip_ascii_ws(bytes, cursor + 1);
        let Some(&quote) = bytes.get(cursor) else {
            break;
        };
        if quote != b'"' && quote != b'\'' {
            index = cursor + 1;
            continue;
        }
        let value_start = cursor + 1;
        let mut value_end = value_start;
        while value_end < bytes.len() && bytes[value_end] != quote {
            value_end += 1;
        }
        if value_end <= bytes.len() {
            segments.push(TemplateSegment {
                start: value_start,
                text: template_content[value_start..value_end].to_string(),
            });
        }
        index = value_end + 1;
    }
    segments.sort_by_key(|segment| segment.start);
    segments
}

fn identifier_occurrences(segment_text: &str, name: &str) -> Vec<usize> {
    let blanked = blank_script_noise(segment_text);
    let mut occurrences = Vec::new();
    let mut from = 0usize;
    while let Some(relative) = blanked[from..].find(name) {
        let at = from + relative;
        from = at + 1;
        let before = if at == 0 {
            None
        } else {
            blanked.as_bytes().get(at - 1).copied()
        };
        let after = blanked.as_bytes().get(at + name.len()).copied();
        if before.is_some_and(is_js_ident) || before == Some(b'.') {
            continue;
        }
        if after.is_some_and(is_js_ident) {
            continue;
        }
        if follows_object_key(&blanked[at + name.len()..]) {
            continue;
        }
        occurrences.push(at);
    }
    occurrences
}

fn is_shadowed_in_template(template_content: &str, name: &str) -> bool {
    for attr in directive_attrs(template_content) {
        let is_scope = attr.name == "v-for"
            || attr.name == "v-slot"
            || attr.name.starts_with("v-slot:")
            || attr.name.starts_with('#');
        if is_scope && contains_standalone_identifier(&attr.value, name) {
            return true;
        }
    }
    false
}

fn total_template_occurrences(template_content: &str, name: &str) -> usize {
    let mut count = 0usize;
    let mut from = 0usize;
    while let Some(relative) = template_content[from..].find(name) {
        let at = from + relative;
        from = at + 1;
        let before = if at == 0 {
            None
        } else {
            template_content.as_bytes().get(at - 1).copied()
        };
        let after = template_content.as_bytes().get(at + name.len()).copied();
        if !before.is_some_and(is_js_ident) && !after.is_some_and(is_js_ident) {
            count += 1;
        }
    }
    count
}

struct DirectiveAttr {
    name: String,
    value: String,
}

fn directive_attrs(template_content: &str) -> Vec<DirectiveAttr> {
    let bytes = template_content.as_bytes();
    let mut attrs = Vec::new();
    let mut index = 0usize;
    while index < bytes.len() {
        let name_start = if index == 0 || bytes[index].is_ascii_whitespace() {
            index + usize::from(bytes[index].is_ascii_whitespace())
        } else {
            index += 1;
            continue;
        };
        if !bytes
            .get(name_start)
            .is_some_and(|byte| matches!(byte, b'v' | b':' | b'@' | b'#'))
        {
            index += 1;
            continue;
        }
        let mut name_end = name_start;
        while name_end < bytes.len()
            && !bytes[name_end].is_ascii_whitespace()
            && !matches!(bytes[name_end], b'"' | b'\'' | b'<' | b'>' | b'/' | b'=')
        {
            name_end += 1;
        }
        let mut cursor = skip_ascii_ws(bytes, name_end);
        if bytes.get(cursor) != Some(&b'=') {
            index = name_end.max(index + 1);
            continue;
        }
        cursor = skip_ascii_ws(bytes, cursor + 1);
        let Some(&quote) = bytes.get(cursor) else {
            break;
        };
        if quote != b'"' && quote != b'\'' {
            index = cursor + 1;
            continue;
        }
        let value_start = cursor + 1;
        let mut value_end = value_start;
        while value_end < bytes.len() && bytes[value_end] != quote {
            value_end += 1;
        }
        attrs.push(DirectiveAttr {
            name: template_content[name_start..name_end].to_string(),
            value: template_content[value_start..value_end].to_string(),
        });
        index = value_end + 1;
    }
    attrs
}

fn contains_standalone_identifier(text: &str, name: &str) -> bool {
    let mut from = 0usize;
    while let Some(relative) = text[from..].find(name) {
        let at = from + relative;
        from = at + 1;
        let before = if at == 0 {
            None
        } else {
            text.as_bytes().get(at - 1).copied()
        };
        let after = text.as_bytes().get(at + name.len()).copied();
        if !before.is_some_and(is_js_ident) && !after.is_some_and(is_js_ident) {
            return true;
        }
    }
    false
}

fn follows_object_key(rest: &str) -> bool {
    let rest = rest.trim_start();
    rest.starts_with(':') && !rest.starts_with("::")
}

fn skip_ascii_ws(bytes: &[u8], mut index: usize) -> usize {
    while bytes
        .get(index)
        .copied()
        .is_some_and(|byte| byte.is_ascii_whitespace())
    {
        index += 1;
    }
    index
}

fn is_js_ident(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

pub fn line_starts_of(text: &str) -> Vec<usize> {
    let mut starts = vec![0usize];
    for (index, byte) in text.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(index + 1);
        }
    }
    starts
}

pub fn index_to_line_col(text: &str, line_starts: &[usize], index: usize) -> (usize, usize) {
    let line_index = match line_starts.binary_search(&index) {
        Ok(index) => index,
        Err(index) => index.saturating_sub(1),
    };
    let column = text[line_starts[line_index]..index].chars().count() + 1;
    (line_index + 1, column)
}

pub fn line_col_to_index(
    text: &str,
    line_starts: &[usize],
    line: usize,
    column: usize,
) -> Option<usize> {
    let start = *line_starts.get(line.checked_sub(1)?)?;
    let end = if line < line_starts.len() {
        line_starts[line]
    } else {
        text.len()
    };
    let mut remaining = column.checked_sub(1)?;
    let mut offset = 0usize;
    for code_point in text[start..end].chars() {
        if remaining == 0 {
            break;
        }
        remaining -= 1;
        offset += code_point.len_utf8();
    }
    (remaining == 0).then_some(start + offset)
}

pub fn describe_seeded_span(
    seeded_text: &str,
    line_starts: &[usize],
    start: usize,
    end: usize,
) -> SpanDescription {
    let (line, column) = index_to_line_col(seeded_text, line_starts, start);
    let (end_line, end_column) = index_to_line_col(seeded_text, line_starts, end);
    SpanDescription {
        span: [start, end],
        line,
        column,
        end_line,
        end_column,
    }
}

pub fn map_offset_through_edits(
    offset: usize,
    edits: &[EditRecord],
    is_end: bool,
) -> (usize, bool) {
    let mut mapped = offset as isize;
    for edit in edits {
        let [start, end] = edit.span;
        if start == end {
            if offset > start || (offset == start && !is_end) {
                mapped += edit.delta;
            }
        } else if offset >= end {
            mapped += edit.delta;
        } else if offset > start {
            return (mapped as usize, true);
        }
    }
    (mapped as usize, false)
}

pub fn span_overlaps_edits(start: usize, end: usize, edits: &[EditRecord]) -> bool {
    edits.iter().any(|edit| {
        let [edit_start, edit_end] = edit.span;
        edit_start != edit_end && start < edit_end && end > edit_start
    })
}

pub fn describe_mapped_span(
    seeded_text: &str,
    edits: &[EditRecord],
    original_start: usize,
    original_end: usize,
) -> Option<SpanDescription> {
    let (start, start_overlap) = map_offset_through_edits(original_start, edits, false);
    let (end, end_overlap) = map_offset_through_edits(original_end, edits, true);
    if start_overlap || end_overlap {
        return None;
    }
    let starts = line_starts_of(seeded_text);
    Some(describe_seeded_span(seeded_text, &starts, start, end))
}

pub fn diagnostic_key(row: &DiagnosticRow) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}|{}",
        row.path, row.rule_id, row.severity, row.line, row.column, row.end_line, row.end_column
    )
}

pub fn sort_diagnostics(mut rows: Vec<DiagnosticRow>) -> Vec<DiagnosticRow> {
    rows.sort();
    rows
}

pub fn flatten_lint_json(report: &Value) -> Result<Vec<DiagnosticRow>, String> {
    let entries = report
        .as_array()
        .ok_or_else(|| "vize lint JSON output is not an array".to_string())?;
    let mut rows = Vec::new();
    for file_result in entries {
        let file = file_result
            .get("file")
            .and_then(Value::as_str)
            .ok_or_else(|| "lint JSON file entry is missing file".to_string())?
            .replace('\\', "/");
        let messages = file_result
            .get("messages")
            .and_then(Value::as_array)
            .ok_or_else(|| "lint JSON file entry is missing messages".to_string())?;
        for message in messages {
            rows.push(DiagnosticRow {
                path: file.clone(),
                rule_id: string_field(message, "ruleId")?,
                severity: integer_field(message, "severity")?,
                line: usize_field(message, "line")?,
                column: usize_field(message, "column")?,
                end_line: usize_field(message, "endLine")?,
                end_column: usize_field(message, "endColumn")?,
            });
        }
    }
    Ok(sort_diagnostics(rows))
}

fn string_field(value: &Value, field: &str) -> Result<String, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| format!("lint message missing {field}"))
}

fn integer_field(value: &Value, field: &str) -> Result<i64, String> {
    value
        .get(field)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("lint message missing {field}"))
}

fn usize_field(value: &Value, field: &str) -> Result<usize, String> {
    let raw = integer_field(value, field)?;
    usize::try_from(raw).map_err(|_| format!("lint message {field} must be non-negative"))
}

pub fn resolve_vize_cli(repo_root: &Path) -> VizeCli {
    let mut candidates = Vec::new();
    if let Some(value) = std::env::var_os("VIZE_BIN") {
        candidates.push(value.to_string_lossy().to_string());
    }
    candidates.extend([
        repo_root.join("target/ci/vize").display().to_string(),
        repo_root.join("target/release/vize").display().to_string(),
        repo_root.join("target/debug/vize").display().to_string(),
        "vize".to_string(),
    ]);
    for candidate in candidates {
        if Command::new(&candidate)
            .arg("--version")
            .current_dir(repo_root)
            .output()
            .is_ok_and(|output| output.status.success())
        {
            return VizeCli {
                command: candidate,
                prefix: Vec::new(),
            };
        }
    }
    VizeCli {
        command: "cargo".to_string(),
        prefix: vec![
            "run".to_string(),
            "-q".to_string(),
            "-p".to_string(),
            "vize".to_string(),
            "--".to_string(),
        ],
    }
}

pub fn run_vize_lint_json(cli: &VizeCli, cwd: &Path, files: &[String]) -> Result<Value, String> {
    let output = Command::new(&cli.command)
        .args(&cli.prefix)
        .args(["lint", "--no-config", "--format", "json"])
        .args(files)
        .current_dir(cwd)
        .output()
        .map_err(|error| format!("failed to run vize lint: {error}"))?;
    let status = output.status.code().unwrap_or(1);
    if status != 0 && status != 1 {
        return Err(format!(
            "vize lint exited with status {status}:\n{}\n{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    serde_json::from_slice(&output.stdout).map_err(|error| {
        format!(
            "vize lint emitted non-JSON output: {error}\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

pub fn load_lint_rows(
    hook_path: Option<&Path>,
    cli: Option<&VizeCli>,
    cwd: &Path,
    files: &[String],
) -> Result<Vec<DiagnosticRow>, String> {
    let json = if let Some(path) = hook_path {
        common::read_json(path)?
    } else {
        run_vize_lint_json(
            cli.ok_or_else(|| "vize cli is required".to_string())?,
            cwd,
            files,
        )?
    };
    flatten_lint_json(&json)
}

fn count_by_key(rows: &[DiagnosticRow]) -> BTreeMap<String, (DiagnosticRow, usize)> {
    let mut counts = BTreeMap::new();
    for row in rows {
        let key = diagnostic_key(row);
        counts
            .entry(key)
            .and_modify(|(_, count)| *count += 1)
            .or_insert((row.clone(), 1));
    }
    counts
}

fn multiset_difference(
    a: &BTreeMap<String, (DiagnosticRow, usize)>,
    b: &BTreeMap<String, (DiagnosticRow, usize)>,
) -> Vec<DiagnosticRow> {
    let mut out = Vec::new();
    for (key, (row, count)) in a {
        let other = b.get(key).map(|(_, count)| *count).unwrap_or(0);
        for _ in 0..count.saturating_sub(other) {
            out.push(row.clone());
        }
    }
    sort_diagnostics(out)
}

pub fn assert_seeded_tree(
    manifest: &SeedManifest,
    out_dir: &Path,
    cli: Option<&VizeCli>,
    baseline_hook: Option<&Path>,
    seeded_hook: Option<&Path>,
) -> Result<SeedAssertReport, String> {
    let files = manifest
        .files
        .iter()
        .map(|file| file.path.clone())
        .collect::<Vec<_>>();
    let baseline_rows = load_lint_rows(baseline_hook, cli, &out_dir.join("original"), &files)?;
    let seeded_rows = load_lint_rows(seeded_hook, cli, &out_dir.join("seeded"), &files)?;
    let (shifted, unmappable) = shift_baseline(&baseline_rows, manifest, out_dir)?;
    let class_a_rows = expected_class_a_rows(manifest);
    let expected_rows = sort_diagnostics(
        shifted
            .iter()
            .cloned()
            .chain(class_a_rows.iter().map(|(row, _)| row.clone()))
            .collect(),
    );
    let expected_counts = count_by_key(&expected_rows);
    let actual_counts = count_by_key(&seeded_rows);
    let missing_rows = multiset_difference(&expected_counts, &actual_counts);
    let unexpected = multiset_difference(&actual_counts, &expected_counts);
    let class_a_keys = class_a_rows
        .iter()
        .map(|(row, identifier)| (diagnostic_key(row), identifier.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut class_a_misses = Vec::new();
    let mut baseline_misses = Vec::new();
    for row in missing_rows {
        if let Some(identifier) = class_a_keys.get(&diagnostic_key(&row)) {
            class_a_misses.push(ClassAMiss::from((&row, identifier.clone())));
        } else {
            baseline_misses.push(row);
        }
    }
    let class_b_injections = manifest
        .injections
        .iter()
        .filter(|injection| injection.class_name == CLASS_B)
        .collect::<Vec<_>>();
    let actual_spans = seeded_rows
        .iter()
        .map(|row| {
            format!(
                "{}|{}|{}|{}|{}",
                row.path, row.line, row.column, row.end_line, row.end_column
            )
        })
        .collect::<BTreeSet<_>>();
    let class_b_detected = class_b_injections
        .iter()
        .filter(|injection| {
            actual_spans.contains(&format!(
                "{}|{}|{}|{}|{}",
                injection.path,
                injection.expected.line,
                injection.expected.column,
                injection.expected.end_line,
                injection.expected.end_column
            ))
        })
        .count();
    let pass = class_a_misses.is_empty()
        && baseline_misses.is_empty()
        && unmappable.is_empty()
        && unexpected.is_empty();
    Ok(SeedAssertReport {
        schema_version: 1,
        tool: "tools/commands/davinci/seed-defects.rs --assert".to_string(),
        source: manifest.source.clone(),
        scope: manifest.scope.clone(),
        lint: LintCounts {
            baseline_diagnostics: baseline_rows.len(),
            seeded_diagnostics: seeded_rows.len(),
        },
        class_a: ClassAReport {
            expected: class_a_rows.len(),
            detected: class_a_rows.len() - class_a_misses.len(),
            misses: class_a_misses,
        },
        class_b: ClassBReport {
            expected: class_b_injections.len(),
            detected: class_b_detected,
            note: "not gated: vize_croquis unused_bindings has no lint consumer today (FN ledger)"
                .to_string(),
        },
        baseline_shift: BaselineShiftReport {
            mapped: shifted.len(),
            misses: baseline_misses,
            unmappable,
        },
        unexpected,
        verdict: if pass { "pass" } else { "fail" }.to_string(),
    })
}

fn shift_baseline(
    rows: &[DiagnosticRow],
    manifest: &SeedManifest,
    out_dir: &Path,
) -> Result<(Vec<DiagnosticRow>, Vec<DiagnosticRow>), String> {
    let mut shifted = Vec::new();
    let mut unmappable = Vec::new();
    for row in rows {
        let edits = manifest.edits.get(&row.path).cloned().unwrap_or_default();
        if edits.is_empty() {
            shifted.push(row.clone());
            continue;
        }
        let original_text = common::read_text(out_dir.join("original").join(&row.path))?;
        let original_starts = line_starts_of(&original_text);
        let start = line_col_to_index(&original_text, &original_starts, row.line, row.column);
        let end = line_col_to_index(
            &original_text,
            &original_starts,
            row.end_line,
            row.end_column,
        );
        if start.is_none()
            || end.is_none()
            || span_overlaps_edits(start.unwrap(), end.unwrap(), &edits)
        {
            unmappable.push(row.clone());
            continue;
        }
        let seeded_text = common::read_text(out_dir.join("seeded").join(&row.path))?;
        if let Some(described) =
            describe_mapped_span(&seeded_text, &edits, start.unwrap(), end.unwrap())
        {
            shifted.push(DiagnosticRow {
                path: row.path.clone(),
                rule_id: row.rule_id.clone(),
                severity: row.severity,
                line: described.line,
                column: described.column,
                end_line: described.end_line,
                end_column: described.end_column,
            });
        } else {
            unmappable.push(row.clone());
        }
    }
    Ok((sort_diagnostics(shifted), sort_diagnostics(unmappable)))
}

fn expected_class_a_rows(manifest: &SeedManifest) -> Vec<(DiagnosticRow, String)> {
    manifest
        .injections
        .iter()
        .filter(|injection| injection.class_name == CLASS_A)
        .map(|injection| {
            (
                DiagnosticRow {
                    path: injection.path.clone(),
                    rule_id: injection.expected_rule.clone().unwrap_or_default(),
                    severity: 1,
                    line: injection.expected.line,
                    column: injection.expected.column,
                    end_line: injection.expected.end_line,
                    end_column: injection.expected.end_column,
                },
                injection.identifier.original.clone().unwrap_or_default(),
            )
        })
        .collect()
}

pub fn parse_suppression_line(line_text: &str) -> Option<SuppressionComment> {
    if !line_text.contains("eslint-") {
        return None;
    }
    for (marker, kind) in [
        ("eslint-disable-next-line", "next-line"),
        ("eslint-disable-line", "line"),
        ("eslint-disable", "block"),
        ("eslint-enable", "enable"),
    ] {
        if let Some(at) = line_text.find(marker) {
            return Some(SuppressionComment {
                line: 0,
                kind: kind.to_string(),
                rules: parse_rule_list(&line_text[at + marker.len()..]),
            });
        }
    }
    None
}

fn parse_rule_list(raw: &str) -> Vec<String> {
    let before_reason = raw
        .split("--")
        .next()
        .unwrap_or("")
        .replace("*/", " ")
        .replace("-->", " ");
    before_reason
        .split(|ch: char| ch.is_whitespace() || ch == ',')
        .map(|rule| {
            rule.trim_matches(|ch| "\"'[]{}();".contains(ch))
                .to_string()
        })
        .filter(|rule| !rule.is_empty())
        .collect()
}

pub fn scan_suppressions(source: &str) -> SuppressionScan {
    let mut comments = Vec::new();
    let mut ranges = Vec::new();
    let mut open_blocks = Vec::<usize>::new();
    for (index, line) in source.split('\n').enumerate() {
        let line_number = index + 1;
        let Some(mut parsed) = parse_suppression_line(line) else {
            continue;
        };
        parsed.line = line_number;
        comments.push(parsed.clone());
        let rules = if parsed.rules.is_empty() {
            vec![None]
        } else {
            parsed.rules.iter().cloned().map(Some).collect()
        };
        match parsed.kind.as_str() {
            "next-line" => {
                for rule in rules {
                    ranges.push(SuppressionRange {
                        rule,
                        start_line: line_number + 1,
                        end_line: Some(line_number + 1),
                        comment_line: line_number,
                        kind: parsed.kind.clone(),
                    });
                }
            }
            "line" => {
                for rule in rules {
                    ranges.push(SuppressionRange {
                        rule,
                        start_line: line_number,
                        end_line: Some(line_number),
                        comment_line: line_number,
                        kind: parsed.kind.clone(),
                    });
                }
            }
            "block" => {
                for rule in rules {
                    open_blocks.push(ranges.len());
                    ranges.push(SuppressionRange {
                        rule,
                        start_line: line_number,
                        end_line: None,
                        comment_line: line_number,
                        kind: parsed.kind.clone(),
                    });
                }
            }
            "enable" => {
                for open_index in (0..open_blocks.len()).rev() {
                    let range_index = open_blocks[open_index];
                    let close = parsed.rules.is_empty()
                        || ranges[range_index]
                            .rule
                            .as_ref()
                            .is_some_and(|rule| parsed.rules.contains(rule));
                    if close {
                        ranges[range_index].end_line = Some(line_number);
                        open_blocks.remove(open_index);
                    }
                }
            }
            _ => {}
        }
    }
    SuppressionScan { comments, ranges }
}

pub fn defuse_suppressions(source: &str) -> (String, bool) {
    let mut changed = false;
    let lines = source
        .split('\n')
        .map(|line| {
            if parse_suppression_line(line).is_some() {
                changed = true;
                line.replace("eslint-", "esl1nt-")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>();
    (lines.join("\n"), changed)
}

pub fn load_rule_map(repo_root: &Path) -> Result<RuleMap, String> {
    let fixture = common::read_json(repo_root.join(RULE_MAP_FIXTURE))?;
    let entries = fixture
        .get("entries")
        .and_then(Value::as_object)
        .ok_or_else(|| "rule map fixture missing entries".to_string())?;
    let mut mapped = BTreeMap::new();
    for (eslint_name, entry) in entries {
        if entry.get("status").and_then(Value::as_str) == Some("mapped") {
            let patina = entry
                .get("patinaRule")
                .and_then(Value::as_str)
                .ok_or_else(|| format!("mapped rule {eslint_name} missing patinaRule"))?;
            mapped.insert(eslint_name.clone(), patina.to_string());
        }
    }
    Ok(RuleMap {
        fixture_path: RULE_MAP_FIXTURE.to_string(),
        fixture_mapped_count: mapped.len(),
        core_sidecar_count: 0,
        mapped,
    })
}

fn range_covers(range: &SuppressionRange, line: usize) -> bool {
    line >= range.start_line && range.end_line.is_none_or(|end| line <= end)
}

pub fn intersect_suppressions(
    diagnostics_by_path: &BTreeMap<String, Vec<DiagnosticRow>>,
    suppressions_by_path: &BTreeMap<String, SuppressionScan>,
    rule_map: &RuleMap,
) -> (Vec<SuppressionCandidate>, usize) {
    let mut candidates = Vec::new();
    let mut on_bare_lines = 0usize;
    for (file_path, rows) in diagnostics_by_path {
        let Some(suppressions) = suppressions_by_path.get(file_path) else {
            continue;
        };
        for row in rows {
            let mut is_candidate = false;
            for range in &suppressions.ranges {
                if !range_covers(range, row.line) {
                    continue;
                }
                if range.rule.is_none() {
                    on_bare_lines += 1;
                    continue;
                }
                let rule = range.rule.as_ref().unwrap();
                if rule_map.mapped.get(rule) == Some(&row.rule_id) && !is_candidate {
                    is_candidate = true;
                    candidates.push(SuppressionCandidate {
                        path: file_path.clone(),
                        line: row.line,
                        column: row.column,
                        end_line: row.end_line,
                        end_column: row.end_column,
                        severity: row.severity,
                        vize_rule: row.rule_id.clone(),
                        eslint_rule: rule.clone(),
                        comment_line: range.comment_line,
                        kind: range.kind.clone(),
                    });
                }
            }
        }
    }
    candidates.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then(a.line.cmp(&b.line))
            .then(a.column.cmp(&b.column))
            .then(a.vize_rule.cmp(&b.vize_rule))
    });
    (candidates, on_bare_lines)
}
