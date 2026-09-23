//! Keep the cached default template aligned with its art-variant document.
//!
//! Variant templates are already the checker virtual TypeScript. This only
//! republishes the default variant into `documents.template`.

use crate::virtual_code::VirtualDocuments;

pub(super) fn attach(documents: &mut VirtualDocuments) {
    let Some(default_template_uri) = documents
        .template
        .as_ref()
        .map(|template| template.uri.clone())
    else {
        return;
    };
    if let Some(default_template) = documents
        .art_templates
        .iter()
        .flatten()
        .find(|template| template.uri == default_template_uri)
    {
        documents.template = Some(default_template.clone());
    }
}
