use corsa::api::ProjectSession;
use std::path::PathBuf;
use vize_carton::String;

mod errors;
mod paths;
mod probe;
mod session;

#[cfg(test)]
mod tests;

pub(super) type TypeProbe = corsa::api::TypeProbe;

pub(crate) struct CorsaTypeAwareSession {
    session: ProjectSession,
    project_root: PathBuf,
    session_root: PathBuf,
    virtual_file_path: PathBuf,
    virtual_file_wire: String,
    supports_overlay_updates: bool,
    overlay_version: i32,
    closed: bool,
}

impl Drop for CorsaTypeAwareSession {
    fn drop(&mut self) {
        self.close();
    }
}
