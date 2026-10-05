//! Test-only access to the existing owner's complete diagnostic generation.

use super::{CorsaExecutor, VirtualProject};
use serde_json::{Value, json};

impl CorsaExecutor {
    pub(crate) fn qualify_native_bulk_for_test(&self, project: &VirtualProject) -> Value {
        let mut state = self
            .incremental_session
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(session) = state.session.as_mut() else {
            return json!({"ownerError":"the original session is absent"});
        };
        let mut packet = session
            .client
            .qualify_native_bulk_for_test(&session.snapshot.uris);
        packet["generatedConfig"] = json!({
            "path":project.generated_tsconfig_path(),
            "bytes":std::fs::read_to_string(project.generated_tsconfig_path()).unwrap(),
        });
        packet["generatedFiles"] = json!(
            project
                .virtual_files_sorted()
                .into_iter()
                .map(|file| {
                    json!({"original":file.original_path,"virtual":file.virtual_path,
                "code":file.content.as_str(),"sourceMap":vize_l0::cstr!("{:#?}",file.source_map)})
                })
                .collect::<Vec<_>>()
        );
        packet
    }
}
