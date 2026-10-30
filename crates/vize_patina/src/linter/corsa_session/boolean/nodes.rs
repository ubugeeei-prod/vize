//! Select checker-owned syntax nodes instead of touching punctuation tokens.
//! Wire layouts: TypeScript 7.0.2 protocol 5 and tsrs 0.20.1 protocol 9.
//! Each handle belongs to the same source snapshot queried by the caller.
use corsa::api::NodeHandle;
use vize_l0::cstr;

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
    node_end: usize,
}

impl<'a> Tree<'a> {
    fn read(data: &'a [u8]) -> corsa::Result<Self> {
        let header = match word(data, 0)? >> 24 {
            5 => 44,
            9 => 64,
            _ => return Err(invalid("unsupported protocol version")),
        };
        if data.len() < header {
            return Err(invalid("truncated header"));
        }
        let strings = word(data, 24)? as usize;
        let text = word(data, 28)? as usize;
        let extended = word(data, 32)? as usize;
        let structured = word(data, 36)? as usize;
        let nodes = word(data, 40)? as usize;
        // Protocol 9 can append opaque binder data after its node records.
        // Those bytes are not syntax nodes and must never produce handles.
        let binder = if header == 64 {
            word(data, 60)? as usize
        } else {
            0
        };
        let node_end = if binder == 0 { data.len() } else { binder };
        if strings != header
            || !(strings <= text
                && text <= extended
                && extended <= structured
                && structured <= nodes
                && nodes <= node_end
                && node_end <= data.len())
            || !(text - strings).is_multiple_of(8)
            || !(node_end - nodes).is_multiple_of(NODE)
            || node_end - nodes < NODE * 2
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
            node_end,
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
    let count = (tree.node_end - tree.nodes) / NODE;
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
mod tests {
    //! Captured from actual stable and tsrs workers, not fabricated handles.
    use super::{NODE, locations, word};

    const SOURCE: &str =
        include_str!("../../../../tests/fixtures/strict-boolean-nodes/conditions.ts");
    const ENCODED: &[u8] =
        include_bytes!("../../../../tests/fixtures/strict-boolean-nodes/conditions.bin");
    const TSRS9: &[u8] = include_bytes!(
        "../../../../../../tests/_fixtures/differential/linter/native-boolean-tsrs-11147/conditions-tsrs9.bin"
    );

    fn range(expression: &str) -> (u32, u32) {
        let start = SOURCE.find(expression).expect("authored expression");
        (
            SOURCE[..start].encode_utf16().count() as u32,
            SOURCE[..start + expression.len()].encode_utf16().count() as u32,
        )
    }

    fn set(data: &mut [u8], offset: usize, value: u32) {
        data[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    #[test]
    fn selects_whole_element_access_and_call_after_astral_unicode() {
        let handles = locations(ENCODED, SOURCE, &[range("items[0]"), range("getTitle()")])
            .expect("encoded checker nodes");
        let actual: Vec<_> = handles.iter().map(|handle| handle.as_str()).collect();
        assert_eq!(
            actual,
            [
                "22.213./private/tmp/vize-checker-node-fixture/conditions.ts",
                "28.214./private/tmp/vize-checker-node-fixture/conditions.ts",
            ]
        );
    }

    #[test]
    fn actual_tsrs9_nodes_resolve_whole_expressions_in_the_same_snapshot() {
        assert_eq!(
            SOURCE,
            include_str!(
                "../../../../../../tests/_fixtures/differential/linter/native-boolean-tsrs-11147/conditions.ts.txt"
            )
        );
        let receipt: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../../tests/_fixtures/differential/linter/native-boolean-tsrs-11147/conditions-tsrs9.json"
        ))
        .unwrap();
        let handles = locations(TSRS9, SOURCE, &[range("items[0]"), range("getTitle()")])
            .expect("captured protocol9 nodes");
        let actual: Vec<_> = handles.iter().map(|handle| handle.as_str()).collect();
        assert_eq!(serde_json::json!(actual), receipt["locations"]);
        assert_eq!(receipt["binaryBytes"], TSRS9.len());
        assert_eq!(
            receipt["types"],
            serde_json::json!([
                {"id":89,"flags":1048576,"objectFlags":2097168,"isTupleType":false,"value":null,"isThisType":false,"symbol":{"id":2,"file":"106"}},
                {"id":14,"flags":32,"objectFlags":0,"isTupleType":false,"value":null,"isThisType":false,"intrinsicName":"string"}
            ])
        );
    }

    #[test]
    fn tsrs9_truncated_headers_sections_and_parent_links_fail_closed() {
        for end in 0..TSRS9.len() {
            assert!(locations(&TSRS9[..end], SOURCE, &[range("items[0]")]).is_err());
        }
        let nodes = word(TSRS9, 40).unwrap() as usize;
        for (offset, value) in [
            (0, 8 << 24),
            (24, 44),
            (60, nodes as u32 - 1),
            (60, TSRS9.len() as u32 + 1),
            (nodes + 23 * NODE + 16, 23),
        ] {
            let mut changed = TSRS9.to_vec();
            set(&mut changed, offset, value);
            assert!(locations(&changed, SOURCE, &[range("items[0]")]).is_err());
        }
        assert!(locations(TSRS9, "different source", &[range("items[0]")]).is_err());
    }

    #[test]
    fn tsrs9_binder_bytes_cannot_be_selected_as_syntax_nodes() {
        let expected = locations(TSRS9, SOURCE, &[range("items[0]")]).unwrap();
        let mut changed = TSRS9.to_vec();
        set(&mut changed, 60, TSRS9.len() as u32);
        changed.extend_from_slice(&[0xff; 31]);
        assert_eq!(
            locations(&changed, SOURCE, &[range("items[0]")]).unwrap(),
            expected
        );
    }

    #[test]
    fn incomplete_and_unsupported_wire_data_fail_closed() {
        for end in 0..ENCODED.len() {
            assert!(
                locations(&ENCODED[..end], SOURCE, &[range("items[0]")]).is_err(),
                "prefix {end}"
            );
        }
        let mut changed = ENCODED.to_vec();
        set(&mut changed, 0, 7 << 24);
        assert!(locations(&changed, SOURCE, &[range("items[0]")]).is_err());
        set(&mut changed, 0, 5 << 24);
        set(&mut changed, 24, 43);
        assert!(locations(&changed, SOURCE, &[range("items[0]")]).is_err());
    }

    #[test]
    fn mismatched_snapshots_invalid_strings_and_metadata_fail_closed() {
        assert!(locations(ENCODED, "different snapshot", &[range("items[0]")]).is_err());
        let mut changed = ENCODED.to_vec();
        let extended = word(&changed, 32).expect("extended") as usize;
        set(&mut changed, extended + 8, u32::MAX);
        assert!(locations(&changed, SOURCE, &[range("items[0]")]).is_err());
        let mut changed = ENCODED.to_vec();
        let nodes = word(&changed, 40).expect("nodes") as usize;
        let structured = word(&changed, 36).expect("structured");
        set(
            &mut changed,
            nodes + NODE + 20,
            0x8000_0000 | (structured - extended as u32),
        );
        assert!(locations(&changed, SOURCE, &[range("items[0]")]).is_err());
    }

    #[test]
    fn invalid_ranges_and_parent_cycles_fail_closed() {
        for query in [(0, 0), (u32::MAX, u32::MAX), (0, 200)] {
            assert!(locations(ENCODED, SOURCE, &[query]).is_err());
        }
        let mut changed = ENCODED.to_vec();
        let nodes = word(&changed, 40).expect("nodes") as usize;
        set(&mut changed, nodes + 23 * NODE + 16, 23);
        assert!(locations(&changed, SOURCE, &[range("items[0]")]).is_err());
        set(&mut changed, nodes + 23 * NODE + 16, u32::MAX);
        assert!(locations(&changed, SOURCE, &[range("items[0]")]).is_err());
    }
}
