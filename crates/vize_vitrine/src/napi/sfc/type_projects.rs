//! Read-only project watch metadata from the captured configuration authority.

use napi_derive::napi;
use std::path::Path;

#[napi(js_name = "typeProjectOwners")]
pub fn type_project_owners(path: String) -> Vec<String> {
    vize::type_projects::type_project_owners(Path::new(&path))
        .iter()
        .map(|root| root.to_string_lossy().into_owned())
        .collect()
}

#[napi(js_name = "typeProjectInputs")]
pub fn type_project_inputs(
    root: String,
    revision: String,
    known_revision: Option<String>,
) -> napi::Result<Option<Vec<String>>> {
    let revision = revision.parse::<u64>().map_err(|_| {
        napi::Error::new(
            napi::Status::InvalidArg,
            "Invalid TypeScript project revision",
        )
    })?;
    let known_revision = known_revision
        .map(|value| value.parse::<u64>())
        .transpose()
        .map_err(|_| {
            napi::Error::new(
                napi::Status::InvalidArg,
                "Invalid known TypeScript project revision",
            )
        })?;
    Ok(
        vize::type_projects::type_project_inputs(Path::new(&root), revision, known_revision).map(
            |paths| {
                paths
                    .iter()
                    .map(|path| path.to_string_lossy().into_owned())
                    .collect()
            },
        ),
    )
}
