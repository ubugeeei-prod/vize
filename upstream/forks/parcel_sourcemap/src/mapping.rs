use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct OriginalLocation {
    pub original_line: u32,
    pub original_column: u32,
    pub source: u32,
    pub name: Option<u32>,
}

impl OriginalLocation {
    pub fn new(original_line: u32, original_column: u32, source: u32, name: Option<u32>) -> Self {
        Self {
            original_line,
            original_column,
            source,
            name,
        }
    }
}

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Mapping {
    pub generated_line: u32,
    pub generated_column: u32,
    pub original: Option<OriginalLocation>,
}
