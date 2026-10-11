#![deny(clippy::all)]

pub mod mapping;
pub mod mapping_line;
pub mod sourcemap_error;
pub mod utils;
mod vlq_utils;

use crate::utils::make_relative_path;
#[cfg(feature = "json")]
use data_url::DataUrl;
pub use mapping::{Mapping, OriginalLocation};
use mapping_line::MappingLine;
pub use sourcemap_error::{SourceMapError, SourceMapErrorType};
#[cfg(feature = "json")]
use std::borrow::Cow;
use std::io;

use rkyv::{
    archived_root,
    ser::{
        serializers::{AlignedSerializer, AllocScratch, CompositeSerializer},
        Serializer,
    },
    AlignedVec, Archive, Deserialize, Infallible, Serialize,
};

use vlq_utils::{is_mapping_separator, read_relative_vlq};

#[derive(Archive, Serialize, Deserialize, Debug, Default, Clone)]
pub struct SourceMapInner {
    pub sources: Vec<String>,
    pub sources_content: Vec<String>,
    pub names: Vec<String>,
    pub mapping_lines: Vec<MappingLine>,
}

#[derive(Debug, Clone)]
pub struct SourceMap {
    pub project_root: String,
    inner: SourceMapInner,
}

impl SourceMap {
    pub fn new(project_root: &str) -> Self {
        Self {
            project_root: String::from(project_root),
            inner: SourceMapInner::default(),
        }
    }

    fn ensure_lines(&mut self, generated_line: usize) {
        let mut line = self.inner.mapping_lines.len();
        if line <= generated_line {
            self.inner
                .mapping_lines
                .reserve(generated_line - self.inner.mapping_lines.len() + 1);
            while line <= generated_line {
                self.inner.mapping_lines.push(MappingLine::new());
                line += 1;
            }
        }
    }

    pub fn add_mapping(
        &mut self,
        generated_line: u32,
        generated_column: u32,
        original: Option<OriginalLocation>,
    ) {
        // TODO: Create new public function that validates if source and name exist?
        self.ensure_lines(generated_line as usize);
        self.inner.mapping_lines[generated_line as usize].add_mapping(generated_column, original);
    }

    pub fn add_mapping_with_offset(
        &mut self,
        mapping: Mapping,
        line_offset: i64,
        column_offset: i64,
    ) -> Result<(), SourceMapError> {
        let (generated_line, generated_line_overflowed) =
            (mapping.generated_line as i64).overflowing_add(line_offset);
        if generated_line_overflowed || generated_line > (u32::MAX as i64) {
            return Err(SourceMapError::new_with_reason(
                SourceMapErrorType::UnexpectedlyBigNumber,
                "mapping.generated_line + line_offset",
            ));
        }

        if generated_line < 0 {
            return Err(SourceMapError::new_with_reason(
                SourceMapErrorType::UnexpectedNegativeNumber,
                "mapping.generated_line + line_offset",
            ));
        }

        let (generated_column, generated_column_overflowed) =
            (mapping.generated_column as i64).overflowing_add(column_offset);
        if generated_column_overflowed || generated_column > (u32::MAX as i64) {
            return Err(SourceMapError::new_with_reason(
                SourceMapErrorType::UnexpectedlyBigNumber,
                "mapping.generated_column + column_offset",
            ));
        }

        if generated_column < 0 {
            return Err(SourceMapError::new_with_reason(
                SourceMapErrorType::UnexpectedNegativeNumber,
                "mapping.generated_column + column_offset",
            ));
        }

        self.add_mapping(
            generated_line as u32,
            generated_column as u32,
            mapping.original,
        );
        Ok(())
    }

    pub fn find_closest_mapping(
        &mut self,
        generated_line: u32,
        generated_column: u32,
    ) -> Option<Mapping> {
        if let Some(line) = self.inner.mapping_lines.get_mut(generated_line as usize) {
            if let Some(line_mapping) = line.find_closest_mapping(generated_column) {
                return Some(Mapping {
                    generated_line,
                    generated_column: line_mapping.generated_column,
                    original: line_mapping.original,
                });
            }
        }

        None
    }

    pub fn get_mappings(&self) -> Vec<Mapping> {
        let mut mappings = Vec::new();
        for (generated_line, mapping_line) in self.inner.mapping_lines.iter().enumerate() {
            for mapping in mapping_line.mappings.iter() {
                mappings.push(Mapping {
                    generated_line: generated_line as u32,
                    generated_column: mapping.generated_column,
                    original: mapping.original,
                });
            }
        }
        mappings
    }

    pub fn write_vlq<W>(&mut self, output: &mut W) -> Result<(), SourceMapError>
    where
        W: io::Write,
    {
        let mut last_generated_line: u32 = 0;
        let mut previous_source: i64 = 0;
        let mut previous_original_line: i64 = 0;
        let mut previous_original_column: i64 = 0;
        let mut previous_name: i64 = 0;

        for (generated_line, line_content) in self.inner.mapping_lines.iter_mut().enumerate() {
            let mut previous_generated_column: u32 = 0;
            let cloned_generated_line = generated_line as u32;
            if cloned_generated_line > 0 {
                // Write a ';' for each line between this and last line, way more efficient than storing empty lines or looping...
                output.write_all(
                    &b";".repeat((cloned_generated_line - last_generated_line) as usize),
                )?;
            }

            line_content.ensure_sorted();

            let mut is_first_mapping: bool = true;
            for mapping in &line_content.mappings {
                let generated_column = mapping.generated_column;
                let original_location_option = &mapping.original;
                if !is_first_mapping {
                    output.write_all(b",")?;
                }

                vlq::encode(
                    (generated_column - previous_generated_column) as i64,
                    output,
                )?;
                previous_generated_column = generated_column;

                // Source should only be written if there is any
                if let Some(original) = &original_location_option {
                    let original_source = original.source as i64;
                    vlq::encode(original_source - previous_source, output)?;
                    previous_source = original_source;

                    let original_line = original.original_line as i64;
                    vlq::encode((original_line - previous_original_line) as i64, output)?;
                    previous_original_line = original_line;

                    let original_column = original.original_column as i64;
                    vlq::encode(original_column - previous_original_column, output)?;
                    previous_original_column = original_column;

                    if let Some(name) = original.name {
                        let original_name = name as i64;
                        vlq::encode(original_name - previous_name, output)?;
                        previous_name = original_name;
                    }
                }

                is_first_mapping = false;
            }

            last_generated_line = cloned_generated_line;
        }

        Ok(())
    }

    pub fn add_source(&mut self, source: &str) -> u32 {
        let relative_source = make_relative_path(self.project_root.as_str(), source);
        match self
            .inner
            .sources
            .iter()
            .position(|s| relative_source.eq(s))
        {
            Some(i) => i as u32,
            None => {
                self.inner.sources.push(relative_source);
                (self.inner.sources.len() - 1) as u32
            }
        }
    }

    pub fn add_sources<I: AsRef<str>>(&mut self, sources: Vec<I>) -> Vec<u32> {
        self.inner.sources.reserve(sources.len());
        let mut result_vec = Vec::with_capacity(sources.len());
        for s in sources.iter() {
            result_vec.push(self.add_source(s.as_ref()));
        }
        result_vec
    }

    pub fn get_source_index(&self, source: &str) -> Result<Option<u32>, SourceMapError> {
        let normalized_source = make_relative_path(self.project_root.as_str(), source);
        Ok(self
            .inner
            .sources
            .iter()
            .position(|s| normalized_source.eq(s))
            .map(|v| v as u32))
    }

    pub fn get_source(&self, index: u32) -> Result<&str, SourceMapError> {
        self.inner
            .sources
            .get(index as usize)
            .map(|v| v.as_str())
            .ok_or_else(|| SourceMapError::new(SourceMapErrorType::SourceOutOfRange))
    }

    pub fn get_sources(&self) -> &Vec<String> {
        &self.inner.sources
    }

    pub fn add_name(&mut self, name: &str) -> u32 {
        return match self.inner.names.iter().position(|s| name.eq(s)) {
            Some(i) => i as u32,
            None => {
                self.inner.names.push(String::from(name));
                (self.inner.names.len() - 1) as u32
            }
        };
    }

    pub fn add_names<I: AsRef<str>>(&mut self, names: Vec<I>) -> Vec<u32> {
        self.inner.names.reserve(names.len());
        return names.iter().map(|n| self.add_name(n.as_ref())).collect();
    }

    pub fn get_name_index(&self, name: &str) -> Option<u32> {
        self.inner
            .names
            .iter()
            .position(|n| name.eq(n))
            .map(|v| v as u32)
    }

    pub fn get_name(&self, index: u32) -> Result<&str, SourceMapError> {
        self.inner
            .names
            .get(index as usize)
            .map(|v| v.as_str())
            .ok_or_else(|| SourceMapError::new(SourceMapErrorType::NameOutOfRange))
    }

    pub fn get_names(&self) -> &Vec<String> {
        &self.inner.names
    }

    pub fn set_source_content(
        &mut self,
        source_index: usize,
        source_content: &str,
    ) -> Result<(), SourceMapError> {
        if self.inner.sources.is_empty() || source_index > self.inner.sources.len() - 1 {
            return Err(SourceMapError::new(SourceMapErrorType::SourceOutOfRange));
        }

        let sources_content_len = self.inner.sources_content.len();
        if sources_content_len > source_index {
            self.inner.sources_content[source_index] = String::from(source_content);
        } else {
            self.inner
                .sources_content
                .reserve((source_index + 1) - sources_content_len);
            let items_to_add = source_index - sources_content_len;
            for _n in 0..items_to_add {
                self.inner.sources_content.push(String::from(""));
            }
            self.inner
                .sources_content
                .push(String::from(source_content));
        }

        Ok(())
    }

    pub fn get_source_content(&self, index: u32) -> Result<&str, SourceMapError> {
        self.inner
            .sources_content
            .get(index as usize)
            .map(|v| v.as_str())
            .ok_or_else(|| SourceMapError::new(SourceMapErrorType::SourceOutOfRange))
    }

    pub fn get_sources_content(&self) -> &Vec<String> {
        &self.inner.sources_content
    }

    // Write the sourcemap instance to a buffer
    pub fn to_buffer(&self, output: &mut AlignedVec) -> Result<(), SourceMapError> {
        output.clear();
        let mut serializer = CompositeSerializer::new(
            AlignedSerializer::new(output),
            AllocScratch::default(),
            Infallible,
        );
        serializer.serialize_value(&self.inner)?;
        Ok(())
    }

    // Create a sourcemap instance from a buffer
    pub fn from_buffer(project_root: &str, buf: &[u8]) -> Result<SourceMap, SourceMapError> {
        let archived = unsafe { archived_root::<SourceMapInner>(buf) };
        // TODO: see if we can use the archived data directly rather than deserializing at all...
        let inner = archived.deserialize(&mut Infallible)?;
        Ok(SourceMap {
            project_root: String::from(project_root),
            inner,
        })
    }

    pub fn add_sourcemap(
        &mut self,
        sourcemap: &mut SourceMap,
        line_offset: i64,
    ) -> Result<(), SourceMapError> {
        self.inner.sources.reserve(sourcemap.inner.sources.len());
        let mut source_indexes = Vec::with_capacity(sourcemap.inner.sources.len());
        let sources = std::mem::take(&mut sourcemap.inner.sources);
        for s in sources.iter() {
            source_indexes.push(self.add_source(s));
        }

        self.inner.names.reserve(sourcemap.inner.names.len());
        let mut names_indexes = Vec::with_capacity(sourcemap.inner.names.len());
        let names = std::mem::take(&mut sourcemap.inner.names);
        for n in names.iter() {
            names_indexes.push(self.add_name(n));
        }

        self.inner
            .sources_content
            .reserve(sourcemap.inner.sources_content.len());
        let sources_content = std::mem::take(&mut sourcemap.inner.sources_content);
        for (i, source_content_str) in sources_content.iter().enumerate() {
            if let Some(source_index) = source_indexes.get(i) {
                self.set_source_content(*source_index as usize, source_content_str)?;
            }
        }

        let mapping_lines = std::mem::take(&mut sourcemap.inner.mapping_lines);
        for (line, mapping_line) in mapping_lines.into_iter().enumerate() {
            let generated_line = (line as i64) + line_offset;
            if generated_line >= 0 {
                let mut line = mapping_line;
                for mapping in line.mappings.iter_mut() {
                    match &mut mapping.original {
                        Some(original_mapping_location) => {
                            original_mapping_location.source = match source_indexes
                                .get(original_mapping_location.source as usize)
                            {
                                Some(new_source_index) => *new_source_index,
                                None => {
                                    return Err(SourceMapError::new(
                                        SourceMapErrorType::SourceOutOfRange,
                                    ));
                                }
                            };

                            original_mapping_location.name = match original_mapping_location.name {
                                Some(name_index) => match names_indexes.get(name_index as usize) {
                                    Some(new_name_index) => Some(*new_name_index),
                                    None => {
                                        return Err(SourceMapError::new(
                                            SourceMapErrorType::NameOutOfRange,
                                        ));
                                    }
                                },
                                None => None,
                            };
                        }
                        None => {}
                    }
                }

                self.ensure_lines(generated_line as usize);
                self.inner.mapping_lines[generated_line as usize] = line;
            }
        }

        Ok(())
    }

    pub fn extends(&mut self, original_sourcemap: &mut SourceMap) -> Result<(), SourceMapError> {
        self.inner
            .sources
            .reserve(original_sourcemap.inner.sources.len());
        let mut source_indexes = Vec::with_capacity(original_sourcemap.inner.sources.len());
        for s in original_sourcemap.inner.sources.iter() {
            source_indexes.push(self.add_source(s));
        }

        self.inner
            .names
            .reserve(original_sourcemap.inner.names.len());
        let mut names_indexes = Vec::with_capacity(original_sourcemap.inner.names.len());
        for n in original_sourcemap.inner.names.iter() {
            names_indexes.push(self.add_name(n));
        }

        self.inner
            .sources_content
            .reserve(original_sourcemap.inner.sources_content.len());
        for (i, source_content_str) in original_sourcemap.inner.sources_content.iter().enumerate() {
            if let Some(source_index) = source_indexes.get(i) {
                self.set_source_content(*source_index as usize, source_content_str)?;
            }
        }

        for (_generated_line, line_content) in self.inner.mapping_lines.iter_mut().enumerate() {
            for mapping in line_content.mappings.iter_mut() {
                let original_location_option = &mut mapping.original;
                if let Some(original_location) = original_location_option {
                    let found_mapping = original_sourcemap.find_closest_mapping(
                        original_location.original_line,
                        original_location.original_column,
                    );
                    match found_mapping {
                        Some(original_mapping) => match original_mapping.original {
                            Some(original_mapping_location) => {
                                *original_location_option = Some(OriginalLocation::new(
                                    original_mapping_location.original_line,
                                    original_mapping_location.original_column,
                                    match source_indexes
                                        .get(original_mapping_location.source as usize)
                                    {
                                        Some(new_source_index) => *new_source_index,
                                        None => {
                                            return Err(SourceMapError::new(
                                                SourceMapErrorType::SourceOutOfRange,
                                            ));
                                        }
                                    },
                                    match original_mapping_location.name {
                                        Some(name_index) => {
                                            match names_indexes.get(name_index as usize) {
                                                Some(new_name_index) => Some(*new_name_index),
                                                None => {
                                                    return Err(SourceMapError::new(
                                                        SourceMapErrorType::NameOutOfRange,
                                                    ));
                                                }
                                            }
                                        }
                                        None => None,
                                    },
                                ));
                            }
                            None => {
                                *original_location_option = None;
                            }
                        },
                        None => {
                            *original_location_option = None;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn add_vlq_map<I: AsRef<str>>(
        &mut self,
        input: &[u8],
        sources: Vec<I>,
        sources_content: Vec<I>,
        names: Vec<I>,
        line_offset: i64,
        column_offset: i64,
    ) -> Result<(), SourceMapError> {
        let mut generated_line: i64 = line_offset;
        let mut generated_column: i64 = column_offset;
        let mut original_line = 0;
        let mut original_column = 0;
        let mut source = 0;
        let mut name = 0;

        let source_indexes: Vec<u32> = self.add_sources(sources);
        let name_indexes: Vec<u32> = self.add_names(names);

        self.inner.sources_content.reserve(sources_content.len());
        for (i, source_content) in sources_content.iter().enumerate() {
            if let Some(source_index) = source_indexes.get(i) {
                self.set_source_content(*source_index as usize, source_content.as_ref())?;
            }
        }

        let mut input = input.iter().cloned().peekable();
        while let Some(byte) = input.peek().cloned() {
            match byte {
                b';' => {
                    generated_line += 1;
                    generated_column = column_offset;
                    input.next().unwrap();
                }
                b',' => {
                    input.next().unwrap();
                }
                _ => {
                    // First is a generated column that is always present.
                    read_relative_vlq(&mut generated_column, &mut input)?;

                    // Read source, original line, and original column if the
                    // mapping has them.
                    let original = if input.peek().cloned().map_or(true, is_mapping_separator) {
                        None
                    } else {
                        read_relative_vlq(&mut source, &mut input)?;
                        read_relative_vlq(&mut original_line, &mut input)?;
                        read_relative_vlq(&mut original_column, &mut input)?;
                        Some(OriginalLocation::new(
                            original_line as u32,
                            original_column as u32,
                            match source_indexes.get(source as usize) {
                                Some(v) => *v,
                                None => {
                                    return Err(SourceMapError::new(
                                        SourceMapErrorType::SourceOutOfRange,
                                    ));
                                }
                            },
                            if input.peek().cloned().map_or(true, is_mapping_separator) {
                                None
                            } else {
                                read_relative_vlq(&mut name, &mut input)?;
                                Some(match name_indexes.get(name as usize) {
                                    Some(v) => *v,
                                    None => {
                                        return Err(SourceMapError::new(
                                            SourceMapErrorType::NameOutOfRange,
                                        ));
                                    }
                                })
                            },
                        ))
                    };

                    if generated_line >= 0 {
                        self.add_mapping(generated_line as u32, generated_column as u32, original);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn offset_columns(
        &mut self,
        generated_line: u32,
        generated_column: u32,
        generated_column_offset: i64,
    ) -> Result<(), SourceMapError> {
        match self.inner.mapping_lines.get_mut(generated_line as usize) {
            Some(line) => line.offset_columns(generated_column, generated_column_offset),
            None => Ok(()),
        }
    }

    pub fn offset_lines(
        &mut self,
        generated_line: u32,
        generated_line_offset: i64,
    ) -> Result<(), SourceMapError> {
        if generated_line_offset == 0 || self.inner.mapping_lines.is_empty() {
            return Ok(());
        }

        let (start_line, overflowed) =
            (generated_line as i64).overflowing_add(generated_line_offset);
        if overflowed || start_line > (u32::MAX as i64) {
            return Err(SourceMapError::new_with_reason(
                SourceMapErrorType::UnexpectedNegativeNumber,
                "column + column_offset cannot be negative",
            ));
        }

        let line = generated_line as usize;
        let abs_offset = generated_line_offset.unsigned_abs() as usize;
        if generated_line_offset > 0 {
            if line > self.inner.mapping_lines.len() {
                self.ensure_lines(line + abs_offset);
            } else {
                self.inner
                    .mapping_lines
                    .splice(line..line, (0..abs_offset).map(|_| MappingLine::new()));
            }
        } else {
            self.inner.mapping_lines.drain(line - abs_offset..line);
        }

        Ok(())
    }

    pub fn add_empty_map(
        &mut self,
        source: &str,
        source_content: &str,
        line_offset: i64,
    ) -> Result<(), SourceMapError> {
        let source_index = self.add_source(source);
        self.set_source_content(source_index as usize, source_content)?;

        for (line_count, _line) in source_content.lines().enumerate() {
            let generated_line = (line_count as i64) + line_offset;
            if generated_line >= 0 {
                self.add_mapping(
                    generated_line as u32,
                    0,
                    Some(OriginalLocation::new(
                        line_count as u32,
                        0,
                        source_index,
                        None,
                    )),
                )
            }
        }

        Ok(())
    }

    #[cfg(feature = "json")]
    pub fn from_json(project_root: &str, input: &str) -> Result<Self, SourceMapError> {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct JSONSourceMap<'a> {
            mappings: &'a str,
            #[serde(borrow)]
            sources: Vec<Cow<'a, str>>,
            #[serde(borrow, default)]
            sources_content: Vec<Option<Cow<'a, str>>>,
            names: Vec<Cow<'a, str>>,
        }

        let json: JSONSourceMap = serde_json::from_str(input)?;
        let mut sources_content = Vec::with_capacity(json.sources.len());
        for i in 0..json.sources.len() {
            sources_content.push(if let Some(Some(content)) = json.sources_content.get(i) {
                content.to_owned()
            } else {
                "".into()
            });
        }

        let mut sm = Self::new(project_root);
        sm.add_vlq_map(
            json.mappings.as_bytes(),
            json.sources,
            sources_content,
            json.names,
            0,
            0,
        )?;
        Ok(sm)
    }

    #[cfg(feature = "json")]
    pub fn to_json(&mut self, source_root: Option<&str>) -> Result<String, SourceMapError> {
        let mut vlq_output: Vec<u8> = Vec::new();
        self.write_vlq(&mut vlq_output)?;

        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct JSONSourceMap<'a> {
            version: u8,
            source_root: Option<&'a str>,
            mappings: &'a str,
            sources: &'a Vec<String>,
            sources_content: &'a Vec<String>,
            names: &'a Vec<String>,
        }

        let sm = JSONSourceMap {
            version: 3,
            source_root,
            mappings: unsafe { std::str::from_utf8_unchecked(&vlq_output) },
            sources: self.get_sources(),
            sources_content: self.get_sources_content(),
            names: self.get_names(),
        };

        Ok(serde_json::to_string(&sm)?)
    }

    #[cfg(feature = "json")]
    pub fn from_data_url(project_root: &str, data_url: &str) -> Result<Self, SourceMapError> {
        let url = DataUrl::process(data_url)?;
        let mime = url.mime_type();
        if mime.type_ != "application" || mime.subtype != "json" {
            return Err(SourceMapError::new(SourceMapErrorType::DataUrlError));
        }

        let (data, _) = url
            .decode_to_vec()
            .map_err(|_| SourceMapError::new(SourceMapErrorType::DataUrlError))?;
        let input = unsafe { std::str::from_utf8_unchecked(data.as_slice()) };

        Self::from_json(project_root, input)
    }

    #[cfg(feature = "json")]
    pub fn to_data_url(&mut self, source_root: Option<&str>) -> Result<String, SourceMapError> {
        let buf = self.to_json(source_root)?;
        let b64 = base64_simd::Base64::STANDARD.encode_to_boxed_str(buf.as_bytes());
        Ok(format!(
            "data:application/json;charset=utf-8;base64,{}",
            b64
        ))
    }
}

#[allow(non_fmt_panics)]
#[test]
fn test_buffers() {
    let map = SourceMap::new("/");
    let mut output = AlignedVec::new();
    match map.to_buffer(&mut output) {
        Ok(_) => {}
        Err(err) => panic!(err),
    }
    match SourceMap::from_buffer("/", &output) {
        Ok(map) => {
            println!("{:?}", map)
        }
        Err(err) => panic!(err),
    }
}

#[cfg(feature = "json")]
#[test]
fn test_to_json() {
    let mut map = SourceMap::new("/");
    map.add_mapping(1, 1, None);
    let json = map.to_json(Some("/")).unwrap();
    assert_eq!(
        json,
        r#"{"version":3,"sourceRoot":"/","mappings":";C","sources":[],"sourcesContent":[],"names":[]}"#
    );
}

#[cfg(feature = "json")]
#[test]
fn test_from_json() {
    let map = SourceMap::from_json("/", r#"{"version":3,"sourceRoot":"/","mappings":";C","sources":[],"sourcesContent":[],"names":[]}"#).unwrap();
    let mappings = map.get_mappings();
    assert_eq!(
        mappings,
        vec![Mapping {
            generated_line: 1,
            generated_column: 1,
            original: None
        }]
    );
}

#[cfg(feature = "json")]
#[test]
fn test_from_json_no_sources_content() {
    let map = SourceMap::from_json(
        "/",
        r#"{"version":3,"sourceRoot":"/","mappings":";C","sources":["file.js"],"names":[]}"#,
    )
    .unwrap();
    assert_eq!(map.get_sources_content(), &vec![""]);
    let map = SourceMap::from_json(
        "/",
        r#"{"version":3,"sourceRoot":"/","mappings":";C","sources":["file.js"],"sourcesContent":[null],"names":[]}"#,
    )
    .unwrap();
    assert_eq!(map.get_sources_content(), &vec![""]);
}

#[cfg(feature = "json")]
#[test]
fn test_to_data_url() {
    let mut map = SourceMap::new("/");
    map.add_mapping(1, 1, None);
    let url = map.to_data_url(Some("/")).unwrap();
    println!("{}", url);
    assert_eq!(
        url,
        r#"data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VSb290IjoiLyIsIm1hcHBpbmdzIjoiO0MiLCJzb3VyY2VzIjpbXSwic291cmNlc0NvbnRlbnQiOltdLCJuYW1lcyI6W119"#
    );
}

#[cfg(feature = "json")]
#[test]
fn test_from_data_url() {
    let map = SourceMap::from_data_url("/", r#"data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VSb290IjoiLyIsIm1hcHBpbmdzIjoiO0MiLCJzb3VyY2VzIjpbXSwic291cmNlc0NvbnRlbnQiOltdLCJuYW1lcyI6W119"#).unwrap();
    let mappings = map.get_mappings();
    assert_eq!(
        mappings,
        vec![Mapping {
            generated_line: 1,
            generated_column: 1,
            original: None
        }]
    );
}
