//! Test-only binding to one complete diagnosing descriptor in a real snapshot.
//! The production singleton snapshot admission remains unchanged.
#![expect(
    clippy::unwrap_used,
    reason = "test-only complete native metadata corruption controls"
)]

use super::super::{SnapshotSourceOwner, SnapshotSourceProject, SourceTextRefusal, source_members};
use corsa::{CorsaError, api::ProjectResponse, runtime::block_on};
use serde_json::{Value, json};
use std::path::Path;
use vize_l0::String;

impl SnapshotSourceOwner<'_> {
    pub(crate) fn whole_batch_snapshot_projects_for_test(&self) -> &[ProjectResponse] {
        &self.snapshot.projects
    }

    pub(crate) fn batch_project_for_test(
        &self,
        config: &Path,
        diagnosing: &Value,
    ) -> Result<Result<SnapshotSourceProject<'_>, SourceTextRefusal>, CorsaError> {
        if self.released {
            return Ok(Err(SourceTextRefusal::ReleasedSnapshot));
        }
        let Some(descriptor) = exact_project(&self.snapshot.projects, config, diagnosing) else {
            return Ok(Err(SourceTextRefusal::ProjectIdentity));
        };
        let response = block_on(self.client.raw_json_request(
            "getSourceFileNames",
            json!({"snapshot":self.snapshot.handle,"project":descriptor.id}),
        ))?;
        let Ok(names) = serde_json::from_value::<Vec<String>>(response) else {
            return Ok(Err(SourceTextRefusal::SourceNames));
        };
        let Some(sources) = source_members(&names) else {
            return Ok(Err(SourceTextRefusal::SourceNames));
        };
        Ok(Ok(SnapshotSourceProject {
            owner: self,
            descriptor,
            sources,
            source_names: names,
        }))
    }

    /// Corrupt only captured metadata; these refusals grant no native read credit.
    pub(crate) fn batch_project_identity_refusals_for_test(
        &self,
        config: &Path,
        diagnosing: &Value,
    ) -> Value {
        let original = json!(self.snapshot.projects);
        let mut duplicate = original.as_array().unwrap().clone();
        duplicate.push(diagnosing.clone());
        let duplicate: Vec<ProjectResponse> = serde_json::from_value(json!(duplicate)).unwrap();
        let mut foreign = diagnosing.clone();
        foreign["configFileName"] = json!("/__vize_foreign_snapshot__/tsconfig.json");
        let mut changed = diagnosing.clone();
        changed["compilerOptions"]["strict"] = json!(false);
        json!({
            "scope":"refusal controls over the complete captured native project vector; local metadata corruptions are not native answers",
            "wholeOriginalProjects":original,"wholeDiagnosingDescriptor":diagnosing,
            "wholeDuplicateProjects":duplicate,"wholeForeignDescriptor":foreign,
            "wholeChangedDescriptor":changed,
            "absentProjectRefused":exact_project(&[],config,diagnosing).is_none(),
            "foreignDescriptorRefused":exact_project(&self.snapshot.projects,config,&foreign).is_none(),
            "duplicateProjectRefused":exact_project(&duplicate,config,diagnosing).is_none(),
            "changedWholeDescriptorRefused":exact_project(&self.snapshot.projects,config,&changed).is_none(),
        })
    }
}

fn exact_project<'project>(
    projects: &'project [ProjectResponse],
    config: &Path,
    diagnosing: &Value,
) -> Option<&'project ProjectResponse> {
    let config = config
        .to_str()
        .filter(|path| Path::new(path).is_absolute())?;
    let mut matching = projects
        .iter()
        .filter(|project| project.config_file_name.as_str() == config);
    let selected = matching.next()?;
    if matching.next().is_some() || json!(selected) != *diagnosing {
        return None;
    }
    Some(selected)
}

#[test]
fn selected_config_requires_unique_whole_descriptor_not_first_project() {
    let descriptor = json!({"id":"/workspace/tsconfig.json","configFileName":"/workspace/tsconfig.json",
        "compilerOptions":{"strict":true},"rootFiles":["/workspace/source.ts"]});
    let configured: ProjectResponse = serde_json::from_value(descriptor.clone()).unwrap();
    let unrelated: ProjectResponse = serde_json::from_value(json!({"id":"/dev/null/inferred",
        "configFileName":"/dev/null/inferred","compilerOptions":{},"rootFiles":[]}))
    .unwrap();
    let projects = [unrelated, configured];
    let config = Path::new("/workspace/tsconfig.json");
    assert!(exact_project(&projects, config, &descriptor).is_some());
    assert!(exact_project(&projects, Path::new("tsconfig.json"), &descriptor).is_none());
    assert!(exact_project(&[], config, &descriptor).is_none());
    let mut changed = descriptor.clone();
    changed["rootFiles"] = json!(["/foreign/source.ts"]);
    assert!(exact_project(&projects, config, &changed).is_none());
    let duplicate: ProjectResponse = serde_json::from_value(descriptor.clone()).unwrap();
    let mut duplicated = projects.to_vec();
    duplicated.push(duplicate);
    assert!(exact_project(&duplicated, config, &descriptor).is_none());
}
