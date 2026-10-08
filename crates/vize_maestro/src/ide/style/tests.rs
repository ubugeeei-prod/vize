//! Product regression laws exercise the original resident SFC and LSP packets.

mod completion;
mod hover;
#[cfg(feature = "native")]
mod resolve;

use tower_lsp::lsp_types::{
    ClientCapabilities, CompletionItem, CompletionTextEdit, Documentation, Hover, HoverContents,
    MarkupKind, TextEdit, Url,
};

use crate::{ide::IdeContext, server::ServerState};

struct Document {
    state: ServerState,
    uri: Url,
    offset: usize,
}

impl Document {
    fn css(marked: &str, lazy: bool) -> Self {
        let source = [
            "<template><div /></template>\n<style scoped>\n",
            marked,
            "\n</style>",
        ]
        .concat();
        Self::marked(&source, lazy)
    }

    fn marked(source: &str, lazy: bool) -> Self {
        let offset = source
            .find('|')
            .expect("test must identify the actual cursor");
        let source = source.replacen('|', "", 1);
        let state = ServerState::new();
        state.apply_lsp_initialization_options(Some(&serde_json::json!({ "typecheck": false })));
        assert!(!state.is_lsp_typecheck_enabled());
        if lazy {
            let capabilities: ClientCapabilities = serde_json::from_value(serde_json::json!({
                "textDocument": { "completion": { "completionItem": {
                    "resolveSupport": { "properties": ["documentation"] }
                } } }
            }))
            .unwrap();
            state.record_client_capabilities(&capabilities);
            assert_eq!(
                state.supports_completion_documentation_resolve(),
                cfg!(feature = "native")
            );
        } else {
            assert!(!state.supports_completion_documentation_resolve());
        }
        let uri = Url::parse("file:///css-documentation/App.vue").unwrap();
        state.documents.open(uri.clone(), source, 1, "vue".into());
        Self { state, uri, offset }
    }

    fn fixture(needle: &str, offset: usize) -> Self {
        let source = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/css-documentation/App.vue"
        ));
        let position = source.find(needle).unwrap() + offset;
        let marked = [
            source.get(..position).unwrap(),
            "|",
            source.get(position..).unwrap(),
        ]
        .concat();
        Self::marked(&marked, false)
    }

    fn context(&self) -> IdeContext<'_> {
        IdeContext::new(&self.state, &self.uri, self.offset).unwrap()
    }

    fn complete(&self) -> Vec<CompletionItem> {
        let ctx = IdeContext::at_completion(&self.state, &self.uri, self.offset).unwrap();
        super::complete(&ctx, 0)
    }
}

fn item<'a>(items: &'a [CompletionItem], label: &str) -> &'a CompletionItem {
    items
        .iter()
        .find(|item| item.label == label)
        .unwrap_or_else(|| {
            panic!(
                "missing CSS item {label}; got {:?}",
                items.iter().map(|item| &item.label).collect::<Vec<_>>()
            )
        })
}

fn edit(item: &CompletionItem) -> &TextEdit {
    match item
        .text_edit
        .as_ref()
        .expect("ordinary CSS completion must replace its token")
    {
        CompletionTextEdit::Edit(edit) => edit,
        CompletionTextEdit::InsertAndReplace(_) => panic!("unexpected insert/replace edit"),
    }
}

fn documentation(item: &CompletionItem) -> &str {
    match item
        .documentation
        .as_ref()
        .expect("selected CSS completion needs documentation")
    {
        Documentation::MarkupContent(content) => {
            assert_eq!(content.kind, MarkupKind::Markdown);
            &content.value
        }
        Documentation::String(_) => panic!("CSS documentation must be rich Markdown"),
    }
}

fn hover_documentation(hover: &Hover) -> &str {
    match &hover.contents {
        HoverContents::Markup(content) => {
            assert_eq!(content.kind, MarkupKind::Markdown);
            &content.value
        }
        _ => panic!("CSS hover must be rich Markdown"),
    }
}
