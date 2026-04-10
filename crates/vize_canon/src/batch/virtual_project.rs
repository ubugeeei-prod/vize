//! Virtual project management for Corsa-backed type checking.
//!
//! This module materializes a mirrored TypeScript project in
//! `node_modules/.vize/canon/` so Corsa can type-check Vue SFCs together with
//! regular TypeScript sources, ambient declarations, and emitted `.d.ts` files.

use std::path::{Path, PathBuf};

use super::error::{CorsaError, CorsaResult};
use super::import_rewriter::ImportRewriter;
use super::source_map::{CompositeSourceMap, SfcBlockRange, SfcSourceMap};
use super::SfcBlockType;
use crate::virtual_ts::{generate_virtual_ts_with_offsets, VirtualTsOptions};
use oxc_span::SourceType;
use serde_json::{Map, Value};
use vize_atelier_core::parser::parse;
use vize_atelier_sfc::{parse_sfc, SfcDescriptor, SfcParseOptions};
use vize_carton::{cstr, Bump, FxHashMap, String as CompactString, ToCompactString};
use vize_croquis::{Analyzer, AnalyzerOptions, ImportStatementInfo, ReExportInfo, TypeExport};

/// A virtual file in the project.
#[derive(Debug)]
pub struct VirtualFile {
    /// Generated or rewritten source code used by Corsa.
    pub content: CompactString,
    /// Source map for position mapping.
    pub source_map: CompositeSourceMap,
    /// Original file path.
    pub original_path: PathBuf,
    /// Materialized file path inside the virtual project.
    pub virtual_path: PathBuf,
}

/// Original position after mapping.
#[derive(Debug, Clone)]
pub struct OriginalPosition {
    /// Original file path.
    pub path: PathBuf,
    /// Line number (0-based).
    pub line: u32,
    /// Column number (0-based).
    pub column: u32,
    /// SFC block type if applicable.
    pub block_type: Option<SfcBlockType>,
}

/// Virtual project for Corsa-backed type checking.
pub struct VirtualProject {
    /// Project root directory.
    project_root: PathBuf,

    /// Virtual project root (`node_modules/.vize/canon`).
    virtual_root: PathBuf,

    /// Explicit tsconfig path, if one was provided by the caller.
    tsconfig_path: Option<PathBuf>,

    /// Global virtual TS options applied to every Vue file.
    virtual_ts_options: VirtualTsOptions,

    /// Virtual files keyed by materialized path.
    virtual_files: FxHashMap<PathBuf, VirtualFile>,

    /// Import rewriter for `.vue` specifiers inside TypeScript sources.
    rewriter: ImportRewriter,
}

impl VirtualProject {
    /// Create a new virtual project.
    pub fn new(project_root: &Path) -> CorsaResult<Self> {
        let project_root = project_root
            .canonicalize()
            .unwrap_or_else(|_| project_root.to_path_buf());
        let virtual_root = project_root
            .join("node_modules")
            .join(".vize")
            .join("canon");

        Ok(Self {
            project_root,
            virtual_root,
            tsconfig_path: None,
            virtual_ts_options: VirtualTsOptions::default(),
            virtual_files: FxHashMap::default(),
            rewriter: ImportRewriter::new(),
        })
    }

    /// Set the tsconfig path to extend.
    pub fn set_tsconfig_path(&mut self, tsconfig_path: Option<PathBuf>) {
        self.tsconfig_path = tsconfig_path;
    }

    /// Set the shared virtual TS options.
    pub fn set_virtual_ts_options(&mut self, options: VirtualTsOptions) {
        self.virtual_ts_options = options;
    }

    /// Get the project root.
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// Get the virtual root.
    pub fn virtual_root(&self) -> &Path {
        &self.virtual_root
    }

    /// Register a supported file path.
    pub fn register_path(&mut self, path: &Path) -> CorsaResult<()> {
        let content = std::fs::read_to_string(path)?;
        self.register_path_with_content(path, &content)
    }

    /// Register a supported file path with already-loaded content.
    pub fn register_path_with_content(&mut self, path: &Path, content: &str) -> CorsaResult<()> {
        if path.extension().and_then(|extension| extension.to_str()) == Some("vue") {
            return self.register_vue_file(path, content);
        }

        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".d.ts"))
        {
            return self.register_declaration_file(path, content);
        }

        let source_type = source_type_for_path(path).ok_or_else(|| CorsaError::PathError {
            path: path.to_path_buf(),
        })?;
        self.register_script_file(path, content, source_type)
    }

    /// Register a `.vue` file.
    pub fn register_vue_file(&mut self, path: &Path, content: &str) -> CorsaResult<()> {
        let descriptor = parse_sfc(
            content,
            SfcParseOptions {
                filename: path.to_string_lossy().to_compact_string(),
                ..Default::default()
            },
        )
        .map_err(|error| CorsaError::SfcParse(error.message.to_compact_string()))?;

        let effective_options =
            virtual_ts_options_for_descriptor(&self.virtual_ts_options, &descriptor);
        let generated = generate_vue_virtual_ts(path, content, &descriptor, &effective_options)?;
        let rewritten = self.rewriter.rewrite(&generated.code, SourceType::ts());
        let source_map = CompositeSourceMap::new_vue(
            SfcSourceMap::new(generated.mappings, collect_sfc_block_ranges(&descriptor)),
            rewritten.source_map,
        );
        let virtual_path = virtual_vue_path(&self.project_root, &self.virtual_root, path)?;

        self.virtual_files.insert(
            virtual_path.clone(),
            VirtualFile {
                content: rewritten.code,
                source_map,
                original_path: path.to_path_buf(),
                virtual_path,
            },
        );

        Ok(())
    }

    /// Register a `.ts`/`.tsx`/`.mts`/`.cts` file.
    pub fn register_ts_file(&mut self, path: &Path) -> CorsaResult<()> {
        let content = std::fs::read_to_string(path)?;
        let source_type = source_type_for_path(path).ok_or_else(|| CorsaError::PathError {
            path: path.to_path_buf(),
        })?;
        self.register_script_file(path, &content, source_type)
    }

    /// Register a `.d.ts` file.
    pub fn register_declaration_file(&mut self, path: &Path, content: &str) -> CorsaResult<()> {
        self.register_script_file(path, content, SourceType::ts())
    }

    /// Register a non-Vue source file.
    pub fn register_script_file(
        &mut self,
        path: &Path,
        content: &str,
        source_type: SourceType,
    ) -> CorsaResult<()> {
        let rewritten = self.rewriter.rewrite(content, source_type);
        let virtual_path = mirrored_virtual_path(&self.project_root, &self.virtual_root, path)?;

        self.virtual_files.insert(
            virtual_path.clone(),
            VirtualFile {
                content: rewritten.code,
                source_map: CompositeSourceMap::new_script(rewritten.source_map),
                original_path: path.to_path_buf(),
                virtual_path,
            },
        );

        Ok(())
    }

    /// Materialize the virtual project to disk for diagnostics collection.
    pub fn materialize(&self) -> CorsaResult<()> {
        if self.virtual_root.exists() {
            std::fs::remove_dir_all(&self.virtual_root)?;
        }
        std::fs::create_dir_all(&self.virtual_root)?;

        for file in self.virtual_files.values() {
            if let Some(parent) = file.virtual_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&file.virtual_path, &file.content)?;
        }

        self.write_tsconfig_file(&self.virtual_root.join("tsconfig.json"), None, false)?;
        Ok(())
    }

    /// Write a declaration-emitting tsconfig and return its path.
    pub fn write_declaration_tsconfig(
        &self,
        out_dir: &Path,
        declaration_map: bool,
    ) -> CorsaResult<PathBuf> {
        let config_path = self.virtual_root.join("tsconfig.declaration.json");
        self.write_tsconfig_file(&config_path, Some(out_dir), declaration_map)?;
        Ok(config_path)
    }

    /// Find a virtual file by its original path.
    pub fn find_by_original(&self, original_path: &Path) -> Option<&VirtualFile> {
        self.virtual_files
            .values()
            .find(|file| file.original_path == original_path)
    }

    /// Return virtual files sorted by original path for deterministic output.
    pub fn virtual_files_sorted(&self) -> Vec<&VirtualFile> {
        let mut files: Vec<_> = self.virtual_files.values().collect();
        files.sort_by(|left, right| left.original_path.cmp(&right.original_path));
        files
    }

    /// Map a virtual position to the original position.
    pub fn map_to_original(
        &self,
        virtual_path: &Path,
        line: u32,
        column: u32,
    ) -> Option<OriginalPosition> {
        let file = self.virtual_files.get(virtual_path)?;
        let virtual_offset = super::source_map::line_col_to_offset(&file.content, line, column)?;
        let (original_offset, _, block_type) =
            file.source_map.get_original_position(virtual_offset)?;
        let original_content = std::fs::read_to_string(&file.original_path).ok()?;
        let (original_line, original_column) =
            super::source_map::offset_to_line_col(&original_content, original_offset)?;

        Some(OriginalPosition {
            path: file.original_path.clone(),
            line: original_line,
            column: original_column,
            block_type,
        })
    }

    /// Map an original position to the virtual position.
    pub fn map_to_virtual(
        &self,
        original_path: &Path,
        line: u32,
        column: u32,
    ) -> Option<(PathBuf, u32, u32)> {
        let file = self.find_by_original(original_path)?;
        let original_content = std::fs::read_to_string(&file.original_path).ok()?;
        let original_offset =
            super::source_map::line_col_to_offset(&original_content, line, column)?;
        let virtual_offset = if let Some(ref sfc_map) = file.source_map.sfc_map {
            for block in [
                SfcBlockType::ScriptSetup,
                SfcBlockType::Script,
                SfcBlockType::Template,
            ] {
                if let Some(virtual_offset) = sfc_map.get_virtual_offset(original_offset, block) {
                    let rewritten_offset = file
                        .source_map
                        .import_map
                        .get_virtual_offset(virtual_offset);
                    if let Some((virtual_line, virtual_column)) =
                        super::source_map::offset_to_line_col(&file.content, rewritten_offset)
                    {
                        return Some((file.virtual_path.clone(), virtual_line, virtual_column));
                    }
                }
            }
            return None;
        } else {
            file.source_map
                .import_map
                .get_virtual_offset(original_offset)
        };

        let (virtual_line, virtual_column) =
            super::source_map::offset_to_line_col(&file.content, virtual_offset)?;
        Some((file.virtual_path.clone(), virtual_line, virtual_column))
    }

    /// Get the number of registered files.
    pub fn file_count(&self) -> usize {
        self.virtual_files.len()
    }

    /// Check if the project has any files.
    pub fn is_empty(&self) -> bool {
        self.virtual_files.is_empty()
    }

    fn write_tsconfig_file(
        &self,
        path: &Path,
        out_dir: Option<&Path>,
        declaration_map: bool,
    ) -> CorsaResult<()> {
        let tsconfig = self.generate_tsconfig_value(out_dir, declaration_map)?;
        std::fs::write(path, serde_json::to_string_pretty(&tsconfig)?)?;
        Ok(())
    }

    fn generate_tsconfig_value(
        &self,
        out_dir: Option<&Path>,
        declaration_map: bool,
    ) -> CorsaResult<Value> {
        let mut config = Map::new();
        let original_tsconfig = self.resolved_tsconfig_path();
        if let Some(ref tsconfig_path) = original_tsconfig {
            config.insert(
                "extends".into(),
                Value::String(tsconfig_path.to_string_lossy().into_owned()),
            );
        }

        let mut compiler_options = self.load_compiler_options(original_tsconfig.as_deref())?;
        compiler_options.insert("allowImportingTsExtensions".into(), Value::Bool(true));

        if let Some(out_dir) = out_dir {
            compiler_options.insert("noEmit".into(), Value::Bool(false));
            compiler_options.insert("declaration".into(), Value::Bool(true));
            compiler_options.insert("emitDeclarationOnly".into(), Value::Bool(true));
            compiler_options.insert("declarationMap".into(), Value::Bool(declaration_map));
            compiler_options.insert(
                "rootDir".into(),
                Value::String(
                    self.common_virtual_source_dir()
                        .to_string_lossy()
                        .into_owned(),
                ),
            );
            compiler_options.insert(
                "outDir".into(),
                Value::String(out_dir.to_string_lossy().into_owned()),
            );
        } else {
            compiler_options.remove("declaration");
            compiler_options.remove("emitDeclarationOnly");
            compiler_options.remove("declarationMap");
            compiler_options.remove("outDir");
            compiler_options.insert("noEmit".into(), Value::Bool(true));
        }

        config.insert("compilerOptions".into(), Value::Object(compiler_options));
        config.insert(
            "include".into(),
            Value::Array(
                self.include_paths()
                    .into_iter()
                    .map(|path| Value::String(path.into()))
                    .collect(),
            ),
        );
        config.insert("exclude".into(), Value::Array(Vec::new()));

        Ok(Value::Object(config))
    }

    fn include_paths(&self) -> Vec<CompactString> {
        let mut includes: Vec<_> = self
            .virtual_files
            .keys()
            .filter_map(|path| path.strip_prefix(&self.virtual_root).ok())
            .map(|path| path.to_string_lossy().to_compact_string())
            .collect();
        includes.sort();
        includes
    }

    fn common_virtual_source_dir(&self) -> PathBuf {
        let mut parents = self
            .virtual_files
            .keys()
            .filter_map(|path| path.parent().map(Path::to_path_buf));
        let Some(mut common) = parents.next() else {
            return self.virtual_root.clone();
        };

        for parent in parents {
            while !parent.starts_with(&common) {
                if !common.pop() {
                    return self.virtual_root.clone();
                }
            }
        }

        common
    }

    fn resolved_tsconfig_path(&self) -> Option<PathBuf> {
        if let Some(ref tsconfig_path) = self.tsconfig_path {
            return Some(tsconfig_path.clone());
        }

        let tsconfig = self.project_root.join("tsconfig.json");
        tsconfig.exists().then_some(tsconfig)
    }

    #[allow(clippy::disallowed_types)]
    fn load_compiler_options(
        &self,
        tsconfig_path: Option<&Path>,
    ) -> CorsaResult<Map<std::string::String, Value>> {
        let Some(tsconfig_path) = tsconfig_path else {
            return Ok(Map::new());
        };

        if !tsconfig_path.exists() {
            return Ok(Map::new());
        }

        let content = std::fs::read_to_string(tsconfig_path)?;
        let config = parse_jsonc_value(&content)?;
        Ok(config
            .get("compilerOptions")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default())
    }
}

struct GeneratedVueFile {
    code: CompactString,
    mappings: Vec<crate::virtual_ts::VizeMapping>,
}

fn generate_vue_virtual_ts(
    path: &Path,
    source: &str,
    descriptor: &SfcDescriptor,
    options: &VirtualTsOptions,
) -> CorsaResult<GeneratedVueFile> {
    let (script_content, script_offset) = merged_script_content(descriptor);
    let script_content_ref = script_content.as_deref();

    let allocator = Bump::new();
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    let has_both_scripts = descriptor.script.is_some() && descriptor.script_setup.is_some();

    if let Some(ref script) = descriptor.script {
        analyzer.analyze_script_plain(&script.content);
    }

    let plain_spans: Option<(Vec<ImportStatementInfo>, Vec<ReExportInfo>, Vec<TypeExport>)> =
        if has_both_scripts {
            Some((
                analyzer.summary().import_statements.clone(),
                analyzer.summary().re_exports.clone(),
                analyzer.summary().type_exports.clone(),
            ))
        } else {
            None
        };

    if let Some(ref script_setup) = descriptor.script_setup {
        let generic = script_setup
            .attrs
            .get("generic")
            .map(|value| value.as_ref());
        analyzer.analyze_script_setup_with_generic(&script_setup.content, generic);
    }

    let template_offset = descriptor
        .template
        .as_ref()
        .map(|template| template.loc.start as u32)
        .unwrap_or(0);
    let template_ast = descriptor.template.as_ref().map(|template| {
        let (root, _) = parse(&allocator, &template.content);
        analyzer.analyze_template(&root);
        root
    });

    let mut summary = analyzer.finish();

    if let (Some((plain_imports, plain_reexports, plain_types)), Some(script)) =
        (plain_spans, descriptor.script.as_ref())
    {
        let plain_len = script.content.len() as u32 + 1;
        shift_module_spans(&mut summary.import_statements, plain_len);
        shift_module_spans(&mut summary.re_exports, plain_len);
        shift_module_spans(&mut summary.type_exports, plain_len);
        summary.import_statements.extend(plain_imports);
        summary.re_exports.extend(plain_reexports);
        summary.type_exports.extend(plain_types);
    }

    let output = generate_virtual_ts_with_offsets(
        &summary,
        script_content_ref,
        template_ast.as_ref(),
        script_offset,
        template_offset,
        options,
    );

    let _ = source;
    let _ = path;

    Ok(GeneratedVueFile {
        code: output.code,
        mappings: output.mappings,
    })
}

fn shift_module_spans<T>(items: &mut [T], delta: u32)
where
    T: ModuleSpan,
{
    for item in items {
        item.shift(delta);
    }
}

trait ModuleSpan {
    fn shift(&mut self, delta: u32);
}

impl ModuleSpan for ImportStatementInfo {
    fn shift(&mut self, delta: u32) {
        self.start += delta;
        self.end += delta;
    }
}

impl ModuleSpan for ReExportInfo {
    fn shift(&mut self, delta: u32) {
        self.start += delta;
        self.end += delta;
    }
}

impl ModuleSpan for TypeExport {
    fn shift(&mut self, delta: u32) {
        self.start += delta;
        self.end += delta;
    }
}

fn merged_script_content(descriptor: &SfcDescriptor) -> (Option<CompactString>, u32) {
    match (descriptor.script.as_ref(), descriptor.script_setup.as_ref()) {
        (Some(script), Some(script_setup)) => (
            Some(cstr!("{}\n{}", script.content, script_setup.content)),
            script.loc.start as u32,
        ),
        (Some(script), None) => (
            Some(script.content.to_compact_string()),
            script.loc.start as u32,
        ),
        (None, Some(script_setup)) => (
            Some(script_setup.content.to_compact_string()),
            script_setup.loc.start as u32,
        ),
        (None, None) => (None, 0),
    }
}

fn collect_sfc_block_ranges(descriptor: &SfcDescriptor) -> Vec<SfcBlockRange> {
    let mut blocks = Vec::with_capacity(3);
    if let Some(template) = descriptor.template.as_ref() {
        push_block_range(
            &mut blocks,
            template.loc.start as u32,
            template.content.len() as u32,
            SfcBlockType::Template,
        );
    }
    if let Some(script) = descriptor.script.as_ref() {
        push_block_range(
            &mut blocks,
            script.loc.start as u32,
            script.content.len() as u32,
            SfcBlockType::Script,
        );
    }
    if let Some(script_setup) = descriptor.script_setup.as_ref() {
        push_block_range(
            &mut blocks,
            script_setup.loc.start as u32,
            script_setup.content.len() as u32,
            SfcBlockType::ScriptSetup,
        );
    }
    blocks
}

fn push_block_range(
    blocks: &mut Vec<SfcBlockRange>,
    start: u32,
    len: u32,
    block_type: SfcBlockType,
) {
    if len == 0 {
        return;
    }
    blocks.push(SfcBlockRange {
        start,
        end: start + len,
        block_type,
    });
}

fn virtual_ts_options_for_descriptor(
    base: &VirtualTsOptions,
    descriptor: &SfcDescriptor,
) -> VirtualTsOptions {
    let css_modules: Vec<CompactString> = descriptor
        .styles
        .iter()
        .filter_map(|style| {
            style
                .module
                .as_ref()
                .map(|module| module.to_compact_string())
        })
        .collect();
    if css_modules.is_empty() {
        return base.clone();
    }

    let mut options = base.clone();
    options.css_modules = css_modules;
    options
}

fn mirrored_virtual_path(
    project_root: &Path,
    virtual_root: &Path,
    path: &Path,
) -> CorsaResult<PathBuf> {
    let relative = path.strip_prefix(project_root)?;
    Ok(virtual_root.join(relative))
}

fn virtual_vue_path(project_root: &Path, virtual_root: &Path, path: &Path) -> CorsaResult<PathBuf> {
    let mut virtual_path = mirrored_virtual_path(project_root, virtual_root, path)?;
    let file_name = virtual_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| cstr!("{name}.ts"))
        .ok_or_else(|| CorsaError::PathError {
            path: path.to_path_buf(),
        })?;
    virtual_path.set_file_name(file_name.as_str());
    Ok(virtual_path)
}

fn source_type_for_path(path: &Path) -> Option<SourceType> {
    let file_name = path.file_name()?.to_str()?;
    if file_name.ends_with(".tsx") {
        return Some(SourceType::tsx());
    }
    if file_name.ends_with(".ts")
        || file_name.ends_with(".d.ts")
        || file_name.ends_with(".mts")
        || file_name.ends_with(".cts")
    {
        return Some(SourceType::ts());
    }
    None
}

fn parse_jsonc_value(content: &str) -> CorsaResult<Value> {
    let stripped = strip_json_comments(content);
    let normalized = strip_trailing_commas(&stripped);
    Ok(serde_json::from_str(&normalized)?)
}

fn strip_json_comments(content: &str) -> CompactString {
    let mut output = CompactString::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;
    let mut line_comment = false;
    let mut block_comment = false;

    while let Some(ch) = chars.next() {
        if line_comment {
            if ch == '\n' {
                line_comment = false;
                output.push('\n');
            }
            continue;
        }

        if block_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                let _ = chars.next();
                block_comment = false;
            } else if ch == '\n' {
                output.push('\n');
            }
            continue;
        }

        if in_string {
            output.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
            output.push(ch);
            continue;
        }

        if ch == '/' && chars.peek() == Some(&'/') {
            let _ = chars.next();
            line_comment = true;
            continue;
        }

        if ch == '/' && chars.peek() == Some(&'*') {
            let _ = chars.next();
            block_comment = true;
            continue;
        }

        output.push(ch);
    }

    output
}

fn strip_trailing_commas(content: &str) -> CompactString {
    let mut output = CompactString::with_capacity(content.len());
    let chars: Vec<char> = content.chars().collect();
    let mut index = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    while index < chars.len() {
        let ch = chars[index];
        if in_string {
            output.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            index += 1;
            continue;
        }

        if ch == '"' {
            in_string = true;
            output.push(ch);
            index += 1;
            continue;
        }

        if ch == ',' {
            let mut lookahead = index + 1;
            while lookahead < chars.len() && chars[lookahead].is_whitespace() {
                lookahead += 1;
            }
            if lookahead < chars.len() && matches!(chars[lookahead], '}' | ']') {
                index += 1;
                continue;
            }
        }

        output.push(ch);
        index += 1;
    }

    output
}

#[cfg(test)]
mod tests {
    use super::{parse_jsonc_value, source_type_for_path, strip_json_comments, VirtualProject};
    use std::fs;
    use std::path::{Path, PathBuf};
    use vize_carton::cstr;

    fn unique_case_dir(name: &str) -> PathBuf {
        static NEXT_CASE_ID: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
        let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("__agent_only")
            .join("tests")
            .join(cstr!("{name}-{}-{case_id}", std::process::id()).as_str())
    }

    #[test]
    fn test_virtual_project_new() {
        let case_dir = unique_case_dir("new");
        let _ = fs::remove_dir_all(&case_dir);
        fs::create_dir_all(&case_dir).unwrap();

        let project = VirtualProject::new(&case_dir).unwrap();

        assert_eq!(project.project_root(), case_dir.as_path());
        assert!(project.virtual_root().ends_with("node_modules/.vize/canon"));

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn test_register_vue_file_rewrites_child_imports() {
        let case_dir = unique_case_dir("register-vue");
        let _ = fs::remove_dir_all(&case_dir);
        let src_dir = case_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let vue_path = src_dir.join("App.vue");
        let vue_content = r#"<script setup lang="ts">
import Child from './Child.vue'
const count = 1
</script>

<template>
  <Child :count="count" />
</template>
"#;
        fs::write(&vue_path, vue_content).unwrap();

        let mut project = VirtualProject::new(&case_dir).unwrap();
        project.register_vue_file(&vue_path, vue_content).unwrap();

        let virtual_file = project.find_by_original(&vue_path).unwrap();
        assert!(virtual_file.content.contains("./Child.vue.ts"));

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn test_materialize_writes_tsconfig_and_virtual_files() {
        let case_dir = unique_case_dir("materialize");
        let _ = fs::remove_dir_all(&case_dir);
        let src_dir = case_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let vue_path = src_dir.join("App.vue");
        fs::write(
            &vue_path,
            r#"<script setup lang="ts">
const message = 'Hello'
</script>

<template>
  <div>{{ message }}</div>
</template>
"#,
        )
        .unwrap();

        let mut project = VirtualProject::new(&case_dir).unwrap();
        project.register_path(&vue_path).unwrap();
        project.materialize().unwrap();

        assert!(case_dir
            .join("node_modules/.vize/canon/src/App.vue.ts")
            .exists());
        assert!(case_dir
            .join("node_modules/.vize/canon/tsconfig.json")
            .exists());

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn test_parse_jsonc_value_handles_comments_and_trailing_commas() {
        let value = parse_jsonc_value(
            r#"{
  // comment
  "compilerOptions": {
    "strict": true,
    "paths": {
      "@/*": ["src/*",],
    },
  },
}"#,
        )
        .unwrap();

        assert_eq!(
            value["compilerOptions"]["paths"]["@/*"][0],
            serde_json::Value::String("src/*".into())
        );
    }

    #[test]
    fn test_strip_json_comments_preserves_strings() {
        let stripped = strip_json_comments(r#"{ "url": "https://example.com" }"#);
        assert!(stripped.contains("https://example.com"));
    }

    #[test]
    fn test_source_type_for_path() {
        assert_eq!(
            source_type_for_path(Path::new("foo.ts")),
            Some(oxc_span::SourceType::ts())
        );
        assert_eq!(
            source_type_for_path(Path::new("foo.tsx")),
            Some(oxc_span::SourceType::tsx())
        );
        assert_eq!(source_type_for_path(Path::new("foo.vue")), None);
    }
}
