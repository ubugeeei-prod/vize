//! Compose native module origins with the current SWC AST's printed origins.
use swc_core::{
    common::{BytePos, DUMMY_SP, LineCol, Span},
    ecma::visit::{Visit, VisitMut},
};
use vize_carton::{FxHashSet, String, cstr};

pub struct Remapper<'a> {
    code: &'a str,
    lines: Vec<usize>,
    positions: Vec<((u32, u32), BytePos)>,
}

impl<'a> Remapper<'a> {
    pub fn new(
        code: &'a str,
        json: &str,
        mut printed: Vec<(BytePos, LineCol)>,
    ) -> Result<Self, String> {
        let map = oxc_sourcemap::SourceMap::from_json_string(json)
            .map_err(|error| cstr!("Invalid Vize source map: {error}"))?;
        printed.sort_by_key(|(_, loc)| (loc.line, loc.col));
        let positions = map
            .get_tokens()
            .map(|token| {
                let original = if token.get_source_id().is_some() {
                    let key = (token.get_src_line(), token.get_src_col());
                    printed
                        .partition_point(|(_, loc)| (loc.line, loc.col) <= key)
                        .checked_sub(1)
                        .and_then(|index| printed.get(index))
                        .filter(|(_, loc)| loc.line == key.0)
                        .map(|(pos, _)| *pos)
                        .unwrap_or(DUMMY_SP.lo)
                } else {
                    DUMMY_SP.lo
                };
                ((token.get_dst_line(), token.get_dst_col()), original)
            })
            .collect();
        let mut lines = vec![0];
        lines.extend(code.match_indices('\n').map(|(index, _)| index + 1));
        Ok(Self {
            code,
            lines,
            positions,
        })
    }

    pub fn position(&self, pos: BytePos) -> BytePos {
        let Some(offset) = pos.0.checked_sub(1).map(|value| value as usize) else {
            return DUMMY_SP.lo;
        };
        let Some(line) = self
            .lines
            .partition_point(|start| *start <= offset)
            .checked_sub(1)
        else {
            return DUMMY_SP.lo;
        };
        let Some(prefix) = self
            .lines
            .get(line)
            .and_then(|start| self.code.get(*start..offset))
        else {
            return DUMMY_SP.lo;
        };
        let key = (line as u32, prefix.encode_utf16().count() as u32);
        self.positions
            .partition_point(|(loc, _)| *loc <= key)
            .checked_sub(1)
            .and_then(|index| self.positions.get(index))
            .filter(|(loc, _)| loc.0 == key.0)
            .map(|(_, pos)| *pos)
            .unwrap_or(DUMMY_SP.lo)
    }

    pub fn span(&self, span: Span) -> Span {
        let lo = self.position(span.lo);
        if lo == DUMMY_SP.lo {
            return DUMMY_SP;
        }
        Span::new(lo, self.position(span.hi).max(lo))
    }
}
impl VisitMut for Remapper<'_> {
    fn visit_mut_span(&mut self, span: &mut Span) {
        *span = self.span(*span);
    }
}

#[derive(Default)]
pub struct CommentPositions(pub FxHashSet<BytePos>);
impl Visit for CommentPositions {
    fn visit_span(&mut self, span: &Span) {
        self.0.insert(span.lo);
        self.0.insert(span.hi);
    }
}
