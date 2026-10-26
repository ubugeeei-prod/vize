use std::sync::Arc;

use tower_lsp::lsp_types::Url;
use vize_l0::String;

use crate::ide::IdeContext;
use crate::virtual_code::{VirtualDocument, VirtualDocuments};

#[derive(Clone, Copy)]
pub(super) enum VirtualDocumentKind {
    Template,
    ArtTemplate(usize),
    Script,
    ScriptSetup,
}

enum CurrentVirtualDocument<'a> {
    Template(&'a VirtualDocument),
    Script(&'a VirtualDocument),
    ScriptSetup(&'a VirtualDocument),
}

impl<'a> CurrentVirtualDocument<'a> {
    fn document(&self) -> &'a VirtualDocument {
        match self {
            Self::Template(doc) | Self::Script(doc) | Self::ScriptSetup(doc) => doc,
        }
    }
}

pub(super) enum MatchedVirtualDocument<'a> {
    Current {
        uri: &'a Url,
        content: &'a str,
        document: &'a VirtualDocument,
    },
    Cached {
        uri: Url,
        content: String,
        documents: Arc<VirtualDocuments>,
        kind: VirtualDocumentKind,
    },
}

impl MatchedVirtualDocument<'_> {
    pub(super) fn uri(&self) -> &Url {
        match self {
            Self::Current { uri, .. } => uri,
            Self::Cached { uri, .. } => uri,
        }
    }

    pub(super) fn content(&self) -> &str {
        match self {
            Self::Current { content, .. } => content,
            Self::Cached { content, .. } => content.as_str(),
        }
    }

    pub(super) fn document(&self) -> Option<&VirtualDocument> {
        match self {
            Self::Current { document, .. } => Some(document),
            Self::Cached {
                documents, kind, ..
            } => match kind {
                VirtualDocumentKind::Template => documents.template.as_ref(),
                VirtualDocumentKind::ArtTemplate(index) => documents.art_template(*index),
                VirtualDocumentKind::Script => documents.script.as_ref(),
                VirtualDocumentKind::ScriptSetup => documents.script_setup.as_ref(),
            },
        }
    }
}

pub(super) fn match_virtual_document<'a>(
    ctx: &'a IdeContext<'_>,
    uri: &str,
) -> Option<MatchedVirtualDocument<'a>> {
    if let Some(current) = match_current_virtual_document(ctx, uri) {
        return Some(MatchedVirtualDocument::Current {
            uri: ctx.uri,
            content: &ctx.content,
            document: current.document(),
        });
    }

    let (authored_uri, kind) = virtual_document_target(uri)?;
    let documents = ctx.state.get_virtual_docs(&authored_uri)?;
    let content = ctx.state.documents.text(&authored_uri).or_else(|| {
        authored_uri
            .to_file_path()
            .ok()
            .and_then(|path| std::fs::read_to_string(path).ok())
    })?;
    let target = MatchedVirtualDocument::Cached {
        uri: authored_uri,
        content: content.into(),
        documents,
        kind,
    };
    target.document()?;
    Some(target)
}

pub(super) fn is_virtual_document_uri(uri: &str) -> bool {
    virtual_document_target(uri).is_some()
}

fn virtual_document_target(uri: &str) -> Option<(Url, VirtualDocumentKind)> {
    let mut parsed = Url::parse(uri).ok()?;
    let path = parsed.path();
    let (authored_path, kind) = if let Some(without_template) = path.strip_suffix(".template.ts") {
        if let Some((authored, index)) = without_template.rsplit_once(".art_variant_") {
            (
                authored,
                VirtualDocumentKind::ArtTemplate(index.parse().ok()?),
            )
        } else {
            (without_template, VirtualDocumentKind::Template)
        }
    } else if let Some(authored) = path.strip_suffix(".script.ts") {
        (authored, VirtualDocumentKind::Script)
    } else {
        let authored = path.strip_suffix(".setup.ts")?;
        (authored, VirtualDocumentKind::ScriptSetup)
    };
    if !authored_path.ends_with(".vue") {
        return None;
    }

    let authored_path = authored_path.to_string();
    parsed.set_path(&authored_path);
    parsed.set_query(None);
    parsed.set_fragment(None);
    Some((parsed, kind))
}

fn match_current_virtual_document<'a>(
    ctx: &'a IdeContext<'_>,
    uri: &str,
) -> Option<CurrentVirtualDocument<'a>> {
    let path = virtual_document_path(uri)?;
    let virtual_docs = ctx.virtual_docs.as_ref()?;
    let is = |request: String| is_request_path(&path, &request);

    if is(super::template_request_path(ctx.uri)) {
        return virtual_docs
            .template
            .as_ref()
            .map(CurrentVirtualDocument::Template);
    }

    for (variant_index, template) in virtual_docs.art_templates.iter().enumerate() {
        if is(super::art_template_request_path(ctx.uri, variant_index)) {
            return template.as_ref().map(CurrentVirtualDocument::Template);
        }
    }

    if is(super::script_request_path(ctx.uri, false)) {
        return virtual_docs
            .script
            .as_ref()
            .map(CurrentVirtualDocument::Script);
    }

    if is(super::script_request_path(ctx.uri, true)) {
        return virtual_docs
            .script_setup
            .as_ref()
            .map(CurrentVirtualDocument::ScriptSetup);
    }

    None
}

/// Request names reuse the authored URI's encoded path as a file path, so the
/// native answer encodes it once more (`%5B` becomes `%255B`); a batch-opened
/// projection is spelled with a single encoding.
fn is_request_path(path: &str, request: &str) -> bool {
    path == request
        || vize_canon::decode_uri_path(path).is_some_and(|decoded| {
            decoded == request || vize_canon::decode_uri_path(request).as_ref() == Some(&decoded)
        })
}

pub(super) fn virtual_document_path(uri: &str) -> Option<String> {
    if let Ok(parsed) = Url::parse(uri) {
        return Some(parsed.path().to_string().into());
    }

    if let Some(path) = uri.strip_prefix("vize-virtual://") {
        return Some(path.to_string().into());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::is_request_path;

    #[test]
    fn special_request_names_match_their_native_spelling() {
        let request = "/app/pages/%5Bid%3Dint%5D.art.vue.setup.ts";
        for path in [
            request,
            "/app/pages/%255Bid%253Dint%255D.art.vue.setup.ts",
            "/app/pages/%5Bid=int%5D.art.vue.setup.ts",
        ] {
            assert!(is_request_path(path, request), "{path}");
        }
        assert!(!is_request_path(
            "/app/pages/%5Bother%5D.art.vue.setup.ts",
            request
        ));
    }
}
