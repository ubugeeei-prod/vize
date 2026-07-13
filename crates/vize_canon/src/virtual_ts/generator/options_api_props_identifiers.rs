//! Options API identifier `props:` support for virtual TypeScript emission.

use vize_carton::String;

pub(super) struct PropsConstAssertions {
    offsets: Vec<usize>,
    index: usize,
}

impl PropsConstAssertions {
    pub(super) fn new(
        facts: Option<&vize_atelier_sfc::SfcScriptGeneratorFacts>,
        options_api: bool,
    ) -> Self {
        let offsets = if options_api {
            facts
                .map(|facts| facts.props_const_assertion_offsets().to_vec())
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        Self { offsets, index: 0 }
    }

    pub(super) fn splice_line(&mut self, line: &str, line_start: usize) -> Option<String> {
        while self.index < self.offsets.len() && self.offsets[self.index] <= line_start {
            self.index += 1;
        }

        let line_end = line_start + line.len();
        if self.index >= self.offsets.len() || self.offsets[self.index] > line_end {
            return None;
        }

        let mut output = String::default();
        let mut copied_until = 0usize;
        let mut spliced = false;
        while self.index < self.offsets.len() {
            let offset = self.offsets[self.index];
            if offset > line_end {
                break;
            }
            self.index += 1;
            if offset < line_start {
                continue;
            }
            let column = offset - line_start;
            if !line.is_char_boundary(column) {
                continue;
            }
            output.push_str(&line[copied_until..column]);
            output.push_str(" as const");
            copied_until = column;
            spliced = true;
        }

        if !spliced {
            return None;
        }
        output.push_str(&line[copied_until..]);
        Some(output)
    }

    pub(super) fn splice_output_line<'a>(
        &mut self,
        output_line: &mut std::borrow::Cow<'a, str>,
        line_start: usize,
    ) {
        if let Some(spliced) = self.splice_line(output_line.as_ref(), line_start) {
            *output_line = std::borrow::Cow::Owned(spliced.into());
        }
    }
}
