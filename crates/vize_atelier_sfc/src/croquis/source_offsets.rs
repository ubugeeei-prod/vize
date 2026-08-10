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
    fn from_analysis(
        descriptor: &SfcDescriptor<'_>,
        source_start: u32,
        script_content: Option<&str>,
    ) -> Self {
        let split_setup = descriptor
            .script
            .as_ref()
            .zip(descriptor.script_setup.as_ref())
            .filter(|(script, setup)| {
                script_content.is_some_and(|content| {
                    content.len() == script.content.len() + 1 + setup.content.len()
                        && content.starts_with(script.content.as_ref())
                        && content.ends_with(setup.content.as_ref())
                })
            })
            .map(|(script, setup)| SplitScriptSetupOffset {
                synthetic_start: script.content.len() as u32 + 1,
                source_start: setup.loc.start as u32,
            });
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
    fn script_offset_mapper(&self, descriptor: &SfcDescriptor<'_>) -> ScriptOffsetMapper {
        ScriptOffsetMapper::from_analysis(
            descriptor,
            self.script_offset,
            self.script_content.as_deref(),
        )
    }

    #[inline]
    pub fn script_content_ref(&self) -> Option<&str> {
        self.script_content.as_deref()
    }

    #[inline]
    pub fn script_source_offset(&self, descriptor: &SfcDescriptor<'_>, offset: u32) -> u32 {
        self.script_offset_mapper(descriptor)
            .to_source_offset(offset)
    }

    #[inline]
    pub fn script_source_len(&self, descriptor: &SfcDescriptor<'_>, start: u32, end: u32) -> u32 {
        self.script_offset_mapper(descriptor).source_len(start, end)
    }

    #[inline]
    pub fn split_script_setup_offsets(
        &self,
        descriptor: &SfcDescriptor<'_>,
    ) -> Option<(usize, usize)> {
        self.script_offset_mapper(descriptor).split_setup_offsets()
    }
}
