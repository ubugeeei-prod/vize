//! Recording own template facts as files enter the analyzer.

use super::template::TemplateComplexity;
use crate::analyzer::CrossFileAnalyzer;
use crate::registry::FileId;

impl CrossFileAnalyzer {
    /// Record (or replace) the own template complexity of an SFC.
    ///
    /// The `add_file*` entry points call this for every `.vue` path with
    /// the source they receive. A host that registers only an SFC's script
    /// (the WASM binding does) calls it again with the whole SFC.
    pub fn record_template_complexity(&mut self, file_id: FileId, sfc_source: &str) {
        match TemplateComplexity::from_sfc(sfc_source) {
            Some(template) => {
                self.template_complexity.insert(file_id, template);
            }
            None => {
                self.template_complexity.remove(&file_id);
            }
        }
    }

    /// The recorded own template complexity of `file_id`.
    pub fn template_complexity_of(&self, file_id: FileId) -> Option<&TemplateComplexity> {
        self.template_complexity.get(&file_id)
    }
}
