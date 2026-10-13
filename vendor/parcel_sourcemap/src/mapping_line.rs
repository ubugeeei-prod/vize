use crate::mapping::OriginalLocation;
use crate::sourcemap_error::{SourceMapError, SourceMapErrorType};
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub struct LineMapping {
    pub generated_column: u32,
    pub original: Option<OriginalLocation>,
}

#[derive(Archive, Serialize, Deserialize, Debug, Default, Clone)]
pub struct MappingLine {
    pub mappings: Vec<LineMapping>,
    pub last_column: u32,
    pub is_sorted: bool,
}

impl MappingLine {
    pub fn new() -> Self {
        Self {
            mappings: Vec::new(),
            last_column: 0,
            is_sorted: true,
        }
    }

    pub fn add_mapping(&mut self, generated_column: u32, original: Option<OriginalLocation>) {
        if self.is_sorted && self.last_column > generated_column {
            self.is_sorted = false;
        }

        self.mappings.push(LineMapping {
            generated_column,
            original,
        });

        self.last_column = generated_column;
    }

    pub fn ensure_sorted(&mut self) {
        if !self.is_sorted {
            self.mappings
                .sort_by(|a, b| a.generated_column.cmp(&b.generated_column));
            self.is_sorted = true
        }
    }

    pub fn find_closest_mapping(&mut self, generated_column: u32) -> Option<LineMapping> {
        if self.mappings.is_empty() {
            return None;
        }

        self.ensure_sorted();
        let index = match self
            .mappings
            .binary_search_by(|m| m.generated_column.cmp(&generated_column))
        {
            Ok(index) => index,
            Err(index) => {
                if index == 0 || index == self.mappings.len() {
                    return Some(LineMapping {
                        generated_column: 0,
                        original: self.mappings[0].original,
                    });
                }

                index - 1
            }
        };

        Some(self.mappings[index])
    }

    pub fn offset_columns(
        &mut self,
        generated_column: u32,
        generated_column_offset: i64,
    ) -> Result<(), SourceMapError> {
        let (start_column, overflowed) =
            (generated_column as i64).overflowing_add(generated_column_offset);
        if overflowed || start_column > (u32::MAX as i64) {
            return Err(SourceMapError::new_with_reason(
                SourceMapErrorType::UnexpectedNegativeNumber,
                "column + column_offset cannot be negative",
            ));
        }

        self.ensure_sorted();
        let mut index = match self
            .mappings
            .binary_search_by(|m| m.generated_column.cmp(&generated_column))
        {
            Ok(index) => index,
            Err(index) => index,
        };

        if generated_column_offset < 0 {
            let u_start_column = start_column as u32;
            let start_index = match self
                .mappings
                .binary_search_by(|m| m.generated_column.cmp(&u_start_column))
            {
                Ok(index) => index,
                Err(index) => index,
            };

            self.mappings.drain(start_index..index);
            index = start_index;
        }

        let abs_offset = generated_column_offset.unsigned_abs() as u32;
        for i in index..self.mappings.len() {
            let mapping = &mut self.mappings[i];
            mapping.generated_column = if generated_column_offset < 0 {
                mapping.generated_column - abs_offset
            } else {
                mapping.generated_column + abs_offset
            };
        }

        Ok(())
    }
}
