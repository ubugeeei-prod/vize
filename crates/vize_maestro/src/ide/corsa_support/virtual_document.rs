use std::sync::Arc;

use tower_lsp::lsp_types::Url;
use vize_carton::String;

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

    if path == super::template_request_path(ctx.uri).as_str() {
        return virtual_docs
            .template
            .as_ref()
            .map(CurrentVirtualDocument::Template);
    }

    for (variant_index, template) in virtual_docs.art_templates.iter().enumerate() {
        if path == super::art_template_request_path(ctx.uri, variant_index).as_str() {
            return template.as_ref().map(CurrentVirtualDocument::Template);
        }
    }

    if path == super::script_request_path(ctx.uri, false).as_str() {
        return virtual_docs
            .script
            .as_ref()
            .map(CurrentVirtualDocument::Script);
    }

    if path == super::script_request_path(ctx.uri, true).as_str() {
        return virtual_docs
            .script_setup
            .as_ref()
            .map(CurrentVirtualDocument::ScriptSetup);
    }

    None
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
