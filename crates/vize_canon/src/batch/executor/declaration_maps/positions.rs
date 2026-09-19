//! Compose both sides of a declaration source map using UTF-16 coordinates.

use super::MapSourceRewrite;
use crate::batch::{CorsaResult, VirtualProject, import_rewriter::ImportSourceMap};
use oxc_sourcemap::{SourceMap, Token};
use serde_json::Value;
use std::{fs, ops::Range, path::Path};
use vize_carton::line_index::LineIndex;

pub(super) fn rewrite_source_positions(
    json: &mut Value,
    project: &VirtualProject,
    rewrites: &[(usize, MapSourceRewrite<'_>)],
) -> CorsaResult<()> {
    let inputs: Vec<_> = rewrites
        .iter()
        .filter_map(|(id, rewrite)| {
            Some((
                *id as u32,
                project.find_by_virtual(rewrite.virtual_path)?,
                LineIndex::new(rewrite.source_content?),
            ))
        })
        .map(|(id, file, original)| (id, file, VirtualProject::declaration_input(file), original))
        .collect();
    let indices: Vec<_> = inputs
        .iter()
        .map(|(id, file, input, original)| {
            (*id, file, input, LineIndex::new(&input.code), original)
        })
        .collect();
    transform(json, |token| {
        let Some((_, file, input, generated, original)) = indices
            .iter()
            .find(|(id, ..)| Some(*id) == token.get_source_id())
        else {
            return Some(token);
        };
        let authored = generated
            .line_col_to_offset(token.get_src_line(), token.get_src_col())
            .map(|offset| input.source_map.get_original_offset(offset as u32))
            .and_then(|offset| file.source_map.get_original_position(offset))
            .map(|(offset, ..)| original.line_col(offset as usize));
        Some(match authored {
            Some((line, col)) => Token::new(
                token.get_dst_line(),
                token.get_dst_col(),
                line,
                col,
                token.get_source_id(),
                token.get_name_id(),
            ),
            None => Token::new(token.get_dst_line(), token.get_dst_col(), 0, 0, None, None),
        })
    })
}

pub(in crate::batch::executor) fn rewrite_generated_positions(
    declaration: &Path,
    before: &str,
    after: &str,
    imports: &ImportSourceMap,
    removed: &[Range<usize>],
    prefix_len: usize,
) -> CorsaResult<()> {
    let mut path = declaration.as_os_str().to_os_string();
    path.push(".map");
    let path = Path::new(&path);
    if !path.is_file() {
        return Ok(());
    }
    let mut json: Value = serde_json::from_str(&fs::read_to_string(path)?)?;
    let old = LineIndex::new(before);
    let new = LineIndex::new(after);
    transform(&mut json, |token| {
        let offset = old.line_col_to_offset(token.get_dst_line(), token.get_dst_col())?;
        let offset = imports.get_virtual_offset(offset as u32) as usize;
        if removed.iter().any(|range| range.contains(&offset)) {
            return None;
        }
        let deleted: usize = removed
            .iter()
            .filter(|range| range.end <= offset)
            .map(|range| range.len())
            .sum();
        let (line, col) = new.line_col(offset.checked_sub(deleted)? + prefix_len);
        Some(Token::new(
            line,
            col,
            token.get_src_line(),
            token.get_src_col(),
            token.get_source_id(),
            token.get_name_id(),
        ))
    })?;
    fs::write(path, serde_json::to_string(&json)?)?;
    Ok(())
}

fn transform(json: &mut Value, mut map: impl FnMut(Token) -> Option<Token>) -> CorsaResult<()> {
    let raw = serde_json::to_string(json)?;
    let mut parts = SourceMap::from_json_string(&raw)
        .map_err(std::io::Error::other)?
        .into_parts();
    parts.tokens = parts.tokens.iter().copied().filter_map(&mut map).collect();
    parts.token_chunks = None;
    let encoded = SourceMap::from_parts(parts).to_json();
    json["mappings"] = Value::String(encoded.mappings);
    Ok(())
}
