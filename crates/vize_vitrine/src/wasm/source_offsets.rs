use super::utf8_byte_to_utf16_offset;

#[inline]
pub(crate) fn to_sfc_utf16_range(
    source: &str,
    base_offset: u32,
    start: u32,
    end: u32,
) -> (u32, u32) {
    let start = base_offset.saturating_add(start);
    let end = base_offset.saturating_add(end);
    (
        utf8_byte_to_utf16_offset(source, start),
        utf8_byte_to_utf16_offset(source, end),
    )
}

#[derive(Debug, Clone, Copy)]
struct SplitScriptSetupOffset {
    synthetic_start: u32,
    source_start: u32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ScriptOffsetMapper {
    source_start: u32,
    split_setup: Option<SplitScriptSetupOffset>,
}

impl ScriptOffsetMapper {
    pub(crate) fn from_descriptor(
        descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
        source_start: u32,
    ) -> Self {
        let split_setup = if let (Some(script), Some(script_setup)) =
            (&descriptor.script, &descriptor.script_setup)
        {
            Some(SplitScriptSetupOffset {
                synthetic_start: script.content.len() as u32 + 1,
                source_start: script_setup.loc.start as u32,
            })
        } else {
            None
        };

        Self {
            source_start,
            split_setup,
        }
    }

    #[inline]
    fn to_source_byte_offset(self, offset: u32) -> u32 {
        if let Some(setup) = self.split_setup
            && offset >= setup.synthetic_start
        {
            return setup
                .source_start
                .saturating_add(offset.saturating_sub(setup.synthetic_start));
        }

        self.source_start.saturating_add(offset)
    }

    #[inline]
    pub(crate) fn to_utf16_range(self, source: &str, start: u32, end: u32) -> (u32, u32) {
        (
            utf8_byte_to_utf16_offset(source, self.to_source_byte_offset(start)),
            utf8_byte_to_utf16_offset(source, self.to_source_byte_offset(end)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{ScriptOffsetMapper, to_sfc_utf16_range};
    use vize_atelier_sfc::{SfcParseOptions, parse_sfc};

    #[test]
    fn test_to_sfc_utf16_range_applies_block_offset_before_utf16_conversion() {
        let source = "<script setup>\nconst 😀value = 1\n</script>";
        let block_start = source.find("const").expect("script body should exist") as u32;
        let emoji_start_in_source = source.find('😀').expect("emoji should exist");
        let local_emoji_start = (emoji_start_in_source as u32).saturating_sub(block_start);
        let local_emoji_end = local_emoji_start + '😀'.len_utf8() as u32;

        let (start, end) =
            to_sfc_utf16_range(source, block_start, local_emoji_start, local_emoji_end);

        assert_eq!(
            start,
            source[..emoji_start_in_source].encode_utf16().count() as u32
        );
        assert_eq!(
            end,
            source[..emoji_start_in_source + '😀'.len_utf8()]
                .encode_utf16()
                .count() as u32
        );
    }

    #[test]
    fn test_script_offset_mapper_maps_split_setup_offsets_to_source_block() {
        let source = r#"<script>
const plain = 1
</script>
<!-- multibyte gap: あ -->
<script setup>
const setup = 'ready'
</script>"#;
        let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
        let script_offset = descriptor.script.as_ref().unwrap().loc.start as u32;
        let script_content_len = descriptor.script.as_ref().unwrap().content.len() as u32;
        let setup_content = descriptor.script_setup.as_ref().unwrap().content.as_ref();
        let setup_source_start = descriptor.script_setup.as_ref().unwrap().loc.start;
        let setup_local_start = setup_content.find("setup").unwrap();
        let setup_source_ident_start = setup_source_start + setup_local_start;
        let setup_synthetic_ident_start = script_content_len + 1 + setup_local_start as u32;

        let mapper = ScriptOffsetMapper::from_descriptor(&descriptor, script_offset);
        let (start, end) = mapper.to_utf16_range(
            source,
            setup_synthetic_ident_start,
            setup_synthetic_ident_start + "setup".len() as u32,
        );

        assert_eq!(
            start,
            source[..setup_source_ident_start].encode_utf16().count() as u32
        );
        assert_eq!(
            end,
            source[..setup_source_ident_start + "setup".len()]
                .encode_utf16()
                .count() as u32
        );
    }
}
