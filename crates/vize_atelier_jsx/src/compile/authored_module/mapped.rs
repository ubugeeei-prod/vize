use oxc_sourcemap::SourceMap;
use vize_atelier_core::codegen::source_map::SourceMapBuilder;
use vize_l0::String;

use super::{JsxComponent, JsxDiagnostic, error};

pub(super) struct ModuleWriter<'a> {
    source: &'a str,
    components: &'a [JsxComponent],
    origins: &'a [crate::emission::RootSourceMap],
    maps: Vec<Option<SourceMap<'a>>>,
    code: String,
    map: Option<SourceMapBuilder>,
}

impl<'a> ModuleWriter<'a> {
    pub(super) fn new(
        source: &'a str,
        components: &'a [JsxComponent],
        origins: &'a [crate::emission::RootSourceMap],
        map: bool,
    ) -> Result<Self, JsxDiagnostic> {
        let maps = if map {
            components
                .iter()
                .enumerate()
                .map(|(index, component)| {
                    let raw = component
                        .map()
                        .or_else(|| origins.get(index).and_then(|origin| origin.json.as_deref()));
                    let Some(raw) = raw else {
                        return Err(error(
                            0,
                            0,
                            "requested JSX backend source origins are missing",
                        ));
                    };
                    let parsed = SourceMap::from_json_string(raw)
                        .map_err(|_| error(0, 0, "invalid JSX render source map"))?;
                    if parsed.get_sources().count() != 1
                        || parsed.get_source_content(0) != Some(source)
                    {
                        return Err(error(
                            0,
                            0,
                            "JSX render map does not belong to the authored module",
                        ));
                    }
                    Ok(Some(parsed))
                })
                .collect::<Result<Vec<_>, _>>()?
        } else {
            Vec::new()
        };
        Ok(Self {
            source,
            components,
            origins,
            maps,
            code: String::default(),
            map: map.then(SourceMapBuilder::new),
        })
    }

    pub(super) fn synthetic(&mut self, text: &str) {
        self.code.push_str(text);
    }

    pub(super) fn authored(&mut self, start: u32, end: u32) -> Result<(), JsxDiagnostic> {
        let text = self
            .source
            .get(start as usize..end as usize)
            .ok_or_else(|| error(start, end, "invalid authored JSX module span"))?;
        if let Some(map) = &mut self.map {
            for (offset, _) in text.char_indices() {
                map.add_raw(self.code.len() + offset, start + offset as u32);
            }
        }
        self.code.push_str(text);
        Ok(())
    }

    pub(super) fn generated(
        &mut self,
        index: usize,
        start: usize,
        end: usize,
    ) -> Result<(), JsxDiagnostic> {
        let component = self
            .components
            .get(index)
            .ok_or_else(|| error(0, 0, "missing mapped JSX component"))?;
        let text = component
            .code()
            .get(start..end)
            .ok_or_else(|| error(0, 0, "invalid generated JSX render span"))?;
        if let Some(builder) = &mut self.map {
            let map = self
                .maps
                .get(index)
                .and_then(Option::as_ref)
                .ok_or_else(|| error(0, 0, "missing JSX render source map"))?;
            let code_start = self
                .origins
                .get(index)
                .map_or(0, |origin| origin.code_start);
            let mapped_code = component
                .code()
                .get(code_start..)
                .ok_or_else(|| error(0, 0, "invalid JSX backend map offset"))?;
            for token in map.get_source_view_tokens() {
                let generated = byte_offset(mapped_code, token.get_dst_line(), token.get_dst_col())
                    .ok_or_else(|| error(0, 0, "invalid generated UTF-16 source-map position"))?
                    + code_start;
                let original = byte_offset(self.source, token.get_src_line(), token.get_src_col())
                    .ok_or_else(|| error(0, 0, "invalid authored UTF-16 source-map position"))?;
                if start <= generated && generated < end {
                    let position = self.code.len() + generated - start;
                    if let Some(name) = token.get_name() {
                        builder.add_named(position, original as u32, name);
                    } else {
                        builder.add_raw(position, original as u32);
                    }
                }
            }
        }
        self.code.push_str(text);
        Ok(())
    }

    pub(super) fn finish(self, filename: &str) -> (String, Option<String>) {
        let map = self
            .map
            .map(|m| m.finish(&self.code, filename, self.source));
        (self.code, map)
    }
}

/// Source maps use UTF-16 columns, while all syntax spans use UTF-8 bytes.
fn byte_offset(text: &str, line: u32, column: u32) -> Option<usize> {
    let start = if line == 0 {
        0
    } else {
        text.match_indices('\n').nth(line as usize - 1)?.0 + 1
    };
    let mut units = 0;
    for (offset, ch) in text.get(start..)?.char_indices() {
        if units == column {
            return Some(start + offset);
        }
        if ch == '\n' {
            return None;
        }
        units += ch.len_utf16() as u32;
        if units > column {
            return None;
        }
    }
    (units == column).then_some(text.len())
}
