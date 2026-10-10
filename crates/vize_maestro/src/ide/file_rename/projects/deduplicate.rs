//! Typed hashed identities preserve edit order and annotation differences.
use tower_lsp::lsp_types::{
    DocumentChangeOperation, DocumentChanges, OneOf, TextEdit, Url, WorkspaceEdit,
};
use vize_l0::{FxHashSet, String};

#[derive(Hash, Eq, PartialEq)]
struct Identity {
    uri: Url,
    start: (u32, u32),
    end: (u32, u32),
    text: String,
    annotation: Option<String>,
}
impl Identity {
    fn new(uri: &Url, edit: &TextEdit, annotation: Option<&str>) -> Self {
        Self {
            uri: uri.clone(),
            start: (edit.range.start.line, edit.range.start.character),
            end: (edit.range.end.line, edit.range.end.character),
            text: edit.new_text.as_str().into(),
            annotation: annotation.map(Into::into),
        }
    }
}

pub(super) fn edits(edit: &mut WorkspaceEdit) {
    if let Some(changes) = &mut edit.changes {
        for (uri, edits) in changes {
            let mut seen = FxHashSet::default();
            edits.retain(|edit| seen.insert(Identity::new(uri, edit, None)));
        }
    }
    let mut seen = FxHashSet::default();
    let mut keep = |edit: &mut tower_lsp::lsp_types::TextDocumentEdit| {
        edit.edits.retain(|change| {
            let identity = match change {
                OneOf::Left(text) => Identity::new(&edit.text_document.uri, text, None),
                OneOf::Right(annotated) => Identity::new(
                    &edit.text_document.uri,
                    &annotated.text_edit,
                    Some(&annotated.annotation_id),
                ),
            };
            seen.insert(identity)
        });
        !edit.edits.is_empty()
    };
    match &mut edit.document_changes {
        Some(DocumentChanges::Edits(edits)) => edits.retain_mut(&mut keep),
        Some(DocumentChanges::Operations(operations)) => {
            operations.retain_mut(|operation| match operation {
                DocumentChangeOperation::Edit(edit) => keep(edit),
                DocumentChangeOperation::Op(_) => true,
            })
        }
        None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::edits;
    use serde_json::json;
    use tower_lsp::lsp_types::{AnnotatedTextEdit, DocumentChanges, OneOf, WorkspaceEdit};

    #[test]
    fn exact_duplicates_collapse_but_ranges_text_and_annotation_ids_remain_distinct() {
        let uri = "file:///workspace/App.vue";
        let text = json!({"range":{"start":{"line":1,"character":2},"end":{"line":1,"character":4}},"newText":"next"});
        let a = json!({"range":text["range"],"newText":"next","annotationId":"a"});
        let b = json!({"range":text["range"],"newText":"next","annotationId":"b"});
        let distinct = json!({"range":text["range"],"newText":"other"});
        let document = |edits| json!({"textDocument":{"uri":uri,"version":2},"edits":edits});
        let mut edit: WorkspaceEdit = serde_json::from_value(
            json!({"documentChanges":[document(json!([text,text,a,a,b,distinct]))]}),
        )
        .unwrap();
        // OneOf's generic JSON decoder prefers TextEdit and ignores extra
        // fields. Construct its typed annotated variant to test real identity.
        let Some(DocumentChanges::Edits(documents)) = &mut edit.document_changes else {
            panic!("document edits")
        };
        let annotated =
            |value| OneOf::Right(serde_json::from_value::<AnnotatedTextEdit>(value).unwrap());
        documents.first_mut().unwrap().edits = vec![
            OneOf::Left(serde_json::from_value(text.clone()).unwrap()),
            OneOf::Left(serde_json::from_value(text.clone()).unwrap()),
            annotated(a.clone()),
            annotated(a.clone()),
            annotated(b.clone()),
            OneOf::Left(serde_json::from_value(distinct.clone()).unwrap()),
        ];
        edits(&mut edit);
        assert_eq!(
            serde_json::to_value(edit).unwrap(),
            json!({"documentChanges":[document(json!([text,a,b,distinct]))]})
        );
        let mut edit =
            serde_json::from_value(json!({"changes":{uri:[text,text,distinct]}})).unwrap();
        edits(&mut edit);
        assert_eq!(
            serde_json::to_value(edit).unwrap(),
            json!({"changes":{uri:[text,distinct]}})
        );
    }
}
