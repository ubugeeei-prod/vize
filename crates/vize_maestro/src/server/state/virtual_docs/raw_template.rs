//! Virtual documents projected from the persistent raw-template frontend.

use tower_lsp::lsp_types::Url;

use crate::virtual_code::{TemplateCodeGenerator, VirtualDocuments};

use super::ServerState;

impl ServerState {
    /// Generate and cache virtual documents for standalone HTML files.
    pub(super) fn update_standalone_html_virtual_docs(&self, uri: &Url, content: &str) {
        self.ensure_artifact_source(uri, content);
        let Some(syntax) = self.raw_template_relief(uri) else {
            self.remove_virtual_docs(uri);
            return;
        };
        let Some(syntax) = syntax.as_ref() else {
            self.remove_virtual_docs(uri);
            return;
        };
        let allocator = vize_carton::Bump::new();
        let ast = syntax.snapshot().materialize(&allocator);
        let base_uri = uri.path();

        let mut template_gen = TemplateCodeGenerator::new();
        template_gen.set_block_offset(0);
        let mut template_doc = template_gen.generate(&ast, content);
        template_doc.uri = vize_carton::cstr!("{base_uri}.__template.ts").to_string();

        let mut docs = VirtualDocuments::new();
        docs.template = Some(template_doc);
        self.virtual_docs_cache.insert(uri.clone(), docs);
    }
}
