use std::cell::RefCell;
use std::path::{Path, PathBuf};

use serde_json::Value;
use vize_carton::typescript_project::{CompilerConfigHost, parse_jsonc_value};
use vize_carton::{FxHashMap, String, cstr};

use super::super::super::{FileStamp, file_stamp};

#[derive(Default)]
pub(super) struct FileHost {
    observations: RefCell<FxHashMap<PathBuf, FileStamp>>,
}

impl FileHost {
    pub(super) fn observations(&self) -> Vec<(PathBuf, FileStamp)> {
        self.observations
            .borrow()
            .iter()
            .map(|(path, stamp)| (path.clone(), *stamp))
            .collect()
    }
}

impl CompilerConfigHost for FileHost {
    type Error = String;

    fn read_jsonc(&self, path: &Path) -> Result<Value, String> {
        self.observe(path);
        let source = std::fs::read_to_string(path)
            .map_err(|error| cstr!("cannot read TypeScript project {}: {error}", path.display()))?;
        parse_jsonc_value(&source)
            .map_err(|error| cstr!("invalid TypeScript project {}: {error}", path.display()))
    }

    fn configuration_error(&self, message: String) -> String {
        message
    }

    fn observe(&self, path: &Path) {
        self.observations
            .borrow_mut()
            .entry(path.to_path_buf())
            .or_insert_with(|| file_stamp(path));
    }
}
