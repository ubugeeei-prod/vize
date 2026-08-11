use std::fs;

use tower_lsp::lsp_types::TextDocumentContentChangeEvent;

use super::{
    CHILD_SOURCE, PARENT_SOURCE, RealCorsaRenameSession, assert_exact_event_edit, strict_rename,
};
use crate::ide::IdeContext;

const CHILD_SOURCE_V2: &str = r#"<script setup lang="ts">
const generationTwo = true;
defineEmits<{ saveItem: [id: string] }>();
</script>
"#;

impl RealCorsaRenameSession {
    pub(in crate::ide::rename::corsa_session_tests) fn assert_parent_queries_before_child_change(
        &self,
    ) -> Result<(), String> {
        assert_parent_rename(
            self,
            CHILD_SOURCE,
            "generationOne",
            "generation-one",
            "generationOne",
        )?;
        assert_parent_rename(
            self,
            CHILD_SOURCE,
            "stableGeneration",
            "stable-generation",
            "stableGeneration",
        )
    }

    pub(in crate::ide::rename::corsa_session_tests) fn assert_child_change_rearms_parent_rename(
        &mut self,
    ) -> Result<(), String> {
        let changed = self.state.documents.apply_changes(
            &self.child_uri,
            vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: CHILD_SOURCE_V2.to_owned(),
            }],
            2,
        );
        if !changed {
            return Err("child overlay did not advance to generation two".to_owned());
        }
        self.state
            .update_virtual_docs(&self.child_uri, CHILD_SOURCE_V2);
        let child_path = self
            .child_uri
            .to_file_path()
            .map_err(|()| "child URI is not a file path".to_owned())?;
        fs::write(&child_path, CHILD_SOURCE_V2).map_err(|error| error.to_string())?;

        assert_parent_rename(
            self,
            CHILD_SOURCE_V2,
            "changedEvent",
            "changed-event",
            "changedEvent",
        )?;
        assert_parent_rename(
            self,
            CHILD_SOURCE_V2,
            "settledEvent",
            "settled-event",
            "settledEvent",
        )?;
        self.expected_readiness_generations += 1;
        Ok(())
    }
}

fn assert_parent_rename(
    session: &RealCorsaRenameSession,
    child_source: &str,
    new_name: &str,
    expected_parent: &str,
    expected_child: &str,
) -> Result<(), String> {
    let parent_start = PARENT_SOURCE
        .find("save-item")
        .ok_or_else(|| "parent event marker missing".to_owned())?;
    let parent_ctx = IdeContext::new(&session.state, &session.parent_uri, parent_start + 2)
        .ok_or_else(|| "missing parent context".to_owned())?;
    let parent_edit = strict_rename(&parent_ctx, session.bridge.as_ref(), new_name)?;
    assert_exact_event_edit(
        &parent_edit,
        (
            &session.parent_uri,
            PARENT_SOURCE,
            "save-item",
            expected_parent,
        ),
        (&session.child_uri, child_source, "saveItem", expected_child),
    )
}
