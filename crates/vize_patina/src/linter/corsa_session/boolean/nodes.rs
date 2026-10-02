//! Select checker-owned syntax nodes instead of touching punctuation tokens.
//! Wire layout: TypeScript 7.0.2, encoder.go at 2bd066d87f5bafd315be9f40889d0a60b9e58e0b.
use corsa::api::NodeHandle;
use vize_l0::cstr;

const HEADER: usize = 44;
const NODE: usize = 28;

fn invalid(reason: &str) -> corsa::CorsaError {
    corsa::CorsaError::Protocol(cstr!("Invalid boolean checker AST: {reason}"))
}

fn word(data: &[u8], offset: usize) -> corsa::Result<u32> {
    let bytes = data
        .get(
            offset
                ..offset
                    .checked_add(4)
                    .ok_or_else(|| invalid("offset overflow"))?,
        )
        .ok_or_else(|| invalid("truncated field"))?;
    Ok(u32::from_le_bytes(
        bytes.try_into().map_err(|_| invalid("truncated field"))?,
    ))
}

struct Tree<'a> {
    data: &'a [u8],
    strings: usize,
    text: usize,
    extended: usize,
    structured: usize,
    nodes: usize,
}

impl<'a> Tree<'a> {
    fn read(data: &'a [u8]) -> corsa::Result<Self> {
        if data.len() < HEADER || word(data, 0)? >> 24 != 5 {
            return Err(invalid("unsupported protocol version"));
        }
        let strings = word(data, 24)? as usize;
        let text = word(data, 28)? as usize;
        let extended = word(data, 32)? as usize;
        let structured = word(data, 36)? as usize;
        let nodes = word(data, 40)? as usize;
        if strings != HEADER
            || !(strings <= text
                && text <= extended
                && extended <= structured
                && structured <= nodes)
            || nodes > data.len()
            || !(text - strings).is_multiple_of(8)
            || !(data.len() - nodes).is_multiple_of(NODE)
            || data.len() - nodes < NODE * 2
        {
            return Err(invalid("invalid section bounds"));
        }
        Ok(Self {
            data,
            strings,
            text,
            extended,
            structured,
            nodes,
        })
    }

    fn string(&self, index: u32) -> corsa::Result<&'a [u8]> {
        // The encoder indexes the u32 offset table, whose entries are pairs.
        if !index.is_multiple_of(2) {
            return Err(invalid("unaligned string index"));
        }
        let offset = self
            .strings
            .checked_add(
                (index as usize)
                    .checked_mul(4)
                    .ok_or_else(|| invalid("string overflow"))?,
            )
            .ok_or_else(|| invalid("string overflow"))?;
        if offset.checked_add(8).is_none_or(|end| end > self.text) {
            return Err(invalid("invalid string index"));
        }
        let start = word(self.data, offset)? as usize;
        let end = word(self.data, offset + 4)? as usize;
        if start > end || end > self.extended - self.text {
            return Err(invalid("invalid string bounds"));
        }
        self.data
            .get(self.text + start..self.text + end)
            .ok_or_else(|| invalid("invalid string bounds"))
    }

    fn field(&self, index: usize, field: usize) -> corsa::Result<u32> {
        word(self.data, self.nodes + index * NODE + field)
    }
}

pub(super) fn locations(
    data: &[u8],
    source: &str,
    ranges: &[(u32, u32)],
) -> corsa::Result<Vec<NodeHandle>> {
    let tree = Tree::read(data)?;
    let root_data = tree.field(1, 20)?;
    if root_data & 0xc000_0000 != 0x8000_0000 {
        return Err(invalid("missing source-file metadata"));
    }
    let root = tree
        .extended
        .checked_add((root_data & 0x3fff_ffff) as usize)
        .ok_or_else(|| invalid("metadata overflow"))?;
    if root.checked_add(12).is_none_or(|end| end > tree.structured) {
        return Err(invalid("truncated source-file metadata"));
    }
    if tree.string(word(data, root)?)? != source.as_bytes() {
        return Err(invalid("source snapshot differs"));
    }
    let path = std::str::from_utf8(tree.string(word(data, root + 8)?)?)
        .map_err(|_| invalid("invalid source path"))?;
    if path.is_empty() {
        return Err(invalid("empty source path"));
    }
    let length = u32::try_from(source.encode_utf16().count())
        .map_err(|_| invalid("source length overflow"))?;
    let count = (data.len() - tree.nodes) / NODE;
    let mut ordered = Vec::new();
    for index in 1..count {
        if tree.field(index, 12)? as usize >= count || tree.field(index, 16)? as usize >= count {
            return Err(invalid("invalid syntax link"));
        }
        let kind = tree.field(index, 0)?;
        let start = tree.field(index, 4)?;
        let end = tree.field(index, 8)?;
        if kind != 0 && kind != u32::MAX && start <= end && end <= length {
            ordered.push((start, index));
        }
    }
    ordered.sort_unstable();
    ranges
        .iter()
        .map(|&(start, end)| {
            if start >= end || end > length {
                return Err(invalid("invalid expression range"));
            }
            let before = ordered.partition_point(|&(position, _)| position <= start);
            let mut index = ordered
                .get(before.wrapping_sub(1))
                .ok_or_else(|| invalid("missing expression node"))?
                .1;
            for _ in 0..count {
                let kind = tree.field(index, 0)?;
                if index > 1
                    && kind != u32::MAX
                    && tree.field(index, 4)? <= start
                    && tree.field(index, 8)? >= end
                {
                    return Ok(NodeHandle::from(cstr!("{index}.{kind}.{path}").as_str()));
                }
                index = tree.field(index, 16)? as usize;
                if index == 0 || index >= count {
                    return Err(invalid("missing expression node"));
                }
            }
            Err(invalid("cyclic syntax parents"))
        })
        .collect()
}

#[cfg(test)]
mod tests;
