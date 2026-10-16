//! Test evidence uses the production source owner on the diagnosing attachment.
use super::super::{
    EditorLspSession,
    snapshot_source::{
        SnapshotSourceOwner, SnapshotSourceProject, SourceTextOutcome, SourceTextRefusal,
    },
};
use crate::{file_uri::path_to_file_uri, lsp_client::session::uri_document_identifier};
use corsa::{api::ProjectResponse, runtime::block_on};
use serde_json::{Value, json};
use std::{fs, path::Path};
use vize_l0::{String, cstr};

impl EditorLspSession {
    pub(in crate::lsp_client) fn batch_source_receipt(
        &self,
        uri: &str,
        authority: &Value,
    ) -> Value {
        let Some(attachment) = self.configured_api.as_ref() else {
            return json!({"qualified":false,"error":"no retained diagnosing attachment"});
        };
        let mut owner = match SnapshotSourceOwner::create(&attachment.client) {
            Ok(owner) => owner,
            Err(error) => return json!({"qualified":false,"wholeSnapshotError":cstr!("{error}")}),
        };
        let mut observation = json!({"qualified":false,"attachment":attachment.session,
            "snapshot":owner.handle(),"snapshotProjects":owner.projects(),
            "scope":"complete current SDK descriptor; requests use the same native snapshot"});
        let result = (|| -> Result<(), String> {
            let native = block_on(attachment.client.get_default_project_for_file(
                owner.handle().clone(),
                uri_document_identifier(uri),
            ))
            .map_err(|error| cstr!("{error}"))?
            .ok_or_else(|| cstr!("diagnosing document has no native default project"))?;
            observation["nativeDefaultProject"] = json!(native);
            observation["sameDiagnosingDescriptor"] = json!(json!(native) == authority["project"]);
            observation["sameDiagnosingAttachment"] =
                json!(json!(attachment.session) == authority["attachment"]);
            let config = Path::new(native.config_file_name.as_str());
            observation["originalSingletonAdmission"] = match owner.project(config) {
                Ok(Ok(project)) => json!({"status":"admitted","project":project.descriptor()}),
                Ok(Err(reason)) => json!({"status":"refused","reason":cstr!("{reason:?}")}),
                Err(error) => json!({"status":"error","error":cstr!("{error}")}),
            };
            observation["descriptorRefusals"] = native_refusals(&owner, config, &native);
            let project = owner
                .project_matching(config, &native)
                .map_err(|error| cstr!("{error}"))?
                .map_err(|reason| cstr!("{reason:?}"))?;
            observation["selectedProject"] = json!(project.descriptor());
            observation["sourceFileNames"] = json!(project.source_names());
            observation["sameDiagnosingSourceNames"] = json!(
                json!(project.source_names())
                    == authority["sourceFileNamesResponse"]["wholeResponse"]
            );
            let leaf: Vec<_> = project
                .source_names()
                .iter()
                .filter(|name| {
                    Path::new(name.as_str()).ends_with("node_modules/leaf-types/index.d.ts")
                })
                .map(|name| read_source(&project, name))
                .collect();
            let authored: Vec<_> = project
                .source_names()
                .iter()
                .filter(|name| path_to_file_uri(Path::new(name.as_str())).as_str() == uri)
                .map(|name| read_source(&project, name))
                .collect();
            let exact = authored.len() == 1
                && leaf.len() <= 1
                && leaf
                    .iter()
                    .chain(&authored)
                    .all(|source| source["nativeMatchesPhysical"] == true);
            observation["loadedLeafSources"] = json!(leaf);
            observation["loadedAuthoredSources"] = json!(authored);
            observation["absentSourceRefused"] = json!(matches!(
                project.read("file:///__vize_absent_source__/source.ts"),
                Ok(SourceTextOutcome::Refused {
                    reason: SourceTextRefusal::SourceIdentity,
                    encoded: None
                })
            ));
            observation["qualified"] = json!(
                exact
                    && observation["sameDiagnosingDescriptor"] == true
                    && observation["sameDiagnosingAttachment"] == true
                    && observation["sameDiagnosingSourceNames"] == true
                    && observation["absentSourceRefused"] == true
                    && observation["descriptorRefusals"]
                        .as_object()
                        .unwrap()
                        .values()
                        .all(|value| value == true)
            );
            Ok(())
        })();
        if let Err(error) = result {
            observation["wholeSourceError"] = json!(error);
        }
        if let Err(error) = owner.release() {
            observation["wholeReleaseError"] = json!(cstr!("{error}"));
            observation["qualified"] = json!(false);
        }
        if let Ok(native) =
            serde_json::from_value::<ProjectResponse>(observation["nativeDefaultProject"].clone())
        {
            let refused = matches!(
                owner.project_matching(Path::new(native.config_file_name.as_str()), &native),
                Ok(Err(SourceTextRefusal::ReleasedSnapshot))
            );
            observation["releasedProjectRefused"] = json!(refused);
            if !refused {
                observation["qualified"] = json!(false);
            }
        }
        observation
    }
}

fn native_refusals(
    owner: &SnapshotSourceOwner<'_>,
    config: &Path,
    native: &ProjectResponse,
) -> Value {
    let refused = |config: &Path, descriptor: &ProjectResponse| {
        matches!(
            owner.project_matching(config, descriptor),
            Ok(Err(SourceTextRefusal::ProjectIdentity))
        )
    };
    let mut changed_options = native.clone();
    changed_options.compiler_options["strict"] =
        json!(!native.compiler_options["strict"].as_bool().unwrap_or(false));
    let mut changed_roots = native.clone();
    changed_roots
        .root_files
        .push("/__vize_absent_source__/source.ts".into());
    let mut changed_handle = native.clone();
    changed_handle.id = cstr!("{}-foreign", native.id.as_str()).as_str().into();
    json!({"absentConfigRefused":refused(Path::new("/__vize_absent_config__/tsconfig.json"),native),
        "relativeConfigRefused":refused(Path::new("tsconfig.json"),native),
        "foreignHandleRefused":refused(config,&changed_handle),
        "changedOptionsRefused":refused(config,&changed_options),
        "changedRootsRefused":refused(config,&changed_roots)})
}

fn read_source(project: &SnapshotSourceProject<'_>, name: &str) -> Value {
    let uri = path_to_file_uri(Path::new(name));
    match project.read(uri.as_str()) {
        Ok(SourceTextOutcome::Complete(text)) => match fs::read(text.file_name()) {
            Ok(bytes) => {
                json!({"nativeSourceName":name,"fileName":text.file_name(),"path":text.path(),
                "uri":text.uri(),"snapshot":text.snapshot_handle(),"project":text.project_descriptor(),
                "completeNativeText":text.text().as_bytes(),"wholeEncodedPayload":text.encoded_bytes(),
                "completePhysicalBytes":bytes,"nativeMatchesPhysical":bytes==text.text().as_bytes()})
            }
            Err(error) => {
                json!({"nativeSourceName":name,"nativeMatchesPhysical":false,"physicalError":cstr!("{error}")})
            }
        },
        Ok(SourceTextOutcome::Refused { reason, encoded }) => json!({"nativeSourceName":name,
            "nativeMatchesPhysical":false,"nativeRefusal":cstr!("{reason:?}"),
            "wholeEncodedRefusal":encoded.as_ref().map(|encoded|encoded.as_bytes())}),
        Err(error) => {
            json!({"nativeSourceName":name,"nativeMatchesPhysical":false,"nativeError":cstr!("{error}")})
        }
    }
}
