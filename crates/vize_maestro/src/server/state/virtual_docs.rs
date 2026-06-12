//! Virtual document generation and caching.

use std::path::PathBuf;

use dashmap::DashMap;
use tower_lsp::lsp_types::Url;

use crate::utils::is_standalone_html_path;
use crate::virtual_code::VirtualDocuments;

use super::ServerState;

impl ServerState {
    /// Generate and cache virtual documents for a document.
    pub fn update_virtual_docs(&self, uri: &Url, content: &str) {
        if uri.path().ends_with(".art.vue") {
            self.update_art_virtual_docs(uri, content);
            return;
        }

        if is_standalone_html_path(uri.path()) {
            self.update_standalone_html_virtual_docs(uri, content);
            return;
        }

        if crate::utils::is_jsx_path(uri.path()) {
            self.update_jsx_virtual_docs(uri, content);
            return;
        }

        let options = vize_atelier_sfc::SfcParseOptions {
            filename: uri.path().to_string().into(),
            ..Default::default()
        };

        let Ok(descriptor) = vize_atelier_sfc::parse_sfc(content, options) else {
            self.remove_virtual_docs(uri);
            return;
        };

        let base_uri = uri.path();
        let virtual_docs = self.virtual_gen.write().generate(&descriptor, base_uri);
        self.virtual_docs_cache.insert(uri.clone(), virtual_docs);
    }

    /// Generate and cache virtual documents for standalone HTML files.
    fn update_standalone_html_virtual_docs(&self, uri: &Url, content: &str) {
        use crate::virtual_code::{TemplateCodeGenerator, VirtualDocuments};

        let allocator = vize_carton::Bump::new();
        let (ast, _errors) = vize_armature::parse(&allocator, content);
        let base_uri = uri.path();

        let mut template_gen = TemplateCodeGenerator::new();
        template_gen.set_block_offset(0);
        let mut template_doc = template_gen.generate(&ast, content);
        template_doc.uri = vize_carton::cstr!("{base_uri}.__template.ts").to_string();

        let mut docs = VirtualDocuments::new();
        docs.template = Some(template_doc);
        self.virtual_docs_cache.insert(uri.clone(), docs);
    }

    /// Generate and cache virtual documents for a `.jsx`/`.tsx` document.
    ///
    /// JSX/TSX components are not SFCs, so the only embedded-language virtual
    /// documents they produce are the CSS blocks of any `<style scoped>` (#1495,
    /// #1498). The type-aware features build their own per-request virtual TS
    /// (see [`crate::ide::JsxService`]); this cache only needs to expose the
    /// scoped CSS so the editor's CSS service gets diagnostics + source mapping,
    /// mirroring the SFC style virtual-document path.
    fn update_jsx_virtual_docs(&self, uri: &Url, content: &str) {
        let styles = crate::ide::JsxScopedStyleService::virtual_css_documents(content, uri);
        if styles.is_empty() {
            self.remove_virtual_docs(uri);
            return;
        }
        let mut docs = VirtualDocuments::new();
        docs.styles = styles;
        self.virtual_docs_cache.insert(uri.clone(), docs);
    }

    /// Generate and cache virtual documents for an art file (*.art.vue).
    ///
    /// Uses the default variant's template as the synthetic template block,
    /// and generates virtual docs for script_setup if present.
    fn update_art_virtual_docs(&self, uri: &Url, content: &str) {
        use crate::virtual_code::{ScriptCodeGenerator, TemplateCodeGenerator, VirtualDocuments};

        let allocator = vize_carton::Bump::new();
        let Ok(art_desc) =
            vize_musea::parse_art(&allocator, content, vize_musea::ArtParseOptions::default())
        else {
            self.remove_virtual_docs(uri);
            return;
        };

        let base_uri = uri.path();
        let mut docs = VirtualDocuments::new();

        // Generate one virtual template per variant so editor features remain correct even when
        // the cursor is inside a non-default variant.
        docs.art_templates.resize(art_desc.variants.len(), None);

        for (index, variant) in art_desc.variants.iter().enumerate() {
            let template_content = variant.template;
            if template_content.trim().is_empty() {
                continue;
            }

            let template_allocator = vize_carton::Bump::new();
            let (ast, _errors) = vize_armature::parse(&template_allocator, template_content);

            let template_ptr = template_content.as_ptr() as usize;
            let source_ptr = content.as_ptr() as usize;
            let block_offset = (template_ptr - source_ptr) as u32;

            let mut template_gen = TemplateCodeGenerator::new();
            template_gen.set_block_offset(block_offset);
            let mut template_doc = template_gen.generate(&ast, template_content);
            template_doc.uri =
                vize_carton::cstr!("{base_uri}.art_variant_{index}.template.ts").to_string();

            if variant.is_default || docs.template.is_none() {
                docs.template = Some(template_doc.clone());
            }

            docs.art_templates[index] = Some(template_doc);
        }

        // Generate script_setup virtual doc using SFC parser
        // (SFC parser handles script blocks even in art files)
        let sfc_options = vize_atelier_sfc::SfcParseOptions {
            filename: uri.path().to_string().into(),
            ..Default::default()
        };
        if let Ok(descriptor) = vize_atelier_sfc::parse_sfc(content, sfc_options) {
            if let Some(ref script_setup) = descriptor.script_setup {
                let isolate = art_script_setup_isolated(script_setup);
                let mut script_doc = generate_art_script_setup_virtual_doc(
                    base_uri,
                    script_setup.content.as_ref(),
                    script_setup.loc.start,
                    art_desc.variants.len(),
                    isolate,
                );
                script_doc.uri = vize_carton::cstr!("{base_uri}.__script_setup.ts").to_string();
                docs.script_setup = Some(script_doc);
            }
            if let Some(ref script) = descriptor.script {
                let mut script_gen = ScriptCodeGenerator::new();
                let mut script_doc = script_gen.generate(script, false);
                script_doc.uri = vize_carton::cstr!("{base_uri}.__script.ts").to_string();
                docs.script = Some(script_doc);
            }
        }

        self.virtual_docs_cache.insert(uri.clone(), docs);
    }

    /// Get cached virtual documents for a document.
    pub fn get_virtual_docs(
        &self,
        uri: &Url,
    ) -> Option<dashmap::mapref::one::Ref<'_, Url, VirtualDocuments>> {
        self.virtual_docs_cache.get(uri)
    }

    /// Remove cached virtual documents when a document is closed.
    pub fn remove_virtual_docs(&self, uri: &Url) {
        self.virtual_docs_cache.remove(uri);
    }

    /// Clear all cached virtual documents.
    pub fn clear_virtual_docs(&self) {
        self.virtual_docs_cache.clear();
    }

    /// Cache of parsed imported-component metadata, keyed by resolved path.
    /// Used by template completion to avoid re-parsing imported components on
    /// every keystroke. Callers handle staleness via the entry's file stamp.
    pub(crate) fn component_metadata_cache(
        &self,
    ) -> &DashMap<PathBuf, crate::ide::completion::template::CachedComponentMetadata> {
        &self.component_metadata_cache
    }
}

fn art_script_setup_isolated(script_setup: &vize_atelier_sfc::SfcScriptBlock<'_>) -> bool {
    !script_setup
        .attrs
        .get("isolate")
        .is_some_and(|value| value.as_ref().eq_ignore_ascii_case("false"))
}

fn generate_art_script_setup_virtual_doc(
    base_uri: &str,
    script: &str,
    source_start: usize,
    variant_count: usize,
    isolate: bool,
) -> crate::virtual_code::VirtualDocument {
    use crate::virtual_code::{
        MappingFeatures, SourceMap, SourceMapping, SourceRange, VirtualDocument, VirtualLanguage,
        analyze_art_script_setup,
    };

    let parts = analyze_art_script_setup(script, source_start, isolate);
    let mut content = String::new();
    let mut mappings = Vec::new();

    content.push_str("// Virtual TypeScript for .art.vue <script setup>\n");
    content.push_str("// Generated by vize_maestro\n\n");

    for import in &parts.shared_imports {
        let generated_start = content.len() as u32;
        content.push_str(&import.text);
        let generated_end = content.len() as u32;
        mappings.push(SourceMapping::with_features(
            SourceRange::new(import.source_start as u32, import.source_end as u32),
            SourceRange::new(generated_start, generated_end),
            MappingFeatures::all(),
        ));
        if !content.ends_with('\n') {
            content.push('\n');
        }
    }
    if !content.ends_with("\n\n") {
        content.push('\n');
    }

    if parts.isolate {
        let count = variant_count.max(1);
        for index in 0..count {
            vize_carton::append!(content, "function __VIZE_art_variant_{index}_setup() {{\n");
            for chunk in &parts.isolated_body {
                let generated_start = content.len() as u32;
                content.push_str(&chunk.text);
                let generated_end = content.len() as u32;
                mappings.push(SourceMapping::with_features(
                    SourceRange::new(chunk.source_start as u32, chunk.source_end as u32),
                    SourceRange::new(generated_start, generated_end),
                    MappingFeatures::all(),
                ));
                if !content.ends_with('\n') {
                    content.push('\n');
                }
            }
            content.push_str("}\n\n");
        }
    } else {
        content.push_str("// Shared <script setup isolate=\"false\">\n");
        for chunk in &parts.isolated_body {
            let generated_start = content.len() as u32;
            content.push_str(&chunk.text);
            let generated_end = content.len() as u32;
            mappings.push(SourceMapping::with_features(
                SourceRange::new(chunk.source_start as u32, chunk.source_end as u32),
                SourceRange::new(generated_start, generated_end),
                MappingFeatures::all(),
            ));
            if !content.ends_with('\n') {
                content.push('\n');
            }
        }
    }

    VirtualDocument {
        uri: vize_carton::cstr!("{base_uri}.__script_setup.ts").to_string(),
        content,
        language: VirtualLanguage::ScriptSetup,
        source_map: SourceMap::from_mappings(mappings),
    }
}
