use crate::types::SfcDescriptor;

/// Maps Croquis' merged script coordinate space back to authored SFC bytes.
#[derive(Debug, Clone, Copy)]
pub(super) struct ScriptOffsetMapper {
    source_start: u32,
    split_setup: Option<SplitScriptSetupOffset>,
}

#[derive(Debug, Clone, Copy)]
struct SplitScriptSetupOffset {
    synthetic_start: u32,
    source_start: u32,
}

impl ScriptOffsetMapper {
    pub(super) fn from_descriptor(
        descriptor: &SfcDescriptor<'_>,
        source_start: u32,
        merge_scripts: bool,
    ) -> Self {
        let split_setup = if merge_scripts {
            descriptor
                .script
                .as_ref()
                .zip(descriptor.script_setup.as_ref())
                .map(|(script, setup)| SplitScriptSetupOffset {
                    synthetic_start: script.content.len() as u32 + 1,
                    source_start: setup.loc.start as u32,
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
    pub(super) fn to_source_offset(self, offset: u32) -> u32 {
        if let Some(setup) = self.split_setup
            && offset >= setup.synthetic_start
        {
            return setup
                .source_start
                .saturating_add(offset - setup.synthetic_start);
        }
        self.source_start.saturating_add(offset)
    }

    #[inline]
    pub(super) fn source_len(self, start: u32, end: u32) -> u32 {
        self.to_source_offset(end)
            .saturating_sub(self.to_source_offset(start))
    }

    #[inline]
    pub(super) fn split_setup_offsets(self) -> Option<(usize, usize)> {
        self.split_setup
            .map(|setup| (setup.synthetic_start as usize, setup.source_start as usize))
    }
}

impl super::SfcCroquisAnalysis {
    #[inline]
    pub fn script_content_ref(&self) -> Option<&str> {
        self.script_content.as_deref()
    }

    #[inline]
    pub fn script_source_offset(&self, offset: u32) -> u32 {
        self.script_offset_mapper.to_source_offset(offset)
    }

    #[inline]
    pub fn script_source_len(&self, start: u32, end: u32) -> u32 {
        self.script_offset_mapper.source_len(start, end)
    }

    #[inline]
    pub fn split_script_setup_offsets(&self) -> Option<(usize, usize)> {
        self.script_offset_mapper.split_setup_offsets()
    }
}
